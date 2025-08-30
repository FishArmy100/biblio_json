use std::{ops::Deref, str::FromStr};

use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::core::{StrongsNumber, VerseId};

lazy_static::lazy_static!
{
    static ref HREF_MODULE_ENTRY_REGEX: Regex = Regex::new("^(?P<module>[a-zA-Z_][a-zA-Z_0-9]*):(?P<entry>\\d+)$").unwrap();
    static ref IMAGE_REF_REGEX: Regex = Regex::new("^[a-zA-Z_][a-zA-Z_0-9]*$").unwrap();
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block
{
    Paragraph(Vec<Inline>),
    Heading { level: u8, content: Vec<Inline> },
    List { ordered: bool, items: Vec<Vec<Inline>> },
    HorizontalRule,
}

impl Block 
{
    pub fn to_html(&self) -> String 
    {
        match self 
        {
            Block::Paragraph(inls) => format!("<p>{}</p>", inls.iter().map(Inline::to_html).join("")),
            Block::Heading { level, content } => format!("<h{l}>{}</h{l}>", content.iter().map(Inline::to_html).join(""), l = level),
            Block::List { ordered, items } => {
                let tag = if *ordered { "ol" } else { "ul" };
                let body = items.iter().map(|it| format!("<li>{}</li>", it.iter().map(Inline::to_html).join(""))).collect::<String>();
                format!("<{t}>{b}</{t}>", t = tag, b = body)
            }
            Block::HorizontalRule => "<hr>".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inline 
{
    Text(String),
    Underline(Vec<Inline>),
    Italic(Vec<Inline>),
    Bold(Vec<Inline>),
    Strike(Vec<Inline>),
    Image { src: AssetIdName, alt: Option<String> },
    Anchor {
        href: HRefSrc,
        content: Vec<Inline>,
    },
    LineBreak,
}

impl Inline
{
    pub fn to_html(&self) -> String 
    {
        match self
        {
            Inline::Text(t) => escape_text(t),
            Inline::Underline(v) => format!("<u>{}</u>", v.iter().map(Self::to_html).join("")),
            Inline::Italic(v) => format!("<i>{}</i>", v.iter().map(Self::to_html).join("")),
            Inline::Bold(v) => format!("<b>{}</b>", v.iter().map(Self::to_html).join("")),
            Inline::Strike(v) => format!("<s>{}</s>", v.iter().map(Self::to_html).join("")),
            Inline::LineBreak => "<br>".into(),
            Inline::Image { src, alt } => {
                if let Some(alt_text) = alt {
                    format!("<img src=\"{}\" alt=\"{}\">", escape_attr(&src), escape_attr(alt_text))
                } else {
                    format!("<img src=\"{}\">", escape_attr(&src))
                }
            },
            Inline::Anchor { href, content } => format!("<a href=\"{}\">{}</a>", escape_attr(&href.to_string()), content.iter().map(Self::to_html).join("")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetIdName(String);

impl Deref for AssetIdName
{
    type Target = str;

    fn deref(&self) -> &Self::Target 
    {
        &self.0
    }
}

impl std::fmt::Display for AssetIdName
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "{}", self)
    }
}

impl FromStr for AssetIdName
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        if IMAGE_REF_REGEX.is_match(s) 
        {
            Ok(Self(s.into()))
        }
        else 
        {
            Err(format!("src '{}' is not formatted properly", s))    
        }
    }
}

impl Serialize for AssetIdName
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for AssetIdName
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HRefSrc
{
    VerseId(VerseId),
    Strongs(StrongsNumber),
    ModuleRef {
        module_alias: AssetIdName,
        entry_id: u32,
    }
}

impl std::fmt::Display for HRefSrc
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            HRefSrc::VerseId(verse_id) => write!(f, "{}", verse_id),
            HRefSrc::Strongs(strongs_number) => write!(f, "{}", strongs_number),
            HRefSrc::ModuleRef { module_alias, entry_id } => write!(f, "{}:{}", module_alias, entry_id),
        }
    }
}

impl FromStr for HRefSrc
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        if let Ok(verse_id) = VerseId::from_str(s)
        {
            Ok(Self::VerseId(verse_id))
        }
        else if let Ok(strongs) = StrongsNumber::from_str(s)
        {
            Ok(Self::Strongs(strongs))
        }
        else if let Some(captures) = HREF_MODULE_ENTRY_REGEX.captures(s)
        {
            let module_alias = captures.name("module").unwrap().as_str().to_owned();
            let entry_id = captures.name("entry").unwrap().as_str().parse::<u32>().unwrap();
            Ok(Self::ModuleRef { module_alias: AssetIdName::from_str(&module_alias).unwrap(), entry_id })
        }
        else 
        {
            Err(format!("href '{}' is not formatted properly", s))    
        }
    }
}

impl Serialize for HRefSrc
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for HRefSrc
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

fn escape_text(t: &str) -> String 
{
    // minimal escaping for text content
    t.chars().map(|c| match c {
        '&' => "&amp;".to_string(),
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        _ => c.to_string(),
    }).collect::<String>()
}

fn escape_attr(t: &str) -> String 
{
    // escaping for attribute values
    t.chars().map(|c| match c {
        '&' => "&amp;".to_string(),
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        '"' => "&quot;".to_string(),
        '\'' => "&#39;".to_string(),
        _ => c.to_string(),
    }).collect::<String>()
}
