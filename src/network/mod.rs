pub mod ping_result;
pub mod udp_ping;
pub mod tcp_ping;
pub mod host;
pub mod dns;

use crate::state::store::SharedAppStateStore;
use crate::network::host::Host;
use crate::config::AppConfig;
use crate::state::actions::AppAction;
use crate::GLOBAL_RUN;

use std::time::Duration;
use std::thread;
use std::sync::{
    atomic::Ordering,
};

use threadpool::ThreadPool;
use pnet::ipnetwork::IpNetwork;

use std::net::Ipv4Addr;

pub fn input_parse(input: &str) -> Result<Vec<Ipv4Addr>, String> {
    if let Ok(IpNetwork::V4(ipn)) = input.parse::<IpNetwork>() {
        Ok(ipn.iter().collect())
    } else {
        Err("Please provide a valid IPv4 CIDR network".to_string())
    }
}

// TODO: should this just loop and wait for signals? Or be called each time?
// This could be a good candidate for async
pub fn init_host_search(store: SharedAppStateStore) {
    thread::spawn(move || {
        let config = AppConfig::default();

        // Do we need to clone here?
        let mut lstore = store.lock().unwrap();
        let hosts = lstore.state.hosts.clone();
        lstore.dispatch(AppAction::SetHostSearchRun(true));
        drop(lstore);

        let pool = ThreadPool::new(config.nworkers);

        for host in hosts {
            let lstore = store.lock().unwrap();
            if !GLOBAL_RUN.load(Ordering::Acquire) || !lstore.state.search_run {
                break
            }
            drop(lstore);

            let store_copy = store.clone();
            thread::sleep(Duration::from_millis(50));
            pool.execute(move || {
                let h = Host::host_ping(host.ip);
                store_copy
                    .lock().unwrap()
                    .dispatch(AppAction::UpdateHost(h));
            });

        }

    });
}
