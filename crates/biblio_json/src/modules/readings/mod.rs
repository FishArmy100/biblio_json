pub mod date;

use core::fmt;
use std::str::FromStr;

use itertools::{Either, Itertools};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{core::RefId, html_text::HtmlText, modules::{EntryId, ExternalModuleData, ModuleValidationError, readings::date::ReadingsDate}, utils, validation::ValidationContextBuilder};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingsConfig
{
    pub name: String,
    pub authors: Option<Vec<String>>,
    pub description: Option<HtmlText>,
    pub data_source: Option<String>,
    pub pub_year: Option<u32>,
    pub license: Option<String>,
    pub format: ReadingsFormat,
    #[serde(default)]
    pub external: ExternalModuleData,
}

#[derive(Debug)]
pub enum LeapYearHandler
{
    Skip,
    Index(u32),
}

impl ToString for LeapYearHandler
{
    fn to_string(&self) -> String 
    {
        match self 
        {
            Self::Skip => "skip".into(),
            Self::Index(i) => i.to_string(),
        }
    }
}

impl FromStr for LeapYearHandler
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        if s == "skip"
        {
            Ok(Self::Skip)
        }
        else if let Ok(i) = s.parse::<u32>()
        {
            Ok(Self::Index(i))
        }
        else 
        {
            Err(format!("LeapYearHandler must either be a number or \"skip\""))
        }
    }
}


impl Serialize for LeapYearHandler 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for LeapYearHandler 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReadingsFormat
{
    /// Represents readings for `count` number of days. Must have exactly `count` number of entries
    Daily
    {
        count: u32,
    },

    /// Represents readings for `count` number of weeks and are mapped to different days of the weeks.
    /// Must have exactly `count * 7` entries \
    /// Example: 
    /// - 0 = Sunday
    /// - 1 = Monday
    /// - 7 = Sunday2
    /// - ...
    Weekly
    {
        count: u32,
    },

    ///  Each day is mapped based on the current day of the month. Must have exactly 31 entries, each mapped to a day.
    Monthly,

    /// Each day is mapped based on the current day of the year. 
    /// Must have exactly `count * 365` or `count * 365 + 1` number of entires. 
    /// If leap year handler is set to Index(...), it must point to a valid entry.
    Yearly
    {
        count: u32,
        leap_year: LeapYearHandler
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadingsEntry
{
    pub id: EntryId,
    pub index: u32,
    pub readings: Vec<RefId>,
}

impl ReadingsEntry
{
    pub fn from_file(path: &str) -> Result<Vec<Self>, String>
    {
        let ret = utils::load_json_lines(path)?
            .into_iter()
            .map(|(l, _)| l)
            .collect();

        Ok(ret)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadingsModule
{
    pub config: ReadingsConfig,
    pub entries: Vec<ReadingsEntry>,
}

impl ReadingsModule
{
    pub fn load_json(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: ReadingsConfig = utils::load_toml(config_path)?;

        let dictionary_path = format!("{}/{}.jsonl", dir_path, name);
        let entries = ReadingsEntry::from_file(&dictionary_path)?;

        Ok(Self { 
            config,
            entries,
        })
    }

    pub fn get_reading(&self, start_date: ReadingsDate, date: ReadingsDate) -> Option<&ReadingsEntry>
    {
        if start_date > date
        {
            panic!("Start date must be either the same date as, or before the current date");
        }

        match &self.config.format
        {
            ReadingsFormat::Daily { count } => {
                let index = date.days_since(start_date) % count;
                Some(self.entries.iter().find(|e| e.index == index).unwrap())
            },
            ReadingsFormat::Weekly { count } => {
                let week_index = date.weeks_since(start_date) % count;
                let index = week_index * 7 + date.day_of_week() as u32 - 1;
                Some(self.entries.iter().find(|e| e.index == index).unwrap())
            },
            ReadingsFormat::Monthly => {
                let index = date.day() as u32 - 1;
                Some(self.entries.iter().find(|e| e.index == index).unwrap())
            },
            ReadingsFormat::Yearly { count, leap_year } => {
                if date.is_leap_day()
                {
                    match leap_year
                    {
                        LeapYearHandler::Skip => return None,
                        LeapYearHandler::Index(i) => return Some(self.entries.iter().find(|e| e.index == *i).unwrap()),
                    }
                }
                let year_index = (date.year() - start_date.year()) as u32 % count;
                let index = year_index * 365 + date.day_of_year() - 1;
                Some(self.entries.iter().find(|e| e.index == index).unwrap())
            },
        }
    }

    pub fn validate(&self, builder: &ValidationContextBuilder) -> Result<(), Vec<ModuleValidationError>>
    {
        let context = builder.build(Some(&self.config.name), &self.config.external);
        let mut errors = self.config.description.as_ref()
            .map(|d| d.validate(&context).err())
            .flatten()
            .map(|errors| errors.into_iter().map(|error| ModuleValidationError::HtmlError { error }).collect_vec())
            .unwrap_or_default();

        let entry_count = self.entries.len() as u32;
        match &self.config.format
        {
            ReadingsFormat::Daily { count } => {
                if entry_count != *count
                {
                    errors.push(ReadingsModuleValidationError::InvalidReadingsCount { 
                        required: Either::Left(*count), 
                        found: entry_count 
                    }.into());
                }
            },
            ReadingsFormat::Weekly { count } => {
                if entry_count != count * 7
                {
                    errors.push(ReadingsModuleValidationError::InvalidReadingsCount { 
                        required: Either::Left(*count * 7), 
                        found: entry_count 
                    }.into());
                }
            },
            ReadingsFormat::Monthly => {
                if entry_count != 31
                {
                    errors.push(ReadingsModuleValidationError::InvalidReadingsCount { 
                        required: Either::Left(31), 
                        found: entry_count 
                    }.into());
                }
            },
            ReadingsFormat::Yearly { count, leap_year } => match leap_year {
                LeapYearHandler::Skip => {
                    if entry_count != *count * 365
                    {
                        errors.push(ReadingsModuleValidationError::InvalidReadingsCount { 
                            required: Either::Left(*count * 365), 
                            found: entry_count 
                        }.into());
                    }
                },
                LeapYearHandler::Index(index) => {
                    if *index >= entry_count 
                    {
                        errors.push(ReadingsModuleValidationError::InvalidReadingsIndex { index: *index }.into());
                    }

                    if entry_count != *count * 365 && entry_count != *count * 365 + 1
                    {
                        errors.push(ReadingsModuleValidationError::InvalidReadingsCount { 
                            required: Either::Right((*count * 365, *count * 365 + 1)), 
                            found: entry_count 
                        }.into());
                    } 
                },
            },
        }

        let mut entry_indexes = self.entries.iter().map(|e| e.index).collect_vec();
        entry_indexes.sort();
        let deduped_indexes = entry_indexes.iter().dedup().collect_vec();

        if entry_indexes.len() != deduped_indexes.len()
        {
            errors.push(ReadingsModuleValidationError::InvalidEntryIndexes.into());
        }

        for (i, entry_index) in entry_indexes.iter().enumerate()
        {
            if i as u32 != *entry_index
            {
                errors.push(ReadingsModuleValidationError::InvalidEntryIndexes.into());
                break;
            }
        }

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

#[derive(Debug)]
pub enum ReadingsModuleValidationError
{
    InvalidReadingsIndex {
        index: u32,
    },
    InvalidReadingsCount
    {
        required: Either<u32, (u32, u32)>,
        found: u32,
    },
    InvalidEntryIndexes,
    InvalidRefId
    {
        id: RefId,
    }
}

impl fmt::Display for ReadingsModuleValidationError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            Self::InvalidReadingsCount { required, found } => match required {
                Either::Left(r) => write!(f, "Modules requires {} number of readings, but found {}", r, found),
                Either::Right((r_a, r_b)) => write!(f, "Module requires {} or {} number of readings, but found {}", r_a, r_b, found),
            },
            Self::InvalidReadingsIndex { index } => write!(f, "Invalid readings index {}", index),
            Self::InvalidEntryIndexes => write!(f, "Readings modules has invalid index values for its readings entries"),
            Self::InvalidRefId { id } => write!(f, "RefId {} cannot have a Bible version", id)
        }
    }
}