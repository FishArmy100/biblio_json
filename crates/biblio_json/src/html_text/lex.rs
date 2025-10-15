use std::collections::HashMap;

use crate::html_text::{ErrorKind, ParseError};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    OpenTag {
        tag: String,
        attrs: HashMap<String, String>,
    },
    CloseTag(String),
    SelfClose {
        tag: String,
        attrs: HashMap<String, String>,
    },
    Text(String),
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        while self.pos < self.chars.len() {
            if self.chars[self.pos] == '<' {
                tokens.push(self.parse_tag()?);
            } else {
                let mut s = String::new();
                while self.pos < self.chars.len() && self.chars[self.pos] != '<' {
                    s.push(self.chars[self.pos]);
                    self.pos += 1;
                }
                tokens.push(Token::Text(s));
            }
        }
        Ok(tokens)
    }

    fn parse_tag(&mut self) -> Result<Token, ParseError> {
        let start = self.pos;
        self.pos += 1; // consume '<'

        if self.pos >= self.chars.len() {
            return Err(ParseError {
                kind: ErrorKind::UnexpectedEOF,
                pos: start,
            });
        }

        let is_close = self.chars[self.pos] == '/';
        if is_close {
            self.pos += 1;
        }

        let mut tag_content = String::new();
        let mut in_quote = false;
        let mut quote_char = '"';

        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];

            if ch == '>' && !in_quote {
                break;
            }

            if (ch == '"' || ch == '\'') && !in_quote {
                in_quote = true;
                quote_char = ch;
            } else if ch == quote_char && in_quote {
                in_quote = false;
            }

            tag_content.push(ch);
            self.pos += 1;
        }

        if self.pos >= self.chars.len() {
            return Err(ParseError {
                kind: ErrorKind::UnexpectedEOF,
                pos: start,
            });
        }

        self.pos += 1; // consume '>'

        let tag_content = tag_content.trim();

        if is_close {
            return Ok(Token::CloseTag(tag_content.to_ascii_lowercase()));
        }

        let self_closing = tag_content.ends_with('/');
        let tag_content = if self_closing {
            tag_content.trim_end_matches('/').trim()
        } else {
            tag_content
        };

        let (tag_name, attrs) = self.parse_tag_and_attrs(tag_content)?;
        let tag_name = tag_name.to_ascii_lowercase();

        let inherently_self_closing = matches!(tag_name.as_str(), "br" | "hr" | "img");

        if self_closing || inherently_self_closing {
            Ok(Token::SelfClose { tag: tag_name, attrs })
        } else {
            Ok(Token::OpenTag { tag: tag_name, attrs })
        }
    }

    fn parse_tag_and_attrs(&self, content: &str) -> Result<(String, HashMap<String, String>), ParseError> {
        let mut parts = content.splitn(2, char::is_whitespace);
        let tag_name = parts.next().unwrap_or("").to_string();

        let mut attrs = HashMap::new();
        if let Some(attr_str) = parts.next() {
            attrs = self.parse_attributes(attr_str.trim())?;
        }

        Ok((tag_name, attrs))
    }

    fn parse_attributes(&self, attr_str: &str) -> Result<HashMap<String, String>, ParseError> {
        let mut attrs = HashMap::new();
        let mut chars = attr_str.chars().peekable();

        while chars.peek().is_some() {
            while matches!(chars.peek(), Some(&' ') | Some(&'\t') | Some(&'\n')) {
                chars.next();
            }

            if chars.peek().is_none() {
                break;
            }

            let mut name = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '=' || ch.is_whitespace() {
                    break;
                }
                name.push(chars.next().unwrap());
            }

            if name.is_empty() {
                break;
            }

            while matches!(chars.peek(), Some(&' ') | Some(&'\t') | Some(&'\n')) {
                chars.next();
            }

            if chars.peek() != Some(&'=') {
                attrs.insert(name.to_ascii_lowercase(), String::new());
                continue;
            }

            chars.next(); // consume '='

            while matches!(chars.peek(), Some(&' ') | Some(&'\t') | Some(&'\n')) {
                chars.next();
            }

            let mut value = String::new();
            if matches!(chars.peek(), Some(&'"') | Some(&'\'')) {
                let quote = chars.next().unwrap();
                while let Some(ch) = chars.next() {
                    if ch == quote {
                        break;
                    }
                    value.push(ch);
                }
            } else {
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() {
                        break;
                    }
                    value.push(chars.next().unwrap());
                }
            }

            attrs.insert(name.to_ascii_lowercase(), value);
        }

        Ok(attrs)
    }
}