use time::{Date, util::is_leap_year};


pub type ReadingsMonth = time::Month;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    pub fn day_of_week(&self) -> u8 { self.0.weekday().number_days_from_sunday() + 1 }
    pub fn month(&self) -> ReadingsMonth { self.0.month() }
    pub fn year(&self) -> i32 { self.0.year() }
    pub fn day_of_year(&self) -> u32 { self.0.to_ordinal_date().1 as u32 }

    pub fn days_since(&self, other: Self) -> u32 
    {
        (self.0 - other.0).whole_days().clamp(0, u32::MAX as i64) as u32
    }

    pub fn weeks_since(&self, other: Self) -> u32 
    {
        (self.0 - other.0).whole_weeks().clamp(0, u32::MAX as i64) as u32
    }

    pub fn is_leap_day(&self) -> bool
    {
        self.is_leap_year() && self.month() == ReadingsMonth::February && self.day() == 29
    }
}