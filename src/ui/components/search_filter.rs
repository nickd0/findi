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
use crate::ui::{
    components::selectable_title::selectable_title,
    pages::PageContent
};

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum SearchFilterOption {
    ShowAll,
    ShowFound,
    HasPort(usize)
}

impl Default for SearchFilterOption {
    fn default() -> Self {
        Self::ShowAll
    }
}

pub fn draw_search_filter<B: Backend>(store: &AppStateStore, rect: Rect, f: &mut Frame<B>, title: &str, txt: &str, content: PageContent) {
    let filter_style = Style::default().fg(Color::Green);

    let span = Span::styled(
        format!("{} ▼", txt), 
        filter_style
    );

    let style = border_style(store.state.curr_focus == content);

    let control_block = Block::default()
        .borders(Borders::ALL)
        .style(style)
        .title(selectable_title(title, Style::default()));

    let filter_option = Paragraph::new(span)
        .alignment(Alignment::Center)
        .block(control_block);
    
    f.render_widget(filter_option, rect);
}
