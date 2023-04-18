//! CalServer is an actor, it manages different calendars and all of the
//! connections associated with each calendar.

use actix::prelude::*;
use chrono::{DateTime, Utc};
use rand::{self, rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::collections::{HashMap, HashSet};

use super::calendar::Calendar;
use super::event::*;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ServerError {
    #[error("Calendar does not exist")]
    CalendarNotFound,

    #[error("Invalid Client ID")]
    InvalidClientID,

    #[error("Failed to create the calendar")]
    CalendarCreationFailed,

    #[error("Event does not exist")]
    EventNotFound,

    #[error("Event failed to serialize to JSON")]
    EventFailedToSerialize,
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
#[derive(Debug, Message, Serialize, Deserialize)]
#[rtype(result = "Result<EventID, ServerError>")]
pub struct AddEvent {
    cal_name: String,

    e_name: String,

    start: DateTime<Utc>,
    end: DateTime<Utc>,

    // client id
    id: usize,
}

/// Send delete request of event to calendar session:
/// Potential Fields: Time, name
/// Possible Returns: pop() nature could be helpful, else void or bool on success
#[derive(Debug, Message, Serialize, Deserialize)]
#[rtype(result = "Result<(), ServerError>")]
pub struct DeleteEvent {
    // client id
    id: usize,

    cal_name: String,

    // eventID
    eid: EventID,
}

#[derive(Debug, Message, Serialize, Deserialize)]
#[rtype(result = "Result<String, ServerError>")]
pub struct GetEvent {
    cal_name: String,
    eid: EventID,
}

#[derive(Debug, Message, Serialize, Deserialize)]
#[rtype(result = "Result<String, ServerError>")]
pub struct GetEventsInRange {
    cal_name: String,
    range: EventRange,
}

/// Create a calendar
#[derive(Debug, Message, Serialize, Deserialize)]
#[rtype(result = "Result<String, ServerError>")]
pub struct CreateCal {
    /// Client id
    pub id: usize,

    /// Name of calendar created
    pub name: String,
}

/// Join calendar, if the calendar does not exists send an error back
#[derive(Debug, Message, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientMessage {
    ListCals,
    CreateCal(CreateCal),
    AddEvent(AddEvent),
    GetEvent(GetEvent),
    GetEventsInRange(GetEventsInRange),
    DeleteEvent(DeleteEvent),
    Join(Join),
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

impl Handler<Join> for CalServer {
    type Result = Result<String, ServerError>;

    fn handle(&mut self, msg: Join, _ctx: &mut Self::Context) -> Self::Result {
        // Attempt to connect user to calendar

        let (id, cal_name) = (msg.id, msg.name);

        // remove the user from the other calendars
        self._cals.iter_mut().for_each(|c| {
            c.1 .1.remove(&id);
        });

        if let Some((_, sessions)) = self._cals.get_mut(&cal_name) {
            sessions.insert(id);
            Ok(format!("Successfully joined calendar: {}", &cal_name))
        } else {
            Err(ServerError::CalendarNotFound)
        }
    }
}

impl Handler<AddEvent> for CalServer {
    type Result = Result<EventID, ServerError>;

    fn handle(&mut self, msg: AddEvent, ctx: &mut Self::Context) -> Self::Result {
        let (cal, sessions) = match self._cals.get_mut(&msg.cal_name) {
            Some(inner) => inner,
            None => return Err(ServerError::CalendarNotFound),
        };

        let event = Event::new(msg.e_name, msg.start, msg.end);
        let uid = event.uid();

        cal.add_event(event);

        Ok(uid)
    }
}

impl Handler<DeleteEvent> for CalServer {
    type Result = Result<(), ServerError>;

    fn handle(&mut self, msg: DeleteEvent, ctx: &mut Self::Context) -> Self::Result {
        let (cal, _) = match self._cals.get_mut(&msg.cal_name) {
            Some(inner) => inner,
            None => return Err(ServerError::CalendarNotFound),
        };

        match cal.remove(msg.eid) {
            Some(_) => Ok(()),
            None => Err(ServerError::EventNotFound),
        }
    }
}

impl Handler<GetEvent> for CalServer {
    type Result = Result<String, ServerError>;

    fn handle(&mut self, msg: GetEvent, ctx: &mut Self::Context) -> Self::Result {
        let (cal, _) = match self._cals.get_mut(&msg.cal_name) {
            Some(inner) => inner,
            None => return Err(ServerError::CalendarNotFound),
        };

        match cal.get(msg.eid) {
            Some(e) => match serde_json::to_string(e) {
                Ok(s) => Ok(s),
                Err(_) => Err(ServerError::EventFailedToSerialize),
            },
            None => Err(ServerError::EventNotFound),
        }
    }
}

impl Handler<GetEventsInRange> for CalServer {
    type Result = Result<String, ServerError>;

    fn handle(&mut self, msg: GetEventsInRange, ctx: &mut Self::Context) -> Self::Result {
        let (cal, _) = match self._cals.get_mut(&msg.cal_name) {
            Some(inner) => inner,
            None => return Err(ServerError::CalendarNotFound),
        };

        let (ok, err): (Vec<_>, Vec<_>) = cal
            .range(msg.range)
            .map(|e| serde_json::to_string(e))
            .partition(Result::is_ok);

        if !err.is_empty() {
            Err(ServerError::EventFailedToSerialize)
        } else {
            serde_json::to_string(&ok.into_iter().map(Result::unwrap).collect::<Vec<String>>())
                .or(Err(ServerError::EventFailedToSerialize))
        }
    }
}

impl Handler<CreateCal> for CalServer {
    type Result = Result<String, ServerError>;

    fn handle(&mut self, msg: CreateCal, ctx: &mut Self::Context) -> Self::Result {
        let (id, new_cal_name) = (msg.id, msg.name);

        log::info!("User {id} is attempting to create a new calendar: \"{new_cal_name}\"");

        // if the calendar name is empty then return an error
        if new_cal_name.is_empty() {
            log::error!("Calendar creation failed: Invalid Name");
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
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

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

    #[actix_rt::test]
    async fn server_add_event_test() -> anyhow::Result<()> {
        // Test to ensure that trying to create an invalid calendar fails
        let client_id = 0;
        let svr = CalServer::new().start();

        // try to make a new calendar with no name
        svr.send(super::CreateCal {
            id: client_id,
            name: "main".to_string(),
        })
        .await?;

        let event_st = DateTime::from_utc(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2023, 6, 19).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ),
            Utc,
        );

        let event_end = DateTime::from_utc(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2023, 6, 19).unwrap(),
                NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
            ),
            Utc,
        );

        let test = svr
            .send(super::AddEvent {
                e_name: "Michael's Birthday".to_string(),
                cal_name: "main".to_string(),

                start: event_st,
                end: event_end,

                id: client_id,
            })
            .await?;

        assert!(test.is_ok());

        Ok(())
    }
}
