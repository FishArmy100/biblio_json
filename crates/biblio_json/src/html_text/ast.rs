use itertools::Itertools;

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
    Image { src: String, alt: Option<String> },
    Anchor {
        href: String,
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
                    format!("<img src=\"{}\" alt=\"{}\">", escape_attr(src), escape_attr(alt_text))
                } else {
                    format!("<img src=\"{}\">", escape_attr(src))
                }
            },
            Inline::Anchor { href, content } => format!("<a href=\"{}\">{}</a>", escape_attr(href), content.iter().map(Self::to_html).join("")),
        }
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
