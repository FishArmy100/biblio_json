use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

use crate::core::{OsisBook, VerseId};

#[derive(Debug, PartialEq, Clone, Hash, Eq, Copy)]
pub enum RefId 
{
    Single(Atom),
    Range { from: Atom, to: Atom },
}

impl RefId
{
    pub fn has_verse_word(&self, verse_id: &VerseId, word_id: NonZeroU32) -> bool 
    {
        match self 
        { 
            RefId::Single(atom) => match atom 
            { 
                Atom::Book { book } => verse_id.book == *book, 
                Atom::Chapter { book, chapter } => verse_id.book == *book && verse_id.chapter == *chapter, 
                Atom::Verse { book, chapter, verse } => verse_id.book == *book && verse_id.chapter == *chapter && verse_id.verse == *verse, 
                Atom::Word { book, chapter, verse, word } => verse_id.book == *book && verse_id.chapter == *chapter && verse_id.verse == *verse && word_id == *word, 
            }, 
            RefId::Range { from, to } => 
            { 
                if verse_id.book < from.book() || verse_id.book > to.book() { return false; } 
                if verse_id.book == from.book() 
                { 
                    if let Some(chapter) = from.chapter() 
                    { 
                        if verse_id.chapter < chapter { return false } 
                        if verse_id.chapter == chapter 
                        { 
                            if let Some(verse) = from.verse() 
                            { 
                                if verse_id.verse < verse { return false } 
                                if verse_id.verse == verse 
                                { 
                                    if let Some(word) = from.word() 
                                    { 
                                        if word_id < word { return false } 
                                    } 
                                } 
                            } 
                        } 
                    } 
                } 
                
                if verse_id.book == to.book() 
                { 
                    if let Some(chapter) = to.chapter() 
                    { 
                        if verse_id.chapter > chapter { return false } 
                        if verse_id.chapter == chapter 
                        { 
                            if let Some(verse) = to.verse() 
                            { 
                                if verse_id.verse > verse { return false } 
                                if verse_id.verse == verse 
                                { 
                                    if let Some(word) = to.word() 
                                    { 
                                        if word_id > word { return false } 
                                    } 
                                } 
                            } 
                        } 
                    } 
                } true 
            }
        }
    }

    /// Will check if the RefId includes the given parameter `verse_id`
    /// ```
    /// let verse_id = VerseId::from_str("Gen.1.1");
    /// RefId::from_str("Gen.1.1#1").unwrap().has_verse(verse_id); // true
    /// RefId::from_str("Gen.1.1-Gen.1.5").unwrap().has_verse(verse_id); // true
    /// RefId::from_str("Gen.1.2-Gen.1.5").unwrap().has_verse(verse_id); // false
    /// ```
    pub fn has_verse(&self, verse_id: &VerseId) -> bool 
    {
        match self 
        { 
            RefId::Single(atom) => match atom 
            { 
                Atom::Book { book } => verse_id.book == *book, 
                Atom::Chapter { book, chapter } => verse_id.book == *book && verse_id.chapter == *chapter, 
                Atom::Verse { book, chapter, verse } => verse_id.book == *book && verse_id.chapter == *chapter && verse_id.verse == *verse, 
                Atom::Word { book, chapter, verse, .. } => verse_id.book == *book && verse_id.chapter == *chapter && verse_id.verse == *verse, 
            }, 
            RefId::Range { from, to } => 
            { 
                if verse_id.book < from.book() || verse_id.book > to.book() { return false; } 
                if verse_id.book == from.book() 
                { 
                    if let Some(chapter) = from.chapter() 
                    { 
                        if verse_id.chapter < chapter { return false } 
                        if verse_id.chapter == chapter 
                        { 
                            if let Some(verse) = from.verse() 
                            { 
                                if verse_id.verse < verse { return false }
                            } 
                        } 
                    } 
                } 
                
                if verse_id.book == to.book() 
                { 
                    if let Some(chapter) = to.chapter() 
                    { 
                        if verse_id.chapter > chapter { return false } 
                        if verse_id.chapter == chapter 
                        { 
                            if let Some(verse) = to.verse() 
                            { 
                                if verse_id.verse > verse { return false }
                            } 
                        } 
                    } 
                } true 
            }
        }
    }


    pub fn is_valid(&self) -> bool
    {
        match self 
        {
            Self::Single(_) => true,
            Self::Range { from, to } => match (from, to)
            {
                (Atom::Book { book: _ }, Atom::Book { book: _ }) => true,
                (Atom::Chapter { book: _, chapter: _ }, Atom::Chapter { book: _, chapter: _ }) => true,
                (Atom::Verse { book: _, chapter: _, verse: _ }, Atom::Verse { book: _, chapter: _, verse: _ }) => true,
                (Atom::Word { book: _, chapter: _, verse: _, word: _ }, Atom::Word { book: _, chapter: _, verse: _, word: _ }) => true,
                _ => false,
            }
        }
    }

    pub fn from_verse_id(verse: VerseId, word: Option<NonZeroU32>) -> Self
    {
        match word
        {
            Some(word) => Self::Single(Atom::Word { 
                book: verse.book, 
                chapter: verse.chapter, 
                verse: verse.verse,
                 word: word 
            }),
            None => Self::Single(Atom::Verse { 
                book: verse.book, 
                chapter: verse.chapter, 
                verse: verse.verse,
            }),
        }
    }

    pub fn is_range(&self) -> bool 
    {
        match self 
        {
            Self::Range { from: _, to: _ } => true,
            _ => false,
        }
    }

    pub fn is_book(&self) -> bool 
    {  
        match self 
        {
            RefId::Single(atom) => atom.is_book(),
            RefId::Range { from, to } => from.is_book() && to.is_book(),
        }
    }

    pub fn is_chapter(&self) -> bool 
    {
        match self 
        {
            RefId::Single(atom) => atom.is_chapter(),
            RefId::Range { from, to } => from.is_chapter() && to.is_chapter(),
        }
    }

    pub fn is_verse(&self) -> bool 
    {
        match self 
        {
            RefId::Single(atom) => atom.is_verse(),
            RefId::Range { from, to } => from.is_verse() && to.is_verse(),
        }
    }

    pub fn is_word(&self) -> bool 
    {
        match self 
        {
            RefId::Single(atom) => atom.is_word(),
            RefId::Range { from, to } => from.is_word() && to.is_word(),
        }
    }

    pub fn get_verse_components(&self) -> Option<(&OsisBook, u32, u32)>
    {
        if let Self::Single(Atom::Verse { book, chapter, verse }) = self 
        {
            Some((book, chapter.get(), verse.get()))
        }
        else 
        {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, Copy)]
pub enum Atom 
{
    Book { book: OsisBook },
    Chapter { book: OsisBook, chapter: NonZeroU32 },
    Verse { book: OsisBook, chapter: NonZeroU32, verse: NonZeroU32 },
    Word { book: OsisBook, chapter: NonZeroU32, verse: NonZeroU32, word: NonZeroU32 },
}

impl Atom
{
    pub fn is_book(&self) -> bool
    {
        match self 
        {
            Self::Book { book: _ } => true,
            _ => false,
        }
    }

    pub fn is_chapter(&self) -> bool 
    {
        match self 
        {
            Self::Chapter { book: _, chapter: _ } => true,
            _ => false,
        }
    }

    pub fn is_verse(&self) -> bool 
    {
        match self 
        {
            Self::Verse { book: _, chapter: _, verse: _ } => true,
            _ => false,
        }
    }

    pub fn is_word(&self) -> bool
    {
        match self 
        {
            Self::Word { book: _, chapter: _, verse: _, word: _ } => true,
            _ => false,
        }
    }

    pub fn book(&self) -> OsisBook 
    {
        match self 
        {
            Atom::Book { book } => *book,
            Atom::Chapter { book, chapter: _ } => *book,
            Atom::Verse { book, chapter: _, verse: _ } => *book,
            Atom::Word { book, chapter: _, verse: _, word: _ } => *book,
        }
    }

    pub fn chapter(&self) -> Option<NonZeroU32>
    {
        match self 
        {
            Atom::Chapter { book: _, chapter } => Some(*chapter),
            Atom::Verse { book: _, chapter, verse: _ } => Some(*chapter),
            Atom::Word { book: _, chapter, verse: _, word: _ } => Some(*chapter),
            _ => None
        }
    }

    pub fn verse(&self) -> Option<NonZeroU32>
    {
        match self 
        {
            Atom::Verse { book: _, chapter: _, verse } => Some(*verse),
            Atom::Word { book: _, chapter: _, verse, word: _ } => Some(*verse),
            _ => None,
        }
    }

    pub fn word(&self) -> Option<NonZeroU32>
    {
        match self 
        {
            Atom::Word { book: _, chapter: _, verse: _, word } => Some(*word),
            _ => None,
        }
    }

    pub fn into_components(&self) -> (OsisBook, Option<NonZeroU32>, Option<NonZeroU32>, Option<NonZeroU32>)
    {
        (
            self.book(),
            self.chapter(),
            self.verse(),
            self.word(),
        )
    }
}

impl fmt::Display for Atom 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        match self {
            Atom::Book { book } => write!(f, "{}", book),
            Atom::Chapter { book, chapter } => write!(f, "{}.{}", book, chapter),
            Atom::Verse { book, chapter, verse } => write!(f, "{}.{}.{}", book, chapter, verse),
            Atom::Word { book, chapter, verse, word } => {
                write!(f, "{}.{}.{}#{}", book, chapter, verse, word)
            }
        }
    }
}

impl fmt::Display for RefId 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        match self {
            RefId::Single(atom) => write!(f, "{}", atom),
            RefId::Range { from, to } => write!(f, "{}-{}", from, to),
        }
    }
}

impl FromStr for Atom 
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        let (main, word_opt) = if let Some((main, word)) = s.split_once('#') {
            (main, Some(word))
        } 
        else 
        {
            (s, None)
        };

        let parts: Vec<&str> = main.split('.').collect();
        match (parts.len(), word_opt) {
            (1, None) => Ok(Atom::Book {
                book: parts[0].to_string().parse()?,
            }),
            (2, None) => Ok(Atom::Chapter {
                book: parts[0].to_string().parse()?,
                chapter: parts[1].parse().map_err(|_| "Invalid chapter")?,
            }),
            (3, None) => Ok(Atom::Verse {
                book: parts[0].to_string().parse()?,
                chapter: parts[1].parse().map_err(|_| "Invalid chapter")?,
                verse: parts[2].parse().map_err(|_| "Invalid verse")?,
            }),
            (3, Some(word)) => Ok(Atom::Word {
                book: parts[0].to_string().parse()?,
                chapter: parts[1].parse().map_err(|_| "Invalid chapter")?,
                verse: parts[2].parse().map_err(|_| "Invalid verse")?,
                word: word.parse().map_err(|_| "Invalid word")?,
            }),
            _ => Err(format!("Unrecognized Atom format: {}", s)),
        }
    }
}

impl FromStr for RefId 
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        if let Some((from, to)) = s.split_once('-') {
            Ok(RefId::Range {
                from: Atom::from_str(from.trim())?,
                to: Atom::from_str(to.trim())?,
            })
        } 
        else 
        {
            Ok(RefId::Single(Atom::from_str(s.trim())?))
        }
    }
}

impl From<VerseId> for RefId
{
    fn from(value: VerseId) -> Self 
    {
        RefId::Single(Atom::Verse { 
            book: value.book, 
            chapter: value.chapter, 
            verse: value.verse 
        })
    }
}

impl Serialize for RefId 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RefId 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        RefId::from_str(&s).map_err(serde::de::Error::custom)
    }
}
