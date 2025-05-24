use std::str::Chars;

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

impl JsonWalker<'_> {
    #[track_caller]
    pub fn consume_char(&mut self, arg: char) {
        let Self { chars, .. } = self;
        assert_eq!(Some(arg), chars.next());
    }

    #[track_caller]
    pub fn consume_either(&mut self, a: char, b: char) {
        let Self { chars, .. } = self;
        match chars.next() {
            Some(var) if var == a => {}
            Some(var) if var == b => {}
            other @ _ => {
                assert!(false, "Expected {a} or {b} got {other:?}")
            }
        }
    }

    #[track_caller]
    pub fn consume_maybe(&mut self, c: char) {
        let Self { chars, .. } = self;
        if peek(chars) == c {
            let _ = chars.next();
        }
    }

    pub fn consume_i32(&mut self) -> i32 {
        let Self { chars, buffer } = self;
        buffer.clear();

        let mut c = peek(chars);
        while match c {
            '0'..='9' | '+' | '-' => true,
            _ => false,
        } {
            buffer.push(c);
            chars.next();
            c = peek(chars);
        }
        buffer
            .parse::<i32>()
            .expect("Couldn't parse as i32: '{buffer}'")
    }

    pub fn consume_u32(&mut self) -> u32 {
        let Self { chars, buffer } = self;
        buffer.clear();
        let mut c = peek(chars);
        while match c {
            '0'..='9' | '+' => true,
            _ => false,
        } {
            buffer.push(c);
            chars.next();
            c = peek(chars);
        }
        buffer
            .parse::<u32>()
            .expect("Couldn't parse as i32: '{buffer}'")
    }

    pub fn consume_f32(&mut self) -> f32 {
        let Self { chars, buffer } = self;
        buffer.clear();
        let mut c = peek(chars);
        while match c {
            '0'..'9' | '+' | '-' | '.' => true,
            _ => false,
        } {
            buffer.push(c);
            chars.next();
            c = peek(chars);
        }
        buffer
            .parse::<f32>()
            .expect("Couldn't parse as i32: '{buffer}'")
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
}
