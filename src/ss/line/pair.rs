use std::net::Shutdown;

use crate::ss::{network_event::NetWorkEvent, Line};

impl Line {
    pub fn set_pair_id(&mut self,id:usize) {
        let m = format!("pair_id change from {} to {},{:?},{:?}",self.pair_id,id,self.status,self.line_type);
        self.pair_id = id;
        self.log(m);
        self.flush_log();
    }

    pub fn on_pair_close(&mut self,id:usize) {
        self.log(format!("your pair[{}] now close {},{:?},{},{},{}",id,self.pair_id,self.line_type,self.mem_data.len(),self.net_data.len(),self.garbage.len()));
        self.flush_log();
        match self.line_type {
            crate::ss::LineType::MainLand => self.shut_down(Shutdown::Both),
            _ => {},
        }
    }
}