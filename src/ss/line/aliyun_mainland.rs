use crate::{log, ss::{network_event::NetWorkEvent, u8r, Line, Status}};

impl Line {
    pub fn aliyun_consume_mainland_data(&mut self) {
        let na = self.net_data.len();
        self.log(format!("aliyun_consume_mainland_data {} {:?}",na,self.status));
        match self.status {
            Status::Raw => self.unexpect(),
            Status::Established => self.website_addr(),
            Status::FirstPackDone => self.unexpect(),
            Status::WaitingDnsResult => {},
            Status::SecondPackDone => self.client_hello(),
            Status::EncryptDone  => self.move_network_data_to_garbage(),
            Status::ReadClose | Status::WriteClose | Status::Close | Status::Dead => todo!(),
        }
    }

    pub fn waiting_for_dns_result(&mut self) {
        self.set_status(Status::WaitingDnsResult);
    }

    pub fn dns_query_fail(&mut self) {
        self.log(format!("dns_query_fail {:?}",self.website_host));
    }

    pub fn dns_query_success(&mut self,id:usize) {
        self.log(format!("dns_query_success {:?}",self.website_host));
        self.set_pair_id(id);
        self.set_status(Status::SecondPackDone);
        if self.net_data.len() > 0 {
            self.client_hello();
        }
    }

    pub fn say_hello_to_mainland(&mut self) {
        let id = self.id as u16;
        self.write(&id.to_be_bytes());
    }
}

impl Line {
    fn website_addr(&mut self) {
        self.update_host_name();
        self.set_status(Status::FirstPackDone);
        let m = format!("[{}]{:?}:{}",self.id,self.website_host,self.website_port);
        log::im(m);
    }

    fn client_hello(&mut self) {
        self.garbage.clear();
        let n = self.net_data.len();
        for _ in 0..n {
            let input = self.net_data.pop_front().unwrap();
            self.garbage.push(u8r(input));
        }
        self.set_status(Status::EncryptDone);
        self.log(format!("client_hello {}",self.garbage.len()));
    }

    fn update_host_name(&mut self) {
        let mut host = Vec::new();
        let mut p = Vec::new();

        let atyp = self.net_data.pop_front().unwrap();
        let len = u8r(self.net_data.pop_front().unwrap()) as usize;
        assert_eq!(atyp,3);
        
        for v in self.net_data.iter() {
            let v = u8r(*v);
            if host.len() < len {
                host.push(v)
            } else {
                p.push(v)
            }
        }

        self.clear_network_data();
        
        match String::from_utf8(host) {
            Ok(ret) => self.website_host = ret,
            Err(e) => log::im(format!("[{}]{:?}",e,self.id)),
        }
        
        self.website_port = u16::from_be_bytes([p[0],p[1]]);
        
    }
}