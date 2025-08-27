pub(crate) mod utils;
pub mod modules;
pub mod core;
pub mod html_text;
use std::{collections::HashMap, fmt::Display, path::Path, sync::Arc};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{core::RefId, modules::{Module, bible::BibleModule, commentary::CommentaryModule, dict::DictModule, strongs::{StrongsDefsModule, StrongsLinksModule}, xrefs::{XRef, XRefModule}}};

pub const PACKAGE_FILE_NAME: &str = "biblio-json.toml";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PackageConfig
{
    pub name: String,
    pub authors: Vec<String>,
    pub license: String,
    pub module_paths: Option<ModulePaths>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ModulePaths
{
    pub bibles: Option<String>,
    pub dictionaries: Option<String>,
    pub xrefs: Option<String>,
    pub strongs_defs: Option<String>,
    pub strongs_links: Option<String>,
    pub commentaries: Option<String>,
}

pub enum PackageValidationError
{
    InvalidRefId
    {
        id: RefId,
        bible_name: String,
        xref_name: String,
        line: usize,
    }
}

impl Display for PackageValidationError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            Self::InvalidRefId { id, bible_name, xref_name, line } => {
                write!(f, "RefId {} in xref module {} on line {} does not exist in Bible {}", id, xref_name, line, bible_name)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadPackageError
{
    pub file: Option<String>,
    pub message: String,
}

impl LoadPackageError
{
    pub fn new(message: String) -> Self 
    {
        Self {
            file: None,
            message,
        }
    }
}

impl Display for LoadPackageError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match &self.file
        {
            Some(file) => write!(f, "Error in file {}\n{}", file, self.message),
            None => write!(f, "{}", self.message)
        }
    }
}

#[derive(Debug)]
pub struct Package 
{
    pub name: String,
    pub authors: Vec<String>,
    pub license: String,
    pub modules: HashMap<String, Module>
}

impl Package
{
    pub fn load(dir_path: &str) -> Result<Self, Vec<LoadPackageError>>
    {
        let path = Path::new(dir_path);

        if !path.is_dir()
        {
            return Err(vec![LoadPackageError::new(format!("Provided path: {dir_path}, must be a directory"))]);
        }

        let config_path = path.join(Path::new(PACKAGE_FILE_NAME));
        let file = utils::load_file(&config_path).map_err(|e| vec![LoadPackageError {
            file: config_path.to_str().map(|s| s.to_owned()),
            message: e.to_string(),
        }])?;
        
        let config = match toml::from_str::<PackageConfig>(&file) {
            Ok(ok) => ok,
            Err(e) => return Err(vec![LoadPackageError::new(e.to_string())])
        };

        let modules = match &config.module_paths {
            Some(paths) => Self::load_modules(dir_path, paths)?,
            None => vec![]
        };

        Ok(Self {
            name: config.name,
            authors: config.authors,
            license: config.license,
            modules: modules.into_iter().map(|m| (m.get_name().to_owned(), m)).collect()
        })
    }

    pub fn get_mod(&self, name: &str) -> Option<&Module>
    {
        self.modules.get(name)
    }

    fn load_modules(root: &str, paths: &ModulePaths) -> Result<Vec<Module>, Vec<LoadPackageError>>
    {
        let mut modules = vec![];
        let mut errors = vec![];
        
        if let Some(bibles_path) = &paths.bibles
        {
            let result = Self::load_module(root, &bibles_path, |dir, name| 
            {
                Ok(Module::Bible(Arc::new(BibleModule::load(dir, name)?)))
            });

            match result
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(dictionary_paths) = &paths.dictionaries
        {
            let result =  Self::load_module(root, &dictionary_paths, |dir, name| 
            {
                Ok(Module::Dictionary(Arc::new(DictModule::load(dir, name)?)))
            });

            match result
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(xref_paths) = &paths.xrefs
        {
            let result =  Self::load_module(root, &xref_paths, |dir, name| 
            {
                Ok(Module::XRef(Arc::new(XRefModule::load(dir, name)?)))
            });

            match result
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(strongs_defs) = &paths.strongs_defs
        {
            let result = Self::load_module(root, strongs_defs, |dir, name| {
                Ok(Module::StrongsDefs(Arc::new(StrongsDefsModule::load(dir, name)?)))
            });

            match result 
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(strongs_links) = &paths.strongs_links
        {
            let result = Self::load_module(root, strongs_links, |dir, name| {
                Ok(Module::StrongsLinks(Arc::new(StrongsLinksModule::load(dir, name)?)))
            });

            match result 
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(commentaries) = &paths.commentaries
        {
            let result = Self::load_module(root, commentaries, |dir, name| {
                Ok(Module::Commentary(Arc::new(CommentaryModule::load(dir, name)?)))
            });

            match result 
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if errors.len() > 0
        {
            Err(errors)
        }
        else 
        {
            Ok(modules)    
        }
    }

    pub fn validate(&self) -> Result<(), Vec<PackageValidationError>>
    {
        let mut errors = vec![];

        let bibles = self.modules.values().filter_map(|m| match m {
            Module::Bible(b) => Some(b),
            _ => None,
        }).collect_vec();

        let xrefs = self.modules.values().filter_map(|m| match m {
            Module::XRef(b) => Some(b),
            _ => None,
        }).collect_vec();

        for bible in bibles
        {
            let bible_name = &bible.config.name;
            for xref in &xrefs
            {
                let xref_name = &xref.config.name;

                xref.refs.iter().enumerate().map(|(i, r)| match r {
                    XRef::Directed { source, source_text: _, targets, note: _ } => {
                        let mut ids = targets.iter().map(|t| (i, t.clone())).collect_vec();
                        ids.push((i, source.clone()));
                        ids
                    },
                    XRef::Mutual { refs, note: _ } => refs.iter().map(|r| (i, r.id.clone())).collect_vec(),
                })
                .flatten()
                .for_each(|(i, id)| {
                    if !bible.source.id_exists(&id)
                    {
                        errors.push(PackageValidationError::InvalidRefId { 
                            id: id.clone(), 
                            bible_name: bible_name.clone(), 
                            xref_name: xref_name.clone(),
                            line: i + 1,
                        });
                    }
                });
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

    fn load_module(base_dir: &str, pattern: &str, f: impl Fn(&str, &str) -> Result<Module, String>) -> Result<Vec<Module>, LoadPackageError>
    {
        let full_path = format!("{}/{}", base_dir, pattern);

        glob::glob(&full_path).map_err(|e| LoadPackageError::new(e.to_string()))?.filter_map(|entry| -> Option<Result<Module, LoadPackageError>> {
            let entry = match entry {
                Ok(ok) => ok,
                Err(e) => return Some(Err(LoadPackageError::new(e.to_string()))),
            };

            let path = Path::new(&entry);

            let ext = path.extension().map(|s| s.to_str()).flatten();
            if ext != Some("toml")
            {
                return None;
            }
            
            let dir = match path.parent() {
                Some(s) => s,
                None => return Some(Err(LoadPackageError::new(format!("Expected path {} to have a parent", path.display()))))
            }.to_str().unwrap();

            let name = match path.file_stem() {
                Some(s) => s,
                None => return Some(Err(LoadPackageError::new(format!("Expected path {} to have a stem", path.display()))))
            }.to_str().unwrap();

            Some(f(dir, name).map_err(|e| LoadPackageError { 
                file: path.canonicalize().ok().map(|p| p.display().to_string()), 
                message: e
            }))
        }).collect()
    }
}