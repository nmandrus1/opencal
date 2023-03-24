use chrono::{DateTime, NaiveDateTime, Utc};
use icalendar::{Component, Event};
use std::{
    collections::{BTreeSet, HashMap},
    ops::RangeBounds,
};

use slotmap::{DefaultKey, Key, KeyData, SlotMap};

#[derive(PartialEq, Eq, Hash)]
pub struct EventID(usize);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash)]
struct CalKey {
    inner: DefaultKey,
    start: chrono::DateTime<Utc>,
}

impl PartialOrd for CalKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.start.partial_cmp(&other.start)
    }
}

impl Ord for CalKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

impl std::convert::From<KeyData> for CalKey {
    fn from(value: KeyData) -> Self {
        let inner = DefaultKey::from(value);
        let start = chrono::DateTime::default();

        Self { inner, start }
    }
}

unsafe impl Key for CalKey {
    fn data(&self) -> KeyData {
        self.inner.data()
    }
}

#[derive(Debug)]
pub struct EventRange {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl EventRange {
    /// Construct a new range from start to end, if either parameters
    /// are Option::None then the corresponding time will be set to either
    /// the [MAX_UTC](https://docs.rs/chrono/latest/chrono/struct.DateTime.html#associatedconstant.MAX_UTC)
    /// or [MIN_UTC](https://docs.rs/chrono/latest/chrono/struct.DateTime.html#associatedconstant.MIN_UTC) time
    pub fn from(start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self {
        Self {
            start: start.unwrap_or(DateTime::<Utc>::MIN_UTC),
            end: end.unwrap_or(DateTime::<Utc>::MAX_UTC),
        }
    }
}

#[derive(Debug)]
struct CalKeyRange {
    start: CalKey,
    end: CalKey,
}

impl From<EventRange> for CalKeyRange {
    fn from(value: EventRange) -> Self {
        // Creates two CalKeys from the EventRange
        // will null keys, it is INVALID to try to use these keys
        Self {
            start: CalKey {
                inner: DefaultKey::null(),
                start: value.start,
            },
            end: CalKey {
                inner: DefaultKey::null(),
                start: value.end,
            },
        }
    }
}

impl RangeBounds<CalKey> for CalKeyRange {
    fn start_bound(&self) -> std::ops::Bound<&CalKey> {
        std::ops::Bound::Included(&self.start)
    }

    fn end_bound(&self) -> std::ops::Bound<&CalKey> {
        std::ops::Bound::Included(&self.end)
    }
}

/// A calendar represented as a Set of Events
#[derive(Default)]
pub struct Calendar {
    arena: SlotMap<CalKey, Event>,

    /// Event set keeps the Events sorted by start time, this
    /// allows us to easily retrieve a range of events given
    /// a start and end time
    event_set: BTreeSet<CalKey>,

    /// A hashmap of events for random access based on an Event's ID
    event_map: HashMap<EventID, CalKey>,
}

impl Calendar {
    pub fn new() -> Self {
        Self::default()
    }
    /// Add an event to the calendar
    ///
    /// If the Event is already in the Calendar, then [None](https://doc.rust-lang.org/nightly/core/option/enum.Option.hmtl) is returned
    pub fn add_event(&mut self, eid: EventID, event: Event) -> Option<Event> {
        // if the event is already in the map return None
        if self.event_map.contains_key(&eid) {
            return Some(event);
        }

        let dt_utc: DateTime<Utc> = event.get_start().unwrap().into();

        let mut key = self.arena.insert(event);
        key.start = dt_utc;

        self.event_set.insert(key);
        self.event_map.insert(eid, key);
        None
    }

    /// Get an event to the calendar
    pub fn get(&self, eid: EventID) -> Option<&Event> {
        // first attempts to get the CalKey from the event map, if successful
        // it retreives a reference to the event from the slotmap
        self.event_map
            .get(&eid)
            .and_then(|key| self.arena.get(*key))
    }

    /// Get all events that fall within the time range
    pub fn range(&self, range: EventRange) -> impl Iterator<Item = &Event> {
        // We create two "CalKeys" that we will use to get a range
        // from the HashSet and then map the CalKeys to &Events
        self.event_set
            .range(CalKeyRange::from(range))
            .inspect(|v| println!("{:?}", v))
            .filter_map(|v| self.arena.get(*v))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Days, NaiveDate, NaiveTime};
    use icalendar::EventLike;

    use super::*;

    /// helper function to return the nth day since Jan 1 2023
    fn nth_day_2023(days: u64) -> NaiveDate {
        NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .checked_add_days(Days::new(days))
            .unwrap()
    }

    /// returns the nth hour of a day
    fn nth_hour(hour: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, 0, 0).unwrap()
    }

    #[test]
    fn test_insert() {
        let mut cal = Calendar::default();

        let mut ev1 = icalendar::Event::new();
        let event_date =
            NaiveDateTime::parse_from_str("2023-01-01 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let event_summary = "kulindu cooray loves javascript";

        ev1.starts(event_date);
        ev1.summary(event_summary);

        // bc ev1 is not in cal, add_event should return None
        assert!(cal.add_event(EventID(1), ev1).is_none());

        // EventID(0) is not in the calendar
        assert!(cal.get(EventID(0)).is_none());

        let maybe_event = cal.get(EventID(1));
        assert!(maybe_event.is_some());

        let event = maybe_event.unwrap();

        assert_eq!(event.get_summary(), Some(event_summary));
    }

    #[test]
    fn test_range() {
        let mut cal = Calendar::default();

        let mut ev1 = Event::new();
        let jan_3_10am = NaiveDateTime::new(nth_day_2023(2), nth_hour(10));
        ev1.starts(jan_3_10am);
        let ev1_summary = "Kulindu is not a funny guy";
        ev1.summary(ev1_summary);

        cal.add_event(EventID(1), ev1);

        let mut ev2 = Event::new();
        let jan_2_10am = NaiveDateTime::new(nth_day_2023(1), nth_hour(10));
        ev2.starts(jan_2_10am);
        let ev2_summary = "What funny tshirt should I get?";
        ev2.summary(ev2_summary);

        cal.add_event(EventID(2), ev2);

        let mut ev3 = Event::new();
        let jan_1_10am = NaiveDateTime::new(nth_day_2023(0), nth_hour(10));
        ev3.starts(jan_1_10am);
        let ev3_summary = "I'm running out of ideas";
        ev3.summary(ev3_summary);

        cal.add_event(EventID(3), ev3);

        let mut iter = cal.range(EventRange::from(None, None));
        // let test = EventRange::from(None, None);
        // let test2 = CalKeyRange::from(test);
        // println!("{:#?}", test2);

        // ev3 is should appear first bc its Jan 1 then ev2 and ev3

        // iter.for_each(|v| println!("{:#?}", v));

        assert_eq!(iter.next().unwrap().get_summary(), Some(ev3_summary));
        assert_eq!(iter.next().unwrap().get_summary(), Some(ev2_summary));
        assert_eq!(iter.next().unwrap().get_summary(), Some(ev1_summary));
        assert_eq!(iter.next(), None);
    }
}
