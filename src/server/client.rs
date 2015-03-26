
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
            Ok(msg) => match msg {
                Message::Json(json) => {
                    let peer_addr = self.stream.peer_addr().unwrap();
                    // @todo deserialize the json to an event struct and send it to the monitor thread
                    println!("Sender: {}\n> {}\n", peer_addr, json);
                    Ok(())
                },
                _ => Err(ClientErr::Unknown("Invalid msg format given to 'handle_next_msg'".to_string()))
            }
        }
    }

    fn next_msg(&mut self) -> Result<Message, ClientErr> {
        match consume(self.stream, 4).and_then(parse_buffer).and_then(to_usize) {
            Ok(msg) => match msg {
                Message::Length(len) => consume(self.stream, len).and_then(parse_buffer).and_then(to_json),
                _ => Err(ClientErr::Unknown("Invalid msg format given to 'next_msg'".to_string()))
            },
            Err(e) => Err(e)
        }
    }
}

fn consume(stream: &mut TcpStream, byte_count: usize) -> Result<Message, ClientErr> {
    let mut buffer = vec![4u8; byte_count];
    match stream.read(&mut buffer) {
        Ok(_) => Ok(Message::Buffered(Box::new(buffer))),
        Err(e) => Err(ClientErr::IoErr(e))
    }
}

fn parse_buffer(msg: Message) -> Result<Message, ClientErr> {
    match msg {
        Message::Buffered(ref vec) => match String::from_utf8(*vec.clone()) {
            Ok(size) => Ok(Message::String(size)),
            Err(e) => Err(ClientErr::ParseErr(e))
        },
        _ => Err(ClientErr::Unknown("Unexpected msg format given to 'parse_buffer'.".to_string()))
    }
}

fn to_usize(msg: Message) -> Result<Message, ClientErr> {
    match msg {
        Message::String(s) => match s.parse::<usize>() {
            Ok(size) => Ok(Message::Length(size)),
            Err(e) => Err(ClientErr::ConvertErr(e))
        },
        _ => Err(ClientErr::Unknown("Unexpected msg format given to 'to_usize'.".to_string()))
    }
}

fn to_json(msg: Message) -> Result<Message, ClientErr> {
    match msg {
        Message::String(s) => match json::from_str(s.trim()) {
            Ok(json) => Ok(Message::Json(json)),
            Err(e) => Err(ClientErr::JsonErr(e))
        },
        _ => Err(ClientErr::Unknown("Unexpected msg format given to 'to_json'.".to_string()))
    }
}

#[derive(Debug)]
enum Message {
    Buffered(Box<Vec<u8>>),
    Length(usize),
    String(String),
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
        fmt.debug_struct("ClientErr").field("err", &msg).finish()
    }
}
