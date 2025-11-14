use std::{collections::HashMap, str::FromStr};

use crate::html_text::{ErrorKind, HtmlText, ParseError, ast::{AssetIdName, HRefSrc, HeadingLevel, Node}, lex::Token};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Context {
    Document,
    Paragraph,
    Heading,
    List,
    ListItem,
    Inline,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    pub fn parse(&mut self) -> Result<HtmlText, ParseError> 
    {
        let nodes = self.parse_nodes(Context::Document, None)?;
        Ok(HtmlText { nodes })
    }

    fn parse_nodes(&mut self, context: Context, closing_tag: Option<&str>) -> Result<Vec<Node>, ParseError> {
        let mut nodes = Vec::new();

        loop {
            match self.peek() {
                None => {
                    if let Some(_) = closing_tag {
                        return Err(ParseError {
                            kind: ErrorKind::UnexpectedEOF,
                            pos: self.pos,
                        });
                    }
                    break;
                }
                Some(Token::CloseTag(tag)) if Some(tag.as_str()) == closing_tag => {
                    self.next();
                    break;
                }
                Some(Token::Text(t)) => {
                    // Skip pure whitespace between tags
                    if t.trim().is_empty() {
                        self.next();
                        continue;
                    }

                    match context {
                        Context::Document => {
                            // In document root, implicit paragraph for bare text
                            nodes.push(self.parse_paragraph()?);
                        }
                        Context::List => {
                            // Lists cannot have text directly (only <li>)
                            return Err(ParseError {
                                kind: ErrorKind::InvalidNesting(
                                    "Lists can only contain <li> elements, not text".into(),
                                ),
                                pos: self.pos,
                            });
                        }
                        Context::ListItem | Context::Paragraph | Context::Heading | Context::Inline => {
                            // Inside list item or inline contexts, treat as plain text (no paragraph wrapping)
                            nodes.push(Node::Text(t.clone()));
                            self.next();
                        }
                    }
                }
                Some(Token::OpenTag { tag, .. }) => {
                    match (context, tag.as_str()) {
                        (Context::Document | Context::ListItem, "p") => {
                            nodes.push(self.parse_paragraph()?);
                        }
                        (Context::Document | Context::ListItem, tag) if tag.starts_with('h') && tag.len() == 2 => {
                            nodes.push(self.parse_heading()?);
                        }
                        (Context::Document | Context::ListItem, "ul" | "ol") => {
                            nodes.push(self.parse_list()?);
                        }
                        (Context::List, "li") => {
                            nodes.push(self.parse_list_item()?);
                        }
                        (Context::Paragraph | Context::Heading | Context::ListItem | Context::Inline, "u") => {
                            nodes.push(self.parse_inline_wrapper("u", Node::Underline)?);
                        }
                        (Context::Paragraph | Context::Heading | Context::ListItem | Context::Inline, "i" | "em") => {
                            let tag = tag.clone();
                            nodes.push(self.parse_inline_wrapper(&tag, Node::Italic)?);
                        }
                        (Context::Paragraph | Context::Heading | Context::ListItem | Context::Inline, "b" | "strong") => {
                            let tag = tag.clone();
                            nodes.push(self.parse_inline_wrapper(&tag, Node::Bold)?);
                        }
                        (Context::Paragraph | Context::Heading | Context::ListItem | Context::Inline, "s" | "strike" | "del") => {
                            let tag = tag.clone();
                            nodes.push(self.parse_inline_wrapper(&tag, Node::Strike)?);
                        }
                        (Context::Paragraph | Context::Heading | Context::ListItem | Context::Inline, "a") => {
                            nodes.push(self.parse_anchor()?);
                        }
                        _ => {
                            return Err(ParseError {
                                kind: ErrorKind::InvalidNesting(format!(
                                    "Cannot have <{}> in {:?} context",
                                    tag, context
                                )),
                                pos: self.pos,
                            });
                        }
                    }
                }
                Some(Token::SelfClose { tag, attrs }) => {
                    match (context, tag.as_str()) {
                        (Context::Document | Context::ListItem, "hr") => {
                            self.next();
                            nodes.push(Node::HorizontalRule);
                        }
                        (Context::Document | Context::Paragraph | Context::Heading | Context::ListItem | Context::Inline, "br") => {
                            self.next();
                            nodes.push(Node::LineBreak);
                        }
                        (Context::Paragraph | Context::Heading | Context::ListItem | Context::Inline, "img") => {
                            nodes.push(self.parse_image(attrs.clone())?);
                        }
                        _ => {
                            return Err(ParseError {
                                kind: ErrorKind::InvalidNesting(format!(
                                    "Cannot have <{}/> in {:?} context",
                                    tag, context
                                )),
                                pos: self.pos,
                            });
                        }
                    }
                }
                Some(Token::CloseTag(_)) => {
                    return Err(ParseError {
                        kind: ErrorKind::UnexpectedToken("unexpected close tag".into()),
                        pos: self.pos,
                    });
                }
            }
        }

        Ok(nodes)
    }

    fn parse_paragraph(&mut self) -> Result<Node, ParseError> {
        let start_pos = self.pos;
        match self.next() {
            Some(Token::OpenTag { tag, .. }) if tag == "p" => {}
            Some(Token::Text(t)) => {
                let mut content = vec![Node::Text(t)];
                while let Some(Token::Text(t)) = self.peek() {
                    content.push(Node::Text(t.clone()));
                    self.next();
                }
                return Ok(Node::Paragraph(content));
            }
            _ => {
                return Err(ParseError {
                    kind: ErrorKind::UnexpectedToken("expected <p>".into()),
                    pos: start_pos,
                });
            }
        }

        let content = self.parse_nodes(Context::Paragraph, Some("p"))?;
        Ok(Node::Paragraph(content))
    }

    fn parse_heading(&mut self) -> Result<Node, ParseError> {
        let start_pos = self.pos;
        let level = match self.next() {
            Some(Token::OpenTag { tag, .. }) if tag.len() == 2 && tag.starts_with('h') => {
                tag[1..2].parse::<u8>().unwrap()
            }
            _ => {
                return Err(ParseError {
                    kind: ErrorKind::UnexpectedToken("expected heading tag".into()),
                    pos: start_pos,
                });
            }
        };

        let level = HeadingLevel::try_from(level).map_err(|_| ParseError {
            kind: ErrorKind::UnexpectedToken(format!("h{}", level)),
            pos: start_pos,
        })?;

        let content = self.parse_nodes(Context::Heading, Some(&format!("h{}", level.to_u8())))?;
        Ok(Node::Heading { level, content })
    }

    fn parse_list(&mut self) -> Result<Node, ParseError> {
        let start_pos = self.pos;
        let (ordered, tag) = match self.next() {
            Some(Token::OpenTag { tag, .. }) if tag == "ul" => (false, "ul".to_string()),
            Some(Token::OpenTag { tag, .. }) if tag == "ol" => (true, "ol".to_string()),
            _ => {
                return Err(ParseError {
                    kind: ErrorKind::UnexpectedToken("expected <ul> or <ol>".into()),
                    pos: start_pos,
                });
            }
        };

        let items = self.parse_nodes(Context::List, Some(&tag))?;
        
        // Validate that all children are ListItems
        for item in &items {
            if !matches!(item, Node::ListItem(_)) {
                return Err(ParseError {
                    kind: ErrorKind::InvalidNesting("List can only contain ListItem nodes".into()),
                    pos: self.pos,
                });
            }
        }

        Ok(Node::List { ordered, items })
    }

    fn parse_list_item(&mut self) -> Result<Node, ParseError> {
        let start_pos = self.pos;
        match self.next() {
            Some(Token::OpenTag { tag, .. }) if tag == "li" => {}
            _ => {
                return Err(ParseError {
                    kind: ErrorKind::UnexpectedToken("expected <li>".into()),
                    pos: start_pos,
                });
            }
        }

        let content = self.parse_nodes(Context::ListItem, Some("li"))?;
        Ok(Node::ListItem(content))
    }

    fn parse_inline_wrapper<F>(&mut self, tag: &str, wrapper: F) -> Result<Node, ParseError>
    where
        F: Fn(Vec<Node>) -> Node,
    {
        self.next(); // consume opening tag
        let content = self.parse_nodes(Context::Inline, Some(tag))?;
        Ok(wrapper(content))
    }

    fn parse_anchor(&mut self) -> Result<Node, ParseError> {
        let start_pos = self.pos;
        let href = match self.next() {
            Some(Token::OpenTag { attrs, .. }) => {
                attrs.get("href").ok_or_else(|| ParseError {
                    kind: ErrorKind::MissingRequiredAttr("href required for <a>".into()),
                    pos: start_pos,
                })?
                .clone()
            }
            _ => {
                return Err(ParseError {
                    kind: ErrorKind::UnexpectedToken("expected <a>".into()),
                    pos: start_pos,
                });
            }
        };

        let href = HRefSrc::from_str(&href).map_err(|e| ParseError {
            kind: ErrorKind::InvalidAttrs(e),
            pos: start_pos,
        })?;

        let content = self.parse_nodes(Context::Inline, Some("a"))?;
        Ok(Node::Anchor { href, content })
    }

    fn parse_image(&mut self, attrs: HashMap<String, String>) -> Result<Node, ParseError> {
        let start_pos = self.pos;
        self.next(); // consume self-closing tag

        let src = attrs.get("src").ok_or_else(|| ParseError {
            kind: ErrorKind::MissingRequiredAttr("src required for <img>".into()),
            pos: start_pos,
        })?;

        let src = AssetIdName::from_str(src).map_err(|e| ParseError {
            kind: ErrorKind::InvalidAttrs(e),
            pos: start_pos,
        })?;

        let alt = attrs.get("alt").cloned();

        Ok(Node::Image { src, alt })
    }
}
