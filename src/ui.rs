use super::network::host::Host;
use std::net::IpAddr;

use std::io;
use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::event::{Key};
use tui::{
  backend::TermionBackend,
  layout::{Constraint, Layout, Direction},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, Cell, Row, Table, TableState},
  Terminal,
};

pub struct StatefulTable {
  state: TableState,
  items: Vec<Host>
}

const JUMP_LEN: usize = 20;

impl StatefulTable {
  pub fn new(hosts: Vec<Host>) -> StatefulTable {
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
          self.items.len() - 1
        } else {
          i + JUMP_LEN
        }
      },
      None => 0
    };
    self.state.select(Some(i))
  }
}

pub fn ui_loop(hosts: Vec<IpAddr>) -> Result<(), io::Error> {
  let stdout = io::stdout().into_raw_mode()?;
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let p_hosts = hosts.iter().map(|h| Host::new(*h) ).collect();

  let mut table_state = StatefulTable::new(p_hosts);
  // let mut table_state = StatefulTable::new();

  // Uses termions async stdin for now,
  // Does not work on windows
  let mut stdin = termion::async_stdin().keys();

  terminal.clear()?;
  // TODO control this from a separate thread using an Atomic::Bool
  'outer: loop {

    terminal.draw(|f| {
      let rects = Layout::default()
          .direction(Direction::Vertical)
          .margin(1)
          .constraints(
              [
                  Constraint::Percentage(10),
                  Constraint::Percentage(80)
              ].as_ref()
          )
          .split(f.size());


      let block = Block::default()
          .title("Status")
          .borders(Borders::ALL);
      f.render_widget(block, rects[0]);

      
      let selected_style = Style::default()
          .bg(Color::Green)
          .fg(Color::Black)
          .add_modifier(Modifier::REVERSED);

      let normal_style = Style::default().bg(Color::Blue);

      let header_cells = ["Host IP", "Hostname", "Status", "Ports open"]
          .iter()
          .map(|h| Cell::from(*h));

      let header = Row::new(header_cells)
          .style(normal_style)
          .height(1)
          .bottom_margin(1);

      let rows = table_state.items.iter().map(|host| {
        let cells = vec![Cell::from(host.ip.to_string()), Cell::from("--"), Cell::from("?"), Cell::from("--")];
        Row::new(cells)
      });

      let t = Table::new(rows)
          .header(header)
          .block(Block::default().borders(Borders::ALL).title("Hosts"))
          .highlight_style(selected_style)
          .widths(&[
            Constraint::Length(18),
            Constraint::Percentage(30),
            Constraint::Length(10),
            Constraint::Max(10)
          ]);

      f.render_stateful_widget(t, rects[1], &mut table_state.state);
    })?;

    if let Some(Ok(key)) = stdin.next() {
      match key {
          Key::Ctrl('c') => break 'outer,
          Key::Down => table_state.next(),
          Key::Up => table_state.prev(),
          Key::Char(' ') => table_state.pgdn(),
          Key::PageDown => table_state.pgdn(),
          _ => {}
      }
    }
  }

  Ok(())
}
