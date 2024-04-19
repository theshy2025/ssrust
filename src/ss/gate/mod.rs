use std::collections::HashMap;

use mio::{event::Event, net::{TcpListener, TcpStream}, Events, Interest, Poll, Token};

use crate::{config::{BUFF_SIZE, DNS, GATE}, default_config::{CHICK_INIT_NUM, GATE_PORT}, log::Log};

use super::{Gate, LineType};

mod pc;
mod dns;
mod line;

impl Gate {
    pub fn new() -> Gate {
        let p = Poll::new().unwrap();
        let addr = format!("0.0.0.0:{}",GATE_PORT).parse().unwrap();
        let mut listener = TcpListener::bind(addr).unwrap();
        p.registry().register(&mut listener, GATE, Interest::READABLE).unwrap();
        let logger = Log::create("gate");
        Gate { p , listener , lines:HashMap::new() , dm:None, logger }
    }

    pub fn start(&mut self) {
        self.init();
        loop {
            self.tick();
        }
    }
}

impl Gate {
    fn init(&mut self) {
        assert!(BUFF_SIZE >= 1024);
        let m = format!("listening on {:?}",self.listener.local_addr().unwrap());
        self.logger.add(m);
        
        if CHICK_INIT_NUM > 0 {
            self.create_hk_chicks(CHICK_INIT_NUM);
        } else {
            self.activate_dns_manager();
        }
        
    }

    fn tick(&mut self) {
        self.decouple();
        self.clear_dead_line();
        self.poll();
        self.check_dns_result();
        self.gather_dns_query();
        self.deliver();
    }

    fn poll(&mut self) {
        let mut events = Events::with_capacity(16);
        self.p.poll(&mut events, None).unwrap();
        for event in events.iter() {
            self.on_event(event);
        }
    }

    fn on_event(&mut self,event:&Event) {
        match event.token() {
            GATE => self.gate_take_care(),
            DNS => self.dns_take_care(event),
            _ => self.line_event(event),
        }
    }

    fn gate_take_care(&mut self) {
        loop {
            match self.listener.accept() {
                Ok((stream,_)) => {
                    self.on_john_connect(stream);
                },
                
                Err(_) => {
                    break;
                },
            }
        }
    }

    fn on_john_connect(&mut self,stream:TcpStream) {
        if CHICK_INIT_NUM > 0 {
            self.find_chick_for_john(stream);
        } else {
            let id = stream.peer_addr().unwrap().port() as usize;
            self.new_line(id, 0, LineType::MainLand, stream);
            let line = self.lines.get_mut(&Token(id)).unwrap();
            line.say_hello_to_mainland();
        }
    }

}

impl Drop for Gate {
    fn drop(&mut self) {
        self.logger.flush();
    }
}