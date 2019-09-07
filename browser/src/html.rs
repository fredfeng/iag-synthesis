//! A simple parser for a tiny subset of HTML.
//!
//! Can parse basic opening and closing tags, and text nodes.
//!
//! Not yet supported:
//!
//! * Comments
//! * Doctypes and processing instructions
//! * Non-well-formed markup
//! * Character entities

use crate::dom::{AttributeMap, DocumentNode, DocumentTree};
use std::collections::HashMap;

/// Parse an HTML document, number its tree nodes, and return the initialized
/// document tree.
///
/// See `dom::DocumentNode::number_preorder()` for information on node numbering.
pub fn parse_document(source: String) -> DocumentTree {
    DocumentTree::new(Parser::new(source).parse_document())
}

pub struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    /// The only doctype directive recognized by this parser.
    pub const HTML_DOCTYPE: &'static str = "<!DOCTYPE html>";

    /// Create a fresh HTML parser on the given input string.
    pub fn new(input: String) -> Parser {
        Parser { pos: 0, input }
    }

    /// Parse a whole HTML document.
    pub fn parse_document(&mut self) -> Vec<DocumentNode> {
        self.parse_doctype();
        self.parse_nodes()
    }

    /// Parse the opening doctype directive, if present, in order to ignore it.
    fn parse_doctype(&mut self) {
        self.consume_whitespace();
        if self.starts_with(Parser::HTML_DOCTYPE) {
            for ch in Parser::HTML_DOCTYPE.chars() {
                assert_eq!(self.consume_char(), ch);
            }
        }
    }

    /// Parse a sequence of sibling nodes.
    fn parse_nodes(&mut self) -> Vec<DocumentNode> {
        let mut nodes = vec![];
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }

    /// Parse a single node.
    fn parse_node(&mut self) -> DocumentNode {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        }
    }

    /// Parse a single element, including its open tag, contents, and closing tag.
    fn parse_element(&mut self) -> DocumentNode {
        // Opening tag.
        assert_eq!(self.consume_char(), '<');
        let tag_name = self.parse_identifier();
        let attr_map = self.parse_attributes();
        let children = if self.next_char() == '/' {
            // Self-closing tag.
            assert_eq!(self.consume_char(), '/');
            assert_eq!(self.consume_char(), '>');
            Vec::new()
        } else {
            // Content-enclosing pair of tags.
            assert_eq!(self.consume_char(), '>');
            let nodes = self.parse_nodes();
            assert_eq!(self.consume_char(), '<');
            assert_eq!(self.consume_char(), '/');
            assert_eq!(self.parse_identifier(), tag_name);
            assert_eq!(self.consume_char(), '>');
            nodes
        };

        DocumentNode::new_elem(tag_name, attr_map, children)
    }

    /// Parse a tag or attribute name.
    fn parse_identifier(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => true,
            _ => false,
        })
    }

    /// Parse a list of name="value" pairs, separated by whitespace.
    fn parse_attributes(&mut self) -> AttributeMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if !self.next_char().is_alphanumeric() {
                break;
            }
            let (name, value) = self.parse_attribute();
            attributes.insert(name, value);
        }
        AttributeMap::new(attributes)
    }

    /// Parse a single name="value" pair.
    fn parse_attribute(&mut self) -> (String, String) {
        let name = self.parse_identifier();
        assert_eq!(self.consume_char(), '=');
        let value = self.parse_quotation();
        (name, value)
    }

    /// Parse a quoted value.
    fn parse_quotation(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert_eq!(self.consume_char(), open_quote);
        value
    }

    /// Parse a text node.
    fn parse_text(&mut self) -> DocumentNode {
        DocumentNode::new_text(self.consume_while(|c| c != '<'))
    }

    /// Consume and discard zero or more whitespace characters.
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    /// Consume characters until `test` returns false.
    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    /// Return the current character, and advance self.pos to the next character.
    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    /// Read the current character without consuming it.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    /// Does the current input start with the given string?
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    /// Return true if all input is consumed.
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}
