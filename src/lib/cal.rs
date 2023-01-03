use chrono::NaiveDateTime;
use std::collections::BTreeSet;

use super::event::Event;

/// Represents a calendar of events
#[derive(Default)]
pub struct EventCalendar(BTreeSet<Event>);

impl EventCalendar {
    /// inserts event into calednar, returning true if the event
    /// is new to the calendar and false if the event already exits
    pub fn add_event(&mut self, event: Event) -> bool {
        self.0.insert(event)
    }

    /// return an iterator of all events between start and end
    pub fn events_in_range(
        &self,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> impl Iterator<Item = &Event> {
        self.0.iter().filter(move |evt| {
            (evt.start() >= start && evt.start() <= end) || (evt.end() >= start && evt.end() <= end)
        })
    }

    pub fn first_event(&self) -> Option<&Event> {
        self.0.first()
    }
}
