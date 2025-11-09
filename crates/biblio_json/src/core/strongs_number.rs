use std::{fmt::Display, str::FromStr};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

lazy_static::lazy_static!
{
    static ref STRONGS_REGEX: Regex = Regex::new(r"^(?P<lang>[HG])(?P<number>\d+)$").unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrongsLang
{
    Hebrew,
    Greek,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StrongsNumber
{
    pub lang: StrongsLang,
    pub number: u32,
}

impl Display for StrongsNumber
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        let letter = match self.lang {
            StrongsLang::Hebrew => "H",
            StrongsLang::Greek => "G",
        };

        write!(f, "{}{}", letter, self.number)
    }
}

impl FromStr for StrongsNumber
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        match STRONGS_REGEX.captures(s) 
        {
            Some(captures) => {
                let lang = match captures.name("lang").unwrap().as_str()
                {
                    "H" => StrongsLang::Hebrew,
                    "G" => StrongsLang::Greek,
                    _ => return Err(format!("String {} is not a valid strongs number", s))
                };

                let number = captures.name("number").unwrap().as_str();
                let number = match u32::from_str(number) {
                    Ok(ok) => ok,
                    Err(e) => return Err(e.to_string()),
                };

                Ok(Self {
                    lang,
                    number
                })
            },
            None => Err(format!("String {} is not a valid strongs number", s)),
        }
    }
}

impl Serialize for StrongsNumber 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for StrongsNumber 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        StrongsNumber::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() 
    {
        let sn = StrongsNumber { lang: StrongsLang::Hebrew, number: 123 };
        assert_eq!(sn.to_string(), "H123");

        let sn = StrongsNumber { lang: StrongsLang::Greek, number: 456 };
        assert_eq!(sn.to_string(), "G456");
    }

    #[test]
    fn test_from_str_valid() 
    {
        let sn: StrongsNumber = "H123".parse().unwrap();
        assert_eq!(sn.lang, StrongsLang::Hebrew);
        assert_eq!(sn.number, 123);

        let sn: StrongsNumber = "G456".parse().unwrap();
        assert_eq!(sn.lang, StrongsLang::Greek);
        assert_eq!(sn.number, 456);
    }

    #[test]
    fn test_from_str_invalid() 
    {
        assert!("X123".parse::<StrongsNumber>().is_err());
        assert!("H12A".parse::<StrongsNumber>().is_err());
        assert!("123".parse::<StrongsNumber>().is_err());
        assert!("".parse::<StrongsNumber>().is_err());
    }

    #[test]
    fn test_serde_serialize() 
    {
        let sn = StrongsNumber { lang: StrongsLang::Hebrew, number: 789 };
        let json = serde_json::to_string(&sn).unwrap();
        assert_eq!(json, "\"H789\"");
    }

    #[test]
    fn test_serde_deserialize() 
    {
        let json = "\"G321\"";
        let sn: StrongsNumber = serde_json::from_str(json).unwrap();
        assert_eq!(sn.lang, StrongsLang::Greek);
        assert_eq!(sn.number, 321);
    }

    #[test]
    fn test_serde_deserialize_invalid() 
    {
        let json = "\"X999\"";
        let result: Result<StrongsNumber, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
