/*
Notes:
- Split DNS code into separate files
  - dns/packet.rs
    - question, answers, packet
 - dns/mdns.rs
- Should host lookup be in a different thread?
  - Or somehow ping and host lookup should use the same UdpSocket?
- Standardize error and result types
*/

mod network;
mod ui;
mod state;

use ui::ui_loop;
use network::host::{Host};
use state::store::AppStateStore;
use state::actions::AppAction;

use std::thread;
use std::process::exit;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::env;

use pnet::ipnetwork::IpNetwork;
use pnet::datalink;
use threadpool::ThreadPool;

const TP_WORKERS: usize = 100;

#[allow(dead_code)]
fn start_ui(store: Arc<Mutex<AppStateStore>>, run: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let _ = ui_loop(store, run);
    })
}

fn main() {
    let interfaces = datalink::interfaces();
    let default_iface = interfaces
        .iter()
        .find(|e| {
            e.is_up() && !e.is_loopback()
                && !e.ips.is_empty()
                && e.ips[0].is_ipv4()
        });

    let mut store = AppStateStore::new();

    let hosts: Vec<Ipv4Addr>;
    let query: String;

    if let Some(input) = env::args().nth(1) {
        // hosts = ip_parse(&input).unwrap_or_default();
        if let Ok(IpNetwork::V4(ipn)) = input.parse::<IpNetwork>() {
            hosts = ipn.iter().collect()
        } else {
            return println!("Please provide a valid IPv4 CIDR network")
        }
        query = input;

    } else if default_iface.is_some() {
        if let IpNetwork::V4(ipn) = default_iface.unwrap().ips[0] {
            // TODO: how to handle multiple ips on one interface?
            hosts = ipn.iter().collect();
            query = ipn.to_string();
        } else {
            return println!("Currently only interfaces with an IPv4 address can be used")
        }

    } else {
        println!("No input provided and could not find an available interface!");
        exit(1);
    }

    store.dispatch(AppAction::BuildHosts(hosts.clone()));
    store.dispatch(AppAction::SetQuery(query));

    let shared_store = Arc::new(Mutex::new(store));
    // Should this be part of the central state store?
    // We gain efficiencies by using an AtomicBool vs
    // just a bool in a Mutex
    let run = Arc::new(AtomicBool::new(true));

    #[cfg(feature = "ui")]
    let ui_thread = start_ui(shared_store.clone(), run.clone());

    let pool = ThreadPool::new(TP_WORKERS);
    for host in hosts {
        if !run.load(Ordering::Acquire) {
            break;
        }

        let store_copy = shared_store.clone();
        thread::sleep(Duration::from_millis(50));
        pool.execute(move || {
            let h = Host::host_ping(host);
            let mut store_lock = store_copy.lock().unwrap();
            store_lock.dispatch(AppAction::UpdateHost(h));
        });
    }

    #[cfg(feature = "ui")]
    let _ = ui_thread.join();
}
