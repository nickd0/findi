pub struct AppConfig {
    pub nworkers: usize
}

impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            nworkers: 100
        }
    }
}
