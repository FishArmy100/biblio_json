use std::{num::NonZeroU32, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::core::OsisBook;

lazy_static::lazy_static!
{
    static ref VERSE_ID_REGEX: Regex = Regex::new("^(?P<book>[a-zA-Z]+).(?P<chapter>[1-9]\\d*).(?P<verse>[1-9]\\d*)$").unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerseId 
{
    pub book: OsisBook,
    pub chapter: NonZeroU32,
    pub verse: NonZeroU32,
}

impl std::fmt::Display for VerseId
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "{}.{}.{}", self.book, self.chapter, self.verse)
    }
}

impl FromStr for VerseId
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        let Some(captures) = VERSE_ID_REGEX.captures(s) else {
            return Err(format!("String `{}` is not an OSIS verse", s));
        };

        let book_str = captures.name("book").unwrap().as_str();
        let book = OsisBook::from_str(book_str)?;

        let chapter = captures.name("chapter")
            .unwrap()
            .as_str()
            .parse::<NonZeroU32>()
            .unwrap();

        let verse = captures.name("verse")
            .unwrap()
            .as_str()
            .parse::<NonZeroU32>()
            .unwrap();
        
        Ok(Self {
            book,
            chapter,
            verse
        })
    }
}

impl Serialize for VerseId
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for VerseId
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests 
{
    use super::*;

    #[test]
    fn test_to_string()
    {
        let verse = VerseId {
            book: OsisBook::Kgs1,
            chapter: NonZeroU32::new(1).unwrap(),
            verse: NonZeroU32::new(4).unwrap()
        };

        assert_eq!(verse.to_string(), "1Kgs.1.4");
    }

    #[test]
    fn test_from_string()
    {
        let str = "Col.4.2";
        let v1 = VerseId::from_str(str).unwrap();
        let v2 = VerseId {
            book: OsisBook::Col,
            chapter: NonZeroU32::new(4).unwrap(),
            verse: NonZeroU32::new(2).unwrap(),
        };

        assert_eq!(v1, v2)
    }
}