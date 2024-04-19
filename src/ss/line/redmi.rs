use crate::{log, ss::{network_event::NetWorkEvent, u8r, Line, Status}};

impl Line {
    pub fn redmi_consume_hk_data(&mut self) {
        let na = self.net_data.len();
        self.log(format!("redmi_consume_hk_data {} {:?}",na,self.status));
        match self.status {
            Status::Raw => self.unexpect(),
            Status::Established => self.aliyun_hello(),
            Status::Close | Status::Dead => todo!(),
            _ => self.move_network_data_to_garbage(),
        }
    }

    fn aliyun_hello(&mut self) {
        assert_eq!(self.net_data.len(),2);
        let id = u16::from_be_bytes([self.net_data[0],self.net_data[1]]);
        self.log(format!("aliyun_hello {}",id));
        self.clear_network_data();
        self.set_status(Status::FirstPackDone);
    }
}

impl Line {
    pub fn redmi_consume_pc_data(&mut self) {
        let na = self.net_data.len();
        self.log(format!("redmi_consume_pc_data {} {:?}",na,self.status));
        match self.status {
            Status::Raw => self.unexpect(),
            Status::Established => self.s5_hello(),
            Status::FirstPackDone => self.s5_connect_cmd(),
            Status::SecondPackDone => self.client_hello_from_pc(),
            Status::EncryptDone => self.move_network_data_to_garbage(),
            Status::ReadClose | Status::WriteClose | Status::Close | Status::Dead => todo!(),
            Status::WaitingDnsResult => todo!(),
        }
    }
    
    fn s5_hello(&mut self) {
        assert_eq!(self.net_data.pop_front(), Some(5));
        self.clear_network_data();
        self.write(&[5,0]);
        self.set_status(Status::FirstPackDone);
    }

    fn s5_connect_cmd(&mut self) {
        assert_eq!(self.net_data.pop_front(), Some(5));
        assert_eq!(self.net_data.pop_front(), Some(1));
        assert_eq!(self.net_data.pop_front(), Some(0));
        
        let mut tmp = self.net_data.clone();
        let atyp = tmp.pop_front().unwrap();
        let len = tmp.pop_front().unwrap();
        let p1 = tmp.pop_back().unwrap();
        let p2 = tmp.pop_back().unwrap();
        let host = String::from_utf8(tmp.into()).unwrap();
        let port = u16::from_be_bytes([p2,p1]);
        let m = format!("[{}]{},{},{:?},{}",self.id,atyp,len,host,port);
        self.log(m.clone());
        log::im(m);
        self.clear_garbage();
        self.u8r_network_data_to_garbage();
        self.write(&[5,0,0,1,0,0,0,0,0,0]);
        self.set_status(Status::SecondPackDone);

    }

    fn client_hello_from_pc(&mut self) {
        self.clear_garbage();
        self.u8r_network_data_to_garbage();
        self.set_status(Status::EncryptDone);
    }

    fn u8r_network_data_to_garbage(&mut self) {
        let n = self.net_data.len();
        for _ in 0..n {
            let input = self.net_data.pop_front().unwrap();
            self.garbage.push(u8r(input));
        }
    }
}

impl Line {
    pub fn unexpect(&mut self) {
        self.log(format!("unexpect {:?}",self.status));
    }
}