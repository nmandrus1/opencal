use std::ops::{Deref, DerefMut};

use icalendar::{Event as ICalEvent, EventLike};

pub struct Event(ICalEvent);

impl Deref for Event {
    type Target = ICalEvent;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Event {
    type Target = ICalEvent;
    fn deref_mut(&self) -> &Self::Target {
        &self.0
    }
}
