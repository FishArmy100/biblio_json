use crate::{core::{RefId, StrongsNumber}, html_text::{HtmlText, ast::{HRefSrc, Node}}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlItem<'a>
{
    Word(&'a str),
    Strongs(StrongsNumber),
    RefId(&'a RefId),
    EntryRef {
        module_alias: &'a str,
        entry_id: u32,
    }
}

pub struct HtmlTextIter<'a> 
{
    stack: Vec<NodeIterFrame<'a>>,
}

struct NodeIterFrame<'a> 
{
    iter: std::slice::Iter<'a, Node>,
    pending_href: Option<HtmlItem<'a>>,
}

impl HtmlText 
{
    pub fn iter(&self) -> HtmlTextIter<'_> 
    {
        HtmlTextIter {
            stack: vec![NodeIterFrame {
                iter: self.nodes.iter(),
                pending_href: None,
            }],
        }
    }
}

impl<'a> Iterator for HtmlTextIter<'a> 
{
    type Item = HtmlItem<'a>;

    fn next(&mut self) -> Option<Self::Item> 
    {
        while let Some(frame) = self.stack.last_mut() 
        {
            // Return pending href before content
            if let Some(item) = frame.pending_href.take() {
                return Some(item);
            }

            let node = match frame.iter.next() 
            {
                Some(n) => n,
                None => {
                    self.stack.pop();
                    continue;
                }
            };

            match node 
            {
                Node::Text(text) => {
                    return Some(HtmlItem::Word(text));
                }

                Node::Anchor { href, content } => {
                    let href_item = match href {
                        HRefSrc::RefId(id) => HtmlItem::RefId(id),
                        HRefSrc::Strongs(num) => HtmlItem::Strongs(*num),
                        HRefSrc::ModuleRef { module_alias, entry_id } => {
                            HtmlItem::EntryRef {
                                module_alias,
                                entry_id: *entry_id,
                            }
                        }
                    };

                    self.stack.push(NodeIterFrame {
                        iter: content.iter(),
                        pending_href: Some(href_item),
                    });
                }

                // Inline containers
                Node::Bold(children)
                | Node::Italic(children)
                | Node::Underline(children)
                | Node::Strike(children)
                | Node::Paragraph(children)
                | Node::ListItem(children) => {
                    self.stack.push(NodeIterFrame {
                        iter: children.iter(),
                        pending_href: None,
                    });
                }

                // Block containers
                Node::Heading { content, .. } => {
                    self.stack.push(NodeIterFrame {
                        iter: content.iter(),
                        pending_href: None,
                    });
                }

                Node::List { items, .. } => {
                    self.stack.push(NodeIterFrame {
                        iter: items.iter(),
                        pending_href: None,
                    });
                }

                // Nodes with no output
                Node::HorizontalRule | Node::LineBreak | Node::Image { .. } => {
                    continue;
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::html_text::ast::{AssetIdName, HeadingLevel};

    use super::*;

    fn create_html_text(nodes: Vec<Node>) -> HtmlText {
        HtmlText { nodes }
    }

    #[test]
    fn test_iter_plain_text() {
        let html = create_html_text(vec![
            Node::Paragraph(vec![Node::Text("Hello".to_string())]),
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 1);
        assert!(matches!(items[0], HtmlItem::Word("Hello")));
    }

    #[test]
    fn test_iter_multiple_text_nodes() {
        let html = create_html_text(vec![
            Node::Paragraph(vec![
                Node::Text("Hello".to_string()),
                Node::Text(" ".to_string()),
                Node::Text("World".to_string()),
            ]),
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_iter_bold_italic_underline() {
        let html = create_html_text(vec![
            Node::Paragraph(vec![
                Node::Bold(vec![Node::Text("bold".to_string())]),
                Node::Italic(vec![Node::Text("italic".to_string())]),
                Node::Underline(vec![Node::Text("underline".to_string())]),
            ]),
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_iter_anchor_refid() {
        let html = create_html_text(vec![
            Node::Paragraph(vec![
                Node::Anchor {
                    href: HRefSrc::RefId(RefId::from_str("Gen.1.1").unwrap()),
                    content: vec![Node::Text("link".to_string())],
                },
            ]),
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], HtmlItem::RefId(_)));
        assert!(matches!(items[1], HtmlItem::Word("link")));
    }

    #[test]
    fn test_iter_anchor_strongs() {
        let html = create_html_text(vec![
            Node::Paragraph(vec![
                Node::Anchor {
                    href: HRefSrc::Strongs(StrongsNumber::from_str("G123").unwrap()),
                    content: vec![Node::Text("strongs".to_string())],
                },
            ]),
        ]);
        
        let items: Vec<_> = html.iter().collect(); 
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], HtmlItem::Strongs(_)));
    }

    #[test]
    fn test_iter_anchor_module_ref() {
        let html = create_html_text(vec![
            Node::Paragraph(vec![
                Node::Anchor {
                    href: HRefSrc::ModuleRef {
                        module_alias: "lex".to_string(),
                        entry_id: 42,
                    },
                    content: vec![Node::Text("module".to_string())],
                },
            ]),
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], HtmlItem::EntryRef { module_alias: "lex", entry_id: 42 }));
    }

    #[test]
    fn test_iter_nested_formatting() {
        let html = create_html_text(vec![
            Node::Paragraph(vec![
                Node::Bold(vec![
                    Node::Italic(vec![Node::Text("nested".to_string())]),
                ]),
            ]),
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 1);
        assert!(matches!(items[0], HtmlItem::Word("nested")));
    }

    #[test]
    fn test_iter_heading() {
        let html = create_html_text(vec![
            Node::Heading {
                level: HeadingLevel::H1,
                content: vec![Node::Text("Title".to_string())],
            },
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_iter_list() {
        let html = create_html_text(vec![
            Node::List {
                ordered: false,
                items: vec![
                    Node::ListItem(vec![Node::Text("item1".to_string())]),
                    Node::ListItem(vec![Node::Text("item2".to_string())]),
                ],
            },
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_iter_skip_horizontal_rule() {
        let html = create_html_text(vec![
            Node::Paragraph(vec![Node::Text("before".to_string())]),
            Node::HorizontalRule,
            Node::Paragraph(vec![Node::Text("after".to_string())]),
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_iter_skip_line_break() {
        let html = create_html_text(vec![
            Node::Paragraph(vec![
                Node::Text("line1".to_string()),
                Node::LineBreak,
                Node::Text("line2".to_string()),
            ]),
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_iter_skip_image() {
        let html = create_html_text(vec![
            Node::Paragraph(vec![
                Node::Text("text".to_string()),
                Node::Image {
                    src: AssetIdName::from_str("image1").unwrap(),
                    alt: None,
                },
            ]),
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_iter_empty() {
        let html = create_html_text(vec![]);
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_iter_strike() {
        let html = create_html_text(vec![
            Node::Paragraph(vec![
                Node::Strike(vec![Node::Text("struck".to_string())]),
            ]),
        ]);
        
        let items: Vec<_> = html.iter().collect();
        assert_eq!(items.len(), 1);
    }
}
