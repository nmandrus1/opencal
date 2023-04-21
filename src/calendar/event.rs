use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct EventID(u128);

impl std::fmt::Display for EventID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn start(&self) -> DateTime<Utc> {
        self.start
    }

    pub fn end(&self) -> DateTime<Utc> {
        self.end
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    uid: EventID,

    start: DateTime<Utc>,
    end: DateTime<Utc>,

    name: String,
    description: Option<String>,
}

impl Event {
    pub fn new(name: String, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            uid: EventID(Uuid::new_v4().as_u128()),
            start,
            end,
            name,
            description: None,
        }
    }

    pub fn new_all_day(name: String, date: NaiveDate) -> Self {
        let start = NaiveDateTime::new(date, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let end = NaiveDateTime::new(date, NaiveTime::from_hms_opt(23, 59, 59).unwrap());

        let start = DateTime::from_utc(start, Utc);
        let end = DateTime::from_utc(end, Utc);

        Self {
            uid: EventID(Uuid::new_v4().as_u128()),
            start,
            end,
            name,
            description: None,
        }
    }

    pub fn uid(&self) -> EventID {
        self.uid
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn start(&self) -> DateTime<Utc> {
        self.start
    }

    pub fn end(&self) -> DateTime<Utc> {
        self.end
    }

    pub fn time_span(&self) -> EventRange {
        EventRange::from(Some(self.start), Some(self.end))
    }
}
