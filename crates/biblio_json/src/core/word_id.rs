use std::{num::NonZeroU32, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::core::{ChapterId, OsisBook, VerseId};

lazy_static::lazy_static!
{
    static ref VERSE_ID_REGEX: Regex = Regex::new("^(?P<book>[\\d*a-zA-Z]+)\\.(?P<chapter>[1-9]\\d*)\\.(?P<verse>[1-9]\\d*)#(?P<word>[1-9]\\d*)$").unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WordId 
{
    pub book: OsisBook,
    pub chapter: NonZeroU32,
    pub verse: NonZeroU32,
    pub word: NonZeroU32,
}

impl WordId
{
    pub fn new(book: OsisBook, chapter: NonZeroU32, verse: NonZeroU32, word: NonZeroU32) -> Self 
    {
        Self 
        {
            book,
            chapter,
            verse,
            word
        }
    }

    pub fn chapter_id(&self) -> ChapterId
    {
        ChapterId { 
            book: self.book, 
            chapter: self.chapter 
        }
    }

    pub fn verse_id(&self) -> VerseId
    {
        VerseId { 
            book: self.book, 
            chapter: self.chapter, 
            verse: self.verse 
        }
    }
}

impl std::fmt::Display for WordId
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "{}.{}.{}#{}", self.book, self.chapter, self.verse, self.word)
    }
}

impl FromStr for WordId
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

        let word = captures.name("word")
            .unwrap()
            .as_str()
            .parse::<NonZeroU32>()
            .unwrap();
        
        Ok(Self {
            book,
            chapter,
            verse,
            word
        })
    }
}

impl Serialize for WordId
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WordId
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_id_new() {
        let word_id = WordId::new(
            OsisBook::Matt,
            NonZeroU32::new(1).unwrap(),
            NonZeroU32::new(1).unwrap(),
            NonZeroU32::new(1).unwrap(),
        );
        assert_eq!(word_id.book, OsisBook::Matt);
        assert_eq!(word_id.chapter.get(), 1);
        assert_eq!(word_id.verse.get(), 1);
        assert_eq!(word_id.word.get(), 1);
    }

    #[test]
    fn test_verse_id() {
        let word_id = WordId::new(
            OsisBook::Matt,
            NonZeroU32::new(3).unwrap(),
            NonZeroU32::new(16).unwrap(),
            NonZeroU32::new(5).unwrap(),
        );
        let verse_id = word_id.verse_id();
        assert_eq!(verse_id.book, OsisBook::Matt);
        assert_eq!(verse_id.chapter.get(), 3);
        assert_eq!(verse_id.verse.get(), 16);
    }

    #[test]
    fn test_display() {
        let word_id = WordId::new(
            OsisBook::John,
            NonZeroU32::new(2).unwrap(),
            NonZeroU32::new(5).unwrap(),
            NonZeroU32::new(3).unwrap(),
        );
        assert_eq!(word_id.to_string(), "John.2.5#3");
    }

    #[test]
    fn test_from_str_valid() {
        let word_id: WordId = "Matt.1.2#3".parse().unwrap();
        assert_eq!(word_id.chapter.get(), 1);
        assert_eq!(word_id.verse.get(), 2);
        assert_eq!(word_id.word.get(), 3);
    }

    #[test]
    fn test_from_str_invalid() {
        let result: Result<WordId, _> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization() {
        let word_id = WordId::new(
            OsisBook::Mark,
            NonZeroU32::new(4).unwrap(),
            NonZeroU32::new(8).unwrap(),
            NonZeroU32::new(2).unwrap(),
        );
        let json = serde_json::to_string(&word_id).unwrap();
        let deserialized: WordId = serde_json::from_str(&json).unwrap();
        assert_eq!(word_id, deserialized);
    }
}
