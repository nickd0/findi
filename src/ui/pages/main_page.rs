use clipboard::{ClipboardContext, ClipboardProvider};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::network::{host::Host, input_parse};
use crate::state::actions::AppAction;
use crate::state::store::{AppStateStore, SharedAppStateStore};
use crate::ui::modal::{Modal, ModalType};
use crate::ui::pages::PageContent;
use crate::ui::{
    components::{
        search_filter::{draw_search_filter, SearchFilterOption},
        selectable_title::selectable_title,
    },
    notification::{Notification, NotificationLevel},
};

use std::convert::TryInto;

use crate::ui::event::Key;

const JUMP_LEN: usize = 20;

pub struct StatefulTable<'a> {
    state: &'a TableState,
    items: &'a Vec<&'a Host>,
}

impl<'a> StatefulTable<'a> {
    #[allow(clippy::ptr_arg)]
    pub fn new(state: &'a TableState, items: &'a Vec<&'a Host>) -> StatefulTable<'a> {
        StatefulTable { state, items }
    }

    pub fn next(&self) -> Option<usize> {
        match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    Some(0)
                } else {
                    Some(i + 1)
                }
            }
            None => Some(0),
        }
    }

    pub fn prev(&self) -> Option<usize> {
        match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    Some(self.items.len() - 1)
                } else {
                    Some(i - 1)
                }
            }
            None => Some(self.items.len() - 1),
        }
    }

    // TODO use table window size to paginate
    pub fn pgdn(&self) -> Option<usize> {
        let idx = match self.state.selected() {
            Some(i) => {
                if i + JUMP_LEN > self.items.len() - 1 {
                    0
                } else {
                    i + JUMP_LEN
                }
            }
            None => 0,
        };
        Some(idx)
    }

    pub fn pgup(&self) -> Option<usize> {
        let idx = match self.state.selected() {
            Some(i) => {
                if i < JUMP_LEN {
                    self.items.len() - 1
                } else {
                    i - JUMP_LEN
                }
            }
            None => 0,
        };
        Some(idx)
    }

    pub fn last(&self) -> Option<usize> {
        // TODO: guard
        Some(self.items.len() - 1)
    }

    pub fn first(&self) -> Option<usize> {
        // TODO: guard
        Some(0)
    }
}

pub fn draw_main_page<B: Backend>(store: SharedAppStateStore, f: &mut Frame<B>) {
    let mut lstore = store.lock().unwrap();

    let query = lstore.state.query.clone();
    let parse_err = lstore.state.input_err;
    let curr_focus = lstore.state.curr_focus;

    let rects = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let selected_border_style = Style::default().fg(Color::Yellow);
    let default_border_style = Style::default().fg(Color::White);

    let first_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(rects[0]);

    let input = Paragraph::new(Span::from(query.to_owned())).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(match curr_focus {
                PageContent::QueryInput => {
                    if parse_err {
                        selected_border_style.fg(Color::Red)
                    } else {
                        selected_border_style
                    }
                }
                _ => default_border_style,
            })
            .title(selectable_title("Search", Style::default())),
    );

    f.render_widget(input, first_row[0]);

    // Render filter options
    draw_search_filter(&*lstore, first_row[1], f);

    if let PageContent::QueryInput = curr_focus {
        f.set_cursor(rects[0].x + query.len() as u16 + 1, rects[0].y + 1)
    }

    // Render Gauge //
    let hosts_len = &lstore.state.hosts.len();
    let num_done = &lstore.state.hosts.iter().filter(|&h| h.ping_done).count();

    let pcnt_done = (num_done * 100 / hosts_len) as u16;

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title("Hosts scanned")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(pcnt_done);

    f.render_widget(gauge, rects[2]);

    // Render host table //
    let selected_style = Style::default()
        .bg(Color::Black)
        .fg(Color::Yellow)
        .add_modifier(Modifier::REVERSED);

    let normal_style = Style::default().bg(Color::Rgb(23, 112, 191));

    let header_cells = ["Host IP", "Hostname", "Status", "Ping type", "Ports open"]
        .iter()
        .map(|h| Cell::from(*h));

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    let rows = lstore.state.filtered_hosts().map(|host| {
        let mut style = Style::default();
        let mut status_cell = Cell::from("?");
        if let Some(dur) = host.ping_res {
            status_cell = Cell::from(format!("✓ ({:?} ms)", dur.as_millis()));
            style = style.fg(Color::Green);
        }

        let mut ping_cell = Cell::from("--");
        let mut port_cell = Cell::from("--");
        let mut host_cell = Cell::from("--");
        if let Some(ping_type) = host.ping_type {
            ping_cell = Cell::from(ping_type.to_string());

            port_cell = Cell::from(
                host.tcp_ports
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            );
        }

        if let Some(host_name) = &host.host_name {
            match host_name {
                Ok(hn) => {
                    style = style.fg(Color::Green);
                    host_cell = Cell::from(hn.to_string())
                }
                Err(_) => host_cell = Cell::from("x"),
            }
        }

        let cells = vec![
            Cell::from(host.ip.to_string()),
            host_cell,
            status_cell,
            ping_cell,
            port_cell,
        ];
        Row::new(cells).style(style)
    });

    let table_block = Block::default()
        .borders(Borders::ALL)
        .border_style(match curr_focus {
            PageContent::HostTable => selected_border_style,
            _ => default_border_style,
        })
        .title(selectable_title("Hosts", Style::default()));

    let t = Table::new(rows)
        .header(header)
        .block(table_block)
        .highlight_style(selected_style)
        .widths(&[
            Constraint::Length(18),
            Constraint::Percentage(30),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Max(10),
        ]);

    f.render_stateful_widget(t, rects[1], &mut lstore.state.table_state);
}

// Page events handler
// TODO: use keycode or key event here?
pub fn handle_main_page_event(key: Key, store: &mut AppStateStore, _: SharedAppStateStore) {
    // let mut store = store.lock().unwrap();

    // Components selection shortcuts
    match key {
        Key::Char('h') => store.dispatch(AppAction::ShiftFocus(PageContent::HostTable)),
        Key::Char('s') => store.dispatch(AppAction::ShiftFocus(PageContent::QueryInput)),
        Key::Char('f') => store.dispatch(AppAction::ShiftFocus(PageContent::SearchFilters)),
        Key::Char('?') => {
            let modal = Modal::new(
                "Help",
                "Keyboard shortcuts:\n\
                - Press 'h' to go to hosts table\n\
                - Press 's' to go to query search (return to begin query)\n\
                - Press 'f' to go to query filter\n\
                (Shortcuts are indicated by an underline character)\n\n\

                Filter controls:\n\
                - Use left/right arrows or space bar to cycle through filters\n\
                - Use Down arrow to go back to host table\n\n\

                Navigate the hosts table with arrows or similar to Vim: \n\
                - To go down one list item use down arrow or 'j' key\n\
                - To go up one list item use down arrow or 'k' key\n\
                - Use shift with the 'j' or 'k' key (or Page Up/Page Down) to move 20 items\n\
                - Use the space bar to jump 20 items down\n\
                - Press enter to see more information on that host\n\n\

                In the Host Info modal: \n\
                - Press 't' to go to TCP Port scan\n\
                - Press 'h' to go to host info
                ",
                ModalType::Ok,
            );
            store.dispatch(AppAction::SetModal(Some(modal)))
        }
        _ => {}
    }

    // store.dispatch(AppAction::ShiftFocus(PageContent::QueryInput))
    match store.state.curr_focus {
        PageContent::HostTable => {
            let s_hosts: Vec<&Host> = store.state.filtered_hosts().collect();
            let s_table = StatefulTable::new(&store.state.table_state, &s_hosts);
            if let Some(table_idx) = match key {
                // Char inputs
                Key::Down | Key::Char('j') => s_table.next(),
                Key::Up | Key::Char('k') => s_table.prev(),
                Key::Char(' ') | Key::Shift('J') | Key::PageDown => s_table.pgdn(),
                Key::Ctrl(' ') | Key::Shift('K') | Key::PageUp => s_table.pgup(),
                Key::Shift('G') => s_table.last(),
                Key::Char('g') => s_table.first(),

                // Focus shift
                Key::Tab => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::QueryInput))
                    }
                    None
                }
                Key::BackTab => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::SearchFilters))
                    }
                    None
                }

                // Host drill down
                Key::Enter => {
                    if let Some(idx) = store.state.table_state.selected() {
                        store.dispatch(AppAction::SetSelectedHost(Some(idx)));
                    }
                    None
                }

                // Copy host IP to clipboard
                Key::Char('c') | Key::Shift('C') => {
                    if let Some(host_idx) = store.state.table_state.selected() {
                        let mut notif = Notification::default();

                        match ClipboardProvider::new() {
                            Ok::<ClipboardContext, Box<dyn std::error::Error>>(mut ctx) => {
                                let hosts: Vec<&Host> = store.state.filtered_hosts().collect();
                                match key {
                                    Key::Char('c') => {
                                        let host_ip = hosts[host_idx].ip;
                                        notif.message = format!(
                                            "Address {} copied to clipboard",
                                            host_ip.to_string()
                                        );
                                        ctx.set_contents(host_ip.to_string()).unwrap();
                                    }
                                    Key::Shift('C') => {
                                        if let Some(Ok(hostname)) =
                                            hosts[host_idx].host_name.as_ref()
                                        {
                                            notif.message = format!(
                                                "Hostname {} copied to clipboard",
                                                hostname.to_owned()
                                            );
                                            ctx.set_contents(hostname.to_owned()).unwrap();
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Err(_) => {
                                notif.level = NotificationLevel::Warn;
                                notif.message = "Could not copy to clipboard".to_owned()
                            }
                        }

                        store.dispatch(AppAction::SetNotification(Some(notif)));
                    }

                    None
                }

                _ => None,
            } {
                store.dispatch(AppAction::TableSelect(Some(table_idx)))
            }
        }

        PageContent::SearchFilters => {
            match key {
                // TODO: Don't love is_none check, but do love that events bubble through the UI
                Key::Tab => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::HostTable))
                    }
                }

                Key::BackTab => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::QueryInput))
                    }
                }

                // Filter list includes show all, show resolved, and show each requested TCP port open
                // TODO: revisit this logic, its a little tangled
                Key::Left | Key::Right | Key::Char(' ') => {
                    match store.state.search_filter_opt {
                        SearchFilterOption::ShowAll => {
                            if store.state.port_query.len() > 0 && matches!(key, Key::Left) {
                                store.dispatch(AppAction::SetSearchFilter(
                                    SearchFilterOption::HasPort(store.state.port_query.len() - 1),
                                ))
                            } else {
                                store.dispatch(AppAction::SetSearchFilter(
                                    SearchFilterOption::ShowFound,
                                ))
                            }
                        }
                        SearchFilterOption::ShowFound => {
                            // Reset table select index on filter
                            if store.state.port_query.len() > 0 && matches!(key, Key::Right) {
                                let mut idx = 0;
                                if matches!(key, Key::Left) {
                                    idx = store.state.port_query.len() - 1
                                }
                                store.dispatches(vec![
                                    AppAction::TableSelect(None),
                                    AppAction::SetSearchFilter(SearchFilterOption::HasPort(idx)),
                                ]);
                            } else {
                                store.dispatches(vec![
                                    AppAction::TableSelect(None),
                                    AppAction::SetSearchFilter(SearchFilterOption::ShowAll),
                                ]);
                            }
                        }
                        SearchFilterOption::HasPort(idx) => {
                            let iidx: isize = idx.try_into().unwrap();

                            let nxt = match key {
                                Key::Left => iidx - 1,
                                Key::Right | Key::Char(' ') => iidx + 1,
                                _ => 0,
                            };

                            let iport_len = store.state.port_query.len().try_into().unwrap();

                            if nxt < iport_len && nxt > 0 {
                                store.dispatch(AppAction::SetSearchFilter(
                                    SearchFilterOption::HasPort(nxt.try_into().unwrap()),
                                ))
                            } else if nxt >= iport_len {
                                store.dispatch(AppAction::SetSearchFilter(
                                    SearchFilterOption::ShowAll,
                                ))
                            } else {
                                store.dispatch(AppAction::SetSearchFilter(
                                    SearchFilterOption::ShowFound,
                                ))
                            }
                        }
                    }
                }

                Key::Down => store.dispatch(AppAction::ShiftFocus(PageContent::HostTable)),

                _ => {}
            }
        }

        PageContent::QueryInput => {
            match key {
                // TODO: Don't love is_none check, but do live that events bubble through the UI
                Key::Tab => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::SearchFilters))
                    }
                }

                Key::BackTab => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::HostTable))
                    }
                }

                Key::Backspace => {
                    let qlen = store.state.query.len();
                    if qlen > 0 {
                        let q = store.state.query[..qlen - 1].to_owned();
                        store.dispatch(AppAction::SetQuery(q));
                        store.dispatch(AppAction::SetNotification(None))
                    }
                }

                Key::Enter => {
                    let parsed = input_parse(&store.state.query);
                    store.dispatch(AppAction::SetInputErr(parsed.is_err()));
                    // Check if modal is visible and YES is selected, then parse and send hosts
                    match parsed {
                        Ok(_) => {
                            let mut msg =
                                String::from("Are you sure you want to start a new query?");
                            if store.state.query_state {
                                msg.push_str(" This will discard the current results.")
                            } else {
                                msg.push_str(" This will kill the current query.")
                            }
                            let modal = Modal::new("Confirm", &msg, ModalType::YesNo);
                            store.dispatch(AppAction::SetModal(Some(modal)))
                        }

                        Err(err) => {
                            store.dispatch(AppAction::SetNotification(Some(Notification::new(
                                "Error",
                                &format!("{}", err),
                                NotificationLevel::Warn,
                            ))))
                        }
                    }
                }

                Key::Char(c) => {
                    if c.is_numeric() || c == '.' || c == '/' {
                        let mut q = store.state.query.to_owned();
                        q.push(c);
                        store.dispatch(AppAction::SetQuery(q));
                        store.dispatch(AppAction::SetNotification(None))
                    }
                }

                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::state::application_state::ApplicationState;
    use std::{
        net::Ipv4Addr,
        ops::DerefMut,
        sync::{Arc, Mutex},
    };

    #[test]
    fn test_main_page_host_table_nav() {
        let events: [(Key, usize); 8] = [
            (Key::Down, 0),
            (Key::Down, 1),
            (Key::Char('j'), 2),
            (Key::Up, 1),
            (Key::Char('k'), 0),
            (Key::Shift('J'), 20),
            (Key::Shift('K'), 0),
            (Key::Char(' '), 20),
        ];

        let store = AppStateStore::new();
        main_page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| state.table_state.selected().unwrap(),
        )
    }

    #[test]
    fn test_main_page_focus_shift() {
        let events: [(Key, PageContent); 4] = [
            (Key::Tab, PageContent::QueryInput),
            (Key::Tab, PageContent::SearchFilters),
            (Key::BackTab, PageContent::QueryInput),
            (Key::BackTab, PageContent::HostTable),
        ];

        let store = AppStateStore::new();
        main_page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| state.curr_focus,
        );
    }

    #[test]
    fn test_main_page_focus_shift_modal() {
        // When a modal is active, tab does not shift focus

        let events: [(Key, PageContent); 3] = [
            (Key::Tab, PageContent::HostTable),
            (Key::BackTab, PageContent::HostTable),
            (Key::BackTab, PageContent::HostTable),
        ];

        let mut store = AppStateStore::new();
        store.state.modal = Some(Modal::new("Modal", "Cool message", ModalType::YesNo));

        main_page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| state.curr_focus,
        );
    }

    #[test]
    fn test_main_page_query_input_chars() {
        // Test ignoring alpha characters, accepting numeric and '.'
        let events: [(Key, String); 6] = [
            (Key::Char('H'), "".to_owned()),
            (Key::Char('A'), "".to_owned()),
            (Key::Char('1'), "1".to_owned()),
            (Key::Char('0'), "10".to_owned()),
            (Key::Char('.'), "10.".to_owned()),
            (Key::Backspace, "10".to_owned()),
        ];

        let mut store = AppStateStore::new();
        store.state.curr_focus = PageContent::QueryInput;

        main_page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| state.query.to_owned(),
        );
    }

    #[test]
    fn test_main_page_query_input_enter_ok() {
        let mut store = AppStateStore::new();
        store.state.query = "10.0.1.0".to_owned();
        store.state.curr_focus = PageContent::QueryInput;

        let events: [(Key, String); 1] = [
            // FIXME: test too rigid
            (
                Key::Enter,
                "Are you sure you want to start a new query? This will kill the current query."
                    .to_owned(),
            ),
        ];

        main_page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| state.modal.as_ref().unwrap().message.to_owned(),
        );
    }

    #[test]
    fn test_main_page_query_input_enter_parse_too_large() {
        let mut store = AppStateStore::new();
        store.state.query = "10.0.1.0/2".to_owned();
        store.state.curr_focus = PageContent::QueryInput;

        let events: [(Key, bool); 1] = [(Key::Enter, true)];

        main_page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| {
                state
                    .notification
                    .as_ref()
                    .unwrap()
                    .message
                    .contains("Network is larger than max size")
            },
        );
    }

    #[test]
    fn test_main_page_query_input_enter_parse_format_fail() {
        let mut store = AppStateStore::new();
        store.state.query = "10.0.1.299/".to_owned();
        store.state.curr_focus = PageContent::QueryInput;

        let events: [(Key, bool); 1] = [(Key::Enter, true)];

        main_page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| {
                state
                    .notification
                    .as_ref()
                    .unwrap()
                    .message
                    .contains("Please provide a valid IPv4 CIDR")
            },
        );
    }

    #[test]
    fn test_main_page_filters_table_highlight() {
        let store = AppStateStore::new();

        let events: [(Key, Option<usize>); 6] = [
            (Key::Char(' '), Some(0)),
            (Key::Char(' '), Some(20)),
            (Key::Tab, Some(20)),
            (Key::Tab, Some(20)),
            (Key::Char(' '), Some(20)),
            (Key::Char(' '), None),
        ];

        main_page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| state.table_state.selected(),
        );
    }

    #[test]
    fn test_main_page_filters_navigate() {
        let mut store = AppStateStore::new();

        store.state.curr_focus = PageContent::SearchFilters;
        let events: [(Key, (SearchFilterOption, PageContent)); 3] = [
            (
                Key::Left,
                (SearchFilterOption::ShowFound, PageContent::SearchFilters),
            ),
            (
                Key::Left,
                (SearchFilterOption::ShowAll, PageContent::SearchFilters),
            ),
            (
                Key::Down,
                (SearchFilterOption::ShowAll, PageContent::HostTable),
            ),
        ];

        main_page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| (state.search_filter_opt, state.curr_focus),
        );
    }

    #[test]
    // TODO fix this test so that it doesn't rely on the actual clipboard of the test system
    fn test_main_page_copy_ip() {
        let mut store = AppStateStore::new();

        let mut resolved_host = Host::new(Ipv4Addr::new(10, 0, 1, 0));
        resolved_host.host_name = Some(Ok("foobar.local".to_owned()));
        store.state.hosts.push(resolved_host);

        let events: [(Key, bool); 3] = [
            (Key::Char('j'), false),
            (Key::Char('c'), true),
            (Key::Char('C'), true),
            // (Key::Char('c'), Some("Address 10.0.1.0 copied to clipboard".to_owned())),
            // (Key::Shift('C'), Some("Hostname foobar.local copied to clipboard".to_owned())),
        ];

        main_page_event_assertion(
            &events,
            Arc::new(Mutex::new(store)),
            |state: &ApplicationState| {
                state.notification.is_some()
                // if let Some(notif) = state.notification.as_ref() {
                //     Some(notif.message.to_owned())
                // } else {
                //     None
                // }
            },
        );
    }

    fn main_page_event_assertion<
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
            handle_main_page_event(*event, store.deref_mut(), lstore.clone());
            let res = state_fn(&store.state);
            assert_eq!(
                res, *result,
                "Testing event {:?}: {:?} vs {:?}",
                event, res, *result
            );
        }
    }
}
