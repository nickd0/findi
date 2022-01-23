// UI page for service scan.

use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Direction},
    text::{Span},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, TableState, Table, Paragraph, Gauge},
    Frame,
};

use crate::state::store::{SharedAppStateStore, AppStateStore};
use crate::ui::components::search_filter::draw_search_filter;

pub fn draw_page<B: Backend>(store: SharedAppStateStore, f: &mut Frame<B>) {
	let lstore = store.lock().unwrap();

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

	draw_search_filter(&lstore, rects[0], f, "Select service group");

    // let t = Table::new(rows)
    //     .header(header)
    //     .block(table_block)
    //     .highlight_style(selected_style)
    //     .widths(&[
    //         Constraint::Length(18),
    //         Constraint::Percentage(30),
    //         Constraint::Length(15),
    //         Constraint::Length(10),
    //         Constraint::Max(10)
    //     ]);
	let table = Block::default().title("Results").borders(Borders::ALL);
    // f.render_widget(table, rects[1]);
}
