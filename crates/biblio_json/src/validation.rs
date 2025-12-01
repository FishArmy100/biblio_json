use std::{collections::HashMap, sync::Arc};

use crate::{core::{RefId, RefIdInner}, modules::{ExternalModuleData, Module, ModuleId, bible::BibleModule}};

#[derive(Debug, Clone)]
pub enum RefIdValidationError
{
    DoesNotExist,
    NeedsBible,
}

impl std::fmt::Display for RefIdValidationError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            RefIdValidationError::DoesNotExist => write!(f, "RefId does not exist"),
            RefIdValidationError::NeedsBible => write!(f, "RefId needs a bible reference"),
        }
    }
}

pub struct ValidationContextBuilder<'a>
{
    pub bibles: &'a HashMap<ModuleId, Arc<BibleModule>>,
    pub all_modules: &'a HashMap<ModuleId, Module>,
}

impl<'a> ValidationContextBuilder<'a>
{
    pub fn build(&'a self, default_bible: Option<&'a ModuleId>, external: &'a ExternalModuleData) -> ValidationContext<'a>
    {
        ValidationContext
        {
            bibles: self.bibles,
            all_modules: self.all_modules,
            default_bible,
            external,
        }
    }
} 

#[derive(Debug, Clone)]
pub struct ValidationContext<'a>
{
    pub bibles: &'a HashMap<ModuleId, Arc<BibleModule>>,
    pub all_modules: &'a HashMap<ModuleId, Module>,
    pub default_bible: Option<&'a ModuleId>,
    pub external: &'a ExternalModuleData,
}

impl<'a> ValidationContext<'a>
{
    pub fn validate_ref_id(&self, id: &RefId) -> Result<(), RefIdValidationError>
    {
        if let Some(default_bible) = self.default_bible.as_ref().map(|b| self.bibles.get(*b)).flatten()
        {
            match default_bible.source.id_exists(&id)
            {
                true => Ok(()),
                false => Err(RefIdValidationError::DoesNotExist),
            }
        }
        else if let Some(bible) = id.bible.as_ref().map(|b| self.bibles.get(b)).flatten()
        {
            match bible.source.id_exists(&id)
            {
                true => Ok(()),
                false => Err(RefIdValidationError::DoesNotExist),
            }
        }
        else 
        {
            match &id.id
            {
                RefIdInner::Single(atom) => 
                {
                    if atom.is_word()
                    {
                        return Err(RefIdValidationError::NeedsBible)
                    }
                },
                RefIdInner::Range { from, to } => 
                {
                    if from.book() != to.book()
                    {
                        return Err(RefIdValidationError::NeedsBible);
                    }

                    if from.is_word() || to.is_word()
                    {
                        return Err(RefIdValidationError::NeedsBible)
                    }
                },
            }

            Ok(())
        }
    }
}