use std::{fmt, str::FromStr};

use crate::html_text::{HtmlText, ast::{Block, HRefSrc, AssetIdName, Inline}, lex::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind 
{
    UnexpectedEOF,
    UnexpectedToken(String),
    UnsupportedTag(String),
    InvalidNesting(String),
    InvalidAttrs(String),
    MissingRequiredAttr(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError 
{
    pub kind: ErrorKind,
    pub pos: usize,
} 

impl fmt::Display for ParseError 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        write!(f, "{:?} at byte {}", self.kind, self.pos)
    }
}

impl std::error::Error for ParseError {}

pub struct Parser 
{
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser 
{
    pub fn new(tokens: Vec<Token>) -> Self 
    {
        Self {
            tokens,
            pos: 0,
        }
    }

    fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos) }
    fn next(&mut self) -> Option<Token> { let t = self.tokens.get(self.pos).cloned(); if t.is_some() { self.pos += 1; } t } 

    pub fn parse(&mut self) -> Result<HtmlText, ParseError> 
    {
        let mut blocks = Vec::new();
        while let Some(tok) = self.peek() 
        {
            match tok 
            {
                Token::OpenTag { tag, attrs: _ } => {
                    match tag.as_str() 
                    {
                        n if n.starts_with('h') && n.len() == 2 && n.chars().nth(1).unwrap().is_ascii_digit() => {
                            blocks.push(self.parse_heading()?);
                        },
                        "p" => blocks.push(self.parse_paragraph()?),
                        "ul" | "ol" => blocks.push(self.parse_list()?),
                        _ => return Err(ParseError { kind: ErrorKind::UnsupportedTag(tag.clone()), pos: self.pos }),
                    }
                },
                Token::SelfClose { tag, attrs: _ } => {
                    match tag.as_str() {
                        "hr" => { self.next(); blocks.push(Block::HorizontalRule); },
                        _ => return Err(ParseError { kind: ErrorKind::UnsupportedTag(tag.clone()), pos: self.pos }),
                    }
                },
                Token::Text(t) => 
                {
                    // Allow stray text outside blocks by wrapping in implicit paragraph if non-whitespace
                    if t.trim().is_empty() 
                    {
                        self.next();
                    } 
                    else 
                    {
                        blocks.push(self.parse_paragraph()?)
                    }
                }
                _ => return Err(ParseError { kind: ErrorKind::UnexpectedToken(format!("{:?}", tok)), pos: self.pos }),
            }
        }
        Ok(HtmlText(blocks))
    }

    fn parse_heading(&mut self) -> Result<Block, ParseError> 
    {
        let start_pos = self.pos;
        let level = match self.next() 
        {
            Some(Token::OpenTag { tag, attrs: _ }) => 
            {
                if tag.len() == 2 && tag.starts_with('h') 
                {
                    tag[1..2].parse::<u8>().unwrap()
                } 
                else 
                { 
                    return Err(ParseError { 
                        kind: ErrorKind::UnsupportedTag(tag), pos: start_pos 
                    }); 
                }
            }
            _ => return Err(ParseError { kind: ErrorKind::UnexpectedToken("expected heading open".into()), pos: start_pos }),
        };
        let content = self.parse_inlines_until(&format!("h{}", level))?;
        Ok(Block::Heading { level, content })
    }

    fn parse_paragraph(&mut self) -> Result<Block, ParseError> 
    {
        let start_pos = self.pos;
        match self.next() 
        {
            Some(Token::OpenTag { tag, attrs: _ }) if tag == "p" => {}
            Some(Token::Text(t)) => // implicit paragraph from stray text
            { 
                let mut v = vec![Inline::Text(t)];
                v.append(&mut self.collect_inline_text());
                return Ok(Block::Paragraph(v));
            }
            other => return Err(ParseError { kind: ErrorKind::UnexpectedToken(format!("expected <p> or text, found {:?}", other)), pos: start_pos }),
        }
        let content = self.parse_inlines_until("p")?;
        Ok(Block::Paragraph(content))
    }

    fn collect_inline_text(&mut self) -> Vec<Inline> 
    {
        let mut v = Vec::new();
        while let Some(Token::Text(t)) = self.peek() 
        { 
            v.push(Inline::Text(t.clone())); self.next(); 
        }
        v
    }

    fn parse_list(&mut self) -> Result<Block, ParseError> 
    {
        let start_pos = self.pos;
        let (ordered, tag) = match self.next() 
        {
            Some(Token::OpenTag { tag, attrs: _ }) if tag == "ul" => (false, "ul".to_string()),
            Some(Token::OpenTag { tag, attrs: _ }) if tag == "ol" => (true, "ol".to_string()),
            other => {
                return Err(ParseError {
                    kind: ErrorKind::UnexpectedToken(format!(
                        "expected <ul> or <ol>, found {:?}",
                        other
                    )),
                    pos: start_pos,
                })
            }
        };

        let mut items = Vec::new();

        loop 
        {
            match self.peek() 
            {
                Some(Token::OpenTag { tag, attrs: _ }) if tag == "li" => {
                    self.next(); // consume <li>
                    let blocks = self.parse_blocks_until("li")?;
                    items.push(blocks);
                }
                Some(Token::CloseTag(n)) if n == &tag => {
                    self.next(); // consume </ul> or </ol>
                    break;
                }
                Some(Token::Text(t)) if t.trim().is_empty() => {
                    self.next(); // skip whitespace
                }
                Some(tok) => {
                    return Err(ParseError {
                        kind: ErrorKind::InvalidNesting(format!(
                            "unexpected in <{}>: {:?}",
                            tag, tok
                        )),
                        pos: self.pos,
                    })
                }
                None => {
                    return Err(ParseError {
                        kind: ErrorKind::UnexpectedEOF,
                        pos: self.pos,
                    })
                }
            }
        }

        Ok(Block::List { ordered, items })
    }

    fn parse_inlines_until(&mut self, closing: &str) -> Result<Vec<Inline>, ParseError> 
    {
        let mut v = Vec::new();
        loop 
        {
            match self.peek() 
            {
                Some(Token::CloseTag(n)) if n == closing => { self.next(); break; }
                Some(Token::Text(t)) => { v.push(Inline::Text(t.clone())); self.next(); }
                Some(Token::OpenTag { tag, attrs }) => 
                {
                    let name = tag.clone();
                    match name.as_str() 
                    {
                        "u" => { self.next(); let content = self.parse_inlines_until("u")?; v.push(Inline::Underline(content)); }
                        "i" | "em" => { self.next(); let content = self.parse_inlines_until(&name)?; v.push(Inline::Italic(content)); }
                        "b" | "strong" => { self.next(); let content = self.parse_inlines_until(&name)?; v.push(Inline::Bold(content)); }
                        "s" | "strike" | "del" => { self.next(); let content = self.parse_inlines_until(&name)?; v.push(Inline::Strike(content)); }
                        "a" => { 
                            let attrs = attrs.clone();
                            self.next();
                            let href = attrs.get("href")
                                .ok_or_else(|| ParseError { 
                                    kind: ErrorKind::MissingRequiredAttr("href required for <a> tag".to_string()), 
                                    pos: self.pos 
                                })?
                                .clone();
                            let content = self.parse_inlines_until("a")?;
                            let href = HRefSrc::from_str(&href).map_err(|e| ParseError { 
                                kind: ErrorKind::InvalidAttrs(e),
                                pos: self.pos,
                            })?;
                            v.push(Inline::Anchor { href, content });
                        }
                        // nested block-level tags are not allowed inside inlines here
                        other if other == "p" || other == "ul" || other == "ol" || other.starts_with('h') => 
                        {
                            return Err(ParseError { kind: ErrorKind::InvalidNesting(format!("block <{}> inside inline", other)), pos: self.pos });
                        }
                        other => return Err(ParseError { kind: ErrorKind::UnsupportedTag(other.to_string()), pos: self.pos }),
                    }
                }
                Some(Token::SelfClose { tag, attrs }) => {
                    match tag.as_str() {
                        "br" => { self.next(); v.push(Inline::LineBreak); }
                        "img" => {
                            let attrs = attrs.clone();
                            self.next();
                            let src = attrs.get("src")
                                .ok_or_else(|| ParseError { 
                                    kind: ErrorKind::MissingRequiredAttr("src required for <img> tag".to_string()), 
                                    pos: self.pos 
                                })?
                                .clone();
                            let alt = attrs.get("alt").cloned();

                            let src = AssetIdName::from_str(&src).map_err(|e| ParseError { 
                                kind: ErrorKind::InvalidAttrs(e),
                                pos: self.pos,
                            })?;
                            v.push(Inline::Image { src, alt });
                        }
                        other => return Err(ParseError { kind: ErrorKind::UnsupportedTag(other.to_string()), pos: self.pos }),
                    }
                }
                Some(Token::CloseTag(_)) => return Err(ParseError { kind: ErrorKind::UnexpectedToken("unexpected close tag".into()), pos: self.pos }),
                None => return Err(ParseError { kind: ErrorKind::UnexpectedEOF, pos: self.pos }),
            }
        }
        Ok(v)
    }

    fn parse_blocks_until(&mut self, closing: &str) -> Result<Vec<Block>, ParseError> 
    {
        let mut blocks = Vec::new();

        loop 
        {
            match self.peek() 
            {
                Some(Token::CloseTag(n)) if n == closing => {
                    self.next(); // consume </closing>
                    break;
                }
                Some(Token::OpenTag { tag, .. }) => match tag.as_str() {
                    "p" => blocks.push(self.parse_paragraph()?),
                    "ul" | "ol" => blocks.push(self.parse_list()?),
                    n if n.starts_with('h') && n.len() == 2 && n.chars().nth(1).unwrap().is_ascii_digit() => {
                        blocks.push(self.parse_heading()?);
                    }
                    "li" => {
                        return Err(ParseError {
                            kind: ErrorKind::InvalidNesting(
                                "nested <li> not allowed directly".into(),
                            ),
                            pos: self.pos,
                        })
                    }
                    other => {
                        return Err(ParseError {
                            kind: ErrorKind::UnsupportedTag(other.to_string()),
                            pos: self.pos,
                        })
                    }
                },
                Some(Token::Text(t)) => {
                    if t.trim().is_empty() {
                        self.next(); // skip whitespace text nodes
                    } else {
                        blocks.push(self.parse_paragraph()?);
                    }
                }
                Some(Token::SelfClose { tag, .. }) if tag == "hr" => {
                    self.next();
                    blocks.push(Block::HorizontalRule);
                }
                None => {
                    return Err(ParseError {
                        kind: ErrorKind::UnexpectedEOF,
                        pos: self.pos,
                    })
                }
                _ => {
                    return Err(ParseError {
                        kind: ErrorKind::UnexpectedToken(format!("unexpected token in <{}>", closing)),
                        pos: self.pos,
                    })
                }
            }
        }

        Ok(blocks)
    }
}