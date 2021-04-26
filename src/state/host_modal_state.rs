use crate::network::host::Host;
use crate::ui::event::Key;

use std::time::Duration;

#[derive(Clone, Debug)]
pub enum HostModalAction {
    SetSelected(usize),
    SetPortQueryInput(Key),
    SetPortScanResult(TcpPortScanResult)
}

#[derive(Clone, Debug)]
pub struct TabsState {
    pub titles: Vec<String>,
    pub index: usize,
}

impl TabsState {
    pub fn new(titles: Vec<String>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub type TcpPortScanResult = (u16, Option<Result<Duration, ()>>);

#[derive(Clone, Debug)]
pub struct HostModalState {
    pub tab_state: TabsState,
    pub selected_host: Host,
    pub port_query: String,
    pub selected_component: usize,
    pub ports: Vec<TcpPortScanResult>
}

impl HostModalState {
    pub fn new(host: Host) -> Self {
        HostModalState {
            tab_state: TabsState {
                titles: vec!["Host info".to_owned(), "TCP port scan".to_owned()],
                index: 0,
            },
            selected_host: host,
            port_query: String::new(),
            selected_component: 0,
            ports: Vec::new()
        }
    }
}
