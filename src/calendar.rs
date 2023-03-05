use chrono::NaiveDateTime;
use icalendar::Event;
use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

pub struct EventID(usize);

/// A calendar represented as a Set of Events
#[derive(Default)]
pub struct Calendar {
    /// Event set keeps the Events sorted by start time, this
    /// allows us to easily retrieve a range of events given
    /// a start and end time
    _event_set: BTreeSet<Arc<Event>>,

    /// A hashmap of events for random access based on an Event's ID
    _event_map: HashMap<EventID, Arc<Event>>,
}

impl Calendar {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an event to the calendar
    pub fn add_event(&mut self, eid: EventID, event: Event) -> Option<Event> {
        unimplemented!()
    }

    /// Get an event to the calendar
    pub fn get(&self, eid: EventID) -> Option<Event> {
        unimplemented!()
    }

    /// Get all events that fall within the time range
    pub fn range(&self, start: NaiveDateTime, end: NaiveDateTime) {
        unimplemented!()
    }
}
