
use collections::string::FromUtf8Error;
use std::num::ParseIntError;
use std::io::Error;
use std::io::prelude::*;
use std::net::TcpStream;
use serialize::json;
use serialize::json::{Json, ParserError};
use std::fmt;

pub struct Session<'a> {
    pub stream: &'a mut TcpStream
}

impl <'a> Session<'a> {
    pub fn start(&mut self) {
        loop {
            match self.handle_next_msg() {
                Ok(_) => continue,
                Err(e) => {
                    println!("Processing msg didnt work: {:?}", e);
                    break
                }
            }
        }
    }

    fn handle_next_msg(&mut self) -> Result<(), ClientErr> {
        match self.next_msg() {
            Err(e) => Err(e),
            Ok(opt) => match opt {
                None => Ok(()),
                Some(json) => {
                    let peer_addr = self.stream.peer_addr().unwrap();
                    println!("Sender: {}\n> {}\n", peer_addr, json);
                    Ok(())
                }
            }
        }
    }

    fn next_msg(&mut self) -> Result<Option<Json>, ClientErr> {
        let msg_len: usize = match consume(self.stream, 4).and_then(parse_buffer).and_then(to_usize) {
            Ok(ref msg_size) => match *msg_size {
                MsgSize::Value(size) => size,
                _ => return Ok(None)
            },
            Err(e) => return Err(e)
        };

        match consume(self.stream, msg_len).and_then(parse_buffer).and_then(to_json) {
            Ok(ref msg_size) => match *msg_size {
                MsgSize::Json(ref json) => Ok(Some(json.clone())),
                _ => return Ok(None)
            },
            Err(e) => Err(e)
        }
    }
}

fn consume(stream: &mut TcpStream, byte_count: usize) -> Result<MsgSize, ClientErr> {
    let mut buffer = vec![4u8; byte_count];
    match stream.read(&mut buffer) {
        Ok(_) => {
            Ok(MsgSize::Buffer(Box::new(buffer)))
        },
        Err(e) => Err(ClientErr::IoErr(e))
    }
}

fn parse_buffer(msg_size: MsgSize) -> Result<MsgSize, ClientErr> {
    match msg_size {
        MsgSize::Buffer(ref vec) => match String::from_utf8(*vec.clone()) {
            Ok(size) => Ok(MsgSize::String(size)),
            Err(e) => Err(ClientErr::ParseErr(e))
        },
        _ => Err(ClientErr::Unknown("Unexpected msg_size format.".to_string()))
    }
}

fn to_usize(msg_size: MsgSize) -> Result<MsgSize, ClientErr> {
    match msg_size {
        MsgSize::String(s) => match s.parse::<usize>() {
            Ok(size) => Ok(MsgSize::Value(size)),
            Err(e) => Err(ClientErr::ConvertErr(e))
        },
        _ => Err(ClientErr::Unknown("Unexpected msg_size format.".to_string()))
    }
}

fn to_json(msg_size: MsgSize) -> Result<MsgSize, ClientErr> {
    match msg_size {
        MsgSize::String(s) => match json::from_str(s.trim()) {
            Ok(json) => Ok(MsgSize::Json(json)),
            Err(e) => Err(ClientErr::JsonErr(e))
        },
        _ => Err(ClientErr::Unknown("Unexpected msg_size format.".to_string()))
    }
}

#[derive(Debug)]
enum MsgSize {
    Buffer(Box<Vec<u8>>),
    String(String),
    Value(usize),
    Json(Json)
}

enum ClientErr {
    Unknown(String),
    ParseErr(FromUtf8Error),
    ConvertErr(ParseIntError),
    IoErr(Error),
    JsonErr(ParserError)
}

impl fmt::Debug for ClientErr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            ClientErr::ParseErr(ref e) => e.to_string(),
            ClientErr::IoErr(ref e) => e.to_string(),
            ClientErr::ConvertErr(ref e) => e.to_string(),
            ClientErr::JsonErr(ref e) => e.to_string(),
            ClientErr::Unknown(ref s) => s.to_string()
        };
        fmt.debug_struct("ClientErr")
            .field("err", &msg)
            .finish()
    }
}
