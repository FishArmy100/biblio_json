pub mod ref_id;
pub mod strongs_number;
pub mod verse_id;
pub mod book;
pub mod lang;

use std::{fmt::Display, num::NonZeroU32, str::FromStr};

pub use ref_id::*;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub use strongs_number::*;
pub use verse_id::*;
pub use book::*;


lazy_static::lazy_static!
{
    static ref WORD_RANGE_REGEX: Regex = Regex::new("^(?P<start>[1-9]\\d*)-(?P<end>[1-9]\\d*)$").unwrap();
    static ref WORD_INDEX_REGEX: Regex = Regex::new("^(?P<index>[1-9]\\d*)$").unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WordRange
{
    Single(NonZeroU32),
    Range(NonZeroU32, NonZeroU32),
}

impl Display for WordRange
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            Self::Single(i) => write!(f, "{}", i),
            Self::Range(s, e) => write!(f, "{}-{}", s, e)
        }
    }
}

impl FromStr for WordRange
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        if let Some(caps) = WORD_INDEX_REGEX.captures(s) 
        {
            let index = caps.name("index").unwrap().as_str().parse::<NonZeroU32>().unwrap();
            Ok(Self::Single(index))
        }
        else if let Some(caps) = WORD_RANGE_REGEX.captures(s)
        {
            let start = caps.name("start").unwrap().as_str().parse::<NonZeroU32>().unwrap();
            let end = caps.name("end").unwrap().as_str().parse::<NonZeroU32>().unwrap();
            Ok(Self::Range(start, end))
        }
        else 
        {
            Err(format!("`{}` is not a word range", s))
        }
    }
}

impl Serialize for WordRange 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WordRange 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}