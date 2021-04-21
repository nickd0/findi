use crate::network::host::Host;

#[derive(Clone, Debug)]
pub enum HostModalAction {
    SetSelected(usize)
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

#[derive(Clone, Debug)]
pub struct HostModalState {
    pub tab_state: TabsState,
    pub selected_host: Host,
}

impl HostModalState {
    pub fn new(host: Host) -> Self {
        HostModalState {
            tab_state: TabsState {
                titles: vec!["Host info".to_owned(), "TCP port scan".to_owned()],
                index: 0
            },
            selected_host: host
        }
    }
}
