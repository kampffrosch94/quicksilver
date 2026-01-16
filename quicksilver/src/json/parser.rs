use std::fmt::Debug;
use std::str::{Chars, FromStr};

/// Helper struct for Deserializing
pub struct JsonWalker<'a> {
    pub chars: Chars<'a>,
    /// buffer used to simplify parsing of numbers and stuff
    pub buffer: String,
}

#[track_caller]
pub fn peek(chars: &Chars) -> char {
    chars
        .as_str()
        .chars()
        .next()
        .expect("String ended while parsing.")
}

/// Might be at the end of the string, in which case it returns None
pub fn peek_maybe(chars: &Chars) -> Option<char> {
    chars.as_str().chars().next()
}

impl JsonWalker<'_> {
    #[track_caller]
    pub fn consume_char(&mut self, arg: char) {
        let Self { chars, .. } = self;
        assert_eq!(Some(arg), chars.next());
    }

    #[track_caller]
    pub fn consume_maybe(&mut self, c: char) {
        let Self { chars, .. } = self;
        if peek(chars) == c {
            let _ = chars.next();
        }
    }

    pub fn consume_int<T>(&mut self) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        let Self { chars, buffer } = self;
        buffer.clear();

        let mut c = peek(chars);
        while match c {
            '0'..='9' | '+' | '-' => true,
            _ => false,
        } {
            buffer.push(c);
            let _ = chars.next();
            let Some(next) = peek_maybe(chars) else { break };
            c = next
        }
        buffer
            .parse::<T>()
            .unwrap_or_else(|_| panic!("Couldn't parse as int: '{buffer}'"))
    }

    pub fn consume_float<T>(&mut self) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        let Self { chars, buffer } = self;
        buffer.clear();
        let mut c = peek(chars);
        while match c {
            '0'..='9' | '+' | '-' | '.' => true,
            _ => false,
        } {
            buffer.push(c);
            chars.next();
            c = peek(chars);
        }
        buffer
            .parse::<T>()
            .unwrap_or_else(|_| panic!("Couldn't parse as float: '{buffer}'"))
    }

    #[track_caller]
    pub fn consume_field(&mut self, name: &str) {
        self.consume_char('"');

        let Self { chars, buffer } = self;
        buffer.clear();
        while peek(chars) != '"' {
            buffer.push(chars.next().unwrap());
        }
        assert_eq!(name, buffer);

        self.consume_char('"');
        self.consume_char(':');
    }

    pub fn consume_string(&mut self) -> String {
        self.consume_char('"');

        let Self { chars, buffer } = self;
        buffer.clear();
        let mut escaped = false;
        while peek(chars) != '"' || escaped {
            if peek(chars) == '\\' && !escaped {
                // ignore escape char backslash
                let _ = chars.next().unwrap();
                escaped = true;
                continue;
            }
            escaped = false;
            buffer.push(chars.next().unwrap());
        }

        self.consume_char('"'); // peek already consumed this
        self.buffer.clone()
    }

    pub fn consume_bool(&mut self) -> bool {
        let Self { chars, buffer } = self;
        buffer.clear();
        while peek(chars) != '}' && peek(chars) != ',' && peek(chars) != ']' {
            buffer.push(chars.next().unwrap());
        }
        buffer
            .parse::<bool>()
            .unwrap_or_else(|_| panic!("Couldn't parse as bool: '{buffer}'"))
    }
}
