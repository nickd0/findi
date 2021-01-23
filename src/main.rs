mod ip_parse;
mod udp_pinger;
mod dns;

use ip_parse::ip_parse;
use udp_pinger::udp_ping;
use dns::dns_namelookup;

use std::env;

fn main() {
    let input = env::args().nth(1).expect("Please provide an input");
    let hosts = ip_parse(input).unwrap_or_default();
    for host in hosts {
        let active = udp_ping(host);
        print!("CIDR host: {} ... ", host);
        let (name, service) = match dns_namelookup(host, 22) {
            Ok((n, s)) => (n, s),
            Err(_) => (String::from("--"), String::from("--"))
        };
        print!(" name: {}, service: {} ... ", name, service);
        println!("{}", active);
    }
}
