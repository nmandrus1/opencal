use chrono::{DateTime, Utc};
use std::{
    collections::{BTreeSet, HashMap},
    ops::RangeBounds,
};

use uuid::Uuid;

use slotmap::{DefaultKey, Key, KeyData, SlotMap};

use super::event::{Event, EventID, EventRange};

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
                start: value.start(),
            },
            end: CalKey {
                inner: DefaultKey::null(),
                start: value.end(),
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

    /// String representing the name of a calendar
    name: String,
}

impl Calendar {
    pub fn new(name: String) -> Self {
        let mut slf = Self::default();
        slf.name = name;
        slf
    }

    /// If the Event was not in the calendar then [None](https://doc.rust-lang.org/nightly/core/option/enum.Option.hmtl) is returned
    /// otherwise Some(Event) is returned
    pub fn add_event(&mut self, event: Event) -> Option<Event> {
        // if the event is already in the map return None
        if self.event_map.contains_key(&event.uid()) {
            return Some(event);
        }

        let eid = event.uid();

        let requestid = Uuid::new_v4();
        let add_span = tracing::info_span!(
            "Request_ID: {} - Add request for EventID: {}",
            %requestid,
            %eid
        );

        let _add_span_guard = add_span.enter();
        tracing::info!("Added EventID: {}", eid);

        let dt_utc: DateTime<Utc> = event.start();

        let mut key = self.arena.insert(event);
        key.start = dt_utc;

        self.event_set.insert(key);
        self.event_map.insert(eid, key);

        None
    }

    /// Remove an event from the calendar
    pub fn remove(&mut self, eid: EventID) -> Option<Event> {
        let key = match self.event_map.remove(&eid) {
            Some(k) => k,
            None => return None,
        };

        self.event_set.remove(&key);
        self.arena.remove(key)
    }

    /// Get an event to the calendar
    pub fn get(&self, eid: EventID) -> Option<&Event> {
        let requestid = Uuid::new_v4();

        tracing::info!(
            "Request_ID: {} - Received get reqeust for EventID: {}",
            requestid,
            eid
        );

        // first attempts to get the CalKey from the event map, if successful
        // it retreives a reference to the event from the slotmap
        self.event_map
            .get(&eid)
            .and_then(|key| self.arena.get(*key))
    }

    /// Get all events that fall within the time range
    pub fn range(&self, range: EventRange) -> impl Iterator<Item = &Event> {
        let requestid = Uuid::new_v4();

        tracing::info!(
            "Request_ID {} - Received range request in range: {} -> {}",
            requestid,
            range.start(),
            range.end()
        );

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
    use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime};

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

        let event_date = DateTime::from_utc(
            NaiveDateTime::parse_from_str("2023-01-01 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            Utc,
        );

        let event_date_end = DateTime::from_utc(
            NaiveDateTime::parse_from_str("2023-01-01 23:59:59", "%Y-%m-%d %H:%M:%S").unwrap(),
            Utc,
        );

        let ev1 = Event::new("Kulindu".into(), event_date, event_date_end);
        let ev1_id = ev1.uid();

        let ev2 = Event::new("Michael".into(), event_date, event_date_end);

        // bc ev1 is not in cal, add_event should return None
        assert!(cal.add_event(ev1).is_none());

        // ev2 is not in the calendar
        assert!(cal.get(ev2.uid()).is_none());

        let maybe_event = cal.get(ev1_id);
        assert!(maybe_event.is_some());
    }

    #[test]
    fn test_range() {
        let mut cal = Calendar::default();

        let ev1 = Event::new_all_day("Kulindu BDay".to_string(), nth_day_2023(2));
        let ev1_id = ev1.uid();

        let ev2 = Event::new_all_day("Michael BDay".to_string(), nth_day_2023(1));
        let ev2_id = ev2.uid();

        let ev3 = Event::new_all_day("Niels BDay".to_string(), nth_day_2023(0));
        let ev3_id = ev3.uid();

        cal.add_event(ev1);
        cal.add_event(ev2);
        cal.add_event(ev3);

        let mut iter = cal.range(EventRange::from(None, None));

        // ev3 is should appear first bc its Jan 1 then ev2 and ev3

        assert_eq!(iter.next().unwrap().uid(), ev3_id);
        assert_eq!(iter.next().unwrap().uid(), ev2_id);
        assert_eq!(iter.next().unwrap().uid(), ev1_id);
        assert!(iter.next().is_none());
    }
}
