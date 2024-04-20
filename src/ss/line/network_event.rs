use std::io::{self, Error, Read, Write};

use crate::ss::{network_event::NetWorkEvent, Line, LineType, Status};

impl NetWorkEvent for Line {
    fn event_log(&mut self,s:String) {
        self.log(s);
    }

    fn on_read_closed(&mut self) {
        match self.status {
            Status::ReadClose => {},
            Status::WriteClose => self.turn_into_close(),
            Status::Close => {},
            Status::Dead => todo!(),
            _ => self.turn_into_read_close(),
        }
    }

    fn on_write_closed(&mut self) {
        match self.status {
            Status::WriteClose => {},
            Status::ReadClose => self.turn_into_close(),
            Status::Close => {},
            Status::Dead => todo!(),
            _ => self.set_status(Status::WriteClose),
        }
    }

    fn on_writable(&mut self) {
        match self.status {
            Status::Raw => {
                self.set_status(Status::Established);
            },
            _ => {}
        }

        self.flush_mem_data_to_network();
    }

    fn on_recv(&mut self,buf:&[u8]) {
        self.traffic = self.traffic + buf.len();
        self.net_data.extend(buf.iter());
    }

    fn consume_network_data(&mut self) {
        match self.line_type {
            LineType::Pc => self.redmi_consume_pc_data(),
            LineType::Hk => self.redmi_consume_hk_data(),
            LineType::MainLand => self.aliyun_consume_mainland_data(),
            LineType::World => self.move_network_data_to_garbage(),
        }
    }
    
    fn fetch_error(&self) -> io::Result<Option<Error>> {
        self.stream.take_error()
    }
    
    fn recv(&mut self,buf:&mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
    
    fn write_to_network(&mut self,buf:&[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }
    
    fn line_type(&self) -> String {
        format!("{:?}",self.line_type)
    }
    
    fn id(&self) -> usize {
        self.id
    }
    
    fn flush_log(&mut self) {
        self.logger.flush();
    }

    
}
