// UI page for service scan.

use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Direction},
    text::{Span, Spans},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, TableState, Table, Paragraph, Gauge},
    Frame,
};

use crate::ui::components::{
    selectable_title::selectable_title,
    search_filter::draw_search_filter,
};
use crate::ui::event::Key;
use crate::ui::pages::PageContent;
use crate::state::actions::AppAction;
use crate::state::store::{SharedAppStateStore, AppStateStore};
use crate::network::dns::query::DnsQuestionType;
use crate::service::{
    service::{dispatch_service_search, ServiceDevice},
    service_list::DEFAULT_SERVICES,
};


// struct ServiceStatefulTable<'a> {
//     state: &'a TableState,
//     items: &'a Vec<&'a ServiceDevice>
// }


pub fn setup_page(store: &mut AppStateStore) {
    store.dispatch(AppAction::SelectServiceGroup(0));
}


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

    let first_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref()
        )
        .split(rects[0]);

    let svc_group = &DEFAULT_SERVICES[lstore.state.selected_service_group].0;

	draw_search_filter(
        &lstore,
        first_row[0],
        f,
        "Select service group",
        svc_group,
        PageContent::ServiceSelect,
    );

    let input = Paragraph::new(Span::from("Enter text".to_owned()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(selectable_title("Service name", Style::default()))
        );
    f.render_widget(input, first_row[1]);

    let rows = lstore.state.found_svcs.iter().map(|svc| {
        let mut name_cell = Cell::from(svc.svc_name);
        let mut ipv4_cell = Cell::from("--");
        let mut ipv6_cell = Cell::from("--");
        let mut txt_spans: Vec<Span> = vec![];

        if let Some(packet) = &svc.packet {
            for group in &[&packet.answers, &packet.addn_records] {
                for ans in *group {
                    let ans_txt = format!("{}", ans);
                    match ans.qtype {
                        DnsQuestionType::A => {
                            ipv4_cell = Cell::from(ans_txt)
                        },
                        DnsQuestionType::AAAA => {
                            ipv6_cell = Cell::from(ans_txt)
                        },
                        DnsQuestionType::PTR => {
                            name_cell = Cell::from(ans_txt)
                        },
                        DnsQuestionType::TXT => {
                            txt_spans.push(Span::from(ans_txt))
                        },
                        _ => {},
                    }
                }

            }
        }

        let txt_cell = Cell::from(Spans::from(txt_spans));
        Row::new(vec![name_cell, ipv4_cell, ipv6_cell, txt_cell]).style(Style::default())
    });

    let normal_style = Style::default()
        .bg(Color::Rgb(23, 112, 191));

    let header_cells = ["Name", "IPv4", "IPv6", "TXT"]
        .iter()
        .map(|h| Cell::from(*h));

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

	let table_block = Block::default().title(selectable_title("Results", Style::default())).borders(Borders::ALL);
    let table = Table::new(rows)
        .header(header)
        .block(table_block)
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Length(16),
            Constraint::Length(30),
            Constraint::Min(50),
        ]);

    f.render_widget(table, rects[1]);
}


pub fn handle_page_event(key: Key, store: &mut AppStateStore, shared_store: SharedAppStateStore) {
    match key {
        Key::Char('s') => store.dispatch(AppAction::ShiftFocus(PageContent::ServiceSelect)),
        Key::Down | Key::Up => {
            if store.state.curr_focus == PageContent::ServiceSelect {
                let idx = store.state.selected_service_group;

                let nx_idx: usize = match key {
                    Key::Down => {
                        if idx == DEFAULT_SERVICES.len() - 1 {
                            0
                        } else {
                            idx + 1
                        }
                    },
                    Key::Up => {
                        if idx == 0 {
                            DEFAULT_SERVICES.len() - 1
                        } else {
                            idx - 1
                        }
                    },
                    _ => 0,
                };
                store.dispatch(AppAction::SelectServiceGroup(nx_idx));
            }
        },
        Key::Enter => {
            if store.state.curr_focus == PageContent::ServiceSelect {
                let svcs = &DEFAULT_SERVICES[store.state.selected_service_group].1;
                let name = &DEFAULT_SERVICES[store.state.selected_service_group].0;
                dispatch_service_search(shared_store, name, svcs);
            }
        }
        _ => {},
    }
}
