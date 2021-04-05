use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Direction},
    text::{Span},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, TableState, Table, Paragraph, Gauge},
    Frame,
};

use crate::state::store::{SharedAppStateStore, AppStateStore};
use crate::ui::{
    notification::{Notification, NotificationLevel},
    components::search_filter::{draw_search_filter, SearchFilterOption}
};
use crate::ui::modal::{Modal, ModalType, ModalOpt};
use crate::state::actions::AppAction;
use crate::network::{
    input_parse,
    init_host_search,
    host::{PingType, Host, HostVec}
};
use crate::ui::{
    pages::PageContent
};

use termion::event::{Key};

const JUMP_LEN: usize = 20;

pub struct StatefulTable<'a> {
  state: &'a TableState,
  items: &'a Vec<&'a Host>
}

impl<'a> StatefulTable<'a> {

    #[allow(clippy::ptr_arg)]
    pub fn new(state: &'a TableState, items: &'a Vec<&'a Host>) -> StatefulTable<'a> {
        StatefulTable {
            state,
            items
        }
    }

    pub fn next(&self) -> Option<usize> {
        match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    Some(0)
                } else {
                    Some(i + 1)
                }
            },
            None => Some(0)
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
            },
            None => Some(self.items.len() - 1)
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
            },
            None => 0
        };
        Some(idx)
    }

    pub fn pgup(&self) -> Option<usize> {
        let idx = match self.state.selected() {
            Some(i) => {
                if i < JUMP_LEN {
                    self.items.len() -1
                } else {
                    i - JUMP_LEN
                }
            },
            None => 0
        };
        Some(idx)
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
                Constraint::Length(3)
            ].as_ref()
        )
        .split(f.size());
    
    let selected_border_style = Style::default().fg(Color::Yellow);
    let default_border_style = Style::default().fg(Color::White);

    let first_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref()
        )
        .split(rects[0]);

    let input = Paragraph::new(Span::from(query.to_owned()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    match curr_focus {
                        PageContent::QueryInput => {
                            if parse_err {
                                selected_border_style.fg(Color::Red)
                            } else {
                                selected_border_style
                            }
                        },
                        _ => default_border_style
                    }
                )
                .title("Host search")
        );

    f.render_widget(input, first_row[0]);

    // Render filter options
    draw_search_filter(&*lstore, first_row[1], f);

    if let PageContent::QueryInput = curr_focus {
        f.set_cursor(rects[0].x + query.len() as u16 + 1, rects[0].y + 1)
    }

    // Render Gauge //
    let hosts_len = &lstore.state.hosts.len();
    let num_done = &lstore.state.hosts
        .iter()
        .filter(|&h| h.ping_done)
        .count();
    
    let pcnt_done = (num_done * 100 / hosts_len) as u16;

    let gauge = Gauge::default()
        .block(Block::default().title("Hosts scanned").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(pcnt_done);
    
    f.render_widget(gauge, rects[2]);

    // Render host table // 
    let selected_style = Style::default()
        .bg(Color::Black)
        .fg(Color::Yellow)
        .add_modifier(Modifier::REVERSED);

    let normal_style = Style::default()
        .bg(Color::Rgb(23, 112, 191));

    let header_cells = ["Host IP", "Hostname", "Status", "Ping type", "Ports open"]
        .iter()
        .map(|h| Cell::from(*h));

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    
    let rows = get_selected_hosts(&lstore.state.hosts, &lstore.state.search_filter_opt).map(|host| {
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

            if let PingType::TCP = ping_type {
                port_cell = Cell::from(
                    host.tcp_ports.iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }

        if let Some(host_name) = &host.host_name {
            match host_name {
                Ok(hn) => {
                style = style.fg(Color::Green);
                host_cell = Cell::from(hn.to_string())
                },
                Err(_) => host_cell = Cell::from("x")
            }
        }

        let cells = vec![Cell::from(host.ip.to_string()), host_cell, status_cell, ping_cell, port_cell];
        Row::new(cells).style(style)
    });

    let table_block = Block::default()
        .borders(Borders::ALL)
        .border_style(
          match curr_focus {
            PageContent::HostTable => selected_border_style,
            _ => default_border_style
          }
        )
        .title("Hosts");

    let t = Table::new(rows)
        .header(header)
        .block(table_block)
        .highlight_style(selected_style)
        .widths(&[
            Constraint::Length(18),
            Constraint::Percentage(30),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Max(10)
        ]);

    f.render_stateful_widget(t, rects[1], &mut lstore.state.table_state);
}

// Page events handler
pub fn handle_main_page_event(key: Key, store: &mut AppStateStore, store_mtx: SharedAppStateStore) {
    // let mut store = store.lock().unwrap();
    match store.state.curr_focus {
        PageContent::HostTable => {
            let s_hosts: Vec<&Host> = get_selected_hosts(&store.state.hosts, &store.state.search_filter_opt).collect();
            let s_table = StatefulTable::new(&store.state.table_state, &s_hosts);
            if let Some(table_idx) = match key {
                // Char inputs
                Key::Down | Key::Char('j') => s_table.next(),
                Key::Up | Key::Char('k') => s_table.prev(),
                Key::Char(' ') | Key::Char('J') | Key::PageDown => s_table.pgdn(),
                Key::Ctrl(' ') | Key::Char('K') | Key::PageUp => s_table.pgup(),

                // Focus shift
                Key::Char('\t') => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::QueryInput))
                    }
                    None
                },
                Key::BackTab => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::SearchFilters))
                    }
                    None
                },

                _ => None
            } {
                store.dispatch(AppAction::TableSelect(Some(table_idx)))
            }
        },

        PageContent::SearchFilters => {
            match key {
                // TODO: Don't love is_none check, but do love that events bubble through the UI
                Key::Char('\t') => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::HostTable))
                    }
                },

                Key::BackTab => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::QueryInput))
                    }
                },

                Key::Down | Key::Up | Key::Char(' ') => {
                    match store.state.search_filter_opt {
                        SearchFilterOption::ShowFound => store.dispatch(AppAction::SetSearchFilter(SearchFilterOption::ShowAll)),
                        SearchFilterOption::ShowAll => {
                            // Reset table select index on filter
                            store.dispatches(vec![
                                AppAction::TableSelect(None),
                                AppAction::SetSearchFilter(SearchFilterOption::ShowFound)
                            ]);
                        },
                    }
                }
                _ => {}
            }
        }

        PageContent::QueryInput => {
            match key {
                // TODO: Don't love is_none check, but do live that events bubble through the UI
                Key::Char('\t') => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::SearchFilters))
                    }
                },

                Key::BackTab => {
                    if store.state.modal.is_none() {
                        store.dispatch(AppAction::ShiftFocus(PageContent::HostTable))
                    }
                },

                Key::Backspace => {
                    let qlen = store.state.query.len();
                    if qlen > 0 {
                        let q = store.state.query[..qlen - 1].to_owned();
                        store.dispatch(AppAction::SetQuery(q));
                        store.dispatch(AppAction::SetNotification(None))
                    }
                },

                Key::Char('\n') => {
                    let parsed = input_parse(&store.state.query);
                    store.dispatch(AppAction::SetInputErr(parsed.is_err()));
                    // Check if modal is visible and YES is selected, then parse and send hosts
                    match parsed {
                        Ok(hosts) => {
                            if let Some(modal) = &store.state.modal {
                                // Modal already displayed

                                if let ModalOpt::Yes = modal.selected {
                                    store.dispatches(vec![
                                        AppAction::SetHostSearchRun(false),
                                        AppAction::SetModal(None),
                                        AppAction::BuildHosts(hosts),
                                        AppAction::ShiftFocus(PageContent::HostTable)
                                    ]);
                                    // TODO: Should this be done from some sort of Thunk action?
                                    // Problem is that the store is wrapped in a mutex currently
                                    // and so does not have access to a thread-safe reference
                                    init_host_search(store_mtx)
                                }
                            } else {
                                // No modal

                                let mut msg = String::from("Are you sure you want to start a new query?");
                                if store.state.query_state {
                                    msg.push_str(" This will discard the current results.")
                                } else {
                                    msg.push_str(" This will kill the current query.")
                                }
                                let modal = Modal::new(
                                    "Confirm",
                                    &msg,
                                    ModalType::YesNo
                                );
                                store.dispatch(AppAction::SetModal(Some(modal)))
                            }
                        },
                        
                        Err(err) => {
                            store.dispatch(AppAction::SetNotification(
                                Some(Notification::new(
                                    "Error",
                                    &format!("{}", err),
                                    NotificationLevel::Warn
                                ))
                            ))

                        }
                    }
                },

                Key::Char(c) => {
                    if c.is_numeric() || c == '.' || c == '/' {
                        let mut q = store.state.query.to_owned();
                        q.push(c);
                        store.dispatch(AppAction::SetQuery(q));
                        store.dispatch(AppAction::SetNotification(None))
                    }
                },

                _ => {}
            }
        }
    }
}

#[allow(clippy::ptr_arg)]
fn get_selected_hosts<'a>(hosts: &'a HostVec, search_opt: &'a SearchFilterOption) -> impl Iterator<Item = &'a Host> {
    hosts.iter().filter(move |&h| {
        if matches!(search_opt, SearchFilterOption::ShowFound) {
            h.ping_res.is_some() || h.host_name.as_ref().unwrap_or(&Err(String::new())).is_ok()
        } else {
            true
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::state::application_state::ApplicationState;
    use std::{
        ops::DerefMut,
        sync::{Mutex, Arc},
        net::Ipv4Addr
    };

    #[test]
    fn test_main_page_host_table_nav() {
        let events: [(Key, usize); 8] = [
            (Key::Down, 0),
            (Key::Down, 1),
            (Key::Char('j'), 2),
            (Key::Up, 1),
            (Key::Char('k'), 0),
            (Key::Char('J'), 20),
            (Key::Char('K'), 0),
            (Key::Char(' '), 20),
        ];

        let store = AppStateStore::new();
        main_page_event_assertion(&events, Arc::new(Mutex::new(store)), |state: &ApplicationState| {
            state.table_state.selected().unwrap()
        })
    }

    #[test]
    fn test_main_page_focus_shift() {
        let events: [(Key, PageContent); 4] = [
            (Key::Char('\t'), PageContent::QueryInput),
            (Key::Char('\t'), PageContent::SearchFilters),
            (Key::BackTab, PageContent::QueryInput),
            (Key::BackTab, PageContent::HostTable),
        ];

        let store = AppStateStore::new();
        main_page_event_assertion(&events, Arc::new(Mutex::new(store)), |state: &ApplicationState| {
            state.curr_focus
        });
    }

    #[test]
    fn test_main_page_focus_shift_modal() {
        // When a modal is active, tab does not shift focus

        let events: [(Key, PageContent); 3] = [
            (Key::Char('\t'), PageContent::HostTable),
            (Key::BackTab, PageContent::HostTable),
            (Key::BackTab, PageContent::HostTable),
        ];

        let mut store = AppStateStore::new();
        store.state.modal = Some(Modal::new("Modal", "Cool message", ModalType::YesNo));

        main_page_event_assertion(&events, Arc::new(Mutex::new(store)), |state: &ApplicationState| {
            state.curr_focus
        });
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

        main_page_event_assertion(&events, Arc::new(Mutex::new(store)), |state: &ApplicationState| {
            state.query.to_owned()
        });
    }

    #[test]
    fn test_main_page_query_input_enter_ok() {
        let mut store = AppStateStore::new();
        store.state.query = "10.0.1.0".to_owned();
        store.state.curr_focus = PageContent::QueryInput;

        let events: [(Key, String); 1] = [
            // FIXME: test too rigid
            (Key::Char('\n'), "Are you sure you want to start a new query? This will kill the current query.".to_owned()),
        ];

        main_page_event_assertion(&events, Arc::new(Mutex::new(store)), |state: &ApplicationState| {
            state.modal.as_ref().unwrap().message.to_owned()
        });
        
    }

    #[test]
    fn test_main_page_query_input_enter_parse_too_large() {
        let mut store = AppStateStore::new();
        store.state.query = "10.0.1.0/2".to_owned();
        store.state.curr_focus = PageContent::QueryInput;

        let events: [(Key, bool); 1] = [
            (Key::Char('\n'), true),
        ];

        main_page_event_assertion(&events, Arc::new(Mutex::new(store)), |state: &ApplicationState| {
            state.notification.as_ref().unwrap().message.contains("Network is larger than max size")
        });
    }

    #[test]
    fn test_main_page_query_input_enter_parse_format_fail() {
        let mut store = AppStateStore::new();
        store.state.query = "10.0.1.299/".to_owned();
        store.state.curr_focus = PageContent::QueryInput;

        let events: [(Key, bool); 1] = [
            (Key::Char('\n'), true),
        ];

        main_page_event_assertion(&events, Arc::new(Mutex::new(store)), |state: &ApplicationState| {
            state.notification.as_ref().unwrap().message.contains("Please provide a valid IPv4 CIDR")
        });
    }

    #[test]
    fn test_main_page_filters_table_highlight() {
        let store = AppStateStore::new();

        let events: [(Key, Option<usize>); 5] = [
            (Key::Char(' '), Some(0)),
            (Key::Char(' '), Some(20)),
            (Key::Char('\t'), Some(20)),
            (Key::Char('\t'), Some(20)),
            (Key::Char(' '), None),
        ];

        main_page_event_assertion(&events, Arc::new(Mutex::new(store)), |state: &ApplicationState| {
            state.table_state.selected()
        });
    }

    fn main_page_event_assertion<AssertResult: PartialEq + std::fmt::Debug, F: Fn(&ApplicationState) -> AssertResult>(
        events: &[(Key, AssertResult)], lstore: SharedAppStateStore, state_fn: F
    ) {

        let mut store = lstore.lock().unwrap();
        for n in 0..30u8 {
            store.state.hosts.push(Host::new(Ipv4Addr::new(10, 0, 0, n)))
        }

        for (event, result) in events {
            // let lstore1 = store_mtx.clone();
            handle_main_page_event(*event, store.deref_mut(), lstore.clone());
            let res = state_fn(&store.state);
            assert_eq!(res, *result, "Testing event {:?}: {:?} vs {:?}", event, res, *result);
        }
    }
}
