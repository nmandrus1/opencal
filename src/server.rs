

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

use icalendar::Event;
use std::collections::{HashMap, HashSet};



#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// calendar name
    pub cal: String,
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    // session id
    pub id: usize,
}



pub struct CalServer {
    sessions: HashMap<usize, Recipient<Message>>,
    _cals: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
}

impl CalServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            _cals: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl CalServer {
    fn _send_message(&self, cal: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self._cals.get(cal) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }
}


impl Actor for CalServer {

    type Context = Context<Self>;
}


impl Handler<Connect> for CalServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        let id = self.rng.gen();
        self.sessions.insert(id, msg.addr);

        println!("Connection established\nSession id: {}...", id);

        id
    }
}

impl Handler<Disconnect> for CalServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        println!("Session {} has disconnected", msg.id);

        self.sessions.remove(&msg.id);
    }
}
