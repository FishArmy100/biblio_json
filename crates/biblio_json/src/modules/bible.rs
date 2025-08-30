use std::{collections::{HashMap, HashSet}, num::NonZeroU32};

use serde::{Deserialize, Serialize};

use crate::{core::{Atom, OsisBook, RefId, lang::Language}, html_text::HtmlText, modules::ExternalModuleData, utils};

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
    pub external: ExternalModuleData,
}

#[derive(Debug)]
pub struct BibleModule
{
    pub config: BibleConfig,
    pub source: BibleSource, 
}

impl BibleModule
{
    pub fn load(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: BibleConfig = utils::load_toml(config_path)?;

        let bible_path = format!("{}/{}.jsonl", dir_path, name);
        let source = BibleSource::from_file(&bible_path, &config.books)?;

        Ok(Self { 
            config,
            source,
        })
    }
}

#[derive(Debug)]
pub struct BibleSource
{
    pub book_infos: HashMap<u32, BookInfo>,
    pub verses: HashMap<RefId, Verse>
}

impl BibleSource
{
    pub fn from_file(path: &str, books: &HashMap<OsisBook, String>) -> Result<BibleSource, String>
    {
        let verses: Vec<(Verse, usize)> = utils::load_json_lines(path)?;

        let mut visited_books = HashSet::<OsisBook>::new();
        let mut current_book: Option<&OsisBook> = None;
        let mut book_chapters: Vec<u32> = vec![];

        let mut book_infos = HashMap::new();

        for (verse, line) in verses.iter()
        {
            let Some((book, chapter, verse_idx)) = verse.id.get_verse_components() else {
                return Err(format!("Verse {} in file {} on line {}, is not in the format `book.chapter.verse`.", &verse.id, path, line + 1));
            };

            if Some(book) != current_book
            {
                if let Some(old_book) = current_book
                {
                    let Some(name) = books.get(old_book) else {
                        // line not +1 because it is referring to the previous line
                        return Err(format!("Full book name for {} in file {} on line {}, does not exist in the bible config.", old_book, path, line))
                    };

                    book_infos.insert(visited_books.len() as u32, BookInfo {
                        name: name.clone(),
                        osis_book: old_book.to_owned(),
                        index: visited_books.len() as u32,
                        chapters: book_chapters,
                    });
                }

                if !visited_books.insert(*book)
                {
                    return Err(format!("Book {} in file {} on line {}, has already been defined and is out of order.", book, path, line + 1));
                }

                current_book = Some(book);
                book_chapters = vec![0];
            }

            if chapter == book_chapters.len() as u32 + 1
            {
                book_chapters.push(0);
            }
            else if chapter != book_chapters.len() as u32 
            {
                return Err(format!("Verse {} in file {} on line {}, has a chapter number that is out of order.", verse.id, path, line))
            }

            if verse_idx == *book_chapters.last().unwrap() + 1
            {
                *book_chapters.last_mut().unwrap() += 1;
            }
            else 
            {
                return Err(format!("Verse {} in file {} on line {}, has a verse number that is out of order.", verse.id, path, line))
            }
        }

        if let Some(old_book) = current_book
        {
            let Some(name) = books.get(old_book) else {
                // line not +1 because it is referring to the previous line
                return Err(format!("Full book name for {} in file {} on line {}, does not exist in the bible config.", old_book, path, verses.len()))
            };

            book_infos.insert(visited_books.len() as u32, BookInfo {
                name: name.clone(),
                osis_book: old_book.to_owned(),
                index: visited_books.len() as u32,
                chapters: book_chapters,
            });
        }

        let verses = verses.into_iter()
            .map(|(v, _)| (v.id.clone(), v))
            .collect::<HashMap<_, _>>();

        Ok(Self 
        {
            book_infos,
            verses,
        })
    }

    pub fn id_exists(&self, id: &RefId) -> bool
    {
        match id 
        {
            RefId::Single(atom) => self.id_atom_exists(atom),
            RefId::Range { from, to } => self.id_atom_exists(from) && self.id_atom_exists(to),
        }
    }

    pub fn id_atom_exists(&self, atom: &Atom) -> bool
    {
        let book = atom.book();
        let chapter = atom.chapter().unwrap_or(NonZeroU32::new(1).unwrap());
        let verse = atom.verse().unwrap_or(NonZeroU32::new(1).unwrap());

        let complete_verse = RefId::Single(Atom::Verse { book: *book, chapter, verse });

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
    pub id: RefId,
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

#[derive(Debug)]
pub struct BookInfo
{
    pub name: String,
    pub osis_book: OsisBook,
    pub index: u32,
    pub chapters: Vec<u32>
}