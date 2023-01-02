use chrono::{offset::Utc, NaiveDate, NaiveDateTime, NaiveTime};
// use felyx_lib::Event;

fn main() {
    let first_day_2023 = NaiveDateTime::new(
        NaiveDate::from_ymd(2023, 1, 1),
        NaiveTime::from_hms(0, 0, 0),
    );

    let first_day_2024 = NaiveDateTime::new(
        NaiveDate::from_ymd(2024, 1, 1),
        NaiveTime::from_hms(0, 0, 0),
    );

    println!("{:?}", first_day_2023.cmp(&first_day_2024));
}
