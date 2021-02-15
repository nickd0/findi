use tui::{
    Frame,
    backend::Backend,
    layout::{Rect, Alignment, Layout, Constraint, Direction},
    text::{Span},
    style::{Color, Style},
    widgets::{Paragraph, Block, Borders},
};

use super::styles::border_style;

use crate::state::store::AppStateStore;
use crate::ui::pages::PageContent;

#[derive(Clone)]
pub enum SearchFilterOption {
    ShowAll,
    ShowFound
}

impl Default for SearchFilterOption {
    fn default() -> Self {
        Self::ShowAll
    }
}

pub fn draw_search_filter<B: Backend>(store: &AppStateStore, rect: Rect, f: &mut Frame<B>) {
    let filter_str = match store.state.search_filter_opt {
        SearchFilterOption::ShowAll => "Show all",
        SearchFilterOption::ShowFound => "Show resolved only",
    };

    let filter_style = Style::default().fg(Color::Green);

    let span = Span::from(Span::styled(
        filter_str, 
        filter_style
    ));

    let control_block = Block::default()
        .borders(Borders::ALL)
        .style(
            match store.state.curr_focus {
                PageContent::SearchFilters => border_style(true),
                _ => border_style(false)
            }
        )
        .title("Filter/sort");

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ].as_ref())
        .split(rect);

    let filter_option = Paragraph::new(span)
        .alignment(Alignment::Center)
        .block(control_block);
    
    f.render_widget(filter_option, rect);
}
