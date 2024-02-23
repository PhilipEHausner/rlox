pub struct CharStream<'a> {
    text: &'a str,
    position: usize,
}

impl<'a> CharStream<'a> {
    pub fn new(text: &'a str) -> CharStream {
        CharStream { text, position: 0 }
    }

    pub fn reset(&mut self) {
        self.position = 0;
    }

    // Consume the next char. Return None if stream has ended.
    pub fn next(&mut self) -> Option<char> {
        let result = self.current_char();
        self.position += 1;
        result
    }

    // Revert the last consumed char, s.t. it can be consumed again.
    pub fn revert(&mut self) {
        self.position -= 1;
    }

    pub fn peek(&self) -> Option<char> {
        self.peek_n(1)
    }

    pub fn peek_n(&self, n: usize) -> Option<char> {
        self.text[self.position + n..].chars().next()
    }

    // Check if the next character in the stream matches an expected char. If so, consume the
    // character. Otherwise, leave CharStream as is.
    // Returns true if expected matches the next char in the stream, false otherwise.
    pub fn matches(&mut self, expected: char) -> bool {
        if let Some(c) = self.current_char() {
            if c != expected {
                return false;
            }
            self.position += 1;
            true
        } else {
            false
        }
    }

    pub fn is_exhausted(&self) -> bool {
        self.position >= self.text.len()
    }

    pub fn current_char(&self) -> Option<char> {
        if self.position > self.text.len() {
            return None;
        }
        self.text[self.position..].chars().next()
    }

    pub fn get_position(&self) -> usize {
        self.position
    }
}
