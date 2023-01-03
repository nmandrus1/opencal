use chrono::NaiveDateTime;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::event::Event;

/// Represents a calendar of events
#[derive(Default)]
pub struct EventCalendar(BTreeMap<Uuid, Event>);

impl EventCalendar {
    /// inserts event into calednar, returning None if the event
    /// is new to the calendar and Some(Event) if the event already exits
    pub fn add_event(&mut self, event: Event) -> Option<Event> {
        self.0.insert(*event.id(), event)
    }

    /// return an iterator of all events between start and end
    pub fn events_in_range(
        &self,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> impl Iterator<Item = (&Uuid, &Event)> {
        self.0.iter().filter(move |(_, evt)| {
            (evt.start() >= start && evt.start() <= end) || (evt.end() >= start && evt.end() <= end)
        })
    }

    /// return the first event in the Calendar
    pub fn first_event(&self) -> Option<&Event> {
        self.0.first_key_value().map(|(_, e)| Some(e)).flatten()
    }
}
