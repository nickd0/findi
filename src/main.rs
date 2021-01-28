mod ip_parse;
mod udp_pinger;
mod dns;
mod tcp_ping;

use ip_parse::ip_parse;
use udp_pinger::udp_ping;
use tcp_ping::tcp_ping;

use std::env;

fn main() {
    let input = env::args().nth(1).expect("Please provide an input");
    let hosts = ip_parse(input).unwrap_or_default();
    for host in hosts {
        let active = udp_ping(host);
        let tcpping = tcp_ping(host).is_ok();
        // let tcpping = false;
        println!("CIDR host: {} UDP: {}, TCP: {}", host, active, tcpping);
        // let (name, service) = match dns_namelookup(host, 22) {
        //     Ok((n, s)) => (n, s),
        //     Err(_) => (String::from("--"), String::from("--"))
        // };
        // print!(" name: {}, service: {} ... ", name, service);
    }
}
