
use std::net::TcpListener;
use std::thread;
use server::client::Session;

pub struct Listener<'l> {
    pub listener: &'l mut TcpListener
}

impl <'l> Listener<'l> {
    pub fn listen(&self) {
        // accept connections and process them, spawning a new thread for each one
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move|| {
                        let mut stream = stream;
                        let mut session = Session { stream: &mut stream };
                        session.start();
                    });
                }
                _ => { println!("Connection failed ..." ); }
            }
        }
    }
}
