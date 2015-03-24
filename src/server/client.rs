
use std::{io};
use collections::string::FromUtf8Error;
use std::io::prelude::*;
use std::net::TcpStream;
use serialize::json;
use std::fmt;

pub struct Session<'a> {
    pub stream: &'a mut TcpStream
}

impl <'a> Session<'a> {
    pub fn start(&mut self) {
        match self.read_msg() {
            Ok(msg) => {
                match json::from_str(msg.trim()) {
                    Ok(json) => { self.print_msg(json); },
                    Err(e) => { println!("Parsing msg did not work: {:?}", e); }
                }
            },
            Err(e) => { println!("Reading msg didnt work: {:?}", e); }
        }
    }

    fn read_msg(&mut self) -> Result<String, ClientErr> {
        let mut buffer = vec![];

        match self.stream.read_to_end(&mut buffer) {
            Ok(_) => {
                return match String::from_utf8(buffer)  {
                    Ok(s) => Ok(s),
                    Err(e) => Err(ClientErr::ParseErr(e))
                }
            },
            Err(e) => Err(ClientErr::IoErr(e))
        }
    }

    fn print_msg(&mut self, msg: json::Json) {
        let peer_addr = self.stream.peer_addr().unwrap();

        println!("Sender: {}\n> {}\n", peer_addr, msg);
    }
}

enum ClientErr {
    ParseErr(FromUtf8Error),
    IoErr(io::Error)
}

impl fmt::Debug for ClientErr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            ClientErr::ParseErr(ref e) => e.to_string(),
            ClientErr::IoErr(ref e) => e.to_string()
        };
        fmt.debug_struct("ClientErr")
            .field("err", &msg)
            .finish()
    }
}
