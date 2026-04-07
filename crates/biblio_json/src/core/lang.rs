use std::{ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Language(pub isolang::Language);

impl Language
{
    pub fn new(lang: &str) -> Result<Self, String>
    {
        if let Some(l) = isolang::Language::from_639_1(lang)
        {
            Ok(Self(l))
        }
        else if let Some(l) = isolang::Language::from_639_3(lang)
        {
            Ok(Self(l))
        }
        else if let Some(l) = isolang::Language::from_autonym(lang)
        {
            Ok(Self(l))
        }
        else if let Some(l) = isolang::Language::from_name_lowercase(lang)
        {
            Ok(Self(l))
        }
        else 
        {
            Err(format!("Unknown language code `{}`", lang))    
        }
    }

    pub fn name(&self) -> &'static str 
    {
        self.0.to_name()
    }

    pub fn autonym(&self) -> Option<&'static str>
    {
        self.0.to_autonym()
    }
}

impl Deref for Language
{
    type Target = isolang::Language;

    fn deref(&self) -> &Self::Target 
    {
        &self.0
    }
}

impl std::fmt::Display for Language
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "{}", self.0.to_639_1().unwrap_or(self.0.to_639_3()))
    }
}

impl FromStr for Language
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        Self::new(s)
    }
}

impl Serialize for Language
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Language
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests 
{
    use super::*;

    #[test]
    fn test_from_string()
    {
        assert!(Language::from_str("en").is_ok());
        assert!(Language::from_str("English").is_ok());
        assert!(Language::from_str("english").is_ok());
        assert!(Language::from_str("paoigh").is_err());
    }
}