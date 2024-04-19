use std::{collections::{HashMap, VecDeque}, time::Instant};

use mio::{net::{TcpListener, TcpStream, UdpSocket}, Poll, Token};

use crate::log::Log;
mod network_event;
mod dns_manager;
mod gate;
mod line;

#[derive(Debug)]
pub enum LineType {
    Pc,
    Hk,
    MainLand,
    World,
}

#[derive(Debug)]
pub enum Status {
    Raw,
    Established,
    FirstPackDone,
    WaitingDnsResult,
    SecondPackDone,
    EncryptDone,
    ReadClose,
    WriteClose,
    Close,
    Dead,
}

pub struct DnsManager {
    socket:UdpSocket,
    dns_result:Vec<(usize,Option<String>)>,
    logger:Log,
}

pub struct Line {
    id:usize,
    line_type:LineType,
    pair_id:usize,
    status:Status,
    birth:Instant,
    stream:TcpStream,
    website_host:String,
    website_port:u16,
    net_data:VecDeque<u8>,//data from network
    mem_data:Vec<u8>,//data from pair
    garbage:Vec<u8>,//data should trans to pair (clone and clear) next frame
    logger:Log,
}

pub struct Gate {
    next_id:usize,
    lines:HashMap<Token,Line>,
    p:Poll,
    dm:Option<DnsManager>,
    listener:TcpListener,
    logger:Log,
}


fn u8r(input:u8) -> u8 {
    if input > 45 && input < 255 - 45 {
        255 - input
    } else {
        input
    }
    
}