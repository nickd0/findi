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

use pnet::ipnetwork::IpNetwork;
use pnet::datalink;
use clap::{App, Arg, ArgMatches};

static GLOBAL_RUN: AtomicBool = AtomicBool::new(true);

#[allow(dead_code)]
fn start_ui(store: Arc<Mutex<AppStateStore>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let _ = ui_loop(store);
    })
}

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("findi")
        .version("0.1.0")
        .author("Nick Donald <nickjdonald@gmail.com>")
        .about("A local network discovery tool")

        .arg(Arg::with_name("disable_ui")
            .short("n")
            .long("no-ui")
            .help("Disable the TUI app"))

        .arg(Arg::with_name("custom_cidr")
            .short("c")
            .long("cidr")
            .help("Network host query in CIDR notation")
            .takes_value(true))

        .get_matches()
}

fn main() {

    let matches = parse_args();

    let interfaces = datalink::interfaces();
    let default_iface = interfaces
        .iter()
        .find(|e| {
            e.is_up() && !e.is_loopback() && !e.ips.is_empty() && e.ips.iter().any(|&ip| ip.is_ipv4())
        });

    let mut store = AppStateStore::new();

    let hosts: Vec<Ipv4Addr>;
    let query: String;

    if let Some(input) = matches.value_of("custom_cidr") {
        match input_parse(&input) {
            Ok(hs) => hosts = hs,
            Err(msg) => return println!("{}", msg)
        }

        query = input.to_owned();

    // } else if default_iface.is_some() {
    } else if let Some(default_if_some) = default_iface {
        if let IpNetwork::V4(ipn) = default_if_some.ips[0] {
            // TODO: how to handle multiple ips on one interface?
            hosts = ipn.iter().collect();
            query = ipn.to_string();
        } else {
            eprintln!("Currently only interfaces with an IPv4 address can be used. Current interface: {:?}", default_iface);
            exit(1);
        }

    } else {
        eprintln!("No input provided and could not find an available interface!");
        exit(1);
    }

    store.dispatch(AppAction::BuildHosts(hosts));
    store.dispatch(AppAction::SetQuery(query));
    store.dispatch(AppAction::SetHostSearchRun(true));

    let shared_store = Arc::new(Mutex::new(store));

    #[cfg(feature = "ui")]
    let ui_thread = start_ui(shared_store.clone());

    init_host_search(shared_store);

    #[cfg(feature = "ui")]
    let _ = ui_thread.join();
}
