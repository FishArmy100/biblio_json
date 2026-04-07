pub mod ref_id;
pub mod strongs_number;
pub mod verse_id;
pub mod book;
pub mod lang;
pub mod chapter_id;
pub mod word_id;

use std::{fmt::Display, num::NonZeroU32, str::FromStr};

pub use ref_id::*;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub use strongs_number::*;
pub use book::*;
pub use chapter_id::*;
pub use verse_id::*;

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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_range_single_display() 
    {
        let range = WordRange::Single(NonZeroU32::new(5).unwrap());
        assert_eq!(range.to_string(), "5");
    }

    #[test]
    fn test_word_range_range_display() 
    {
        let range = WordRange::Range(NonZeroU32::new(2).unwrap(), NonZeroU32::new(7).unwrap());
        assert_eq!(range.to_string(), "2-7");
    }

    #[test]
    fn test_word_range_from_str_single() 
    {
        let range: WordRange = "3".parse().unwrap();
        assert_eq!(range, WordRange::Single(NonZeroU32::new(3).unwrap()));
    }

    #[test]
    fn test_word_range_from_str_range() 
    {
        let range: WordRange = "4-8".parse().unwrap();
        assert_eq!(
            range,
            WordRange::Range(NonZeroU32::new(4).unwrap(), NonZeroU32::new(8).unwrap())
        );
    }

    #[test]
    fn test_word_range_from_str_invalid() 
    {
        let result: Result<WordRange, _> = "0".parse();
        assert!(result.is_err());

        let result: Result<WordRange, _> = "a-b".parse();
        assert!(result.is_err());

        let result: Result<WordRange, _> = "1-".parse();
        assert!(result.is_err());

        let result: Result<WordRange, _> = "-2".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_word_range_serde_single() 
    {
        let range = WordRange::Single(NonZeroU32::new(9).unwrap());
        let serialized = serde_json::to_string(&range).unwrap();
        assert_eq!(serialized, "\"9\"");
        let deserialized: WordRange = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, range);
    }

    #[test]
    fn test_word_range_serde_range() 
    {
        let range = WordRange::Range(NonZeroU32::new(1).unwrap(), NonZeroU32::new(3).unwrap());
        let serialized = serde_json::to_string(&range).unwrap();
        assert_eq!(serialized, "\"1-3\"");
        let deserialized: WordRange = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, range);
    }
}
