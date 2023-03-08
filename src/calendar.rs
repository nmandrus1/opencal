use chrono::NaiveDateTime;
use icalendar::{CalendarDateTime, Component, Event, EventLike};
use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use slotmap::{DefaultKey, SlotMap};

#[derive(PartialEq, Eq, Hash)]
pub struct EventID(usize);

/// A calendar represented as a Set of Events
#[derive(Default)]
pub struct Calendar<'a> {
    arena: SlotMap<DefaultKey, Event>,

    /// Event set keeps the Events sorted by start time, this
    /// allows us to easily retrieve a range of events given
    /// a start and end time
    event_set: BTreeSet<&'a Event>,

    /// A hashmap of events for random access based on an Event's ID
    event_map: HashMap<EventID, DefaultKey>,
}

impl<'a> Calendar<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an event to the calendar
    ///
    /// If the Event is already in the Calendar, then [None](https://doc.rust-lang.org/nightly/core/option/enum.Option.hmtl) is returned
    pub fn add_event(&mut self, eid: EventID, event: Event) -> Option<Event> {
        // if the event is already in the map return None
        if self.event_map.contains_key(&eid) {
            Some(event)
        }

        let key = self.arena.insert(event);
        // event was just inserted so retrieving a reference is safe
        let ev = self.arena.get(key).unwrap();

        self.event_set.insert(ev);
        self.event_map.insert(eid, key);
        None
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
