
use collections::string::FromUtf8Error;
use std::num::ParseIntError;
use std::io::Error;
use std::io::prelude::*;
use std::net::{TcpStream, SocketAddr};
use serialize::json;
use serialize::json::{Json, ParserError};
use std::sync::mpsc::Sender;
use std::fmt;

pub struct Session<'a> {
    pub stream: &'a mut TcpStream,
    pub channel: &'a mut Sender<String>
}

impl <'a> Session<'a> {
    pub fn start(&mut self) {
        let ip = match self.stream.peer_addr().unwrap() {
            SocketAddr::V4(addr) => addr.ip().to_string(),
            _ => panic!("[session] Unable to resolve ip address for incoming tcp connection.")
        };
        loop {
            match self.handle_next_msg() {
                Ok(_) => continue,
                Err(e) => {
                    println!("[session][{}] Processing msg didnt work: {:?}", ip, e);
                    break
                }
            }
        }
    }

    fn handle_next_msg(&mut self) -> Result<(), ClientErr> {
        let msg = try!(self.next_msg());
        match msg {
            Message::Json(json) => {
                match self.channel.send(json.to_string()) {
                    Ok(_) => Ok(()),
                    _ => Err(ClientErr::Unknown("Failed to send message to monitor thread.".to_string()))
                }
            },
            _ => Err(ClientErr::Unknown("Invalid msg format given to 'handle_next_msg'".to_string()))
        }
    }

    fn next_msg(&mut self) -> Result<Message, ClientErr> {
        let msg = try!(consume(self.stream, 4).and_then(parse_buffer).and_then(to_usize));
        match msg {
            Message::Length(len) => consume(self.stream, len).and_then(parse_buffer).and_then(to_json),
            _ => Err(ClientErr::Unknown("Invalid msg format given to 'next_msg'".to_string()))
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
            ClientErr::ParseErr(ref e) => format!("{:?}", e),
            ClientErr::IoErr(ref e) => format!("{:?}", e),
            ClientErr::ConvertErr(ref e) => format!("{:?}", e),
            ClientErr::JsonErr(ref e) => format!("{:?}", e),
            ClientErr::Unknown(ref s) => s.to_string()
        };
        fmt.debug_struct("ClientErr").field("err", &msg).finish()
    }
}
