
extern crate hbmon;

use std::net::TcpListener;
use hbmon::server::socket;

fn main() {
    println!("Starting server on port 8888\n");

    socket::Listener {
        listener: &mut TcpListener::bind("127.0.0.1:8888").unwrap()
    }.listen();
}
