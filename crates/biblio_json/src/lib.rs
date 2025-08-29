pub(crate) mod utils;
pub mod modules;
pub mod core;
pub mod html_text;
use std::{collections::HashMap, fmt::Display, path::Path, sync::Arc};
use serde::{Deserialize, Serialize};

use crate::{modules::{Module, ModuleValidationContext, ModuleValidationError, bible::BibleModule, commentary::CommentaryModule, dict::DictModule, strongs::{StrongsDefsModule, StrongsLinksModule}, xrefs::XRefModule}};

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

// #[derive(Debug, Clone)]
// pub struct LoadPackageError
// {
//     pub file: Option<String>,
//     pub message: String,
// }

pub enum LoadPackageError
{
    ModuleLoadingError {
        path: String,
        error: String,
    },
    ModuleValidationError {
        name: String,
        error: ModuleValidationError,
    },
    PackagePathNotDirectory(String),
    PackageConfigNotFound(String),
    PackageConfigError {
        path: String,
        error: String,
    },
    GlobError(String),
    ExpectedParent(String),
    ExpectedStem(String),
    PackagePathDoesNotExist(String),
}

impl Display for LoadPackageError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            LoadPackageError::ModuleLoadingError { path, error } => write!(f, "Error loading module '{}':\n{}", path, error),
            LoadPackageError::ModuleValidationError { name: path, error } => write!(f, "Validation error in module '{}': {}", path, error),
            LoadPackageError::PackagePathNotDirectory(path) => write!(f, "Package path '{}' is not a directory.", path),
            LoadPackageError::PackageConfigNotFound(path) => write!(f, "Package config not found at path '{}'", path),
            LoadPackageError::PackageConfigError { path, error } => write!(f, "Error loading package config '{}':\n{}", path, error),
            LoadPackageError::GlobError(error) => write!(f, "Glob error: {}", error),
            LoadPackageError::ExpectedParent(path) => write!(f, "Expected file {} to have a parent", path),
            LoadPackageError::ExpectedStem(path) => write!(f, "Expected file {} to have a stem", path),
            LoadPackageError::PackagePathDoesNotExist(path) => {
                if let Some(cwd) = std::env::current_dir().map(|p| p.display().to_string()).ok()
                {
                    write!(f, "Path for package does not exist '{}' in current directory '{}'", path, cwd) 
                }
                else 
                {
                    write!(f, "Path for package does not exist '{}'", path)    
                }
            },
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

        if !path.exists()
        {
            return Err(vec![LoadPackageError::PackagePathDoesNotExist(path.display().to_string())]);
        }

        if !path.is_dir()
        {
            return Err(vec![LoadPackageError::PackagePathNotDirectory(path.display().to_string())]);
        }

        let config_path = path.join(Path::new(PACKAGE_FILE_NAME));
        let file = utils::load_file(&config_path).map_err(|e| vec![LoadPackageError::PackageConfigNotFound(e)])?;
        
        let config = match toml::from_str::<PackageConfig>(&file) {
            Ok(ok) => ok,
            Err(e) => return Err(vec![LoadPackageError::PackageConfigError {
                path: config_path.to_str().unwrap().to_owned(),
                error: e.to_string(),
            }])
        };

        let modules = match &config.module_paths {
            Some(paths) => Self::load_modules(dir_path, paths)?,
            None => vec![]
        };

        Self::validate_modules(&modules)?;

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

    fn validate_modules(modules: &Vec<Module>) -> Result<(), Vec<LoadPackageError>>
    {
        let bibles = modules.iter().filter_map(|m| match m {
            Module::Bible(b) => Some(b.clone()),
            _ => None,
        }).map(|b| (b.config.name.clone(), b)).collect::<HashMap<_, _>>();

        let context = ModuleValidationContext {
            bibles: &bibles
        };

        let mut errors = vec![];
        for m in modules
        {
            if let Err(errs) = m.validate(&context)
            {
                errors.extend(errs.into_iter().map(|error| LoadPackageError::ModuleValidationError { 
                    name: m.get_name().to_string(), 
                    error,
                }));
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

    fn load_module(base_dir: &str, pattern: &str, f: impl Fn(&str, &str) -> Result<Module, String>) -> Result<Vec<Module>, LoadPackageError>
    {
        let full_path = format!("{}/{}", base_dir, pattern);

        let paths = glob::glob(&full_path).map_err(|e| LoadPackageError::GlobError(e.to_string()))?;
        paths.filter_map(|entry| -> Option<Result<Module, LoadPackageError>> {
            let entry = match entry {
                Ok(ok) => ok,
                Err(e) => return Some(Err(LoadPackageError::GlobError(e.to_string()))),
            };

            let path = Path::new(&entry);

            let ext = path.extension().map(|s| s.to_str()).flatten();
            if ext != Some("toml")
            {
                return None;
            }
            
            let dir = match path.parent() {
                Some(s) => s,
                None => return Some(Err(LoadPackageError::ExpectedParent(path.display().to_string())))
            }.to_str().unwrap();

            let name = match path.file_stem() {
                Some(s) => s,
                None => return Some(Err(LoadPackageError::ExpectedStem(path.display().to_string())))
            }.to_str().unwrap();

            Some(f(dir, name).map_err(|e| LoadPackageError::ModuleLoadingError { 
                path: path.canonicalize().ok().map(|p| p.display().to_string()).unwrap(), 
                error: e
            }))
        }).collect()
    }
}