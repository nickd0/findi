use crate::network::host::Host;
use crate::ui::event::Key;

use std::time::Duration;

#[derive(Clone, Debug)]
pub enum HostModalAction {
    SetSelected(usize),
    SetPortQueryInput(Key),
    SetPortScanResult(TcpPortScanResult),
    SetCommonPortsForScanning,
}

#[derive(Clone, Debug)]
pub struct TabsState {
    pub titles: Vec<String>,
    pub index: usize,
}

pub type TcpPortScanResult = (u16, Option<Result<Duration, ()>>);

#[derive(Clone, Debug)]
pub struct HostModalState {
    pub tab_state: TabsState,
    pub selected_host: Host,
    pub port_query: String,
    pub selected_component: usize,
    pub ports: Vec<TcpPortScanResult>,
    pub port_query_run: bool,
}

impl HostModalState {
    pub fn new(host: Host) -> Self {
        HostModalState {
            tab_state: TabsState {
                titles: vec![
                    "Host info".to_owned(),
                    "Common TCP port scan".to_owned(),
                    "TCP port scan".to_owned(),
                ],
                index: 0,
            },
            selected_host: host,
            port_query: String::new(),
            selected_component: 0,
            ports: Vec::new(),
            port_query_run: false,
        }
    }
}
