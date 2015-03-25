
use std::io::{Error};
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
        loop {
            match self.read_msg() {
                Ok(opt) => match opt {
                    Some(msg) => {
                        match json::from_str(msg.trim()) {
                            Ok(json) => { self.print_msg(json); },
                            Err(e) => { println!("Parsing msg did not work: {:?}", e); }
                        }
                    },
                    None => break
                },
                Err(e) => { println!("Reading msg didnt work: {:?}", e); }
            }
        }
    }

    fn read_msg(&mut self) -> Result<Option<String>, ClientErr> {
        let mut buffer = &mut vec![4u8; 4];
        let msg_size: usize = match self.stream.read(buffer) {
            Ok(_) => match String::from_utf8(buffer.clone()) {
                Ok(msg) => match msg.parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => 0
                },
                Err(_) => 0
            },
            Err(_) => 0
        };
        /*
            The above code isn't all to elegant ^^ and chaining would be nicer.
            The below wont work, because the value type held by the results is not the same throughout the chain.

        let msg_size: usize = match self.stream.read(buffer)
            .and_then(|| { String::from_utf8(buffer.clone()) })
            .and_then(|msg| { msg.parse::<usize>() })
        {
            Ok(i) => i,
            Err(_) => 0
        };
        */
        if msg_size == 0 {
            return Ok(None);
        }

        let mut buffer = &mut vec![4u8; msg_size];
        match self.stream.read(buffer) {
            Ok(_) => match String::from_utf8(buffer.clone()) {
                Ok(msg) => Ok(Some(msg)),
                Err(e) => Err(ClientErr::ParseErr(e))
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
    IoErr(Error)
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
