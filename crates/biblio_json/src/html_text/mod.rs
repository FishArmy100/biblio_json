//! A tiny HTML AST + parser for a very small subset of HTML.
//! Supported nodes only:
//! - paragraphs: <p> ... </p>
//! - underlined: <u> ... </u>
//! - italics: <i> ... </i> or <em> ... </em>
//! - strikethroughs: <s> ... </s> or <strike> ... </strike> or <del> ... </del>
//! - bold: <b> ... </b> or <strong> ... </strong>
//! - lists: <ul><li>..</li></ul>, <ol><li>..</li></ol>
//! - headings: <h1>..</h1> up to <h6>..</h6>
//! - newlines: <br> or <br/>
//! - header breaks: <hr> or <hr/>
//! - Unicode characters (any text content)
//!
//! No other tags or attributes are supported. Unknown tags cause an error.
//! No attributes, comments, scripts, styles, entities, or inline HTML are supported.
//! This is deliberately minimal and safe to extend.
//! 
//! **NOTE:** This was created by ChatGPT, so errors be be present

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Document(Vec<Block>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph(Vec<Inline>),
    Heading { level: u8, content: Vec<Inline> },
    List { ordered: bool, items: Vec<Vec<Inline>> },
    HorizontalRule,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    Underline(Vec<Inline>),
    Italic(Vec<Inline>),
    Bold(Vec<Inline>),
    Strike(Vec<Inline>),
    LineBreak,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    UnexpectedEOF,
    UnexpectedToken(String),
    UnsupportedTag(String),
    InvalidNesting(String),
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

#[derive(Debug, Clone, PartialEq)]
enum Token {
    OpenTag(String),   // e.g., "p", "h1", "ul"
    CloseTag(String),  // e.g., "/p", "/h1"
    SelfClose(String), // e.g., "br", "hr"
    Text(String),
}

fn is_whitespace(c: char) -> bool { c == ' ' || c == '\n' || c == '\t' || c == '\r' }

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            // parse tag
            let start = i;
            i += 1;
            if i >= bytes.len() { return Err(ParseError { kind: ErrorKind::UnexpectedEOF, pos: start }); }
            let mut tag = String::new();
            let mut is_close = false;
            if bytes[i] == b'/' { is_close = true; i += 1; }
            while i < bytes.len() && bytes[i] != b'>' {
                tag.push(bytes[i] as char);
                i += 1;
            }
            if i >= bytes.len() { return Err(ParseError { kind: ErrorKind::UnexpectedEOF, pos: start }); }
            // consume '>'
            i += 1;
            // normalize: strip spaces and attributes (unsupported)
            let tag = tag.trim();
            // detect self close or attributes shorthand like br/
            let self_closing = tag.ends_with('/') || matches!(tag.to_ascii_lowercase().as_str(), "br" | "hr");
            let mut name = tag.trim_end_matches('/').trim().to_string();
            // cut off attributes if any
            if let Some(idx) = name.find(' ') {
                // attributes are not supported; if present, it's an error
                return Err(ParseError { kind: ErrorKind::UnsupportedTag(name[..idx].to_string()), pos: start });
            }
            name = name.to_ascii_lowercase();
            if is_close {
                tokens.push(Token::CloseTag(name));
            } else if self_closing {
                tokens.push(Token::SelfClose(name));
            } else {
                tokens.push(Token::OpenTag(name));
            }
        } else {
            // text node until next '<'
            let start = i;
            let mut s = String::new();
            while i < bytes.len() && bytes[i] != b'<' {
                s.push(bytes[i] as char);
                i += 1;
            }
            tokens.push(Token::Text(s));
        }
    }
    Ok(tokens)
}

pub fn parse(input: &str) -> Result<Node, ParseError> {
    let tokens = tokenize(input)?;
    let mut p = Parser { tokens, pos: 0 };
    p.parse_document()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos) }
    fn next(&mut self) -> Option<Token> { let t = self.tokens.get(self.pos).cloned(); if t.is_some() { self.pos += 1; } t }
    fn expect_close(&mut self, name: &str, pos: usize) -> Result<(), ParseError> {
        match self.next() {
            Some(Token::CloseTag(n)) if n == name => Ok(()),
            Some(t) => Err(ParseError { kind: ErrorKind::UnexpectedToken(format!("expected </{}>, found {:?}", name, t)), pos }),
            None => Err(ParseError { kind: ErrorKind::UnexpectedEOF, pos }),
        }
    }

    fn parse_document(&mut self) -> Result<Node, ParseError> {
        let mut blocks = Vec::new();
        while let Some(tok) = self.peek() {
            match tok {
                Token::OpenTag(name) => match name.as_str() {
                    n if n.starts_with('h') && n.len() == 2 && n.chars().nth(1).unwrap().is_ascii_digit() => {
                        blocks.push(self.parse_heading()?);
                    }
                    "p" => blocks.push(self.parse_paragraph()?),
                    "ul" | "ol" => blocks.push(self.parse_list()?),
                    "hr" => { self.next(); blocks.push(Block::HorizontalRule); },
                    _ => return Err(ParseError { kind: ErrorKind::UnsupportedTag(name.clone()), pos: self.pos }),
                },
                Token::SelfClose(name) if name == "hr" => { self.next(); blocks.push(Block::HorizontalRule); },
                Token::Text(t) => {
                    // Allow stray text outside blocks by wrapping in implicit paragraph if non-whitespace
                    if t.trim().is_empty() {
                        self.next();
                    } else {
                        blocks.push(self.parse_paragraph()?)
                    }
                }
                _ => return Err(ParseError { kind: ErrorKind::UnexpectedToken(format!("{:?}", tok)), pos: self.pos }),
            }
        }
        Ok(Node::Document(blocks))
    }

    fn parse_heading(&mut self) -> Result<Block, ParseError> {
        let start_pos = self.pos;
        let level = match self.next() {
            Some(Token::OpenTag(n)) => {
                if n.len() == 2 && n.starts_with('h') {
                    n[1..2].parse::<u8>().unwrap()
                } else { return Err(ParseError { kind: ErrorKind::UnsupportedTag(n), pos: start_pos }); }
            }
            _ => return Err(ParseError { kind: ErrorKind::UnexpectedToken("expected heading open".into()), pos: start_pos }),
        };
        let content = self.parse_inlines_until(&format!("h{}", level))?;
        Ok(Block::Heading { level, content })
    }

    fn parse_paragraph(&mut self) -> Result<Block, ParseError> {
        let start_pos = self.pos;
        match self.next() {
            Some(Token::OpenTag(n)) if n == "p" => {}
            Some(Token::Text(t)) => { // implicit paragraph from stray text
                let mut v = vec![Inline::Text(t)];
                v.append(&mut self.collect_inline_text());
                return Ok(Block::Paragraph(v));
            }
            other => return Err(ParseError { kind: ErrorKind::UnexpectedToken(format!("expected <p> or text, found {:?}", other)), pos: start_pos }),
        }
        let content = self.parse_inlines_until("p")?;
        Ok(Block::Paragraph(content))
    }

    fn collect_inline_text(&mut self) -> Vec<Inline> {
        let mut v = Vec::new();
        while let Some(Token::Text(t)) = self.peek() { v.push(Inline::Text(t.clone())); self.next(); }
        v
    }

    fn parse_list(&mut self) -> Result<Block, ParseError> {
        let start_pos = self.pos;
        let (ordered, tag) = match self.next() {
            Some(Token::OpenTag(n)) if n == "ul" => (false, "ul".to_string()),
            Some(Token::OpenTag(n)) if n == "ol" => (true, "ol".to_string()),
            other => return Err(ParseError { kind: ErrorKind::UnexpectedToken(format!("expected <ul> or <ol>, found {:?}", other)), pos: start_pos }),
        };
        let mut items = Vec::new();
        loop {
            match self.peek() {
                Some(Token::OpenTag(n)) if n == "li" => {
                    self.next();
                    let inlines = self.parse_inlines_until("li")?;
                    items.push(inlines);
                }
                Some(Token::CloseTag(n)) if n == &tag => { self.next(); break; }
                Some(Token::Text(t)) if t.trim().is_empty() => { self.next(); }
                Some(tok) => return Err(ParseError { kind: ErrorKind::InvalidNesting(format!("unexpected in <{}>: {:?}", tag, tok)), pos: self.pos }),
                None => return Err(ParseError { kind: ErrorKind::UnexpectedEOF, pos: self.pos }),
            }
        }
        Ok(Block::List { ordered, items })
    }

    fn parse_inlines_until(&mut self, closing: &str) -> Result<Vec<Inline>, ParseError> {
        let mut v = Vec::new();
        loop {
            match self.peek() {
                Some(Token::CloseTag(n)) if n == closing => { self.next(); break; }
                Some(Token::Text(t)) => { v.push(Inline::Text(t.clone())); self.next(); }
                Some(Token::OpenTag(n)) => {
                    let name = n.clone();
                    match name.as_str() {
                        "u" => { self.next(); let content = self.parse_inlines_until("u")?; v.push(Inline::Underline(content)); }
                        "i" | "em" => { self.next(); let content = self.parse_inlines_until(&name)?; v.push(Inline::Italic(content)); }
                        "b" | "strong" => { self.next(); let content = self.parse_inlines_until(&name)?; v.push(Inline::Bold(content)); }
                        "s" | "strike" | "del" => { self.next(); let content = self.parse_inlines_until(&name)?; v.push(Inline::Strike(content)); }
                        // nested block-level tags are not allowed inside inlines here
                        other if other == "p" || other == "ul" || other == "ol" || other.starts_with('h') => {
                            return Err(ParseError { kind: ErrorKind::InvalidNesting(format!("block <{}> inside inline", other)), pos: self.pos });
                        }
                        other => return Err(ParseError { kind: ErrorKind::UnsupportedTag(other.to_string()), pos: self.pos }),
                    }
                }
                Some(Token::SelfClose(n)) if n == "br" => { self.next(); v.push(Inline::LineBreak); }
                Some(Token::SelfClose(n)) => return Err(ParseError { kind: ErrorKind::UnsupportedTag(n.clone()), pos: self.pos }),
                Some(Token::CloseTag(_)) => return Err(ParseError { kind: ErrorKind::UnexpectedToken("unexpected close tag".into()), pos: self.pos }),
                None => return Err(ParseError { kind: ErrorKind::UnexpectedEOF, pos: self.pos }),
            }
        }
        Ok(v)
    }
}

// --- Renderers --- //

impl Node {
    pub fn to_html(&self) -> String {
        match self {
            Node::Document(blocks) => blocks.iter().map(|b| b.to_html()).collect::<Vec<_>>().join("")
        }
    }
}

impl Block {
    pub fn to_html(&self) -> String {
        match self {
            Block::Paragraph(inls) => format!("<p>{}</p>", inlines_to_html(inls)),
            Block::Heading { level, content } => format!("<h{l}>{}</h{l}>", inlines_to_html(content), l = level),
            Block::List { ordered, items } => {
                let tag = if *ordered { "ol" } else { "ul" };
                let body = items.iter().map(|it| format!("<li>{}</li>", inlines_to_html(it))).collect::<String>();
                format!("<{t}>{b}</{t}>", t = tag, b = body)
            }
            Block::HorizontalRule => "<hr>".to_string(),
        }
    }
}

fn inlines_to_html(inls: &[Inline]) -> String 
{
    let mut s = String::new();
    for i in inls {
        match i {
            Inline::Text(t) => s.push_str(&escape_text(t)),
            Inline::Underline(v) => s.push_str(&format!("<u>{}</u>", inlines_to_html(v))),
            Inline::Italic(v) => s.push_str(&format!("<i>{}</i>", inlines_to_html(v))),
            Inline::Bold(v) => s.push_str(&format!("<b>{}</b>", inlines_to_html(v))),
            Inline::Strike(v) => s.push_str(&format!("<s>{}</s>", inlines_to_html(v))),
            Inline::LineBreak => s.push_str("<br>"),
        }
    }
    s
}

fn escape_text(t: &str) -> String {
    // minimal escaping for text-only (no entity decoding)
    t.chars().map(|c| match c {
        '&' => "&amp;".to_string(),
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        '"' => "&quot;".to_string(),
        '\'' => "&#39;".to_string(),
        _ => c.to_string(),
    }).collect::<String>()
}

// --- Example usage (doc test) --- //
/// ```
/// use mini_html_ast::*;
/// let html = r#"
/// <h2>Heading</h2>
/// <p>Hello <b>world</b>!<br>Привет <i>мир</i>.</p>
/// <ul><li>first</li><li><u>second</u></li></ul>
/// <hr>
/// "#;
/// let ast = parse(html).unwrap();
/// // roundtrip
/// let out = ast.to_html();
/// assert_eq!(out, "<h2>Heading</h2><p>Hello <b>world</b>!<br>Привет <i>мир</i>.</p><ul><li>first</li><li><u>second</u></li></ul><hr>");
/// ```
pub mod mini_html_ast { pub use super::{parse, Node, Block, Inline, ParseError, ErrorKind}; }