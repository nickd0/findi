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

const MAX_IPNETWORK_SIZE: u32 = 4096;

pub fn input_parse(input: &str) -> Result<Vec<Ipv4Addr>, String> {
    if let Ok(IpNetwork::V4(ipn)) = input.parse::<IpNetwork>() {
        // This is sort of an arbitrary limit, could be higher?
        if ipn.size() > MAX_IPNETWORK_SIZE {
            return Err(format!("Network is larger than max size of 4096 IP addresses ({})", ipn.size()).to_owned())
        }

        // Validate only private networks for now
        if !ipn.network().is_private() {
            return Err("Only private IP networks as defined in IETF RFC1918 can be scanned for now".to_owned())
        }
        Ok(ipn.iter().collect())
    } else {
        Err("Please provide a valid IPv4 CIDR network".to_owned())
    }
}

// TODO: profile performance here. On the one hand, dont want to have to
// manage another global atomicbool for this thread, on the other hand,
// locking the global state to check the run value and sleep if not set
// doesn't seem like a great way to wait.
// Note, this doens't seem to be working very well right now -- if start
// a new search, takes a long time for the new query to start
// May need to use an additional atomic bool since the search needs to stop before
// other state is updated
// Could use thread park https://doc.rust-lang.org/std/thread/fn.park.html
// Or use an event loop with mpsc
// This could be a good candidate for async

/*
2/15/21
TODO
Currently, when a new thread is launched here from a cancelled query,
the new thread takes over the store lock and the old thread waits on line 80,
then when the new thread terminates, the old one does too and eprints that it was
interrupted. This works, but seems inefficient because the old  thread is just
waiting on the store lock only to be cancelled. Revisit this and consider using an mpsc
*/
pub fn init_host_search(store: SharedAppStateStore) {
    thread::spawn(move || {
        let config = AppConfig::default();


        // Do we need to clone here?
        let mut lstore = store.lock().unwrap();
        let hosts = lstore.state.hosts.clone();
        // Wait for search run to be started

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
                if !store_copy.lock().unwrap().state.search_run {
                    // eprintln!("Interrupted! {:?}", host.ip);
                    return
                }
                let h = Host::host_ping(host.ip);
                store_copy.lock().unwrap().dispatch(AppAction::UpdateHost(h));
            });

        }
        pool.join();
        // Need to check if the query was interrupted or not
        store.lock().unwrap().dispatch(AppAction::QueryComplete);
    });
}
