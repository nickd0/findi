#[derive(Clone)]
pub struct AppConfig {
    // Number of workers for network scans
    pub nworkers: usize,
    // UI tick length (ms)
    pub tick_len: usize,
}

impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            nworkers: 100,
            tick_len: 100,
        }
    }
}
