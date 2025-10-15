//! A tiny HTML AST + parser for a very small subset of HTML.
//! Supported nodes only:
//! - paragraphs: `<p> ... </p>`
//! - underlined: `<u> ... </u>`
//! - italics: `<i> ... </i>` or `<em> ... </em>`
//! - strikethroughs: `<s> ... </s>` or `<strike> ... </strike>` or `<del> ... </del>`
//! - bold: `<b> ... </b>` or `<strong> ... </strong>`
//! - lists: `<ul><li>..</li></ul>`, `<ol><li>..</li></ol>`
//! - headings: `<h1>..</h1>` up to `<h6>..</h6>`
//! - newlines: `<br>` or `<br/>`
//! - header breaks: `<hr>` or `<hr/>`
//! - anchors: `<a href="...">...</a>`
//! - images: `<img src="..." alt="...">`
//! - Unicode characters (any text content)
//!
//! Now supports basic attributes for relevant tags.
//! No comments, scripts, styles, entities, or complex inline HTML are supported.
//! This is deliberately minimal and safe to extend.

use std::{fmt, str::FromStr};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{core::RefId, html_text::{ast::{AssetIdName, HRefSrc, Node}, lex::Lexer, parse::Parser}, modules::EntryId, validation::{RefIdValidationError, ValidationContext}};

pub mod lex;
pub mod parse;
pub mod ast;


#[derive(Debug, Clone, PartialEq)]
pub struct HtmlText 
{
    pub nodes: Vec<Node>,
}

impl HtmlText
{
    pub fn to_html(&self) -> String 
    {
        self.to_string()
    }

    pub fn validate(&self, context: &ValidationContext) -> Result<(), Vec<HtmlValidationError>>
    {
        HtmlValidator::validate(self, context).map_err(|e| vec![e])
    }
}

impl ToString for HtmlText
{
    fn to_string(&self) -> String 
    {
        self.nodes.iter().map(|n| n.to_html()).join("")
    }
}

impl FromStr for HtmlText
{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        let mut lexer = Lexer::new(s);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
}

impl Serialize for HtmlText 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for HtmlText 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    UnexpectedEOF,
    UnexpectedToken(String),
    UnsupportedTag(String),
    InvalidNesting(String),
    InvalidAttrs(String),
    MissingRequiredAttr(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        match self 
        {
            ErrorKind::UnexpectedEOF => write!(f, "Unexpected end of file"),
            ErrorKind::UnexpectedToken(tok) => write!(f, "Unexpected token: {}", tok),
            ErrorKind::UnsupportedTag(tag) => write!(f, "Unsupported tag: <{}>", tag),
            ErrorKind::InvalidNesting(desc) => write!(f, "Invalid nesting: {}", desc),
            ErrorKind::InvalidAttrs(desc) => write!(f, "Invalid attributes: {}", desc),
            ErrorKind::MissingRequiredAttr(attr) => write!(f, "Missing required attribute: {}", attr),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ErrorKind,
    pub pos: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} at byte {}", self.kind, self.pos)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
pub enum HtmlValidationError
{
    InvalidRefId
    {
        id: RefId,
        error: RefIdValidationError
    },
    InvalidModuleAlias
    {
        alias: AssetIdName,
    },
    EntryIdDoesNotExist
    {
        module_name: String,
        entry_id: EntryId,
    }
}

impl std::fmt::Display for HtmlValidationError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        match self 
        {
            Self::InvalidRefId { id, error } => match error {
                RefIdValidationError::DoesNotExist => write!(f, "RefId {} does not exist in bible in the current context", id),
                RefIdValidationError::NeedsBible => write!(f, "RefId {} needs a bible reference", id),
            },
            Self::InvalidModuleAlias { alias } => write!(f, "Module alias '{}' either does not exist, or the module it references does not exist", alias),
            Self::EntryIdDoesNotExist { module_name, entry_id } => write!(f, "Entry '{}' for module '{}' does not exist", entry_id, module_name),
        }
    }
}

pub struct HtmlValidator;

impl HtmlValidator {
    pub fn validate(doc: &HtmlText, context: &ValidationContext) -> Result<(), HtmlValidationError> 
    {
        doc.nodes.iter().try_for_each(|node| Self::visit_node(node, context))
    }

    fn visit_node(node: &Node, context: &ValidationContext) -> Result<(), HtmlValidationError> {
        match node {
            // Block-level nodes
            Node::Paragraph(children) => {
                children.iter().try_for_each(|n| Self::visit_node(n, context))
            }
            Node::Heading { level: _, content } => {
                content.iter().try_for_each(|n| Self::visit_node(n, context))
            }
            Node::List { ordered: _, items } => {
                items.iter().try_for_each(|n| Self::visit_node(n, context))
            }
            Node::ListItem(children) => {
                children.iter().try_for_each(|n| Self::visit_node(n, context))
            }
            Node::HorizontalRule => Ok(()),

            // Inline nodes
            Node::Text(_) => Ok(()),
            Node::Underline(children) => {
                children.iter().try_for_each(|n| Self::visit_node(n, context))
            }
            Node::Italic(children) => {
                children.iter().try_for_each(|n| Self::visit_node(n, context))
            }
            Node::Bold(children) => {
                children.iter().try_for_each(|n| Self::visit_node(n, context))
            }
            Node::Strike(children) => {
                children.iter().try_for_each(|n| Self::visit_node(n, context))
            }
            Node::Image { src: _, alt: _ } => Ok(()),
            Node::LineBreak => Ok(()),
            
            Node::Anchor { href, content } => {
                // First validate all children
                content.iter().try_for_each(|n| Self::visit_node(n, context))?;
                
                // Then validate the href
                match href {
                    HRefSrc::RefId(id) => {
                        context.validate_ref_id(id).map_err(|error| {
                            HtmlValidationError::InvalidRefId {
                                id: id.clone(),
                                error,
                            }
                        })
                    }
                    HRefSrc::Strongs(_) => Ok(()),
                    HRefSrc::ModuleRef { module_alias, entry_id } => {
                        let aliases = &context.external.aliases;
                        let Some(module_name) = aliases.get(module_alias) else {
                            return Err(HtmlValidationError::InvalidModuleAlias {
                                alias: module_alias.clone(),
                            });
                        };

                        let all_modules = &context.all_modules;
                        let Some(module) = all_modules.get(module_name) else {
                            return Err(HtmlValidationError::InvalidModuleAlias {
                                alias: module_alias.clone(),
                            });
                        };

                        if !module.has_entry(*entry_id) {
                            return Err(HtmlValidationError::EntryIdDoesNotExist {
                                module_name: module_name.clone(),
                                entry_id: *entry_id,
                            });
                        }

                        Ok(())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_text::ast::Node;
    use std::str::FromStr;

    fn parse_ok(s: &str) -> HtmlText {
        HtmlText::from_str(s).expect(&format!("should parse: {}", s))
    }

    fn roundtrip(input: &str) {
        let parsed = HtmlText::from_str(input).unwrap();
        let output = parsed.to_string();
        assert_eq!(output, input, "roundtrip failed: input={input}, output={output}");
    }

    // ──────────────────────────────────────────────
    // BASIC BLOCK ELEMENTS
    // ──────────────────────────────────────────────

    #[test]
    fn paragraph_basic() {
        let html = "<p>Hello world</p>";
        let doc = parse_ok(html);
        assert_eq!(
            doc.nodes,
            vec![Node::Paragraph(vec![Node::Text("Hello world".into())])]
        );
        assert_eq!(doc.to_string(), html);
    }

    #[test]
    fn heading_levels() {
        for level in 1..=6 {
            let html = format!("<h{0}>Header {0}</h{0}>", level);
            let doc = parse_ok(&html);
            assert_eq!(
                doc.nodes,
                vec![Node::Heading {
                    level,
                    content: vec![Node::Text(format!("Header {}", level))]
                }]
            );
            assert_eq!(doc.to_string(), html);
        }
    }

    #[test]
    fn list_unordered() {
        let html = "<ul><li>One</li><li>Two</li></ul>";
        let doc = parse_ok(html);
        assert_eq!(
            doc.nodes,
            vec![Node::List {
                ordered: false,
                items: vec![
                    Node::ListItem(vec![Node::Text("One".into())]),
                    Node::ListItem(vec![Node::Text("Two".into())]),
                ]
            }]
        );
        assert_eq!(doc.to_string(), html);
    }

    #[test]
    fn list_ordered() {
        let html = "<ol><li>First</li><li>Second</li></ol>";
        roundtrip(html);
    }

    #[test]
    fn horizontal_rule() {
        let html = "<hr>";
        roundtrip(html);
    }

    // ──────────────────────────────────────────────
    // INLINE STYLING ELEMENTS
    // ──────────────────────────────────────────────

    #[test]
    fn bold_text() {
        let html = "<p><b>bold</b></p>";
        roundtrip(html);
    }

    #[test]
    fn italic_and_underline() {
        let html = "<p><i>italic</i> and <u>underline</u></p>";
        roundtrip(html);
    }

    #[test]
    fn strike_text() {
        let html = "<p><s>cross</s></p>";
        roundtrip(html);
    }

    #[test]
    fn nested_inline_tags() {
        let html = "<p><b>bold and <i>italic</i></b></p>";
        roundtrip(html);
    }

    #[test]
    fn line_break_in_paragraph() {
        let html = "<p>Hello<br>World</p>";
        roundtrip(html);
    }

    // ──────────────────────────────────────────────
    // IMAGES AND ANCHORS
    // ──────────────────────────────────────────────

    #[test]
    fn image_with_alt() {
        let html = "<p><img src=\"example_img\" alt=\"Example\"></p>";
        roundtrip(html);
    }

    #[test]
    fn image_without_alt() {
        let html = "<p><img src=\"example_img\"></p>";
        roundtrip(html);
    }

    #[test]
    fn anchor_with_href_refid() {
        // Pretend RefId supports parsing like "John.3.16"
        let html = "<p><a href=\"John.3.16\">text</a></p>";
        roundtrip(html);
    }

    #[test]
    fn anchor_with_module_ref() {
        let html = "<p><a href=\"lexicon:42\">see</a></p>";
        roundtrip(html);
    }

    #[test]
    fn anchor_with_strongs() {
        let html = "<p><a href=\"G3056\">word</a></p>";
        roundtrip(html);
    }

    // ──────────────────────────────────────────────
    // NESTING / COMPOSITION
    // ──────────────────────────────────────────────

    #[test]
    fn paragraph_with_mixed_content() {
        let html = "<p>Hello <b>bold</b><i>italic</i> world</p>";
        roundtrip(html);
    }

    #[test]
    fn list_items_with_paragraphs() {
        let html = "<ul><li><p>Text</p></li></ul>";
        roundtrip(html);
    }

    #[test]
    fn nested_lists() {
        let html = "<ul><li>Outer<ul><li>Inner</li></ul></li></ul>";
        roundtrip(html);
    }

    #[test]
    fn multiple_blocks() {
        let html = "<h1>Title</h1><p>Text</p><hr><p>More</p>";
        roundtrip(html);
    }

    // ──────────────────────────────────────────────
    // INVALID HTML TESTS
    // ──────────────────────────────────────────────

    #[test]
    fn missing_closing_tag() {
        let html = "<p>Unclosed";
        let err = HtmlText::from_str(html).unwrap_err();
        assert!(matches!(err.kind, ErrorKind::UnexpectedEOF));
    }

    #[test]
    fn invalid_tag_in_paragraph() {
        let html = "<p><ul></ul></p>";
        let err = HtmlText::from_str(html).unwrap_err();
        assert!(matches!(err.kind, ErrorKind::InvalidNesting(_)));
    }

    #[test]
    fn list_with_text_not_li() {
        let html = "<ul>text</ul>";
        let err = HtmlText::from_str(html).unwrap_err();
        assert!(matches!(err.kind, ErrorKind::InvalidNesting(_)));
    }

    #[test]
    fn missing_required_href_attr() {
        let html = "<p><a>missing</a></p>";
        let err = HtmlText::from_str(html).unwrap_err();
        assert!(matches!(err.kind, ErrorKind::MissingRequiredAttr(_)));
    }

    #[test]
    fn invalid_image_src() {
        // src cannot have invalid name characters
        let html = "<p><img src=\"!bad\"></p>";
        let err = HtmlText::from_str(html).unwrap_err();
        assert!(matches!(err.kind, ErrorKind::InvalidAttrs(_)));
    }

    #[test]
    fn unsupported_tag() {
        let html = "<div>test</div>";
        let err = HtmlText::from_str(html).unwrap_err();
        assert!(matches!(err.kind, ErrorKind::InvalidNesting(_)));
    }

    // ──────────────────────────────────────────────
    // ROUNDTRIP PROPERTY TESTS
    // ──────────────────────────────────────────────

    #[test]
    fn roundtrip_minimal_document() {
        let html = "<p>simple</p>";
        roundtrip(html);
    }

    #[test]
    fn roundtrip_complex_document() {
        let html = "<h2>Header</h2><p>Some <b>bold</b> text and <a href=\"H1234\">link</a><br>plus image <img src=\"foo\"></p><ul><li>Item 1</li><li>Item 2</li></ul>";
        roundtrip(html);
    }
}
