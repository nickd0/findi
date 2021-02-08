pub mod components;
mod modal;
mod notification;

use super::network::{
  input_parse,
  host::{PingType, HostVec, Host}
};
use crate::state::store::SharedAppStateStore;
use crate::state::actions::AppAction;
use components::MainPageContent;

use std::io;
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering}
};
use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::event::{Key};
use tui::{
  backend::{TermionBackend},
  layout::{Constraint, Layout, Direction},
  text::{Span},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, Cell, Row, Table, TableState, Paragraph, Gauge},
  Terminal,
};

pub struct StatefulTable {
  state: TableState,
  items: HostVec
}

const JUMP_LEN: usize = 20;

impl StatefulTable {
  pub fn new(hosts: HostVec) -> StatefulTable {
    StatefulTable {
      items: hosts,
      state: TableState::default()
    }
  }

  pub fn next(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i >= self.items.len() - 1 {
          0
        } else {
          i + 1
        }
      },
      None => 0
    };
    self.state.select(Some(i))
  }

  pub fn prev(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i == 0 {
          self.items.len() - 1
        } else {
          i - 1
        }
      },
      None => self.items.len() - 1
    };
    self.state.select(Some(i))
  }

  // TODO use table window size to paginate
  pub fn pgdn(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i + JUMP_LEN > self.items.len() - 1 {
          0
        } else {
          i + JUMP_LEN
        }
      },
      None => 0
    };
    self.state.select(Some(i))
  }

  pub fn pgup(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i < JUMP_LEN {
          self.items.len() -1
        } else {
          i - JUMP_LEN
        }
      },
      None => 0
    };
    self.state.select(Some(i))
  }
}

fn handle_table_input(key: Key, table_state: &mut StatefulTable) {
  match key {
    Key::Down | Key::Char('j') => table_state.next(),
    Key::Up | Key::Char('k') => table_state.prev(),
    Key::Char(' ') | Key::Char('J') | Key::PageDown => table_state.pgdn(),
    Key::Ctrl(' ') | Key::Char('K') | Key::PageUp => table_state.pgup(),
    _ => {}
  }
}

fn handle_field_input(key: Key, store: SharedAppStateStore) {
  let mut lstore = store.lock().unwrap();
  match key {
    Key::Backspace => {
      let qlen = lstore.state.query.len();
      if qlen > 0 {
        let q = lstore.state.query[..qlen - 1].to_owned();
        lstore.dispatch(AppAction::SetQuery(q))
      }
    },
    Key::Char('\n') => {
      let par_err = input_parse(&lstore.state.query).is_err();
      lstore.dispatch(AppAction::SetInputErr(par_err));
    },
    Key::Char(c) => {
      if !c.is_ascii_control() {
        let mut q = lstore.state.query.to_owned();
        q.push(c);
        lstore.dispatch(AppAction::SetQuery(q))
      }
    }
    _ => {}
  }
   
}

pub fn ui_loop(store: SharedAppStateStore, run: Arc<AtomicBool>) -> Result<(), io::Error> {
  let stdout = io::stdout().into_raw_mode()?;
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let lock_store = store.lock().unwrap();
  let mut table_state = StatefulTable::new(lock_store.state.hosts.clone());
  drop(lock_store);

  // Uses termions async stdin for now,
  // Does not work on windows
  let mut stdin = termion::async_stdin().keys();

  let mut curr_focus = MainPageContent::HostsTable;

  terminal.clear()?;
  // TODO control this from a separate thread using an Atomic::Bool

  while run.load(Ordering::Acquire) {

    // Update the stateful table from application state
    // Then release the lock
    let lock_store = store.lock().unwrap();
    table_state.items = lock_store.state.hosts.clone();
    // Clone for now, but maybe we shouldnt drop the store lock until the end of the loop?
    let query = lock_store.state.query.clone();
    let parse_err = lock_store.state.input_err;
    drop(lock_store);

    terminal.draw(|f| {
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
                  MainPageContent::QueryInput => {
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
        MainPageContent::QueryInput => f.set_cursor(rects[0].x + query.len() as u16 + 1, rects[0].y + 1),
        _ => {}
      };

      let num_done = table_state.items
          .iter()
          .filter(|&h| h.ping_done)
          .collect::<Vec<&Host>>()
          .len();

      let gauge = Gauge::default()
          .block(Block::default().title("Hosts scanned").borders(Borders::ALL))
          .gauge_style(Style::default().fg(Color::Yellow))
          .percent((num_done * 100 / table_state.items.len()) as u16);
        
      f.render_widget(gauge, rects[2]);
      
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

      let rows = table_state.items.iter().map(|host| {
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
            MainPageContent::HostsTable => selected_border_style,
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

      f.render_stateful_widget(t, rects[1], &mut table_state.state);

      if parse_err {
        // modal::draw_modal("Query error!".to_string(), f)
        notification::draw_notification("Query error".to_owned(), "Could not parse query input".to_owned(), f)
      }

      if num_done >= 10 {
        modal::draw_modal("Confirm".to_owned(), f);
      }
    })?;

    if let Some(Ok(key)) = stdin.next() {
      match key {
          Key::Char('\t') | Key::BackTab => {
            curr_focus = match curr_focus {
              MainPageContent::HostsTable => MainPageContent::QueryInput,
              _ => MainPageContent::HostsTable
            }
          }
          Key::Ctrl('c') => run.store(false, Ordering::Release),
          Key::Char('q') => run.store(false, Ordering::Release),
          _ => {
            match curr_focus {
              MainPageContent::HostsTable => handle_table_input(key, &mut table_state),
              MainPageContent::QueryInput => handle_field_input(key, store.clone()),
              _ => {}
            }
          }
      }
    }
  }
  terminal.clear()?;

  Ok(())
}
