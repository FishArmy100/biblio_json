use std::{ops::Deref, str::FromStr};

use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::core::{RefId, StrongsNumber, VerseId};

lazy_static::lazy_static! {
    static ref HREF_MODULE_ENTRY_REGEX: Regex = Regex::new("^(?P<module>[a-zA-Z_][a-zA-Z_0-9]*):(?P<entry>\\d+)$").unwrap();
    static ref IMAGE_REF_REGEX: Regex = Regex::new("^[a-zA-Z_][a-zA-Z_0-9]*$").unwrap();
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    // Block-level elements
    Paragraph(Vec<Node>),
    Heading { level: u8, content: Vec<Node> },
    List { ordered: bool, items: Vec<Node> },
    ListItem(Vec<Node>),
    HorizontalRule,
    
    // Inline elements
    Text(String),
    Underline(Vec<Node>),
    Italic(Vec<Node>),
    Bold(Vec<Node>),
    Strike(Vec<Node>),
    Image { src: AssetIdName, alt: Option<String> },
    Anchor { href: HRefSrc, content: Vec<Node> },
    LineBreak,
}

impl Node {
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            Node::Paragraph(_) | Node::Heading { .. } | Node::List { .. } | 
            Node::ListItem(_) | Node::HorizontalRule
        )
    }

    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            Node::Text(_) | Node::Underline(_) | Node::Italic(_) | 
            Node::Bold(_) | Node::Strike(_) | Node::Image { .. } | 
            Node::Anchor { .. } | Node::LineBreak
        )
    }

    pub fn to_html(&self) -> String {
        match self {
            Node::Paragraph(children) => {
                format!("<p>{}</p>", children.iter().map(Node::to_html).join(""))
            }
            Node::Heading { level, content } => {
                format!("<h{l}>{}</h{l}>", content.iter().map(Node::to_html).join(""), l = level)
            }
            Node::List { ordered, items } => {
                let tag = if *ordered { "ol" } else { "ul" };
                let body = items.iter().map(Node::to_html).join("");
                format!("<{t}>{b}</{t}>", t = tag, b = body)
            }
            Node::ListItem(children) => {
                format!("<li>{}</li>", children.iter().map(Node::to_html).join(""))
            }
            Node::HorizontalRule => "<hr>".to_string(),
            Node::Text(t) => escape_text(t),
            Node::Underline(children) => {
                format!("<u>{}</u>", children.iter().map(Node::to_html).join(""))
            }
            Node::Italic(children) => {
                format!("<i>{}</i>", children.iter().map(Node::to_html).join(""))
            }
            Node::Bold(children) => {
                format!("<b>{}</b>", children.iter().map(Node::to_html).join(""))
            }
            Node::Strike(children) => {
                format!("<s>{}</s>", children.iter().map(Node::to_html).join(""))
            }
            Node::LineBreak => "<br>".into(),
            Node::Image { src, alt } => {
                if let Some(alt_text) = alt {
                    format!("<img src=\"{}\" alt=\"{}\">", escape_attr(&src), escape_attr(alt_text))
                } else {
                    format!("<img src=\"{}\">", escape_attr(&src))
                }
            }
            Node::Anchor { href, content } => {
                format!("<a href=\"{}\">{}</a>", escape_attr(&href.to_string()), content.iter().map(Node::to_html).join(""))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetIdName(String);

impl Deref for AssetIdName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for AssetIdName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for AssetIdName {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if IMAGE_REF_REGEX.is_match(s) {
            Ok(Self(s.into()))
        } else {
            Err(format!("src '{}' is not formatted properly", s))
        }
    }
}

impl Serialize for AssetIdName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for AssetIdName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HRefSrc {
    RefId(RefId),  // Simplified - replace with your RefId type
    Strongs(StrongsNumber), // Simplified - replace with your StrongsNumber type
    ModuleRef {
        module_alias: AssetIdName,
        entry_id: u32,
    },
}

impl std::fmt::Display for HRefSrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HRefSrc::RefId(id) => write!(f, "{}", id),
            HRefSrc::Strongs(num) => write!(f, "{}", num),
            HRefSrc::ModuleRef { module_alias, entry_id } => write!(f, "{}:{}", module_alias, entry_id),
        }
    }
}

impl FromStr for HRefSrc 
{
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        // Try module ref pattern first
        if let Some(captures) = HREF_MODULE_ENTRY_REGEX.captures(s) {
            let module_alias = captures.name("module").unwrap().as_str().to_owned();
            let entry_id = captures.name("entry").unwrap().as_str().parse::<u32>()
                .map_err(|e| format!("Invalid entry_id: {}", e))?;
            return Ok(Self::ModuleRef { 
                module_alias: AssetIdName::from_str(&module_alias)?, 
                entry_id 
            });
        }
        
        if let Some(strongs) = StrongsNumber::from_str(s).ok()
        {
            return Ok(Self::Strongs(strongs))
        }
        
        // Default to RefId
        Ok(Self::RefId(RefId::from_str(s)?))
    }
}

impl Serialize for HRefSrc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for HRefSrc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

fn escape_text(t: &str) -> String {
    t.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

fn escape_attr(t: &str) -> String {
    t.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}