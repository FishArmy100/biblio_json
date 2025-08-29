pub mod bible;
pub mod dict;
pub mod xrefs;
pub mod strongs;
pub mod commentary;

use std::{collections::HashMap, fmt::Display, sync::Arc};

use bible::BibleModule;

use crate::{core::RefId, modules::{commentary::CommentaryModule, dict::DictModule, strongs::{StrongsDefsModule, StrongsLinksModule}, xrefs::XRefModule}};

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

#[derive(Debug)]
pub enum Module
{
    Bible(Arc<BibleModule>),
    Dictionary(Arc<DictModule>),
    XRef(Arc<XRefModule>),
    StrongsDefs(Arc<StrongsDefsModule>),
    StrongsLinks(Arc<StrongsLinksModule>),
    Commentary(Arc<CommentaryModule>)
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

    pub fn get_name(&self) -> &str 
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
