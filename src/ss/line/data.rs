use crate::ss::{network_event::NetWorkEvent, Line, Status};

impl Line {
    pub fn set_website_host(&mut self,s:String) {
        self.log(format!("website_host change from {:?} to {:?}",self.website_host,s));
        self.website_host = s;
    }
}

//network data
impl Line {
    pub fn move_network_data_to_garbage(&mut self) {
        self.garbage.extend(self.net_data.iter());
        self.clear_network_data();
    }


    pub fn clear_network_data(&mut self) {
        self.log(format!("clear_network_data {}",self.net_data.len()));
        self.net_data.clear();
    }
}

//memory data
impl Line {
    pub fn on_data_from_mem(&mut self,data:Vec<u8>) {
        let pa = self.mem_data.len();
        self.log(format!("{} bytes data from mem , old len {} {:?}",data.len(),pa,self.status));
        self.append_mem_data(data);
        match self.status {
            Status::Raw => self.unexpect(),
            Status::Close | Status::Dead => self.log(format!("i am dead already")),
            _ => {
                self.flush_mem_data_to_network();
            },
        }
    }

    pub fn flush_mem_data_to_network(&mut self) {
        let len = self.mem_data.len();
        if len > 0 {
            self.log(format!("flush_mem_data_to_network {},{:?},{:?}",len,self.status,self.line_type));
            let data = self.move_out_mem_data();
            let n = self.write(&data);
            if n < data.len() {
                self.append_mem_data(data[n..].to_vec());
            }
        } else {
            self.log(format!("mem data is empty no need flush"));
        }
    }
    
    pub fn move_out_garbage(&mut self) -> Vec<u8> {
        self.log(format!("move_out_garbage {}",self.garbage.len()));
        let ret = self.garbage.clone();
        self.clear_garbage();
        ret
    }
    
    pub fn clear_garbage(&mut self) {
        self.log(format!("clear_garbage {}",self.garbage.len()));
        self.garbage.clear();
    }

    pub fn append_mem_data(&mut self,mut data:Vec<u8>) {
        self.mem_data.append(&mut data);
    }

    pub fn move_out_mem_data(&mut self) -> Vec<u8> {
        let tmp = self.mem_data.clone();
        self.log(format!("move_out_mem_data {}",self.mem_data.len()));
        self.mem_data.clear();
        tmp
    }
}
