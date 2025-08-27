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

pub mod lex;
pub mod parse;
pub mod ast;

use std::{collections::HashMap, fmt, ops::Deref};

use itertools::Itertools;

use crate::html_text::{ast::Block, lex::Lexer, parse::{ParseError, Parser}};

#[derive(Debug, Clone, PartialEq)]
pub struct HtmlText(Vec<Block>);

impl Deref for HtmlText
{
    type Target = Vec<Block>;

    fn deref(&self) -> &Self::Target 
    {
        &self.0
    }
}

impl HtmlText 
{
    pub fn to_html(&self) -> String 
    {
        self.iter().map(|b| b.to_html()).collect::<Vec<_>>().join("")
    }

    pub fn from_str(html: &str) -> Result<HtmlText, ParseError>
    {
        let mut lexer = Lexer::new(html);
        let tokens = lexer.tokenize()?;
        
        let mut p = Parser::new(tokens);
        p.parse()
    }
}
#[cfg(test)]
mod tests {
    use crate::html_text::ast::Inline;

    use super::*;
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;
    use rand::seq::IndexedRandom;

    // Basic functionality tests
    #[test]
    fn test_basic_paragraph() {
        let html = "<p>Hello world</p>";
        let result = HtmlText::from_str(html).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Block::Paragraph(inlines) => {
                assert_eq!(inlines.len(), 1);
                match &inlines[0] {
                    Inline::Text(text) => assert_eq!(text, "Hello world"),
                    _ => panic!("Expected text inline"),
                }
            }
            _ => panic!("Expected paragraph block"),
        }
    }

    #[test]
    fn test_heading_levels() {
        for level in 1..=6 {
            let html = format!("<h{}>Heading {}</h{}>", level, level, level);
            let result = HtmlText::from_str(&html).unwrap();
            match &result[0] {
                Block::Heading { level: l, content } => {
                    assert_eq!(*l, level);
                    assert_eq!(content.len(), 1);
                }
                _ => panic!("Expected heading block"),
            }
        }
    }

    #[test]
    fn test_formatting_tags() {
        let test_cases = vec![
            ("<b>bold</b>", "bold"),
            ("<strong>strong</strong>", "strong"),
            ("<i>italic</i>", "italic"),
            ("<em>emphasis</em>", "emphasis"),
            ("<u>underline</u>", "underline"),
            ("<s>strike</s>", "strike"),
            ("<del>delete</del>", "delete"),
            ("<strike>strikethrough</strike>", "strikethrough"),
        ];

        for (html, expected_text) in test_cases {
            let full_html = format!("<p>{}</p>", html);
            let result = HtmlText::from_str(&full_html).unwrap();
            match &result[0] {
                Block::Paragraph(inlines) => {
                    assert_eq!(inlines.len(), 1);
                    // Check that it parsed as a formatting inline (not just text)
                    match &inlines[0] {
                        Inline::Text(_) => panic!("Expected formatting inline, got text for: {}", html),
                        _ => {
                            // Extract inner text to verify content
                            let html_output = result.to_html();
                            assert!(html_output.contains(expected_text));
                        }
                    }
                }
                _ => panic!("Expected paragraph block"),
            }
        }
    }

    #[test]
    fn test_nested_formatting() {
        let html = "<p><b>bold <i>and italic</i> text</b></p>";
        let result = HtmlText::from_str(html).unwrap();
        assert!(result.to_html().contains("bold") && result.to_html().contains("italic"));
    }

    #[test]
    fn test_lists() {
        // Unordered list
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let result = HtmlText::from_str(html).unwrap();
        match &result[0] {
            Block::List { ordered, items } => {
                assert!(!ordered);
                assert_eq!(items.len(), 2);
            }
            _ => panic!("Expected list block"),
        }

        // Ordered list
        let html = "<ol><li>First</li><li>Second</li></ol>";
        let result = HtmlText::from_str(html).unwrap();
        match &result[0] {
            Block::List { ordered, items } => {
                assert!(ordered);
                assert_eq!(items.len(), 2);
            }
            _ => panic!("Expected list block"),
        }
    }

    #[test]
    fn test_self_closing_tags() {
        let html = "<p>Line 1<br>Line 2</p><hr>";
        let result = HtmlText::from_str(html).unwrap();
        assert_eq!(result.len(), 2);
        
        match &result[1] {
            Block::HorizontalRule => {},
            _ => panic!("Expected horizontal rule"),
        }
    }

    #[test]
    fn test_attributes() {
        // Test anchor with href
        let html = r#"<p><a href="https://example.com">Link</a></p>"#;
        let result = HtmlText::from_str(html).unwrap();
        let html_output = result.to_html();
        assert!(html_output.contains(r#"href="https://example.com""#));

        // Test image with src and alt
        let html = r#"<p><img src="image.jpg" alt="Test image"></p>"#;
        let result = HtmlText::from_str(html).unwrap();
        let html_output = result.to_html();
        assert!(html_output.contains(r#"src="image.jpg""#));
        assert!(html_output.contains(r#"alt="Test image""#));
    }

    #[test]
    fn test_attribute_edge_cases() {
        // Test special characters in attributes
        let html = r#"<p><img src="<test>"></p>"#;
        let result = HtmlText::from_str(html).unwrap();
        let html_output = result.to_html();
        assert!(html_output.contains("&lt;test&gt;"));

        // Test quotes in attributes
        let html = r#"<p><img src='He said "hello"'></p>"#;
        let result = HtmlText::from_str(html).unwrap();
        let html_output = result.to_html();
        assert!(html_output.contains("He said"));
    }

    #[test]
    fn test_implicit_paragraphs() {
        let html = "Just some text";
        let result = HtmlText::from_str(html).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Block::Paragraph(_) => {},
            _ => panic!("Expected implicit paragraph"),
        }
    }

    #[test]
    fn test_whitespace_handling() {
        let html = "   <p>  Text with spaces  </p>   ";
        let result = HtmlText::from_str(html).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_error_cases() {
        // Unclosed tags
        assert!(HtmlText::from_str("<p>unclosed").is_err());
        
        // Unsupported tags
        assert!(HtmlText::from_str("<div>content</div>").is_err());
        
        // Missing required attributes
        assert!(HtmlText::from_str("<a>link</a>").is_err());
        assert!(HtmlText::from_str("<img>").is_err());
        
        // Invalid nesting
        assert!(HtmlText::from_str("<p><ul><li>item</li></ul></p>").is_err());
        
        // Malformed tags
        assert!(HtmlText::from_str("<>content</>").is_err());
    }

    // Fuzzer implementation
    pub struct HtmlFuzzer {
        rng: StdRng,
        max_depth: usize,
        max_siblings: usize,
    }

    impl HtmlFuzzer {
        pub fn new(seed: u64) -> Self {
            Self {
                rng: StdRng::seed_from_u64(seed),
                max_depth: 5,
                max_siblings: 10,
            }
        }

        pub fn generate_html(&mut self) -> String {
            self.generate_blocks(0)
        }

        fn generate_blocks(&mut self, depth: usize) -> String {
            if depth >= self.max_depth {
                return self.generate_text();
            }

            let num_blocks = self.rng.random_range(1..=self.max_siblings.min(5));
            let mut html = String::new();

            for _ in 0..num_blocks {
                html.push_str(&self.generate_block(depth));
            }

            html
        }

        fn generate_block(&mut self, depth: usize) -> String {
            let block_types = vec!["p", "h1", "h2", "h3", "ul", "ol", "hr"];
            let block_type = block_types.choose(&mut self.rng).unwrap();

            match *block_type {
                "p" => format!("<p>{}</p>", self.generate_inlines(depth + 1)),
                "h1" | "h2" | "h3" => format!("<{}>{}</{}>", block_type, self.generate_inlines(depth + 1), block_type),
                "ul" => self.generate_list(false, depth),
                "ol" => self.generate_list(true, depth),
                "hr" => "<hr>".to_string(),
                _ => unreachable!(),
            }
        }

        fn generate_list(&mut self, ordered: bool, depth: usize) -> String {
            let tag = if ordered { "ol" } else { "ul" };
            let num_items = self.rng.random_range(1..=5);
            let mut html = format!("<{}>", tag);

            for _ in 0..num_items {
                html.push_str(&format!("<li>{}</li>", self.generate_inlines(depth + 1)));
            }

            html.push_str(&format!("</{}>", tag));
            html
        }

        fn generate_inlines(&mut self, depth: usize) -> String {
            if depth >= self.max_depth {
                return self.generate_text();
            }

            let num_inlines = self.rng.random_range(1..=5);
            let mut html = String::new();

            for _ in 0..num_inlines {
                html.push_str(&self.generate_inline(depth));
            }

            html
        }

        fn generate_inline(&mut self, depth: usize) -> String {
            let inline_types = vec!["text", "b", "i", "u", "s", "em", "strong", "a", "img", "br"];
            let inline_type = inline_types.choose(&mut self.rng).unwrap();

            match *inline_type {
                "text" => self.generate_text(),
                "b" | "i" | "u" | "s" | "em" | "strong" => {
                    format!("<{}>{}</{}>", inline_type, self.generate_inlines(depth + 1), inline_type)
                }
                "a" => {
                    let href = self.generate_url();
                    format!("<a href=\"{}\">{}</a>", href, self.generate_inlines(depth + 1))
                }
                "img" => {
                    let src = self.generate_url();
                    let alt = if self.rng.random_bool(0.7) {
                        format!(" alt=\"{}\"", self.generate_safe_text())
                    } else {
                        String::new()
                    };
                    format!("<img src=\"{}\"{}>", src, alt)
                }
                "br" => "<br>".to_string(),
                _ => unreachable!(),
            }
        }

        fn generate_text(&mut self) -> String {
            let words = vec![
                "hello", "world", "test", "content", "sample", "data", "text",
                "lorem", "ipsum", "dolor", "sit", "amet", "the", "quick",
                "brown", "fox", "jumps", "over", "lazy", "dog",
            ];

            let num_words = self.rng.random_range(1..=10);
            let selected_words: Vec<&str> = (0..num_words)
                .map(|_| *words.choose(&mut self.rng).unwrap())
                .collect();

            // Sometimes add special characters to test escaping
            let mut text = selected_words.join(" ");
            if self.rng.random_bool(0.2) {
                let specials = vec!["&", "<", ">", "\"", "'"];
                let special = specials.choose(&mut self.rng).unwrap();
                text.push_str(special);
            }

            text
        }

        fn generate_safe_text(&mut self) -> String {
            let words = vec!["safe", "text", "content", "image", "description"];
            let num_words = self.rng.random_range(1..=3);
            (0..num_words)
                .map(|_| *words.choose(&mut self.rng).unwrap())
                .collect::<Vec<&str>>()
                .join(" ")
        }

        fn generate_url(&mut self) -> String {
            let protocols = vec!["http://", "https://", ""];
            let domains = vec!["example.com", "test.org", "sample.net", "localhost"];
            let paths = vec!["", "/path", "/image.jpg", "/page.html", "/test"];

            let protocol = protocols.choose(&mut self.rng).unwrap();
            let domain = domains.choose(&mut self.rng).unwrap();
            let path = paths.choose(&mut self.rng).unwrap();

            // Sometimes add problematic characters to test attribute parsing
            if self.rng.random_bool(0.1) {
                format!("{}{}{}<test>", protocol, domain, path)
            } else {
                format!("{}{}{}", protocol, domain, path)
            }
        }
    }

    #[test]
    fn test_fuzzer_basic() {
        let mut fuzzer = HtmlFuzzer::new(42);
        
        for i in 0..50 {
            let html = fuzzer.generate_html();
            println!("Fuzzer test {}: {}", i, &html[..html.len().min(100)]);
            
            match HtmlText::from_str(&html) {
                Ok(parsed) => {
                    // Verify round-trip doesn't crash
                    let _output = parsed.to_html();
                }
                Err(e) => {
                    // Some generated HTML will be invalid - that's expected
                    println!("Expected error for test {}: {:?}", i, e);
                }
            }
        }
    }

    #[test]
    fn test_fuzzer_stress() {
        let seeds = vec![1, 42, 123, 999, 12345];
        
        for seed in seeds {
            let mut fuzzer = HtmlFuzzer::new(seed);
            let mut valid_count = 0;
            let mut error_count = 0;
            
            for _ in 0..100 {
                let html = fuzzer.generate_html();
                
                match HtmlText::from_str(&html) {
                    Ok(parsed) => {
                        valid_count += 1;
                        // Ensure round-trip works
                        let output = parsed.to_html();
                        assert!(!output.is_empty());
                    }
                    Err(_) => {
                        error_count += 1;
                        // Errors are expected for invalid HTML
                    }
                }
            }
            
            println!("Seed {}: {} valid, {} errors", seed, valid_count, error_count);
            assert!(valid_count > 0, "Should have at least some valid parses");
        }
    }

    #[test]
    fn test_fuzzer_edge_cases() {
        let edge_cases = vec![
            "",  // Empty input
            "   ",  // Whitespace only
            "<",  // Incomplete tag
            ">",  // Stray closing bracket
            "<>",  // Empty tag
            "</p>",  // Closing tag without opening
            "<p><p>nested</p></p>",  // Invalid nesting
            "<p>text</p><p>more text</p>",  // Multiple blocks
            "<p>text with &amp; entities</p>",  // Entities (not fully supported)
            "<p attr='value'>text</p>",  // Unsupported attributes on p tag
            r#"<img src="test" alt="description with "quotes"">"#,  // Complex quotes
            "<a href='javascript:alert(\"xss\")'>link</a>",  // XSS attempt
        ];

        for (i, html) in edge_cases.iter().enumerate() {
            println!("Testing edge case {}: {}", i, html);
            match HtmlText::from_str(html) {
                Ok(parsed) => {
                    let output = parsed.to_html();
                    println!("  -> Success: {}", output);
                }
                Err(e) => {
                    println!("  -> Error: {:?}", e);
                }
            }
        }
    }

    #[test]
    fn test_performance() {
        use std::time::Instant;
        
        let mut fuzzer = HtmlFuzzer::new(999);
        let mut total_time = std::time::Duration::new(0, 0);
        let mut successful_parses = 0;
        
        for _ in 0..1000 {
            let html = fuzzer.generate_html();
            let start = Instant::now();
            
            if let Ok(_) = HtmlText::from_str(&html) {
                successful_parses += 1;
            }
            
            total_time += start.elapsed();
        }
        
        let avg_time = total_time.as_nanos() / 1000;
        println!("Average parse time: {}ns ({} successful)", avg_time, successful_parses);
        
        // Ensure reasonable performance (less than 1ms average)
        assert!(avg_time < 1_000_000, "Parser too slow: {}ns average", avg_time);
    }

    #[test]
    fn test_memory_safety() {
        // Test with deeply nested content
        let mut deep_html = String::new();
        for _ in 0..50 {
            deep_html.push_str("<p><b>");
        }
        deep_html.push_str("deep content");
        for _ in 0..50 {
            deep_html.push_str("</b></p>");
        }
        
        // Should either parse or error gracefully without crashing
        let _ = HtmlText::from_str(&deep_html);
        
        // Test with very long text
        let long_text = "a".repeat(10000);
        let long_html = format!("<p>{}</p>", long_text);
        let result = HtmlText::from_str(&long_html).unwrap();
        assert!(result.to_html().len() > 10000);
    }
}