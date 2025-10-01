use std::{fmt::Display, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};


lazy_static::lazy_static!
{
    static ref COLOR_REGEX: Regex = Regex::new(r"^^#(?P<r>[0-9a-f]{2})(?P<g>[0-9a-f]{2})(?P<b>[0-9a-f]{2})$").unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HighlightColor
{
    pub r: u8,
    pub g: u8,
    pub b: u8,    
}

impl Display for HighlightColor
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl FromStr for HighlightColor
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        let Some(captures) = COLOR_REGEX.captures(s) else {
            return Err(format!("'{}' is not a valid hex color", s));
        };

        let r = u8::from_str_radix(captures.name("r").unwrap().as_str(), 16).unwrap();
        let g = u8::from_str_radix(captures.name("g").unwrap().as_str(), 16).unwrap();
        let b = u8::from_str_radix(captures.name("b").unwrap().as_str(), 16).unwrap();

        Ok(Self {
            r,
            g,
            b,
        })
    }
}

impl Serialize for HighlightColor 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for HighlightColor 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        HighlightColor::from_str(&s).map_err(serde::de::Error::custom)
    }
}
#[cfg(test)]
mod tests 
{
    use super::*;
    use serde_json;
    use std::collections::HashSet;

    #[test]
    fn test_display_fmt() 
    {
        let color = HighlightColor { r: 255, g: 0, b: 128 };
        assert_eq!(color.to_string(), "#ff0080");
        assert_eq!(format!("{}", color), "#ff0080");
    }

    #[test]
    fn test_from_str_valid() 
    {
        let color: HighlightColor = "#ff0080".parse().unwrap();
        assert_eq!(color, HighlightColor { r: 255, g: 0, b: 128 });

        let color: HighlightColor = "#000000".parse().unwrap();
        assert_eq!(color, HighlightColor { r: 0, g: 0, b: 0 });

        let color: HighlightColor = "#abcdef".parse().unwrap();
        assert_eq!(color, HighlightColor { r: 0xab, g: 0xcd, b: 0xef });
    }

    #[test]
    fn test_from_str_invalid() 
    {
        assert!("#ff008".parse::<HighlightColor>().is_err());
        assert!("ff0080".parse::<HighlightColor>().is_err());
        assert!("#gg0080".parse::<HighlightColor>().is_err());
        assert!("#ff00800".parse::<HighlightColor>().is_err());
        assert!("#ff0080ff".parse::<HighlightColor>().is_err());
        assert!("#zzzzzz".parse::<HighlightColor>().is_err());
        assert!("".parse::<HighlightColor>().is_err());
    }

    #[test]
    fn test_serialize() 
    {
        let color = HighlightColor { r: 255, g: 0, b: 128 };
        let json = serde_json::to_string(&color).unwrap();
        assert_eq!(json, "\"#ff0080\"");
    }

    #[test]
    fn test_deserialize_valid() 
    {
        let json = "\"#ff0080\"";
        let color: HighlightColor = serde_json::from_str(json).unwrap();
        assert_eq!(color, HighlightColor { r: 255, g: 0, b: 128 });
    }

    #[test]
    fn test_deserialize_invalid() 
    {
        let json = "\"notacolor\"";
        let result: Result<HighlightColor, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_equality_and_hash() 
    {
        let c1 = HighlightColor { r: 1, g: 2, b: 3 };
        let c2 = HighlightColor { r: 1, g: 2, b: 3 };
        let c3 = HighlightColor { r: 3, g: 2, b: 1 };
        assert_eq!(c1, c2);
        assert_ne!(c1, c3);

        let mut set = HashSet::new();
        set.insert(c1);
        assert!(set.contains(&c2));
        assert!(!set.contains(&c3));
    }
}
