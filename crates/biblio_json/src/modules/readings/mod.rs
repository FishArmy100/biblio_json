pub mod date;

use std::str::FromStr;

use itertools::{Either, Itertools};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{core::RefId, html_text::HtmlText, modules::{EntryId, ExternalModuleData, ModuleValidationError}, utils, validation::ValidationContextBuilder};

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
                    errors.push(ModuleValidationError::InvalidReadingsCount { 
                        required: Either::Left(*count), 
                        found: entry_count 
                    });
                }
            },
            ReadingsFormat::Weekly { count } => {
                if entry_count != count * 7
                {
                    errors.push(ModuleValidationError::InvalidReadingsCount { 
                        required: Either::Left(*count * 7), 
                        found: entry_count 
                    });
                }
            },
            ReadingsFormat::Monthly => {
                if entry_count != 31
                {
                    errors.push(ModuleValidationError::InvalidReadingsCount { 
                        required: Either::Left(31), 
                        found: entry_count 
                    });
                }
            },
            ReadingsFormat::Yearly { count, leap_year } => match leap_year {
                LeapYearHandler::Skip => {
                    if entry_count != *count * 365
                    {
                        errors.push(ModuleValidationError::InvalidReadingsCount { 
                            required: Either::Left(*count * 365), 
                            found: entry_count 
                        });
                    }
                },
                LeapYearHandler::Index(index) => {
                    if *index >= entry_count 
                    {
                        errors.push(ModuleValidationError::InvalidReadingsIndex { index: *index });
                    }

                    if entry_count != *count * 365 || entry_count != *count * 365 + 1
                    {
                        errors.push(ModuleValidationError::InvalidReadingsCount { 
                            required: Either::Right((*count * 365, *count * 365 + 1)), 
                            found: entry_count 
                        });
                    } 
                },
            },
        }

        let mut entry_indexes = self.entries.iter().map(|e| e.index).collect_vec();
        entry_indexes.sort();
        let deduped_indexes = entry_indexes.iter().dedup().collect_vec();

        if entry_indexes.len() != deduped_indexes.len()
        {
            errors.push(ModuleValidationError::InvalidEntryIndexes);
        }

        for (i, entry_index) in entry_indexes.iter().enumerate()
        {
            if i as u32 != *entry_index
            {
                errors.push(ModuleValidationError::InvalidEntryIndexes);
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