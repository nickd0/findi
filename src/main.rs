mod ip_parse;
mod network;

use ip_parse::ip_parse;

use network::host::Host;

use std::thread;

use colored::*;

use std::env;

fn main() {
    let input = env::args().nth(1).expect("Please provide an input");
    let hosts = ip_parse(input).unwrap_or_default();
    let mut threads = vec![];

    for host in hosts {
        let t = thread::spawn(move || {
            // TODO: should this be mut or just receive the ping result value?
            let mut p_host = Host::new(host);
            p_host.ping();

            if p_host.ping_res.unwrap().is_ok() {
                let col_string = &format!("Host {} alive, found by {}", host, p_host.ping_type.unwrap());
                println!("{}", col_string.green());
            } else {
                println!("Host {} is down", host)
            }
        });
        threads.push(t);
    }

    println!("Waiting for threads to resolve...");

    for t in threads {
        let _ = t.join();
    }
}
