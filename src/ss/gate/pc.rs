use mio::{net::TcpStream, Token};

use crate::{default_config::SERVER_IP, log, ss::{Gate, LineType}};

impl Gate {
    pub fn create_hk_chicks(&mut self,n:u8) {
        log::create_dir(LineType::Pc);
        log::create_dir(LineType::Hk);
        for _ in 0..n {
            let id = self.next_id();
            let stream = TcpStream::connect(SERVER_IP.parse().unwrap()).unwrap();
            self.new_line(id, 0, LineType::Hk, stream);
        }
    }

    pub fn find_chick_for_john(&mut self,stream:TcpStream) {
        let chick_id = self.find_idle_hk_chick();
        if chick_id > 0 {
            let id = self.next_id();
            self.new_line(id, chick_id, LineType::Pc, stream);
            let line = self.lines.get_mut(&Token(chick_id)).unwrap();
            line.set_pair_id(id);
            self.logger.add(format!("chick[{}]for john[{}]",chick_id,id))
        } else {
            let m = format!("no chick available");
            self.logger.add(m.clone());
            log::im(m);
        }
    }

    fn find_idle_hk_chick(&self) -> usize {
        for (_,line) in self.lines.iter() {
            if line.is_hk_chick() && line.is_ready() {
                return line.id;
            }
        }
        0
    }
}