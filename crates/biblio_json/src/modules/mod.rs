pub mod bible;
pub mod dict;
pub mod xrefs;
pub mod strongs;
pub mod commentary;
pub mod notebook;

use std::{collections::HashMap, fmt::Display, sync::Arc};

use bible::BibleModule;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{core::{RefId, lang::Language}, html_text::{HtmlText, ast::AssetIdName}, modules::{bible::Verse, commentary::{CommentaryEntry, CommentaryModule}, dict::{DictEntry, DictModule}, strongs::{StrongsDefEntry, StrongsDefsModule, StrongsLinkEntry, StrongsLinksModule}, xrefs::{XRefEntry, XRefModule}}};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ExternalModuleData
{
    #[serde(default)]
    pub aliases: HashMap<AssetIdName, String>,
    #[serde(default)]
    pub assets: HashMap<AssetIdName, String>,
}

#[derive(Debug)]
pub enum ModuleValidationError
{
    BibleNotFound(String),
    WordRefIdInvalid(RefId),
    RefIdDoesNotExist(RefId, String),
}

impl Display for ModuleValidationError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            ModuleValidationError::BibleNotFound(name) => write!(f, "Bible '{}' does not exist.", name),
            ModuleValidationError::WordRefIdInvalid(ref_id) => write!(f, "RefId {} with word indexes is not valid in this context.", ref_id),
            ModuleValidationError::RefIdDoesNotExist(ref_id, bible) => write!(f, "RefId {} does not exist in bible '{}'", ref_id, bible),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleValidationContext<'a>
{
    pub bibles: &'a HashMap<String, Arc<BibleModule>>,
}

impl<'a> ModuleValidationContext<'a>
{
    pub fn ref_id_exists(&self, id: RefId, default_bible: Option<&BibleModule>)
    {
        
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Module
{
    Bible(#[serde_as(as = "Arc<_>")] Arc<BibleModule>),
    Dictionary(#[serde_as(as = "Arc<_>")] Arc<DictModule>),
    XRef(#[serde_as(as = "Arc<_>")] Arc<XRefModule>),
    StrongsDefs(#[serde_as(as = "Arc<_>")] Arc<StrongsDefsModule>),
    StrongsLinks(#[serde_as(as = "Arc<_>")] Arc<StrongsLinksModule>),
    Commentary(#[serde_as(as = "Arc<_>")] Arc<CommentaryModule>)
}

impl Module
{
    pub fn is_bible(&self) -> bool
    {
        match self 
        {
            Self::Bible(_) => true,
            _ => false,
        }
    }

    pub fn as_bible(&self) -> Option<Arc<BibleModule>>
    {
        match self 
        {
            Self::Bible(b) => Some(b.clone()),
            _ => None,
        }
    }

    pub fn is_dict(&self) -> bool
    {
        match self 
        {
            Self::Dictionary(_) => true,
            _ => false,
        }
    }

    pub fn as_dict(&self) -> Option<Arc<DictModule>>
    {
        match self
        {
            Self::Dictionary(d) => Some(d.clone()),
            _ => None,
        }
    }

    pub fn is_xrefs(&self) -> bool
    {
        match self 
        {
            Self::XRef(_) => true,
            _ => false,
        }
    }

    pub fn as_xrefs(&self) -> Option<Arc<XRefModule>>
    {
        match self
        {
            Self::XRef(x) => Some(x.clone()),
            _ => None,
        }
    }

    pub fn is_strongs_defs(&self) -> bool
    {
        match self 
        {
            Self::StrongsDefs(_) => true,
            _ => false,
        }
    }

    pub fn as_strongs_defs(&self) -> Option<Arc<StrongsDefsModule>>
    {
        match self 
        {
            Self::StrongsDefs(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn is_strongs_links(&self) -> bool
    {
        match self 
        {
            Self::StrongsLinks(_) => true,
            _ => false,
        }
    }
    
    pub fn as_strongs_links(&self) -> Option<Arc<StrongsLinksModule>>
    {
        match self 
        {
            Self::StrongsLinks(m) => Some(m.clone()),
            _ => None,
        }
    }

    pub fn is_commentary(&self) -> bool
    {
        match self 
        {
            Self::Commentary(_) => true,
            _ => false,
        }
    }

    pub fn as_commentary(&self) -> Option<Arc<CommentaryModule>>
    {
        match self 
        {
            Self::Commentary(c) => Some(c.clone()),
            _ => None,
        }
    }

    pub fn name(&self) -> &str 
    {
        match self 
        {
            Module::Bible(bible_module) => &bible_module.config.name,
            Module::Dictionary(dict_module) => &dict_module.config.name,
            Module::XRef(xref_module) => &xref_module.config.name,
            Module::StrongsDefs(strongs) => &strongs.config.name,
            Module::StrongsLinks(strongs_links_module) => &strongs_links_module.config.name,
            Module::Commentary(commentary_module) => &commentary_module.config.name,
        }
    }

    pub fn description(&self) -> Option<&HtmlText>
    {
        match self 
        {
            Module::Bible(bible_module) => bible_module.config.description.as_ref(),
            Module::Dictionary(dict_module) => dict_module.config.description.as_ref(),
            Module::XRef(xref_module) => xref_module.config.description.as_ref(),
            Module::StrongsDefs(strongs_defs_module) => strongs_defs_module.config.description.as_ref(),
            Module::StrongsLinks(strongs_links_module) => strongs_links_module.config.description.as_ref(),
            Module::Commentary(commentary_module) => commentary_module.config.description.as_ref(),
        }
    }

    pub fn language(&self) -> Option<Language>
    {
        match self
        {
            Module::Bible(bible_module) => bible_module.config.language,
            Module::Dictionary(dict_module) => dict_module.config.language,
            Module::XRef(xref_module) => xref_module.config.language,
            Module::StrongsDefs(strongs_defs_module) => strongs_defs_module.config.language,
            Module::StrongsLinks(strongs_links_module) => strongs_links_module.config.language,
            Module::Commentary(commentary_module) => commentary_module.config.language,
        }
    }

    pub fn authors(&self) -> Option<&Vec<String>>
    {
        match self
        {
            Module::Bible(bible_module) => bible_module.config.authors.as_ref(),
            Module::Dictionary(dict_module) => dict_module.config.authors.as_ref(),
            Module::XRef(xref_module) => xref_module.config.authors.as_ref(),
            Module::StrongsDefs(strongs_defs_module) => strongs_defs_module.config.authors.as_ref(),
            Module::StrongsLinks(strongs_links_module) => strongs_links_module.config.authors.as_ref(),
            Module::Commentary(commentary_module) => commentary_module.config.authors.as_ref(),
        }
    }

    pub fn pub_year(&self) -> Option<u32>
    {
        match self
        {
            Module::Bible(bible_module) => bible_module.config.pub_year,
            Module::Dictionary(dict_module) => dict_module.config.pub_year,
            Module::XRef(xref_module) => xref_module.config.pub_year,
            Module::StrongsDefs(strongs_defs_module) => strongs_defs_module.config.pub_year,
            Module::StrongsLinks(strongs_links_module) => strongs_links_module.config.pub_year,
            Module::Commentary(commentary_module) => commentary_module.config.pub_year,
        }
    }

    pub fn external(&self) -> &ExternalModuleData
    {
        match self
        {
            Module::Bible(bible_module) => &bible_module.config.external,
            Module::Dictionary(dict_module) => &dict_module.config.external,
            Module::XRef(xref_module) => &xref_module.config.external,
            Module::StrongsDefs(strongs_defs_module) => &strongs_defs_module.config.external,
            Module::StrongsLinks(strongs_links_module) => &strongs_links_module.config.external,
            Module::Commentary(commentary_module) => &commentary_module.config.external,
        }
    }

    pub fn validate(&self, context: &ModuleValidationContext) -> Result<(), Vec<ModuleValidationError>>
    {
        match self 
        {
            Module::Bible(_) => Ok(()),
            Module::Dictionary(_) => Ok(()),
            Module::XRef(xref_module) => xref_module.validate(context),
            Module::StrongsDefs(_) => Ok(()),
            Module::StrongsLinks(strongs_links) => strongs_links.validate(context),
            Module::Commentary(commentary_module) => commentary_module.validate(context),
        }
    }
}

pub type EntryId = u32;

#[derive(Debug, Clone, Copy)]
pub enum ModuleEntry<'a>
{
    Dictionary(&'a DictEntry),
    StrongsDef(&'a StrongsDefEntry),
    StrongsLink(&'a StrongsLinkEntry),
    XRef(&'a XRefEntry),
    Commentary(&'a CommentaryEntry),
    Verse(&'a Verse),
}

impl<'a> ModuleEntry<'a>
{
    pub fn is_dictionary(&self) -> bool
    {
        match self 
        {
            Self::Dictionary(_) => true,
            _ => false,
        }
    }

    pub fn as_dictionary(&self) -> Option<&'a DictEntry>
    {
        match self 
        {
            Self::Dictionary(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleEntryRef
{
    pub module: String,
    pub entry_id: EntryId,
}
