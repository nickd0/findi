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
mod services;
mod ui;
mod state;
mod config;

use ui::{
    pages::Page,
    ui_loop,
};
use network::input_parse;
use state::store::AppStateStore;
use state::actions::AppAction;
use config::AppConfig;

use std::process::exit;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool};

use pnet::{
    datalink,
    ipnetwork::IpNetwork,
};
use clap::{App, Arg, ArgMatches, crate_version, crate_authors};
use colored::Colorize;

static GLOBAL_RUN: AtomicBool = AtomicBool::new(true);

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("findi")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A local network discovery tool")
        .subcommand(
            App::new("host-scan")
                .about("Scans local network for live hosts.")
                .arg(Arg::with_name("custom_cidr")
                    .short("c")
                    .long("cidr")
                    .help("Network host query in CIDR notation")
                    .takes_value(true))

                .arg(Arg::with_name("interface")
                    .short("i")
                    .long("interface")
                    .help("Network interface for query")
                    .takes_value(true))

                .arg(Arg::with_name("scan_ports")
                    .short("p")
                    .long("tcpports")
                    .help("TCP port scan list/range (e.g. -p 22 or -p 22,443 or -p 80-90)")
                    .takes_value(true))

                .arg(Arg::with_name("output_file")
                    .short("o")
                    .long("output")
                    .help("Output file location with extension (csv|json|txt)")
                    .takes_value(true))
        )

        .subcommand(
            App::new("service-scan")
                .about("Scans local network for Bonjor/ZeroConf services.")
        )

        .arg(Arg::with_name("tick_len")
            .short("t")
            .long("ticklen")
            .help("UI timer tick length in ms. If no key events, UI redraws at this interval.")
            .takes_value(true))

        .arg(Arg::with_name("nworkers")
            .short("w")
            .long("numworkers")
            .help("Number of workers for network scanning.")
            .takes_value(true))

        .arg(Arg::with_name("service_scan")
            .short("s")
            .long("servicescan")
            .help("Scan for local network services with mDNS.")
            .takes_value(true))

        .arg(Arg::with_name("disable_ui")
            .short("n")
            .long("no-ui")
            .help("Disable the TUI app"))

        .get_matches()
}

fn main() {

    let matches = parse_args();

    let interfaces = datalink::interfaces();

    // Find a suitable interface and match by name if provided
    let default_iface = interfaces
        .iter()
        .find(|e| {
            e.is_up() &&
            !e.is_loopback() &&
            !e.ips.is_empty() &&
            e.ips.iter().any(|&ip| ip.is_ipv4()) &&
            matches.value_of("interface").unwrap_or(&e.name) == e.name
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

    } else if let Some(default_if_some) = default_iface {
        if let Some(IpNetwork::V4(ipn)) = default_if_some.ips.iter().find(|ip| matches!(ip, IpNetwork::V4(_))) {
            // TODO: how to handle multiple ips on one interface?
            hosts = ipn.iter().collect();
            query = ipn.to_string();
        } else {
            eprintln!("Currently only interfaces with an IPv4 address can be used. Current interface: {:?}", default_iface);
            exit(1);
        }

    } else {
        if let Some(input_if)  = matches.value_of("interface") {
            eprintln!("The interface {} was not suitable. It may be down, loopback, or not have an IPv4 address.", input_if)
        } else {
            eprintln!("No input provided and could not find a suitable interface!")
        }
        exit(1);
    }

    // Get port list from args, ignore a malformed port list
    // TODO: constrain this to at most 10 ports?
    if let Some(port_list) = matches.value_of("scan_ports") {
        store.dispatch(AppAction::SetPortQuery(Some(port_list.to_owned())))
    }

    // Setup user config
    let mut config = AppConfig::default();

    if let Some(nworkers) = matches.value_of("nworkers").and_then(|nw| nw.parse().ok()) {
        config.nworkers = nworkers;
    }

    if let Some(tick_len) = matches.value_of("tick_len").and_then(|tl| tl.parse().ok()) {
        config.tick_len = tick_len;
    }

    store.dispatch(AppAction::SetConfig(config));

    let num_hosts = hosts.len();

    store.dispatch(AppAction::BuildHosts(hosts));
    store.dispatch(AppAction::SetQuery(query));
    store.dispatch(AppAction::SetHostSearchRun(true));

    let shared_store = Arc::new(Mutex::new(store));

    let run_page = match matches.subcommand() {
        ("host-scan", _) => Page::MainPage,
        ("service-scan", _) => Page::ServiceScanPage,
        _ => unreachable!(),
    };

    #[cfg(feature = "ui")]
    if !matches.is_present("disable_ui") {
        // Run UI on main thread
        let _ = ui_loop(shared_store, run_page);
    } else {
        // TODO: move this elsewhere and accept an argument for different types out output
        // ie stdout, csv, json, etc

        let lstore = shared_store;
        let mut hostidx: usize = 0;

        println!("Scanning {} hosts...", num_hosts);

        loop {
            let hstore = lstore.lock().unwrap();
            if hostidx >= hstore.state.hosts.len() {
                break
            }

            let host = &hstore.state.hosts[hostidx];
            if host.ping_done {
                if host.ping_res.is_some() || host.host_name.as_ref().unwrap().is_ok() {
                    let dur_str = match host.ping_res {
                        Some(d) => format!("{:.2?}ms", d.as_millis()),
                        None => "--".to_owned()
                    };
                    println!(
                        "Live host {} {}{}",

                        format!(
                            "{:<28}",

                            format!(
                                "{:<15?} {}",
                                host.ip, dur_str
                            )
                        ),

                        format!("{:<30}",
                            match &host.host_name {
                                Some(Ok(hostname)) => hostname.green(),
                                _ => "--".red()
                            }
                        ),

                        match hstore.state.port_query.len() {
                            0 => String::default(),
                            _ => format!(" TCP ports: {}", host.tcp_ports.iter().map(|&p| p.to_string()).collect::<Vec<String>>().join(","))
                        }
                    )
                }
                hostidx += 1;
            }
        }
    }
}
