use std::time::Instant;

use mio::net::TcpStream;

use crate::log::Log;

use super::{Line, LineType, Status};

mod status;
mod data;
mod pair;
mod network_event;
mod redmi;
mod aliyun_mainland;

impl Line {
    pub fn new(id:usize,pair_id:usize,line_type:LineType,stream:TcpStream) -> Line {
        let logger = Log::create_for_line(&line_type,id);
        Line {
            id,pair_id,line_type,
            status: Status::Raw,
            stream,
            website_port: 0,
            net_data: Default::default(),
            mem_data: Default::default(),
            garbage: Default::default(),
            website_host: Default::default(),
            birth: Instant::now(),
            logger,
        }
    }

    fn log(&mut self,s:String) {
        self.logger.add(format!("[{}]{}",self.pair_id,s));
    }
}

impl Drop for Line {
    fn drop(&mut self) {
        self.logger.flush();
    }
}