use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{core::strongs_number::StrongsNumber, utils};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrongsEntry
{
    pub strongs_ref: StrongsNumber,
    pub word: String,
    pub definitions: Vec<String>,
    pub derivation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrongsDefsConfig
{
    pub name: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub source: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug)]
pub struct StrongsDefsModule
{
    pub config: StrongsDefsConfig,
    pub index_map: HashMap<StrongsNumber, u32>,
    pub defs: Vec<StrongsEntry>,
}

impl StrongsDefsModule
{
    pub fn load(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: StrongsDefsConfig = utils::load_toml(config_path)?;

        let bible_path = format!("{}/{}.jsonl", dir_path, name);
        let defs = StrongsEntry::from_file(&bible_path)?;

        let index_map: HashMap<_, _> = defs.iter().enumerate().map(|(i, d)| {
            (d.strongs_ref.clone(), i as u32)
        }).collect();

        Ok(Self { 
            config,
            index_map,
            defs,
        })
    }

    pub fn get_def(&self, num: &StrongsNumber) -> Option<&StrongsEntry>
    {
        match self.index_map.get(num)
        {
            Some(idx) => Some(&self.defs[*idx as usize]),
            None => None
        }
    }
}

impl StrongsEntry
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