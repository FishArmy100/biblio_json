pub mod alias_config;
pub mod abbrev_config;

use std::{collections::{HashMap, HashSet}, num::NonZeroU32};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{core::{Atom, OsisBook, RefId, RefIdInner, VerseId, lang::Language}, html_text::HtmlText, modules::{EntryId, ExternalModuleData, ModuleValidationError, bible::{abbrev_config::AbbrevConfig, alias_config::AliasConfig}}, utils, validation::ValidationContextBuilder};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct BibleConfig
{
    pub name: String,
    pub authors: Option<Vec<String>>,
    pub language: Option<Language>,
    pub description: Option<HtmlText>,
    pub data_source: Option<String>,
    pub pub_year: Option<u32>,
    pub license: Option<String>,
    pub books: HashMap<OsisBook, String>,

    #[serde(default)]
    pub book_abbreviations: AbbrevConfig,

    #[serde(default)]
    pub book_aliases: AliasConfig,

    #[serde(default)]
    pub external: ExternalModuleData,
}

impl BibleConfig
{
    pub fn get_abbreviated_book(&self, book: OsisBook) -> Option<&str>
    {
        if let Some(book) = self.book_abbreviations.get(&book)
        {
            Some(&book)
        }
        else if let Some(book) = self.books.get(&book)
        {
            Some(&book)
        }
        else
        {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BibleModule
{
    pub config: BibleConfig,
    pub source: BibleSource, 
}

impl BibleModule
{
    pub fn load_json(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: BibleConfig = utils::load_toml(config_path)?;

        let bible_path = format!("{}/{}.jsonl", dir_path, name);
        let source = BibleSource::from_file(&bible_path, &config.books, &config)?;

        Ok(Self { 
            config,
            source,
        })
    }

    pub fn get_abbreviated_book(&self, book: OsisBook) -> Option<&str>
    {
        self.config.get_abbreviated_book(book)
    }

    pub fn validate(&self, builder: &ValidationContextBuilder) -> Result<(), Vec<ModuleValidationError>>
    {
        let context = builder.build(Some(&self.config.name), &self.config.external);
        
        let errors = self.config.description.as_ref().iter()
            .flat_map(|d| d.validate(&context).err())
            .flatten()
            .map(|error| ModuleValidationError::HtmlError { error })
            .collect_vec();

        if errors.len() > 0
        {
            Err(errors)
        }
        else 
        {
            Ok(())    
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BibleSource
{
    pub name: String,
    pub book_infos: Vec<BookInfo>,
    pub verses: HashMap<VerseId, Verse>,
}

impl BibleSource
{
    pub fn from_file(path: &str, books: &HashMap<OsisBook, String>, config: &BibleConfig) -> Result<BibleSource, String>
    {
        let verses: Vec<(Verse, usize)> = utils::load_json_lines(path)
            .stringify_error()?
            .into_iter()
            .map(|l| (l.value, l.line))
            .collect();

        let mut visited_books = HashSet::<OsisBook>::new();
        let mut current_book: Option<&OsisBook> = None;
        let mut book_chapters: Vec<u32> = vec![];

        let mut book_infos = vec![];

        for (verse, line) in verses.iter()
        {
            let VerseId { book, chapter, verse: verse_idx } = &verse.verse_id;

            if Some(book) != current_book
            {
                if let Some(old_book) = current_book
                {
                    let Some(name) = books.get(old_book) else {
                        // line not +1 because it is referring to the previous line
                        return Err(format!("Full book name for {} in file {} on line {}, does not exist in the bible config.", old_book, path, line))
                    };

                    let Some(abbreviation) = config.get_abbreviated_book(*old_book) else {
                        return Err(format!("OsisBook {} does not exist in the config", book));
                    };

                    book_infos.push(BookInfo {
                        name: name.clone(),
                        osis_book: old_book.to_owned(),
                        index: visited_books.len() as u32,
                        chapters: book_chapters,
                        abbreviation: abbreviation.into(),
                    });
                }

                if !visited_books.insert(*book)
                {
                    return Err(format!("Book {} in file {} on line {}, has already been defined and is out of order.", book, path, line + 1));
                }

                current_book = Some(book);
                book_chapters = vec![0];
            }

            if chapter.get() == book_chapters.len() as u32 + 1
            {
                book_chapters.push(0);
            }
            else if chapter.get() != book_chapters.len() as u32 
            {
                return Err(format!("Verse {} in file {} on line {}, has a chapter number that is out of order.", verse.verse_id, path, line))
            }

            if verse_idx.get() == *book_chapters.last().unwrap() + 1
            {
                *book_chapters.last_mut().unwrap() += 1;
            }
            else 
            {
                return Err(format!("Verse {} in file {} on line {}, has a verse number that is out of order.", verse.verse_id, path, line))
            }
        }

        if let Some(old_book) = current_book
        {
            let Some(name) = books.get(old_book) else {
                // line not +1 because it is referring to the previous line
                return Err(format!("Full book name for {} in file {} on line {}, does not exist in the bible config.", old_book, path, verses.len()))
            };

            let Some(abbreviation) = config.get_abbreviated_book(*old_book) else {
                return Err(format!("OsisBook {} does not exist in the config", old_book));
            };

            book_infos.push(BookInfo {
                name: name.clone(),
                osis_book: old_book.to_owned(),
                index: visited_books.len() as u32,
                chapters: book_chapters,
                abbreviation: abbreviation.into(),
            });
        }

        let verses = verses.into_iter()
            .map(|(v, _)| (v.verse_id.clone(), v))
            .collect::<HashMap<_, _>>();

        Ok(Self 
        {
            book_infos,
            verses,
            name: config.name.clone(),
        })
    }

    pub fn id_exists(&self, id: &RefId) -> bool
    {
        match id.id 
        {
            RefIdInner::Single(atom) => self.id_atom_exists(&atom),
            RefIdInner::Range { from, to } => self.id_atom_exists(&from) && self.id_atom_exists(&to),
        }
    }

    pub fn id_atom_exists(&self, atom: &Atom) -> bool
    {
        let book = atom.book();
        let chapter = atom.chapter().unwrap_or(NonZeroU32::new(1).unwrap());
        let verse = atom.verse().unwrap_or(NonZeroU32::new(1).unwrap());

        let complete_verse = VerseId { book, chapter, verse };

        let word = atom.word();

        let Some(verse_data) = self.verses.get(&complete_verse) else {
            return false;
        };

        if let Some(word) = word.map(|w| w.get())
        {
            return word as usize <= verse_data.words.len();
        }

        true
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct Verse
{
    pub id: EntryId,
    pub verse_id: VerseId,
    pub words: Vec<Word>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct Word 
{
    pub red: Option<bool>,
    pub italics: Option<bool>,
    pub begin_punc: Option<String>,
    pub end_punc: Option<String>,
    pub text: String, 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BookInfo
{
    pub name: String,
    pub abbreviation: String,
    pub osis_book: OsisBook,
    pub index: u32,
    pub chapters: Vec<u32>
}