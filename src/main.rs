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
mod config;

use ui::ui_loop;
use network::input_parse;
use state::store::AppStateStore;
use state::actions::AppAction;
use network::init_host_search;

use std::thread;
use std::process::exit;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool};
use std::env;

use pnet::ipnetwork::IpNetwork;
use pnet::datalink;

static GLOBAL_RUN: AtomicBool = AtomicBool::new(true);

#[allow(dead_code)]
fn start_ui(store: Arc<Mutex<AppStateStore>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let _ = ui_loop(store);
    })
}

fn main() {
    let interfaces = datalink::interfaces();
    let default_iface = interfaces
        .iter()
        .find(|e| {
            e.is_up() && !e.is_loopback() && !e.ips.is_empty()
        });

    let mut store = AppStateStore::new();

    let hosts: Vec<Ipv4Addr>;
    let query: String;

    if let Some(input) = env::args().nth(1) {
        match input_parse(&input) {
            Ok(hs) => hosts = hs,
            Err(msg) => return println!("{}", msg)
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
    store.dispatch(AppAction::SetHostSearchRun(true));

    let shared_store = Arc::new(Mutex::new(store));

    #[cfg(feature = "ui")]
    let ui_thread = start_ui(shared_store.clone());

    init_host_search(shared_store.clone());

    #[cfg(feature = "ui")]
    let _ = ui_thread.join();
}
