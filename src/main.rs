mod ip_parse;
mod udp_pinger;
mod tcp_ping;
mod ping_result;

use ip_parse::ip_parse;
use udp_pinger::udp_ping;
use tcp_ping::tcp_ping;
use std::thread;
use std::sync::{Arc, Mutex};
use std::net::UdpSocket;

use colored::*;

use std::env;

fn main() {
    let input = env::args().nth(1).expect("Please provide an input");
    let hosts = ip_parse(input).unwrap_or_default();
    let mut threads = vec![];
    let lock = Arc::new(Mutex::new(0));

    let sock = UdpSocket::bind("0.0.0.0:32524").expect("Couldn't create sock");
    for host in hosts {
        // TOOD need to lock the udp socket
        let sock_clone = sock.try_clone().expect("Couldnt clone udp socket");
        let t = thread::spawn(move || {
            let active = udp_ping(sock_clone, host).is_ok();
            // let tcpping = tcp_ping(host).is_ok();
            // let string = format!("CIDR host: {} UDP: {}, TCP: {}", host, active, tcpping);
            let string = format!("CIDR host: {} UDP: {}", host, active);
            let mut col_string = string.white();
            // if active || tcpping {
            if active {
                col_string = string.green();
            }
            println!("{}", col_string);
        });
        threads.push(t);
    }

    println!("Waiting for threads to resolve...");

    for t in threads {
        let _ = t.join();
    }
}
