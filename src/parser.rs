use std::collections::HashMap;
use substring::Substring;

use crate::{
    css::{Color, Declaration, Rule, Selector, SimpleSelector, Unit, Value},
    dom,
};

pub struct Parser {
    pos: usize,
    input: String,
}

// HTML PARSING
impl Parser {
    // Read character without consuming it.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    // Do the next characters start with the given string?
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    // If the exact string `s` is found at the current position, consume it.
    // Otherwise, panic.
    fn expect(&mut self, s: &str) {
        if self.starts_with(s) {
            self.pos += s.len();
        } else {
            panic!("Expected {:?} at byte {} but it was not found", s, self.pos);
        }
    }

    // Return true if all input is consumed.
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_char(&mut self) -> char {
        let c = self.next_char();
        self.pos += c.len_utf8();
        c
    }

    // Consume character unitl `test` returns false.
    fn consume_while(&mut self, test: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }

        result
    }

    // Consume and discard zero or more whitespace characters.
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    // Parse a tag or attribute name.
    fn parse_name(&mut self) -> String {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
    }

    fn parse_node(&mut self) -> dom::Node {
        if self.starts_with("<!--") {
            self.parse_comment()
        } else if self.starts_with("<") {
            self.parse_element()
        } else {
            self.parse_text()
        }
    }

    fn parse_comment(&mut self) -> dom::Node {
        self.expect("<!--");

        loop {
            self.consume_while(|c| c != '-');
            if self.try_parse_close_comment() {
                break;
            } else {
                self.consume_char();
            }
        }

        dom::comment()
    }

    fn try_parse_close_comment(&mut self) -> bool {
        let end_string = "-->";
        let slice = self.input.substring(self.pos, self.pos + 3);

        slice == end_string
    }

    fn parse_text(&mut self) -> dom::Node {
        dom::text(self.consume_while(|c| c != '<'))
    }

    fn parse_element(&mut self) -> dom::Node {
        // Opening tag.
        self.expect("<");
        let tag_name = self.parse_name();
        let attrs = self.parse_attributes();
        self.expect(">");

        // Contents.
        let children = self.parse_nodes();

        // Closing tag.
        self.expect("</");
        self.expect(&tag_name);
        self.expect(">");

        dom::elem(tag_name, attrs, children)
    }

    // Parse a single name="value" pair.
    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_name();
        self.expect("=");
        let value = self.parse_attr_value();
        return (name, value);
    }

    // Parse a quoted value.
    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        let close_quote = self.consume_char();
        assert_eq!(open_quote, close_quote);
        return value;
    }

    // Parse a list of name="value" pairs, separated by whitespace.
    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }
        return attributes;
    }

    // Parse a sequence of sibling nodes.
    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }

        nodes
    }

    // Parse an HTML document and return the root element.
    pub fn parse(source: String) -> dom::Node {
        let mut nodes = Parser {
            pos: 0,
            input: source,
        }
        .parse_nodes();

        // If the document contains a root element, just return it. Otherwise, create one.
        if nodes.len() == 1 {
            return nodes.remove(0);
        } else {
            return dom::elem("html".to_string(), HashMap::new(), nodes);
        }
    }
}

// CSS PARSING
impl Parser {
    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
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
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }

        selector
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break,
                c => panic!("Unexpected character {} in selector list", c), //start of declarations
            }
        }
        // Return selectors with highest specificity first, for use in matching.
        selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));
        selectors
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        self.expect("{");
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    fn parse_declaration(&mut self) -> Declaration {
        let name = self.parse_identifier();
        self.consume_whitespace();
        self.expect(":");
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        self.expect(";");

        Declaration { name, value }
    }

    fn parse_value(&mut self) -> Value {
        match self.next_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier()),
        }
    }

    fn parse_color(&mut self) -> Value {
        self.expect("#");
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

    fn parse_length(&mut self) -> Value {
        Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f32 {
        self.consume_while(|c| matches!(c, '0'..='9' | '.'))
            .parse()
            .unwrap()
    }

    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Unit::Px,
            _ => panic!("unrecognized unit"),
        }
    }
}

fn valid_identifier_char(c: char) -> bool {
    // TODO: Include U+00A0 and higher.
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}
