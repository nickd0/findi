mod ip_parse;
mod network;
mod ui;

use ip_parse::ip_parse;
use ui::ui_loop;

use network::host::Host;

use std::thread;

use std::env;

fn main() {
    let input = env::args().nth(1).expect("Please provide an input");
    let hosts = ip_parse(input).unwrap_or_default();
    let mut threads = vec![];

    let p_hosts = hosts.clone();

    let ui_thread = thread::spawn(move || {
        let _ = ui_loop(p_hosts);
    });
    // TODO: share the same hosts vec between threads

    for host in hosts {
        let t = thread::spawn(move || {
            // TODO: should this be mut or just receive the ping result value?
            let mut p_host = Host::new(host);
            p_host.ping();

            // if p_host.ping_res.unwrap().is_ok() {
            //     let col_string = &format!("Host {} alive, found by {}", host, p_host.ping_type.unwrap());
            //     println!("{}", col_string.green());
            // } else {
            //     println!("Host {} is down", host)
            // }
        });
        threads.push(t);
    }

    for t in threads {
        let _ = t.join();
    }
    let _ = ui_thread.join();
}

// fn main() {
//     ui_loop();
// }
