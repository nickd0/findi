pub mod dns;
pub mod host;
pub mod ping_result;
pub mod port_list;
pub mod tcp_ping;
pub mod udp_ping;

use crate::state::{
    host_modal_state::{HostModalAction, HostModalState},
    store::SharedAppStateStore,
};

use crate::network::host::Host;
use crate::state::actions::AppAction;
use crate::GLOBAL_RUN;

use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use pnet::ipnetwork::IpNetwork;
use port_list::COMMON_PORTS;
use threadpool::ThreadPool;

use std::net::Ipv4Addr;

const MAX_IPNETWORK_SIZE: u32 = 4096;

pub fn input_parse(input: &str) -> Result<Vec<Ipv4Addr>> {
    if let Ok(IpNetwork::V4(ipn)) = input.parse::<IpNetwork>() {
        // This is sort of an arbitrary limit, could be higher?
        if ipn.size() > MAX_IPNETWORK_SIZE {
            return Err(anyhow!(
                "Network is larger than max size of 4096 IP addresses ({})",
                ipn.size()
            ));
        }

        // Validate only private networks for now
        if !ipn.network().is_private() {
            return Err(anyhow!(
                "Only private IP networks as defined in IETF RFC1918 can be scanned for now"
            ));
        }
        Ok(ipn.iter().collect())
    } else {
        Err(anyhow!("Please provide a valid IPv4 CIDR network"))
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
        // Do we need to clone here?
        let mut lstore = store.lock().unwrap();
        let hosts = lstore.state.hosts.clone();
        let network_nworkers = lstore.state.app_config.nworkers;
        // Wait for search run to be started
        lstore.dispatch(AppAction::SetHostSearchRun(true));
        drop(lstore);

        let pool = ThreadPool::new(network_nworkers);

        for host in hosts {
            let lstore = store.lock().unwrap();
            if !GLOBAL_RUN.load(Ordering::Acquire) || !lstore.state.search_run {
                break;
            }
            drop(lstore);

            let store_copy = store.clone();
            thread::sleep(Duration::from_millis(50));

            pool.execute(move || {
                let localstore = store_copy.lock().unwrap();
                let port_query = localstore.state.port_query.clone();
                if !localstore.state.search_run {
                    return;
                }
                drop(localstore);

                let mut h = Host::host_ping(host.ip);

                for port in port_query {
                    match tcp_ping::tcp_scan_port(&h.ip, port) {
                        Ok(_) => {
                            h.tcp_ports.insert(port);
                        }
                        Err(_) => {}
                    }
                }

                store_copy
                    .lock()
                    .unwrap()
                    .dispatch(AppAction::UpdateHost(h));
            });
        }
        pool.join();
        // Need to check if the query was interrupted or not
        store.lock().unwrap().dispatch(AppAction::QueryComplete);
    });
}

/// TODO: combine with the above for a single shared resouce access func
/// TODO: ensure this isn't dispatched more than once
pub fn dispatch_port_scan(store: SharedAppStateStore) {
    thread::spawn(move || {
        let lstore = store.lock().unwrap();
        let modal_state: HostModalState = lstore.state.modal_state.clone().unwrap();
        drop(lstore);

        for (port, _) in modal_state.ports {
            let port_run = store.lock().unwrap().state.modal_state.is_none();

            if !GLOBAL_RUN.load(Ordering::Acquire) || port_run {
                break;
            }

            match tcp_ping::tcp_scan_port(&modal_state.selected_host.ip, port) {
                Ok(dur) => store.lock().unwrap().dispatch(AppAction::SetModalAction(
                    HostModalAction::SetPortScanResult((port, Some(Ok(dur)))),
                )),
                Err(_) => store.lock().unwrap().dispatch(AppAction::SetModalAction(
                    HostModalAction::SetPortScanResult((port, Some(Err(())))),
                )),
            }

            thread::sleep(Duration::from_millis(10));
        }
    });
}

pub fn dispatch_common_port_scan(store: SharedAppStateStore) {
    thread::spawn(move || {
        let mut lstore = store.lock().unwrap();
        lstore.dispatch(AppAction::SetModalAction(
            HostModalAction::SetCommonPortsForScanning,
        ));
        let modal_state: HostModalState = lstore.state.modal_state.clone().unwrap();
        // don't lock up other threads
        drop(lstore);

        for port in COMMON_PORTS.iter() {
            let port_run = store.lock().unwrap().state.modal_state.is_none();

            if !GLOBAL_RUN.load(Ordering::Acquire) || port_run {
                break;
            }
            match tcp_ping::tcp_scan_port(&modal_state.selected_host.ip, *port) {
                Ok(dur) => store.lock().unwrap().dispatch(AppAction::SetModalAction(
                    HostModalAction::SetPortScanResult((*port, Some(Ok(dur)))),
                )),
                Err(_) => store.lock().unwrap().dispatch(AppAction::SetModalAction(
                    HostModalAction::SetPortScanResult((*port, Some(Err(())))),
                )),
            }
            thread::sleep(Duration::from_millis(10));
        }
    });
}
