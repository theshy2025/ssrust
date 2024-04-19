use std::collections::HashMap;

use mio::{event::Event, net::TcpStream, Interest, Token};

use crate::{default_config::CHICK_INIT_NUM, log, ss::{network_event::NetWorkEvent, Gate, Line, LineType}};

impl Gate {
    pub fn decouple(&mut self) {
        let mut close:Vec<(usize,usize)> = Vec::new();
        for (_,line) in self.lines.iter_mut() {
            if line.is_close() && line.pair_id > 0 {
                close.push((line.id,line.pair_id));
                line.set_pair_id(0);
            }
        }

        for (id,pid) in close {
            match self.lines.get_mut(&Token(pid)) {
                Some(line) => line.on_pair_close(id),
                None => log::im(format!("fail fetch close line {}-{}",id,pid)),
            }
        }
    }

    pub fn clear_dead_line(&mut self) {
        let mut hk = 0;
        let mut working = 0;
        let mut ready = 0;
        
        let mut dead:Vec<usize> = Vec::new();

        for (_,line) in self.lines.iter_mut() {
            let id = line.id;
            
            if line.is_dead() {
                dead.push(id);
            }

            if line.is_hk_chick() {
                hk = hk + 1;
                if line.is_ready() {
                    ready = ready + 1;
                }

                if line.is_working() {
                    working = working + 1;
                } else if line.is_old() {
                    dead.push(id);
                }
            }
        }
        
        if true {
            self.logger.add(format!("total:{},hk:{},working:{},ready:{},dead:{}",self.lines.len(),hk,working,ready,dead.len()));
        }

        for id in dead {
            self.lines.remove(&Token(id));
        }

        if working > 0 && ready < CHICK_INIT_NUM/2 {
            self.create_hk_chicks(1);
        }
    }

    pub fn deliver(&mut self) {
        let mut ret:HashMap<usize, Vec<u8>> = HashMap::new();

        for (_,line) in self.lines.iter_mut() {
            let pid = line.pair_id;
            let len = line.garbage.len();
            if pid > 0 && len > 0 {
                ret.insert(pid, line.move_out_garbage());
            }
        }
        

        for (id,data) in ret {
            match self.lines.get_mut(&Token(id)) {
                Some(line) => line.on_data_from_mem(data),
                None => self.logger.add(format!("fail fetch line {}",id)),
            }
        }
    }

    pub fn line_event(&mut self,event:&Event) {
        let line = self.lines.get_mut( &event.token() ).unwrap();
        line.on_event(event);
    }

    pub fn new_line(&mut self,id:usize,pair_id:usize,line_type:LineType,mut stream:TcpStream) {
        let t = Token(id);
        self.p.registry().register(&mut stream,t,Interest::READABLE | Interest::WRITABLE).unwrap();
        let line = Line::new(id, pair_id, line_type, stream);
        self.lines.insert(t, line);
    }
    
}