use mio::{event::Event, net::TcpStream, Token};

use crate::{log, ss::{network_event::NetWorkEvent, DnsManager, Gate, LineType}};

impl Gate {
    pub fn gather_dns_query(&mut self) {
        let mut names:Vec<(usize,String)> = Vec::new();

        for (_,line) in self.lines.iter_mut() {
            match line.line_type {
                LineType::MainLand => {
                    match line.status {
                        crate::ss::Status::FirstPackDone => {
                            names.push((line.id,line.website_host.clone()));
                            line.waiting_for_dns_result();
                        },
                        _ => {},
                    }
                }
                _ => {},
            }
        }

        match &mut self.dm {
            Some(dm) => {
                for (id,host) in names {
                    dm.new_dns_query(id, host);
                }
            }
            None => {},
        }
    }

    pub fn check_dns_result(&mut self) {
        match &mut self.dm {
            Some(dm) => {
                let n = dm.peek();
                if n > 0 {
                    let mut data = dm.move_out_dns_result();
                    for _ in 0..data.len() {
                        let (id,ip) = data.pop().unwrap();
                        self.on_dns_result(id, ip);
                    }
                }
            },
            None => {},
        }
    }

    pub fn activate_dns_manager(&mut self) {
        log::create_dir(LineType::MainLand);
        log::create_dir(LineType::World);
        let dm = DnsManager::new(&self.p);
        self.dm = Some(dm);
    }

    pub fn dns_take_care(&mut self,event:&Event) {
        self.dm.as_mut().unwrap().on_event(event);
    }

    pub fn on_dns_result(&mut self,id:usize,ret:Option<String>) {
        let line = self.lines.get_mut(&Token(id)).unwrap();
        match ret {
            Some(ip) => {
                let ip_port = format!("{}:{}",ip,line.website_port);
                match ip_port.parse() {
                    Ok(addr) => {
                        match TcpStream::connect(addr) {
                            Ok(stream) => {
                                let new_line_id = stream.local_addr().unwrap().port() as usize;
                                self.new_line(new_line_id, id, LineType::World, stream);
                                let new_line = self.lines.get_mut(&Token(new_line_id)).unwrap();
                                new_line.set_website_host(ip_port);
                                let line = self.lines.get_mut(&Token(id)).unwrap();
                                line.dns_query_success(new_line_id);
                            },
                            Err(e) => self.logger.add(format!("connect to {} fail {}",addr,e)),
                        }
                    },
                    Err(e) => self.logger.add(format!("{} not a valid address {}",ip_port,e)),
                }
            }
            None => line.dns_query_fail(),
        }
    }
}