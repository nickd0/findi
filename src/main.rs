mod ip_parse;
mod udp_pinger;
mod dns;
mod tcp_ping;
mod ping_result;

use ip_parse::ip_parse;
use udp_pinger::udp_ping;
use tcp_ping::tcp_ping;
use std::io::ErrorKind;

use colored::*;

use std::env;

fn main() {
    let input = env::args().nth(1).expect("Please provide an input");
    let hosts = ip_parse(input).unwrap_or_default();
    for host in hosts {
        let active = udp_ping(host).is_ok();
        let tcpping = match tcp_ping(host) {
            Ok(_) => true,
            Err(e) => {
                println!("TCP error: {:?}", e.kind());
                false
            }
        };
        let string = format!("CIDR host: {} UDP: {}, TCP: {}", host, active, tcpping);
        let mut col_string = string.white();
        if active || tcpping {
            col_string = string.green();
        }
        println!("{}", col_string);
    }
}
