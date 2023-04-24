use chrono::NaiveDateTime;
use icalendar::Event;
use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

pub struct EventID(usize);


#[derive(Default)]
pub struct Calendar {
    _event_set: BTreeSet<Arc<Event>>,

    _event_map: HashMap<EventID, Arc<Event>>,
}

impl Calendar2 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, eid: EventID) -> Option<Event> {
        unimplemented!()
    }
    
    pub fn add_event(&mut self, eid: EventID, event: Event) -> Option<Event> {
        unimplemented!()
    }


    pub fn range(&self, start: NaiveDateTime, end: NaiveDateTime) {
        unimplemented!()
    }
}
