use std::{io::{self, Error, ErrorKind}, time::Instant};

use mio::event::Event;

use crate::config::BUFF_SIZE;

pub trait NetWorkEvent {
    fn id(&self) -> usize {0}
    fn line_type(&self) -> String;
    fn on_writable(&mut self){}
    fn on_read_closed(&mut self){}
    fn on_write_closed(&mut self){}
    fn fetch_error(&self) -> io::Result<Option<Error>>;
    fn event_log(&mut self,s:String);
    fn flush_log(&mut self);
    
    fn recv(&mut self,buf:&mut [u8]) -> io::Result<usize>;
    fn on_recv(&mut self,buf:&[u8]);
    fn consume_network_data(&mut self){}
    
    fn write_to_network(&mut self,buf:&[u8]) -> io::Result<usize>;

    fn on_event(&mut self,event:&Event) {
        self.event_log(format!("on_event {:?}",event));
        if event.is_error() {
            self.on_error();
            return;
        }

        if event.is_writable() {
            if event.is_write_closed() {
                self.on_write_closed();
            } else {
                self.on_writable();
            }
        }

        if event.is_readable() {
            if event.is_read_closed() {
                self.on_read_closed();
            } else {
                self.read();
            }
        }
       
    }

    fn on_error(&mut self) {
        let ret = self.fetch_error();
        match ret {
            Ok(err) => {
                match err {
                    Some(e) => {
                        self.event_log(format!("on_error {:?}",e));
                    }
                    None => todo!()
                }
            },
            Err(_) => todo!(),
        }
    }

    fn read(&mut self) {
        loop {
            let mut buf = [0;BUFF_SIZE];
            let t = Instant::now();
            match self.recv(&mut buf) {
                Ok(n) => {
                    self.event_log(format!("recv {} bytes from network {:?},{:?}",n,self.line_type(),t.elapsed().as_micros()));
                    if n > 0 {
                        self.on_recv(&buf[..n]);
                    } else {
                        break;
                    }
                }

                Err(e) => {
                    if e.kind() != ErrorKind::WouldBlock {
                        self.event_log(format!("read err {:?}",e));
                    }
                    break;
                },
            }
        }
        self.consume_network_data();
    }

    fn write(&mut self,buf:&[u8]) -> usize {
        let len = buf.len();
        match self.write_to_network(buf) {
            Ok(n) => {
                if n == len {
                    self.event_log(format!("all {} bytes data put to socket",n));
                } else {
                    let m = format!("[{}]only partial data put to socket {},{}",self.id(),n,len);
                    self.event_log(m.clone());
                }
                n
            },
            Err(e) => {
                let m = format!("[{}]socket write fail {:?},{}",self.id(),e,buf.len());
                self.event_log(m);
                self.flush_log();
                0
            },
        }
    }
}