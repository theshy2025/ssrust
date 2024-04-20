use std::net::Shutdown;

use crate::{log, ss::{Line, LineType, Status}};

impl Line {
    pub fn turn_into_close(&mut self) {
        self.log(format!("turn_into_close"));
        self.set_status(Status::Close);
        log::im(format!("[{}][{}]{}",self.id,self.pair_id,self.traffic));
    }

    pub fn turn_into_read_close(&mut self) {
        self.log(format!("turn_into_read_close {:?},{:?}",self.status,self.line_type));
        self.set_status(Status::Dead);
        self.set_status(Status::ReadClose);

        match self.line_type {
            LineType::Pc | LineType::World => self.shut_down(Shutdown::Write),
            _ => {},
        }
    }

    pub fn shut_down(&mut self,how:Shutdown) {
        self.log(format!("shut_down {:?},{:?},{:?}",how,self.status,self.line_type));
        match self.stream.shutdown(how) {
            Ok(_) => {},
            Err(e) => {
                let m = format!("[{}]shutdown {:?} fail {}",self.id,how,e);
                self.log(m.clone());
                log::im(m);
            },
        }
    }

    pub fn set_status(&mut self,status:Status) {
        let m = format!("change status from {:?} to {:?}",self.status,status);
        self.status = status;
        self.log(m);
        match self.status {
            Status::ReadClose | Status::WriteClose | Status::EncryptDone => self.logger.flush(),
            _ => {},
        }
    }

    pub fn is_old(&self) -> bool {
        match self.status {
            Status::Raw | Status::Established | Status::FirstPackDone => self.birth.elapsed().as_secs() > 60 ,
            _ => false,
        }
    }

    pub fn is_hk_chick(&self) -> bool {
        match self.line_type {
            LineType::Hk => true,
            _ => false,
        }
    }

    pub fn is_working(&self) -> bool {
        if self.pair_id == 0 {
            return false;
        }

        match self.status {
            Status::Established | Status::FirstPackDone | 
            Status::SecondPackDone | Status::EncryptDone => true,
            
            _ => false,
        }
    }

    pub fn is_ready(&self) -> bool {
        if self.is_working() {
            return false;
        }

        if self.is_old() {
            return false;
        }

        match self.status {
            Status::FirstPackDone => true,
            _ => false,
        }
    }

    pub fn is_close(&self) -> bool {
        match self.status {
            Status::Close => true,
            _ => false,
        }
    }

    pub fn is_dead(&self) -> bool {
        match self.status {
            Status::Dead => true,
            _ => false,
        }
    }
}
