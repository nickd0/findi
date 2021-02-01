mod ip_parse;
mod network;

use ip_parse::ip_parse;

use network::udp_ping::udp_ping;
use network::tcp_ping::tcp_ping;

use std::thread;

use colored::*;

use std::env;

fn main() {
    let input = env::args().nth(1).expect("Please provide an input");
    let hosts = ip_parse(input).unwrap_or_default();
    let mut threads = vec![];

    for host in hosts {
        let t = thread::spawn(move || {
            let udp_live = udp_ping(host).is_ok();
            let tcp_live = tcp_ping(host).is_ok();

            let string = format!("CIDR host: {} UDP: {} TCP: {}", host, udp_live, tcp_live);
            let mut col_string = string.white();
            if udp_live || tcp_live {
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
