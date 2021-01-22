mod ip_parse;
mod udp_pinger;

use ip_parse::ip_parse;
use udp_pinger::udp_ping;

use std::env;

fn main() {
    let input = env::args().nth(1).expect("Please provide an input");
    let hosts = ip_parse(input).unwrap_or_default();
    for host in hosts {
        let active = udp_ping(host);
        print!("CIDR host: {} ... ", host);
        println!("{}", active);
    }
}
