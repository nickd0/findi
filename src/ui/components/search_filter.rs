use tui::{
    Frame,
    backend::Backend,
    layout::{Rect, Alignment},
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

    let span = Span::styled(
        format!("{} ▼", filter_str), 
        filter_style
    );

    let control_block = Block::default()
        .borders(Borders::ALL)
        .style(
            match store.state.curr_focus {
                PageContent::SearchFilters => border_style(true),
                _ => border_style(false)
            }
        )
        .title("Filter/sort");

    let filter_option = Paragraph::new(span)
        .alignment(Alignment::Center)
        .block(control_block);
    
    f.render_widget(filter_option, rect);
}
