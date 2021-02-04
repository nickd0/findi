mod ip_parse;
mod network;
mod ui;
mod state;
mod dns;

use ip_parse::ip_parse;
use ui::ui_loop;
use network::host::{Host};
use state::store::AppStateStore;
use state::actions::AppAction;

use std::thread;
use std::sync::{Arc, Mutex};
use std::env;

#[allow(dead_code)]
fn start_ui(store: Arc<Mutex<AppStateStore>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let _ = ui_loop(store);
    })
}

fn main() {
    let input = env::args().nth(1).expect("Please provide an input");
    let hosts = ip_parse(input).unwrap_or_default();

    let mut threads = vec![];

    let mut store = AppStateStore::new();

    store.dispatch(AppAction::BuildHosts(hosts.clone()));

    let shared_store = Arc::new(Mutex::new(store));

    #[cfg(feature = "ui")]
    let ui_thread = start_ui(shared_store.clone());

    // TODO: share the same hosts vec between threads

    // This doesn't work because every thread waits to unlock the
    // ensure host vec
    // TODO: limit this to a certain number of threads
    for host in hosts {
        let store_copy = shared_store.clone();
        let t = thread::spawn(move || {
            // TODO: should this be mut or just receive the ping result value?
            let h = Host::host_ping(host);
            let mut store_lock = store_copy.lock().unwrap();
            store_lock.dispatch(AppAction::UpdateHost(h));
        });
        threads.push(t);
    }

    for t in threads {
        let _ = t.join();
    }

    #[cfg(feature = "ui")]
    let _ = ui_thread.join();
}

// fn main() {
//     ui_loop();
// }
