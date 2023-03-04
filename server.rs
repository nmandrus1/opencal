//! CalServer is an actor, it manages different calendars and all of the
//! connections associated with each calendar.

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

use icalendar::Event;
use std::collections::{HashMap, HashSet};

/// Message sent to a calendar session
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// New cal session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    // session id
    pub id: usize,
}

/// Send message to specific calendar
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

/// Struct representing the Websocket server
/// Responsible for coordinating calendars
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
    /// Send message to all users in the calendar
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

/// Make actor from `ChatServer`
impl Actor for CalServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

// Run when Connect message is sent from a Session
impl Handler<Connect> for CalServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        // assign an id to the session and store it in the hashmap
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
