// https://github.com/fdehau/tui-rs/blob/master/examples/popup.rs

use crate::{
    network::dispatch_common_port_scan,
    state::{
        actions::AppAction,
        host_modal_state::HostModalAction,
        store::{AppStateStore, SharedAppStateStore},
    },
};

use super::components::selectable_title::selectable_title;

use crate::ui::{
    components::text_input::{text_input, InputStyleState},
    event::Key,
    pages::PageContent,
};

use crate::network::{
    dispatch_port_scan, host::Host, init_host_search, input_parse, port_list::get_port_desc,
};

use std::convert::TryInto;

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};

#[derive(Clone)]
pub enum ModalType {
    YesNo,
    Ok,
    Custom,
}

#[derive(Clone)]
pub struct Modal {
    pub modal_type: ModalType,
    pub title: String,
    pub message: String,
    pub selected: ModalOpt,
}

impl Modal {
    pub fn new(title: &str, message: &str, modal_type: ModalType) -> Self {
        Self {
            title: String::from(title),
            message: String::from(message),
            selected: ModalOpt::Yes,
            modal_type,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum ModalOpt {
    Yes,
    No,
}

// Make Modal Opt a trait for different options types
impl ModalOpt {
    pub fn toggle(&self) -> Self {
        match self {
            ModalOpt::Yes => ModalOpt::No,
            ModalOpt::No => ModalOpt::Yes,
        }
    }

    pub fn mut_toggle(&mut self) {
        *self = self.toggle()
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

// TODO: draw specific modal types here
pub fn draw_modal<B: Backend>(modal: Modal, f: &mut Frame<B>) {
    let block = Block::default().title(modal.title).borders(Borders::ALL);
    let area = match modal.modal_type {
        ModalType::YesNo | ModalType::Custom => centered_rect(40, 30, f.size()),
        ModalType::Ok => centered_rect(40, 80, f.size()),
    };

    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(block, area);

    let mut yes_style = Style::default();
    let mut no_style = Style::default();

    match modal.selected {
        ModalOpt::Yes => yes_style = yes_style.fg(Color::Green),
        ModalOpt::No => no_style = no_style.fg(Color::Green),
    };

    let span = match modal.modal_type {
        ModalType::YesNo | ModalType::Custom => Span::styled("Yes", yes_style),
        ModalType::Ok => Span::styled("Ok", yes_style),
    };

    let no_span = Span::styled("No", no_style);

    let yes_btn = Paragraph::new(span).alignment(Alignment::Center);
    let no_btn = Paragraph::new(no_span).alignment(Alignment::Center);

    let msg_spans: Vec<Spans> = modal
        .message
        .split("\n")
        .map(|msgstr| Spans::from(msgstr))
        .collect();
    // let msg_span = Spans::from(Span::from(modal.message));
    // let msg_span = Spans::from(Span::from(modal.message));
    let modal_text = Paragraph::new(msg_spans).wrap(Wrap { trim: true });

    let btn_layout = match modal.modal_type {
        ModalType::YesNo | ModalType::Custom => Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .split(area),
        ModalType::Ok => Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(area),
    };

    let btn_layout_x = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(btn_layout[1]);

    let text_layout = Layout::default()
        .margin(2)
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(btn_layout[0]);

    f.render_widget(modal_text, text_layout[1]);

    if let ModalType::YesNo = modal.modal_type {
        f.render_widget(yes_btn, btn_layout_x[1]);
        f.render_widget(no_btn, btn_layout_x[3]);
    }
}

// TODO: consolidate modal drawing funcs
// TODO: break out this modal into separate file?
pub fn draw_host_modal<B: Backend>(
    modal: Modal,
    host: &Host,
    lstore: SharedAppStateStore,
    f: &mut Frame<B>,
) {
    let block = Block::default().title(modal.title).borders(Borders::ALL);
    let area = centered_rect(30, 70, f.size());
    let store = lstore.lock().unwrap();
    let modal_state = store.state.modal_state.as_ref().unwrap();

    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let btn_layout = Layout::default()
        .margin(2)
        .direction(Direction::Vertical)
        .constraints([Constraint::Max(2), Constraint::Percentage(80)].as_ref())
        .split(area);

    let titles = modal_state
        .tab_state
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(first, Style::default().add_modifier(Modifier::UNDERLINED)),
                Span::styled(rest, Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(modal_state.tab_state.index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, btn_layout[0]);

    // TODO: extract out the modal rendering
    // TODO: don't use tab state index for this, use an enum
    if modal_state.tab_state.index == 0 {
        let fields = vec![
            ("IP", host.ip.to_string()),
            (
                "Response time",
                match host.ping_res {
                    Some(dur) => dur.as_millis().to_string() + " ms",
                    None => "--".to_owned(),
                },
            ),
            (
                "Ping type",
                match host.ping_type {
                    Some(ptype) => ptype.to_string(),
                    None => "--".to_owned(),
                },
            ),
            (
                "Hostanme",
                match &host.host_name {
                    Some(Ok(hostname)) => hostname.to_owned(),
                    _ => "--".to_owned(),
                },
            ),
            (
                "Resolution type",
                match host.res_type {
                    Some(rtype) => rtype.to_string(),
                    None => "--".to_owned(),
                },
            ),
        ];

        const SPACING: usize = 15;

        // array_map is unstable as of now
        let field_spans: Vec<ListItem> = fields
            .iter()
            .map(|(field, val)| {
                ListItem::new(Spans::from(vec![
                    Span::styled(*field, Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" ".repeat(SPACING - field.len() + 3)),
                    // TODO: improve this
                    Span::from(val.to_owned()),
                ]))
            })
            .collect();

        let host_list = List::new(field_spans).start_corner(Corner::TopLeft);

        let text_layout = Layout::default()
            .margin(2)
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(btn_layout[1]);

        f.render_widget(host_list, text_layout[1]);
    } else {
        let layout = Layout::default()
            .margin(2)
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .split(btn_layout[1]);

        if modal_state.tab_state.index == 1 {
            let instructions = Span::from("Press enter to scan for commonly used ports.");

            // Render port input
            let parag = Paragraph::new(instructions).wrap(Wrap { trim: false });
            f.render_widget(parag, layout[0]);
        } else if modal_state.tab_state.index == 2 {
            let instructions = Span::from(
                r#"Enter port numbers comma separated or as a range
            e.g. "22,48,1000-1100""#,
            );

            // Render port input
            let parag = Paragraph::new(instructions).wrap(Wrap { trim: false });
            f.render_widget(parag, layout[0]);

            let input_title = selectable_title("Range", Style::default());

            let input_field = text_input(
                input_title,
                &modal_state.port_query,
                InputStyleState::Focused,
            );
            f.render_widget(input_field, layout[1]);

            f.set_cursor(
                layout[1].x + modal_state.port_query.len() as u16 + 1,
                layout[1].y + 1,
            );
        }

        // Render port scan result table
        let header = Row::new(vec!["Port", "Status", "Common use"]);

        // let rows: Vec<Row> = modal_state.ports.iter().map(|(port, stat)| {
        // TODO: make port lookup an optional feature
        // TODO: measure port lookup performance
        let rows: Vec<Row> = modal_state
            .ports
            .iter()
            .filter(|(_, stat)| matches!(stat, Some(Ok(_))))
            .map(|(port, stat)| {
                Row::new(vec![
                    port.to_string(),
                    format!("✓ ({:?})", stat.unwrap().unwrap()),
                    get_port_desc(port).to_owned(),
                ])
                .style(Style::default().fg(Color::Green))
            })
            .collect();

        let ports_title = selectable_title("Ports", Style::default());

        let table_block = Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .title(ports_title);

        let table = Table::new(rows).header(header).block(table_block).widths(
            [
                Constraint::Length(7),
                Constraint::Length(15),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        );

        f.render_widget(table, layout[2]);

        // Gauge
        let pcnt_done: u16 = (modal_state.ports.iter().filter(|p| p.1.is_some()).count() * 100)
            .checked_div(modal_state.ports.len())
            .unwrap_or(0)
            .try_into()
            .unwrap();

        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(pcnt_done);

        f.render_widget(gauge, layout[3]);
    }
}

// TODO: dispatch these events to the current modal state
pub fn handle_modal_event(key: Key, store: &mut AppStateStore, lstore: SharedAppStateStore) {
    match key {
        Key::BackTab | Key::Tab => {
            let modal = store.state.modal.as_ref().unwrap();
            let mut mclone = modal.clone();
            mclone.selected.mut_toggle();
            store.dispatch(AppAction::SetModal(Some(mclone)))
        }

        Key::Enter => {
            match &store.state.modal_state {
                Some(modal_state) => {
                    if modal_state.tab_state.index == 1 && !modal_state.port_scan_in_progress {
                        dispatch_common_port_scan(lstore);
                    } else if modal_state.tab_state.index == 2 {
                        dispatch_port_scan(lstore)
                    }
                }
                None => {
                    match store.state.modal.as_ref().unwrap().modal_type {
                        ModalType::YesNo => {
                            match store.state.modal.as_ref().unwrap().selected {
                                ModalOpt::No => store.dispatch(AppAction::SetModal(None)),

                                ModalOpt::Yes => {
                                    let parsed = input_parse(&store.state.query);
                                    store.dispatches(vec![
                                        AppAction::SetHostSearchRun(false),
                                        AppAction::SetModal(None),
                                        AppAction::BuildHosts(parsed.unwrap()),
                                        AppAction::ShiftFocus(PageContent::HostTable),
                                    ]);
                                    // TODO: Should this be done from some sort of Thunk action?
                                    // Problem is that the store is wrapped in a mutex currently
                                    // and so does not have access to a thread-safe reference
                                    init_host_search(lstore)
                                }
                            }
                        }
                        ModalType::Ok => store.dispatch(AppAction::SetModal(None)),

                        ModalType::Custom => {}
                    }
                }
            }
            // TODO: just dispatch the enter action and let the modal reducer take care of it?
            // store.dispatch(AppAction::SetModalAction(HostModalAction::SetSelected(idx)));
        }

        Key::Esc => store.dispatch(AppAction::SetModal(None)),

        Key::Char(c) => {
            match &mut store.state.modal_state {
                Some(modal_state) => {
                    let mut idx: usize = modal_state.tab_state.index;
                    for (i, title) in modal_state.tab_state.titles.iter().enumerate() {
                        if title.to_ascii_lowercase().starts_with(c) {
                            idx = i
                        }
                    }
                    store.dispatch(AppAction::SetModalAction(HostModalAction::SetSelected(idx)));

                    // TODO: clean up selected modal logic and use a match here
                    if c.is_ascii_digit() || c == '-' || c == ',' {
                        store.dispatch(AppAction::SetModalAction(
                            HostModalAction::SetPortQueryInput(key),
                        ))
                    }
                }

                None => {}
            }
        }

        Key::Backspace => store.dispatch(AppAction::SetModalAction(
            HostModalAction::SetPortQueryInput(key),
        )),

        _ => {}
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::network::host::Host;
    use crate::state::{application_state::ApplicationState, store::SharedAppStateStore};
    use std::net::Ipv4Addr;
    use std::ops::DerefMut;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_modal_event_tab() {
        let mut store = AppStateStore::new();

        let events: [(Key, ModalOpt); 3] = [
            (Key::Tab, ModalOpt::No),
            (Key::Tab, ModalOpt::Yes),
            (Key::BackTab, ModalOpt::No),
        ];

        store.state.modal = Some(Modal::new("Test modal", "This is a test", ModalType::YesNo));

        page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| state.modal.as_ref().unwrap().selected,
        )
    }

    #[test]
    fn test_modal_yesno_event_enter() {
        let mut store = AppStateStore::new();

        let events: [(Key, bool); 2] = [(Key::Tab, true), (Key::Enter, false)];

        store.state.modal = Some(Modal::new("Test modal", "This is a test", ModalType::YesNo));

        page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| state.modal.is_some(),
        )
    }

    #[test]
    fn test_modal_ok_event_enter() {
        let mut store = AppStateStore::new();

        let events: [(Key, bool); 1] = [(Key::Enter, false)];

        store.state.modal = Some(Modal::new("Test modal", "This is a test", ModalType::Ok));

        page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| state.modal.is_some(),
        )
    }

    #[test]
    fn test_modal_event_esc() {
        let mut store = AppStateStore::new();

        let events: [(Key, bool); 2] = [(Key::Tab, true), (Key::Esc, false)];

        store.state.modal = Some(Modal::new("Test modal", "This is a test", ModalType::YesNo));

        page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| state.modal.is_some(),
        )
    }

    // TODO consolidate this with main_page assertion function
    fn page_event_assertion<
        AssertResult: PartialEq + std::fmt::Debug,
        F: Fn(&ApplicationState) -> AssertResult,
    >(
        events: &[(Key, AssertResult)],
        lstore: SharedAppStateStore,
        state_fn: F,
    ) {
        let mut store = lstore.lock().unwrap();
        for n in 0..30u8 {
            store
                .state
                .hosts
                .push(Host::new(Ipv4Addr::new(10, 0, 0, n)))
        }

        for (event, result) in events {
            // let lstore1 = store_mtx.clone();
            handle_modal_event(*event, store.deref_mut(), lstore.clone());
            let res = state_fn(&store.state);
            assert_eq!(
                res, *result,
                "Testing event {:?}: {:?} vs {:?}",
                event, res, *result
            );
        }
    }
}
