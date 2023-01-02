// General Notes and Ideas
//
// - Fundemental unit is a Month
// - A month contains a vector of Days
//      - Serializable to be stored on disk
//      - Query specific days/weeks
// - A Day contains a list of events
//
// - Server will have a 'Calendar Owner' with full permissions
// - Server will have basic commands to add moderators and Event types

// Because the calendar is crowd sourced and public, events will
// have event owners such that only the owner or a group of owners will
// have permissions to edit the event
//
// FUTURE:
// - Users can open threads on an event to ask questions
//      - Moderated by server owners + event owners

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use thiserror::Error;

use std::collections::BTreeMap;

/// Basic Errors that can occur for events
#[derive(Error, Debug)]
pub enum EventError {
    /// Error for invalid start time for an event
    #[error("start time/date cannot be after end time/date")]
    InvalidStartTime,

    /// Error for invalid end time for an event
    #[error("end time/date cannot be before start time/date")]
    InvalidEndTime,
}

/// Struct to represent a given event on the calendar
#[derive(PartialOrd, PartialEq, Eq)]
pub struct Event {
    name: String,
    start: NaiveDateTime,
    end: NaiveDateTime,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        match (
            self.start.cmp(&other.start),
            self.end.cmp(&other.end),
            self.name.cmp(&other.name),
        ) {
            (Greater, _, _) => Greater,
            (Less, _, _) => Less,
            (Equal, Greater, _) => Greater,
            (Equal, Less, _) => Less,
            (Equal, Equal, Greater) => Greater,
            (Equal, Equal, Less) => Less,
            (Equal, Equal, Equal) => Equal,
        }
    }
}

impl Event {
    /// given a start and end time determine whether they would be valid
    fn start_end_times_valid(st: &NaiveDateTime, end: &NaiveDateTime) -> bool {
        end.signed_duration_since(*st).num_seconds().is_positive()
    }

    /// return the NaiveDate component of the start field
    fn start_nd(&self) -> NaiveDate {
        self.start.date()
    }

    /// return the NaiveDate component of the end field
    fn end_nd(&self) -> NaiveDate {
        self.end.date()
    }

    /// Create an Event with a name and date, defaults to an
    /// all day event starting at 00:00:00 and ending at 23:59:59
    pub fn new(name: String, date: &NaiveDate) -> Self {
        Self {
            name,
            start: NaiveDateTime::new(*date, NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
            end: NaiveDateTime::new(*date, NaiveTime::from_hms_opt(23, 59, 59).unwrap()),
        }
    }

    /// Set/Change an event's start time
    #[must_use]
    pub fn with_start(self, start: NaiveDateTime) -> Result<Self, EventError> {
        // check how many seconds from the start time the end time is, if the value
        // is negative that means the start time is AFTER the end time which
        // results in an InvalidStartTime error, on success returns the new start time
        if Event::start_end_times_valid(&start, &self.end) {
            // lol literally the first time ive used this syntax
            Ok(Event { start, ..self })
        } else {
            // if the new start time is invalid then return an error
            Err(EventError::InvalidStartTime)
        }
    }

    pub fn with_end(self, end: NaiveDateTime) -> Result<Self, EventError> {
        // check how many seconds from the end time the start time is, if the value
        // is negative that means the start time is AFTER the end time which
        // results in an InvalidEndTime error, on success returns new end time
        if Event::start_end_times_valid(&self.start, &end) {
            // previous end time is overwritten
            Ok(Event { end, ..self })
        } else {
            Err(EventError::InvalidEndTime)
        }
    }

    /// Change the name of an event
    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }
}

// NOTE: How to represent events that last multiple days?
// NOTE: In the future it migh be worth trying to remove the Day struct, it feels redundant
//       Maybe a Vector or Hashmap of Events makes sense? Suppose a request
//       was made to get all of the events given some time range,

/// Represents a calendar of events
pub struct EventCalendar {
    curr_month: BTreeMap<String, Event>,
}

// NOTE: this will need to be changed to scan for files
// already present on the server and load those if they exist
impl Default for EventCalendar {
    fn default() -> Self {
        // get current time in standard timezone
        Self {
            curr_month: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::{Datelike, Timelike};

    use super::*;

    // helper functions for test
    /// return the first NaiveDate for 2023
    fn first_day_2023_nd() -> NaiveDate {
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
    }

    /// return the first time of any day 00:00:00
    fn first_time_nt() -> NaiveTime {
        NaiveTime::from_hms_opt(0, 0, 0).unwrap()
    }

    /// return the last time for any day 23:59:59
    fn last_time_nt() -> NaiveTime {
        NaiveTime::from_hms_opt(23, 59, 59).unwrap()
    }

    /// return the first NaiveDateTime for 2023 - 01/01/2023-00:00:00
    fn first_day_2023_ndt() -> NaiveDateTime {
        let nd = first_day_2023_nd();
        let nt = first_time_nt();
        NaiveDateTime::new(nd, nt)
    }

    // ##################################
    // ###           TESTS            ###
    // ##################################

    #[test]
    fn test_new_event() {
        let naive_date = first_day_2023_nd();

        // common times
        let first_time = first_time_nt();
        let last_time = last_time_nt();

        // event being tested
        let event = Event::new(String::from("Birthday Party"), &naive_date);

        // assumed start and end times for testing
        let assumed_start_time = NaiveDateTime::new(naive_date, first_time);
        let assumed_end_time = NaiveDateTime::new(naive_date, last_time);

        assert_eq!(event.start, assumed_start_time);
        assert_eq!(event.end, assumed_end_time);
    }

    #[test]
    fn test_event_start_time_change() {
        // basic date declaration
        let naive_date = first_day_2023_nd();

        // event being tested
        let mut event = Event::new(String::from("Birthday Party"), &naive_date);
        // new start time
        let new_start_time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();

        event = event
            .with_start(NaiveDateTime::new(naive_date, new_start_time))
            .unwrap();
        assert_eq!(event.start, NaiveDateTime::new(naive_date, new_start_time))
    }

    #[test]
    fn test_event_end_time_change() {
        // basic date declaration
        let naive_date = first_day_2023_nd();

        // event being tested
        let mut event = Event::new(String::from("Birthday Party"), &naive_date);
        // new start time
        let new_end_time = NaiveTime::from_hms_opt(22, 30, 0).unwrap();

        event = event
            .with_end(NaiveDateTime::new(naive_date, new_end_time))
            .unwrap();

        assert_eq!(event.end, NaiveDateTime::new(naive_date, new_end_time))
    }

    #[test]
    fn test_invalid_event_time_change() {
        // basic date declaration
        let naive_date = first_day_2023_nd();
        let start_time = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let invalid_end_time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();

        let mut event = Event::new("Birthday".into(), &naive_date);

        event = event
            .with_start(NaiveDateTime::new(naive_date, start_time))
            .unwrap();

        assert_eq!(
            true,
            event
                .with_end(NaiveDateTime::new(naive_date, invalid_end_time))
                .is_err()
        );
    }

    #[test]
    fn invalid_events_test() {
        // basic date declaration
        let naive_date = first_day_2023_nd();

        // common times
        let first_time = first_time_nt();
        let last_time = last_time_nt();

        // event being tested
        let mut event = Event::new(String::from("Birthday Party"), &naive_date);

        // assumed start and end times for testing
        let assumed_start_time = NaiveDateTime::new(naive_date, first_time);
        let assumed_end_time = NaiveDateTime::new(naive_date, last_time);

        assert_eq!(event.start, assumed_start_time);
        assert_eq!(event.end, assumed_end_time);

        // new start time
        let new_start_time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();

        // update start time
        event = event
            .with_start(NaiveDateTime::new(naive_date, new_start_time))
            .unwrap();

        assert_eq!(event.start, NaiveDateTime::new(naive_date, new_start_time));

        // new end time
        let new_end_time = NaiveTime::from_hms_opt(22, 30, 0).unwrap();

        // update end time
        event = event
            .with_end(NaiveDateTime::new(naive_date, new_end_time))
            .unwrap();

        assert_eq!(event.end, NaiveDateTime::new(naive_date, new_end_time));

        // try to set invalid start time
        let status = event.with_start(NaiveDateTime::new(naive_date, last_time));
        assert_eq!(true, status.is_err());

        // try to set invalid end time
        let mut event = Event::new(String::from("Birthday Party"), &naive_date);
        let status = event.with_end(NaiveDateTime::new(naive_date, first_time));
        assert_eq!(true, status.is_err());
    }

    #[test]
    fn test_event_ordering_lt_start_cmp() {
        use std::cmp::Ordering;
        let ndt = first_day_2023_ndt();
        let d1 = Event::new("A".into(), &ndt.date());

        // 01/01/2023-00:00:00 < 01/01/2023-00:00:01
        let mut d2 = Event::new("A".into(), &ndt.date());
        d2 = d2.with_start(d1.start.with_second(1).unwrap()).unwrap();
        assert_eq!(d1.cmp(&d2), Ordering::Less);

        // 01/01/2023-00:00:00 < 01/01/2023-00:01:00
        let mut d3 = Event::new("A".into(), &ndt.date());
        d3 = d3.with_start(d1.start.with_minute(1).unwrap()).unwrap();
        assert_eq!(d1.cmp(&d3), Ordering::Less);

        // 01/01/2023-00:00:00 < 01/01/2023-01:00:00
        let mut d4 = Event::new("A".into(), &ndt.date());
        d4 = d4.with_start(d1.start.with_hour(1).unwrap()).unwrap();
        assert_eq!(d1.cmp(&d4), Ordering::Less);

        // 01/01/2023-00:00:00 < 01/01/2024-00:00:00
        let mut d5 = Event::new("A".into(), &ndt.date().with_year(2024).unwrap());
        assert_eq!(d1.cmp(&d5), Ordering::Less);

        // 01/01/2023-00:00:00 < 01/02/2023-00:00:00
        let mut d6 = Event::new("A".into(), &ndt.date());
        d6 = d6.with_end(d1.start.with_day(3).unwrap()).unwrap();
        d6 = d6.with_start(d1.start.with_day(2).unwrap()).unwrap();
        assert_eq!(d1.cmp(&d6), Ordering::Less);

        // 01/01/2023-00:00:00 < 02/01/2023-00:00:00
        let mut d7 = Event::new("A".into(), &ndt.date());
        d7 = d7.with_end(d1.start.with_month(3).unwrap()).unwrap();
        d7 = d7.with_start(d1.start.with_month(2).unwrap()).unwrap();
        assert_eq!(d1.cmp(&d7), Ordering::Less);
    }
}
