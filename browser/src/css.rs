//! A simple parser for a tiny subset of CSS.
//!
//! To support more CSS syntax, it would probably be easiest to replace this
//! hand-rolled parser with one based on a library or parser generator.

use crate::utility::Color;
use crate::user_agent;

// Data structures:

#[derive(Clone, Debug)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Clone, Debug)]
pub enum Selector {
    Simple(SimpleSelector),
}

#[derive(Clone, Debug)]
pub struct SimpleSelector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    Percent(f32),
    ColorValue(Color),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unit {
    Cm,
    Mm,
    Q,
    In,
    Pc,
    Pt,
    Px,
    // Em,
    // Rem,
}

impl Unit {
    pub fn to_px(self, length: f32) -> f32 {
        use Unit::*;

        match self {
            Cm => In.to_px(length) / 2.54,
            Mm => Cm.to_px(length) / 10.0,
            Q => Cm.to_px(length) / 40.0,
            In => length * 96.0,
            Pc => In.to_px(length) / 6.0,
            Pt => In.to_px(length) / 72.0,
            Px => length,
        }
    }
}

pub type Specificity = (usize, usize, usize);

impl Selector {
    pub fn specificity(&self) -> Specificity {
        // http://www.w3.org/TR/selectors/#specificity
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag.iter().count();
        (a, b, c)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Keyword(ref kw) => f.write_str(kw),
            Value::Length(l, u) => write!(f, "{}{}", l, u),
            Value::Percent(p) => write!(f, "{}%", p),
            Value::ColorValue(c) => write!(f, "{}", c),
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Unit::Cm => f.write_str("cm"),
            Unit::Mm => f.write_str("mm"),
            Unit::Q => f.write_str("Q"),
            Unit::In => f.write_str("in"),
            Unit::Pc => f.write_str("pc"),
            Unit::Pt => f.write_str("pt"),
            Unit::Px => f.write_str("px"),
            // Unit::Em => f.write_str("em"),
        }
    }
}

/// Parse a CSS stylesheet.
pub fn parse(source: String) -> Stylesheet {
    Stylesheet { rules: Parser::new(source).parse_rules() }
}

/// Parse the user agent stylesheet.
pub fn user_agent() -> Stylesheet {
    parse(user_agent::STYLESHEET_SOURCE.to_owned())
}

struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    /// Assemble initial parser state for an owned string of input.
    fn new(input: String) -> Self {
        Parser { pos: 0, input }
    }

    /// Parse a list of rule sets, separated by optional whitespace.
    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.advance();
            if self.eof() {
                break;
            }
            rules.push(self.parse_rule());
        }
        rules
    }

    /// Parse a rule set: `<selectors> { <declarations> }`.
    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    /// Parse a comma-separated list of selectors.
    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.advance();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.advance();
                }
                '{' => break,
                c => panic!("Unexpected character {} in selector list", c),
            }
        }
        // Return selectors with highest specificity first, for use in matching.
        selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));
        selectors
    }

    /// Parse one simple selector, e.g.: `type#id.class1.class2.class3`
    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag: None,
            id: None,
            class: Vec::new(),
        };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    // universal selector
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        selector
    }

    /// Parse a list of declarations enclosed in `{ ... }`.
    fn parse_declarations(&mut self) -> Vec<Declaration> {
        assert_eq!(self.consume_char(), '{');
        let mut declarations = Vec::new();
        loop {
            self.advance();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    /// Parse one `<property>: <value>;` declaration.
    fn parse_declaration(&mut self) -> Declaration {
        let property_name = self.parse_identifier();
        self.advance();
        assert_eq!(self.consume_char(), ':');
        self.advance();
        let value = self.parse_value();
        self.advance();
        if self.next_char() != '}' {
            assert_eq!(self.consume_char(), ';');
        }

        Declaration {
            name: property_name,
            value: value,
        }
    }

    // Methods for parsing values:

    fn parse_value(&mut self) -> Value {
        match self.next_char() {
            '-' | '0'..='9' | '.' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier()),
        }
    }

    fn parse_length(&mut self) -> Value {
        let number = self.parse_float();
        if self.next_char() == '%' {
            self.consume_char();
            Value::Percent(number)
        } else {
            Value::Length(number, self.parse_unit())
        }
    }

    fn parse_float(&mut self) -> f32 {
        self.consume_while(
            |ch| match ch {
                '-' | '0'..='9' | '.' => true,
                _ => false
            }
        ).parse::<f32>().unwrap()
    }

    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "cm" => Unit::Cm,
            "mm" => Unit::Mm,
            "q" => Unit::Q,
            "in" => Unit::In,
            "pc" => Unit::Pc,
            "pt" => Unit::Pt,
            "px" => Unit::Px,
            // "em" => Unit::Em,
            _ => panic!("unrecognized unit"),
        }
    }

    fn parse_color(&mut self) -> Value {
        assert_eq!(self.consume_char(), '#');
        Value::ColorValue(Color {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255,
        })
    }

    /// Parse two hexadecimal digits.
    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos..self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    /// Parse a property name or keyword.
    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    /// Consume and discard syntactic noise (i.e., comments and whitespace).
    fn advance(&mut self) {
        // Consume leading whitespace.
        self.consume_while(char::is_whitespace);
        while self.peek().starts_with("/*") {
            // Consume the opening delimiter.
            self.consume_char();
            self.consume_char();
            // Skip past the comment text.
            self.pos += self.peek().find("*/").expect("Dangling comment!");
            // Consume the closing delimiter.
            self.consume_char();
            self.consume_char();
            // Consume interleaving/trailing whitespace.
            self.consume_while(char::is_whitespace);
        }
    }

    /// Consume characters until `test` returns false.
    fn consume_while<F>(&mut self, pred: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let base = self.pos;
        let view = self.peek();
        self.pos += view.find(|ch| !pred(ch)).unwrap_or(view.len());
        self.input[base..self.pos].to_owned()
    }

    /// Return the current character, and advance self.pos to the next character.
    fn consume_char(&mut self) -> char {
        let mut iter = self.peek().char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    /// Read the current character without consuming it.
    fn next_char(&self) -> char {
        self.peek().chars().next().unwrap()
    }

    /// Peek the upcoming input as a string slice.
    ///
    /// Note that the peeked slice becomes stale once more input is consumed.
    fn peek(&self) -> &str {
        &self.input[self.pos..]
    }

    /// Return true if all input is consumed.
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

fn valid_identifier_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true, // TODO: Include U+00A0 and higher.
        _ => false,
    }
}
