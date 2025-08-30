use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;

use crate::{core::{RefId, lang::Language}, html_text::HtmlText, modules::{ExternalModuleData, ModuleValidationContext, ModuleValidationError}, utils};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct XRefsConfig
{
    pub name: String,
    pub authors: Option<Vec<String>>,
    pub language: Option<Language>,
    pub description: Option<HtmlText>,
    pub data_source: Option<String>,
    pub pub_year: Option<u32>,
    pub license: Option<String>,
    pub bible: Option<String>,
    #[serde(default)]
    pub external: ExternalModuleData,
}

#[derive(Debug, Clone)]
pub enum XRef 
{
    Directed 
    {
        source: RefId,
        targets: Vec<RefId>,
        note: Option<String>,
        id: u32,
    },
    Mutual 
    {
        refs: Vec<RefId>,
        note: Option<String>,
        id: u32,
    },
}

impl XRef
{
    pub fn from_file(path: &str) -> Result<Vec<Self>, String>
    {
        let ret = utils::load_json_lines(path)?
            .into_iter()
            .map(|(l, _)| l)
            .collect();

        Ok(ret)
    }

    pub fn has_source(&self, ref_id: &RefId) -> bool
    {
        match self 
        {
            Self::Directed { source, targets: _, note: _, id: _ } => source == ref_id,
            Self::Mutual { refs, note: _, id: _ } => refs.iter().find(|r| **r == *ref_id).is_some()
        }
    }

    pub fn id(&self) -> u32 
    {
        match self 
        {
            XRef::Directed { source: _, targets: _, note: _, id } => *id,
            XRef::Mutual { refs: _, note: _, id } => *id,
        }
    }

    pub fn collect_ref_ids(&self) -> Vec<RefId>
    {
        match self {
            XRef::Directed { source, targets, note: _, id: _ } => {
                let mut ids = targets.iter().map(|t| t.clone()).collect_vec();
                ids.push(source.clone());
                ids
            },
            XRef::Mutual { refs, note: _, id: _ } => refs.iter().map(|r| r.clone()).collect_vec(),
        }
    }
}

impl Serialize for XRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        // Use human-readable (JSON) for tagged, otherwise compact for bincode
        if serializer.is_human_readable() {
            // Use the default derived implementation for JSON
            #[derive(Serialize)]
            #[serde(tag = "type", rename_all = "snake_case")]
            enum Helper<'a> 
            {
                Directed 
                {
                    source: &'a RefId,
                    targets: &'a Vec<RefId>,
                    note: &'a Option<String>,
                    id: u32,
                },
                Mutual 
                {
                    refs: &'a Vec<RefId>,
                    note: &'a Option<String>,
                    id: u32,
                },
            }
            match self 
            {
                XRef::Directed { source, targets, note, id } => 
                {
                    Helper::Directed { source, targets, note, id: *id }.serialize(serializer)
                }
                XRef::Mutual { refs, note, id } => 
                {
                    Helper::Mutual { refs, note, id: *id }.serialize(serializer)
                }
            }
        } 
        else 
        {
            // Compact binary for bincode
            match self 
            {
                XRef::Directed { source, targets, note, id } => 
                {
                    (0u8, source, targets, note, id).serialize(serializer)
                }
                XRef::Mutual { refs, note, id } => 
                {
                    (1u8, refs, note, id).serialize(serializer)
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for XRef 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        if deserializer.is_human_readable() 
        {
            #[derive(Deserialize)]
            #[serde(tag = "type", rename_all = "snake_case")]
            enum Helper 
            {
                Directed 
                {
                    source: RefId,
                    targets: Vec<RefId>,
                    note: Option<String>,
                    id: u32,
                },
                Mutual 
                {
                    refs: Vec<RefId>,
                    note: Option<String>,
                    id: u32,
                },
            }
            match Helper::deserialize(deserializer)? 
            {
                Helper::Directed { source, targets, note, id } => Ok(XRef::Directed { source, targets, note, id }),
                Helper::Mutual { refs, note, id } => Ok(XRef::Mutual { refs, note, id }),
            }
        } 
        else 
        {
            use serde::de::SeqAccess;
            use serde::de::Visitor;
            use std::fmt;

            struct XRefVisitor;

            impl<'de> Visitor<'de> for XRefVisitor 
            {
                type Value = XRef;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result 
                {
                    formatter.write_str("XRef tuple")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where A: SeqAccess<'de>,
                {
                    let tag: u8 = seq.next_element()?.ok_or_else(|| A::Error::custom("missing tag"))?;
                    match tag 
                    {
                        0 => 
                        {
                            let source: RefId = seq.next_element()?.ok_or_else(|| A::Error::custom("missing source"))?;
                            let targets: Vec<RefId> = seq.next_element()?.ok_or_else(|| A::Error::custom("missing targets"))?;
                            let note: Option<String> = seq.next_element()?.ok_or_else(|| A::Error::custom("missing note"))?;
                            let id: u32 = seq.next_element()?.ok_or_else(|| A::Error::custom("missing id"))?;
                            Ok(XRef::Directed { source, targets, note, id })
                        }
                        1 => 
                        {
                            let refs: Vec<RefId> = seq.next_element()?.ok_or_else(|| A::Error::custom("missing refs"))?;
                            let note: Option<String> = seq.next_element()?.ok_or_else(|| A::Error::custom("missing note"))?;
                            let id: u32 = seq.next_element()?.ok_or_else(|| A::Error::custom("missing id"))?;
                            Ok(XRef::Mutual { refs, note, id })
                        }
                        _ => Err(A::Error::custom("invalid tag for XRef")),
                    }
                }
            }

            deserializer.deserialize_tuple(5, XRefVisitor)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XRefModule
{
    pub config: XRefsConfig,
    pub refs: Vec<XRef>,
}

impl XRefModule
{
    pub fn load_json(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: XRefsConfig = utils::load_toml(config_path)?;

        let dictionary_path = format!("{}/{}.jsonl", dir_path, name);
        let refs = XRef::from_file(&dictionary_path)?;

        Ok(Self { 
            config,
            refs,
        })
    }

    pub fn validate(&self, context: &ModuleValidationContext) -> Result<(), Vec<ModuleValidationError>>
    {
        if let Some(bible_name) = &self.config.bible
        {
            let Some(bible) = context.bibles.get(bible_name) else
            {
                return Err(vec![ModuleValidationError::BibleNotFound(bible_name.clone())])
            };

            let mut errors = vec![];
            for r in self.refs.iter().map(|r| r.collect_ref_ids().into_iter()).flatten()
            {
                if !bible.source.id_exists(&r)
                {
                    errors.push(ModuleValidationError::RefIdDoesNotExist(r, bible_name.clone()));
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
        else  
        {
            let mut errors = vec![];
            for r in self.refs.iter().map(|r| r.collect_ref_ids().into_iter()).flatten()
            {
                if r.is_word()
                {
                    errors.push(ModuleValidationError::WordRefIdInvalid(r));
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
}