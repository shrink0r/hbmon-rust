
extern crate hbmon;

use std::net::Ipv4Addr;
use hbmon::server::socket;

fn main() {
    let port = 8888;
    let ip = Ipv4Addr::new(127, 0, 0, 1);

    println!("Binding port {} to ip {} ...\n", port, ip);

    socket::Listener::new(ip, port).listen();
}
