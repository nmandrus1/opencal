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

use chrono::{Datelike, Month as CMonth, NaiveDate, NaiveDateTime, NaiveTime, Utc, Weekday};
use num_traits::cast::FromPrimitive;

use thiserror::Error;

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
pub struct Event {
    name: String,
    start: NaiveDateTime,
    end: NaiveDateTime,
}

impl Event {
    /// given a start and end time determine whether they would be valid
    fn start_end_times_valid(st: &NaiveDateTime, end: &NaiveDateTime) -> bool {
        end.signed_duration_since(*st).num_seconds().is_positive()
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
    pub fn with_start(&mut self, start: NaiveDateTime) -> Result<&NaiveDateTime, EventError> {
        // check how many seconds from the start time the end time is, if the value
        // is negative that means the start time is AFTER the end time which
        // results in an InvalidStartTime error, on success returns the new start time
        if Event::start_end_times_valid(&start, &self.end) {
            // previous start time is overwritten
            self.start = start;
            Ok(&self.start)
        } else {
            // if the new start time is invalid then return an error
            Err(EventError::InvalidStartTime)
        }
    }

    pub fn with_end(&mut self, end: NaiveDateTime) -> Result<&NaiveDateTime, EventError> {
        // check how many seconds from the end time the start time is, if the value
        // is negative that means the start time is AFTER the end time which
        // results in an InvalidEndTime error, on success returns new end time
        if Event::start_end_times_valid(&self.start, &end) {
            // previous end time is overwritten
            self.end = end;
            Ok(&self.end)
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

/// Struct containing all the events for the day
pub struct Day<'month> {
    events: Option<Vec<&'month Event>>,
    n_date: NaiveDate,
    weekday: Weekday,
}

/// Struct representing a Month
pub struct Month {
    // days: Vec<Day<'month>><'month>,
    events: Vec<Event>,
    // Chrono Month enum
    month: CMonth,
    // the year the month is in
    year: i32,
}

impl Month {
    fn from_ym(year: i32, month: u32) -> Option<Self> {
        // the first day of the mont
        let date = NaiveDate::from_ymd_opt(year, month, 1)?;

        // iterator over every day of the month and create a Vector of Days
        // let days = date
        //     .iter_days()
        //     .take_while(|date| date.month() == month)
        //     .map(|d| Day {
        //         events: None,
        //         n_date: d,
        //         weekday: d.weekday(),
        //     })
        //     .collect::<Vec<Day>>();

        Some(Self {
            // days,
            events: Vec::new(),
            month: CMonth::from_u32(month)?,
            year,
        })
    }
}

/// Represents a calendar of events with the previous,
/// current, and next months events loaded
pub struct EventCalendar {
    curr_month: Month,
}

// NOTE: this will need to be changed to scan for files
// already present on the server and load those if they exist
impl Default for EventCalendar {
    fn default() -> Self {
        // get current time in standard timezone
        let datetime = Utc::now().naive_utc();

        let c_month = chrono::Month::from_u32(datetime.month()).unwrap();

        // the month numbers, previous, next, and current month
        let c_month = datetime.month();

        let curr_month = Month::from_ym(datetime.year(), c_month).unwrap();

        Self { curr_month }
    }
}

#[cfg(test)]
mod test {
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
    fn from_ym_test() {
        let dec_22 = Month::from_ym(2022, 12).unwrap();

        assert_eq!(dec_22.month, CMonth::December);
        // assert_eq!(dec_22.days[0].weekday, Weekday::Thu);
        // assert_eq!(dec_22.days.len(), 31);

        let feb_20 = Month::from_ym(2020, 2).unwrap();
        assert_eq!(feb_20.month, CMonth::February);
        // assert_eq!(feb_20.days[0].weekday, Weekday::Sat);
        // assert_eq!(feb_20.days[28].weekday, Weekday::Sat);
        // assert_eq!(feb_20.days.len(), 29);
    }

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

        event
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

        event
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

        event
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
        event
            .with_start(NaiveDateTime::new(naive_date, new_start_time))
            .unwrap();

        assert_eq!(event.start, NaiveDateTime::new(naive_date, new_start_time));

        // new end time
        let new_end_time = NaiveTime::from_hms_opt(22, 30, 0).unwrap();

        // update end time
        event
            .with_end(NaiveDateTime::new(naive_date, new_end_time))
            .unwrap();

        assert_eq!(event.end, NaiveDateTime::new(naive_date, new_end_time));

        // try to set invalid start time
        let status = event.with_start(NaiveDateTime::new(naive_date, last_time));
        assert_eq!(true, status.is_err());

        // try to set invalid end time
        let status = event.with_end(NaiveDateTime::new(naive_date, first_time));
        assert_eq!(true, status.is_err());
    }
}
