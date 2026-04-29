use std::{num::NonZeroU32, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{core::OsisBook, modules::bible::BookInfo};

lazy_static::lazy_static!
{
    static ref VERSE_ID_REGEX: Regex = Regex::new("^(?P<book>[\\d*a-zA-Z]+)\\.(?P<chapter>[1-9]\\d*)\\.(?P<verse>[1-9]\\d*)$").unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VerseId 
{
    pub book: OsisBook,
    pub chapter: NonZeroU32,
    pub verse: NonZeroU32,
}

impl VerseId
{
    pub fn new(book: OsisBook, chapter: NonZeroU32, verse: NonZeroU32) -> Self 
    {
        Self
        {
            book,
            chapter,
            verse,
        }
    }

    pub fn flatten(self, books: &[BookInfo]) -> u32 
    {
        // Find the book info
        let book_info = books
            .iter()
            .find(|b| b.osis_book == self.book)
            .expect("Book not found");

        // Compute verses in all previous books
        let mut index = 0u32;
        for b in books 
        {
            if b.osis_book == self.book 
            {
                break;
            }
            index += b.chapters.iter().sum::<u32>();
        }

        // Compute verses in previous chapters
        let chapter_idx = (self.chapter.get() - 1) as usize;
        index += book_info.chapters[..chapter_idx].iter().sum::<u32>();

        // Add verse (minus 1 because flatten is 0-based)
        index += self.verse.get() - 1;

        index
    }

    pub fn expand(books: &[BookInfo], mut index: u32) -> Self 
    {
        // Find book by subtracting book sizes
        let mut book_info = None;

        for b in books 
        {
            let book_verse_count: u32 = b.chapters.iter().sum();
            if index < book_verse_count 
            {
                book_info = Some(b);
                break;
            } 
            else 
            {
                index -= book_verse_count;
            }
        }

        let book_info = book_info.expect("Index out of range");

        // Find chapter by subtracting chapter sizes
        let mut chapter_num = 1u32;
        for &count in &book_info.chapters 
        {
            if index < count 
            {
                break;
            } 
            else 
            {
                index -= count;
                chapter_num += 1;
            }
        }

        let verse_num = index + 1; // convert from 0-based to 1-based

        VerseId {
            book: book_info.osis_book,
            chapter: NonZeroU32::new(chapter_num).unwrap(),
            verse: NonZeroU32::new(verse_num).unwrap(),
        }
    }
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

pub struct VerseRangeIter<'a>
{
    books: &'a [BookInfo],
    current: u32,
    end: u32,
}

impl<'a> VerseRangeIter<'a>
{
    pub fn from_flat(books: &'a [BookInfo], from: u32, to: u32) -> Self 
    {
        Self {
            books,
            current: from,
            end: to,
        }
    }

    pub fn from_verses(books: &'a [BookInfo], from: VerseId, to: VerseId) -> Self 
    {
        Self {
            books,
            current: from.flatten(books),
            end: to.flatten(books)
        }
    }
}

impl<'a> Iterator for VerseRangeIter<'a> 
{
    type Item = VerseId;

    fn next(&mut self) -> Option<Self::Item> 
    {
        if self.current > self.end 
        {
            return None;
        }

        let v = VerseId::expand(self.books, self.current);
        self.current += 1;
        Some(v)
    }
}

#[cfg(test)]
mod tests 
{
    use std::collections::HashSet;

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

    #[test]
    fn test_invalid_format() 
    {
        let invalid_strs = [
            "Col4.2",      // missing dot
            "Col.04.2",    // chapter starts with 0
            "Col.4.02",    // verse starts with 0
            "Col..4.2",    // extra dot
            "Col.4",       // missing verse
            "Col.4.2.1",   // too many parts
            "Col.0.2",     // chapter zero
            "Col.4.0",     // verse zero
            "Col.4.two",   // non-numeric verse
            "Col.four.2",  // non-numeric chapter
        ];

        for s in invalid_strs 
        {
            assert!(VerseId::from_str(s).is_err(), "Should fail for: {}", s);
        }
    }

    #[test]
    fn test_serialize_deserialize() 
    {
        let verse = VerseId {
            book: OsisBook::Phlm,
            chapter: NonZeroU32::new(1).unwrap(),
            verse: NonZeroU32::new(25).unwrap(),
        };
        let serialized = serde_json::to_string(&verse).unwrap();
        assert_eq!(serialized, "\"Phlm.1.25\"");
        let deserialized: VerseId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(verse, deserialized);
    }

    #[test]
    fn test_display_trait() {
        let verse = VerseId {
            book: OsisBook::Gen,
            chapter: NonZeroU32::new(50).unwrap(),
            verse: NonZeroU32::new(26).unwrap(),
        };
        assert_eq!(format!("{}", verse), "Gen.50.26");
    }

    #[test]
    fn test_equality_and_hash() 
    {
        let v1 = VerseId {
            book: OsisBook::John,
            chapter: NonZeroU32::new(3).unwrap(),
            verse: NonZeroU32::new(16).unwrap(),
        };
        let v2 = VerseId {
            book: OsisBook::John,
            chapter: NonZeroU32::new(3).unwrap(),
            verse: NonZeroU32::new(16).unwrap(),
        };
        let mut set = HashSet::new();
        set.insert(v1);
        assert!(set.contains(&v2));
    }
}

