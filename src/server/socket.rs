
use std::net::{TcpListener, TcpStream, Ipv4Addr};
use std::thread;
use server::client::Session;
use std::sync::mpsc::{channel, Sender, Receiver};

pub struct Listener {
    listener: TcpListener
}

impl Listener {
    pub fn new(ip: Ipv4Addr, port: usize) -> Listener {
        Listener {
            listener: match TcpListener::bind(format!("{}:{}", ip, port).as_slice()) {
                Ok(listener) => listener,
                Err(e) => panic!("{}", e.to_string())
            }
        }
    }

    pub fn listen(&self) {
        let (tx, rx) = channel::<String>();
        self.spawn_monitor(rx);
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => self.spawn_client_session(stream, tx.clone()),
                _ => { println!("[monitor] Unexpectedly failed to establish incoming connection." ); }
            }
        }
    }

    fn spawn_monitor(&self, channel: Receiver<String>) {
        thread::spawn(move|| {
            loop {
                match channel.recv() {
                    Ok(msg) => println!("[monitor] Received message: {:?}", msg),
                    _ => println!("[monitor] Unexpectedly failed to receive incoming message.")
                }
            }
        });
    }

    fn spawn_client_session(&self, stream: TcpStream, channel: Sender<String>) {
        thread::spawn(move|| {
            let mut stream = stream;
            let mut channel = channel;
            let mut session = Session { stream: &mut stream, channel: &mut channel };
            session.start();
        });
    }
}
