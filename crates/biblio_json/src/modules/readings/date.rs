use std::num::{NonZeroU8, NonZeroU32};

use serde::{Deserialize, Serialize};
use time::{Date, util::is_leap_year};


pub type ReadingsMonth = time::Month;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadingsDate(time::Date);

impl ReadingsDate
{
    pub fn new(year: i32, month: ReadingsMonth, day: u8) -> Option<Self>
    {
        Some(Self(Date::from_calendar_date(year, month, day).ok()?))
    }

    pub fn is_leap_year(&self) -> bool
    {
        is_leap_year(self.0.year())
    }

    pub fn day(&self) -> u8 { self.0.day() }
    pub fn month(&self) -> ReadingsMonth { self.0.month() }
    pub fn year(&self) -> i32 { self.0.year() }
}