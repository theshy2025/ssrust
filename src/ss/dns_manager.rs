use simple_dns::*;

use mio::{net::UdpSocket, Interest, Poll};

use crate::{config::DNS, default_config::SERVER_IP, log::Log};

use super::{network_event::NetWorkEvent, DnsManager};


impl DnsManager {
    pub fn new(p:&Poll) -> DnsManager {
        let addr = SERVER_IP.parse().unwrap();
        let mut socket = UdpSocket::bind("0.0.0.0:0".parse().unwrap()).unwrap();
        socket.connect(addr).unwrap();
        p.registry().register(&mut socket, DNS, Interest::READABLE).unwrap();
        let logger = Log::create("dns");
        DnsManager { socket , dns_result:Vec::new() , logger }
    }
}

impl DnsManager {
    fn log(&mut self,s:String) {
        self.logger.add(s);
    }

    pub fn peek(&self) -> usize {
        let num = self.dns_result.len();
        num
    }

    pub fn new_dns_query(&mut self,id:usize,host:String) {
        self.log(format!("new_dns_query [{}] {:?}",id,host));
        let packet = self.build(id.try_into().unwrap(),host);
        self.write(&packet);
    }

    pub fn move_out_dns_result(&mut self) -> Vec<(usize,Option<String>)> {
        self.log(format!("move_out_dns_result {:?}",self.dns_result));
        let ret = self.dns_result.clone();
        self.clear_dns_result();
        ret
    }

    pub fn clear_dns_result(&mut self) {
        self.log(format!("clear_dns_result {:?}",self.dns_result));
        self.dns_result.clear();
    }

    fn decode(&mut self,data:&[u8]) -> (usize,Option<String>) {
        match Packet::parse(data) {
            Ok(packet) => {
                let id = packet.id() as usize;
                match packet.rcode() {
                    RCODE::NoError => {
                        let ip = get_ip(packet.answers);
                        (id,ip)
                    },
                    other => {
                        self.log(format!("dns server reply with error code {:?}",other));
                        (id,None)
                    },
                }
            },
            Err(e) => {
                self.log(format!("packet parse fail {},{}",e,data.len()));
                (0,None)
            },
        }
    }

    fn build(&self,id:u16,host:String) -> Vec<u8> {
        let mut packet = Packet::new_query(id);
        packet.set_flags(PacketFlag::RECURSION_DESIRED);
        let qname = Name::new(&host).unwrap();
        let qtype = TYPE::A.into();
        let qclass = CLASS::IN.into();
        let question = Question::new(qname, qtype, qclass, false);
        packet.questions.push(question);
        packet.build_bytes_vec_compressed().unwrap()
    }

}

impl NetWorkEvent for DnsManager {
    fn fetch_error(&self) -> std::io::Result<Option<std::io::Error>> {
        todo!()
    }

    fn event_log(&mut self,s:String) {
        self.log(s);
    }

    fn recv(&mut self,buf:&mut [u8]) -> std::io::Result<usize> {
        self.socket.recv(buf)
    }

    fn on_recv(&mut self,buf:&[u8]) {
        let ret = self.decode(buf);
        self.dns_result.push(ret);
    }

    fn write_to_network(&mut self,buf:&[u8]) -> std::io::Result<usize> {
        self.socket.send(buf)
    }
    
    fn line_type(&self) -> String {
        SERVER_IP.to_string()
    }
    
    fn flush_log(&mut self) {
        self.logger.flush();
    }
    
}





fn get_ip(data:Vec<ResourceRecord>) -> Option<String> {
    for v in data {
        match v.rdata {
            rdata::RData::A(a) => {
                let b = a.address.to_be_bytes();
                let ret = format!("{}.{}.{}.{}",b[0],b[1],b[2],b[3]);
                return Some(ret)
            }
            _ => {}
        }
    }

    None
}
