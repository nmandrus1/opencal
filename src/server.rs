//! CalServer is an actor, it manages different calendars and all of the
//! connections associated with each calendar.

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use thiserror::Error;
use uuid::Uuid;

use std::collections::{HashMap, HashSet};

use super::calendar::Calendar;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ServerError {
    #[error("Calendar does not exist")]
    CalendarNotFound,

    #[error("Invalid Client ID")]
    InvalidClientID,

    #[error("Failed to create the calendar")]
    CalendarCreationFailed,
}

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

/// Send Add request of event to calendar session
/// Potential Fields: Time Zone, time, description
/// Possible Returns: Boolean on success, maybe pointer or copy if necessary for other FNs
#[derive(Message)]
#[rtype(result = "()")]
pub struct AddEvent;

/// Send delete request of event to calendar session:
/// Potential Fields: Time, name
/// Possible Returns: pop() nature could be helpful, else void or bool on success
#[derive(Message)]
#[rtype(result = "()")]
pub struct DeleteEvent;

/// Create a calendar
#[derive(Message)]
#[rtype(result = "Result<String, ServerError>")]
pub struct CreateCal {
    /// Client id
    pub id: usize,

    /// Name of calendar created
    pub name: String,
}

/// Join calendar, if the calendar does not exists send an error back
#[derive(Message)]
#[rtype(result = "Result<String, ServerError>")]
pub struct Join {
    /// Client ID
    pub id: usize,

    /// Room name
    pub name: String,
}

#[derive(Message)]
#[rtype(result = "String")]
pub struct ListCals;

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
    _cals: HashMap<String, (Calendar, HashSet<usize>)>,
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
        let requestid = Uuid::new_v4();
        tracing::info!("Request_id: {} - Message sent to {}", requestid, cal);

        if let Some((_, sessions)) = self._cals.get(cal) {
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
        let requestid = Uuid::new_v4();
        tracing::info!("Request_id: {} - Connect message received", requestid);

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
        let requestid = Uuid::new_v4();
        tracing::info!("Request_id: {} - Disconnect message received", requestid);

        println!("Session {} has disconnected", msg.id);

        self.sessions.remove(&msg.id);
    }
}

impl Handler<Join> for CalServer {
    type Result = Result<String, ServerError>;

    fn handle(&mut self, msg: Join, _ctx: &mut Self::Context) -> Self::Result {
        let requestid = Uuid::new_v4();
        tracing::info!("Request_id: {} - Join message received", requestid);

        // Attempt to connect user to calendar

        let (id, cal_name) = (msg.id, msg.name);

        if let Some((_, sessions)) = self._cals.get_mut(&cal_name) {
            sessions.insert(id);
            Ok(format!("Successfully joined calendar: {}", &cal_name))
        } else {
            Err(ServerError::CalendarNotFound)
        }
    }
}

impl Handler<CreateCal> for CalServer {
    type Result = Result<String, ServerError>;

    fn handle(&mut self, msg: CreateCal, ctx: &mut Self::Context) -> Self::Result {
        let (id, new_cal_name) = (msg.id, msg.name);

        tracing::info!("User {id} is attempting to create a new calendar: \"{new_cal_name}\"");

        // if the calendar name is empty then return an error
        if new_cal_name.is_empty() {
            tracing::error!("Calendar creation failed: Invalid Name");
            return Err(ServerError::CalendarCreationFailed);
        }

        let new_cal = Calendar::new(new_cal_name.clone());

        // try to insert calendar
        let ins = self._cals.insert(new_cal_name, (new_cal, HashSet::new()));

        if ins.is_some() {
            Err(ServerError::CalendarCreationFailed)
        } else {
            Ok(String::from("Calendar created Successfully"))
        }
    }
}

impl Handler<ListCals> for CalServer {
    type Result = String;

    fn handle(&mut self, msg: ListCals, ctx: &mut Self::Context) -> Self::Result {
        if self._cals.is_empty() {
            String::from("No Calendars");
        }

        self._cals.keys().fold(String::new(), |mut acc, str| {
            acc.push_str(str);
            acc.push('\n');
            acc
        })
    }
}

#[cfg(test)]
mod tests {

    // The testing strategy for Calendar is to send it messages as
    // if they were coming from a WsCalSession and ensure that
    // the calendar appropriately handles them

    use super::*;
    use anyhow;

    #[actix_rt::test]
    async fn server_create_cal_test() -> anyhow::Result<()> {
        // basic test to test basic calendar creation

        let svr = CalServer::new().start();

        // send CreateCal message
        let test1 = svr
            .send(super::CreateCal {
                id: 0,
                name: "main".to_string(),
            })
            .await?;

        // was the calendar created Successfully
        assert!(test1.is_ok());

        Ok(())
    }

    #[actix_rt::test]
    async fn server_create_invalid_cal_test() -> anyhow::Result<()> {
        // Test to ensure that trying to create an invalid calendar fails

        let svr = CalServer::new().start();

        // try to make a new calendar with no name
        let test2 = svr
            .send(super::CreateCal {
                id: 0,
                name: "".to_string(),
            })
            .await?;

        // this test should fail and return CalendarCreationFailed
        assert!(test2.is_err());
        assert_eq!(
            test2.err().unwrap(),
            super::ServerError::CalendarCreationFailed
        );

        Ok(())
    }
}
