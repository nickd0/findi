use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Direction},
    text::{Span},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, TableState, Table, Paragraph, Gauge},
    Frame,
};

use crate::state::store::{SharedAppStateStore, AppStateStore};
use crate::ui::notification::{Notification, NotificationLevel};
use crate::ui::modal::{Modal, ModalType};
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

enum MainPageContent {
    QueryInput,
    HostTable,
}

const JUMP_LEN: usize = 20;

pub struct StatefulTable<'a> {
  state: &'a TableState,
  items: &'a HostVec
}

impl<'a> StatefulTable<'a> {
    pub fn new(state: &'a TableState, items: &'a HostVec) -> StatefulTable<'a> {
        StatefulTable {
            state,
            items,
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

    f.render_widget(input, rects[0]);

    match curr_focus {
        PageContent::QueryInput => f.set_cursor(rects[0].x + query.len() as u16 + 1, rects[0].y + 1),
        _ => {}
    };

    // Render Gauge //
    let hosts_len = &lstore.state.hosts.len();
    let num_done = &lstore.state.hosts
        .iter()
        .filter(|&h| h.ping_done)
        .collect::<Vec<&Host>>()
        .len();
    
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

      let rows = lstore.state.hosts.iter().map(|host| {
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

          match ping_type {
            PingType::TCP => port_cell = Cell::from(
              host.tcp_ports.iter()
                  .map(|p| p.to_string())
                  .collect::<Vec<String>>()
                  .join(",")
            ),
            _ => {}
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
pub fn handle_main_page_event(key: Key, lstore: &mut AppStateStore) {
    // let mut lstore = store.lock().unwrap();
    match lstore.state.curr_focus {
        PageContent::HostTable => {
            let s_table = StatefulTable::new(&lstore.state.table_state, &lstore.state.hosts);
            if let Some(table_idx) = match key {
                // Char inputs
                Key::Down | Key::Char('j') => s_table.next(),
                Key::Up | Key::Char('k') => s_table.prev(),
                Key::Char(' ') | Key::Char('J') | Key::PageDown => s_table.pgdn(),
                Key::Ctrl(' ') | Key::Char('K') | Key::PageUp => s_table.pgup(),

                // Focus shift
                Key::Char('\t') | Key::BackTab => {
                    if lstore.state.modal.is_none() {
                        lstore.dispatch(AppAction::ShiftFocus(PageContent::QueryInput))
                    }
                    None
                },

                _ => None
            } {
                lstore.dispatch(AppAction::TableSelect(table_idx))
            }
        },

        PageContent::QueryInput => {
            match key {
                // TODO: Don't love is_none check, but do live that events bubble through the UI
                Key::Char('\t') | Key::BackTab => {
                    if lstore.state.modal.is_none() {
                        lstore.dispatch(AppAction::ShiftFocus(PageContent::HostTable))
                    }
                },

                Key::Backspace => {
                    let qlen = lstore.state.query.len();
                    if qlen > 0 {
                        let q = lstore.state.query[..qlen - 1].to_owned();
                        lstore.dispatch(AppAction::SetQuery(q));
                        lstore.dispatch(AppAction::SetNotification(None))
                    }
                },

                Key::Char('\n') => {
                    let parsed = input_parse(&lstore.state.query);
                    lstore.dispatch(AppAction::SetInputErr(parsed.is_err()));
                    // Check if modal is visible and YES is selected, then parse and send hosts
                    match parsed {
                        Ok(_) => {
                            if lstore.state.modal.is_some() {
                                // Assuming YES is selected for now
                                lstore.dispatches(vec![
                                    AppAction::SetHostSearchRun(false),
                                    AppAction::SetModal(None),
                                    AppAction::BuildHosts(parsed.unwrap()),
                                ]);
                                // TODO: start the host search, but we need the mutex and we only have
                                // a ref to the store from here
                            } else {
                                let mut msg = String::from("Are you sure you want to start a new query?");
                                if lstore.state.query_state {
                                    msg.push_str(" This will discard the current results.")
                                } else {
                                    msg.push_str(" This will kill the current query.")
                                }
                                let modal = Modal::new(
                                    "Confirm",
                                    &msg,
                                    ModalType::YesNo
                                );
                                lstore.dispatch(AppAction::SetModal(Some(modal)))
                            }
                        },
                        
                        Err(msg) => {
                            lstore.dispatch(AppAction::SetNotification(
                                Some(Notification::new(
                                    "Notification",
                                    &msg,
                                    NotificationLevel::Warn
                                ))
                            ))

                        }
                    }
                },

                Key::Char(c) => {
                    if c.is_numeric() || c == '.' || c == '/' {
                        let mut q = lstore.state.query.to_owned();
                        q.push(c);
                        lstore.dispatch(AppAction::SetQuery(q));
                        lstore.dispatch(AppAction::SetNotification(None))
                    }
                },
                _ => {}
            }
        },

        _ => {}
    }

    // match key {
    //     Key::Char('\t') | Key::BackTab => {
    //         match lstore.state.curr_focus {
    //             PageContent::HostTable => lstore.dispatch(AppAction::ShiftFocus(PageContent::QueryInput)),
    //             PageContent::QueryInput => lstore.dispatch(AppAction::ShiftFocus(PageContent::HostTable)),
    //             _ => {}
    //         }
    //     },
    //     _ => {}
    // }
}