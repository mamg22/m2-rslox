#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,
    Bang, BangEqual, Equal, EqualEqual,
    Greater, GreaterEqual, Less, LessEqual,
    Identifier, String, Number,
    And, Class, Else, False, For, Fun, If, Nil, Or, Print,
    Return, Super, This, True, Var, While,
}

type TT = TokenType;

pub struct Token<'s> {
    pub token_type: TokenType,
    pub span: &'s str,
    pub line: usize,
}

impl<'s> Token<'s> {
    pub fn new(token_type: TokenType, span: &'s str, line: usize) -> Self {
        Self {
            token_type,
            span,
            line,
        }
    }
}

pub struct Scanner<'s> {
    source: &'s str,
    pub line: usize,
}

type ScanResult<'s> = Result<Option<Token<'s>>, &'static str>;

impl<'s> Scanner<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> ScanResult<'s> {
        self.skip_whitespace();

        if self.source.is_empty() {
            return Ok(None);
        }

        let ch = self.source.chars().next().unwrap();
        let next_ch = self.source.chars().nth(1).unwrap_or('\0');

        let tok = match ch {
            '"' => return self.string(),
            '0'..='9' => return self.number(),
            'A'..='Z' | 'a'..='z' | '_' => return self.identifier(),

            '!' if next_ch == '=' => self.make_token(TT::BangEqual, 2),
            '=' if next_ch == '=' => self.make_token(TT::EqualEqual, 2),
            '<' if next_ch == '=' => self.make_token(TT::LessEqual, 2),
            '>' if next_ch == '=' => self.make_token(TT::GreaterEqual, 2),

            '(' => self.make_token(TT::LeftParen, 1),
            ')' => self.make_token(TT::RightParen, 1),
            '{' => self.make_token(TT::LeftBrace, 1),
            '}' => self.make_token(TT::RightBrace, 1),
            ';' => self.make_token(TT::Semicolon, 1),
            ',' => self.make_token(TT::Comma, 1),
            '.' => self.make_token(TT::Dot, 1),
            '-' => self.make_token(TT::Minus, 1),
            '+' => self.make_token(TT::Plus, 1),
            '/' => self.make_token(TT::Slash, 1),
            '*' => self.make_token(TT::Star, 1),
            '!' => self.make_token(TT::Bang, 1),
            '=' => self.make_token(TT::Equal, 1),
            '<' => self.make_token(TT::Less, 1),
            '>' => self.make_token(TT::Greater, 1),

            _ => {
                self.advance(1);
                return Err("Unexpected character");
            }
        };

        Ok(Some(tok))
    }

    fn make_token(&mut self, token_type: TokenType, length: usize) -> Token<'s> {
        let (span, source) = self.source.split_at(length);
        self.source = source;
        Token::new(token_type, span, self.line)
    }

    fn skip_whitespace(&mut self) {
        loop {
            self.source = self.source
                .trim_start_matches(|c: char| c.is_whitespace() && c != '\n');

            match self.source.chars().next() {
                Some('/') => {
                    if self.source.chars().nth(1).is_some_and(|c| c == '/') {
                        self.source = self.source
                            .trim_start_matches(|c: char| c != '\n')
                    }
                    else {
                        break;
                    }
                },
                Some('\n') => {
                    self.line += 1;
                    self.advance(1);
                },
                _ => { break; },
            }
        }
    }

    fn string(&mut self) -> ScanResult<'s> {
        for (pos, ch) in self.source.char_indices().skip(1) {
            if ch == '\n' {
                self.line += 1;
            }
            if ch == '"' {
                let tok = self.make_token(TT::String, pos + ch.len_utf8());

                return Ok(Some(tok));
            }
        }

        self.advance(1);
        Err("Unterminated string")
    }
    
    fn number(&mut self) -> ScanResult<'s> {
        let mut source_iter = self.source.char_indices();

        let end = source_iter.by_ref()
            .skip_while(|c| c.1.is_ascii_digit())
            .next();

        let next_num = source_iter.next()
            .is_some_and(|c| c.1.is_ascii_digit());

        let length = match end {
            Some((_, '.')) if next_num => {
                let end = source_iter
                    .skip_while(|c| c.1.is_ascii_digit()).next();

                if let Some((pos, _)) = end {
                    pos
                }
                else {
                    self.source.len()
                }
            },
            Some((pos, _)) => pos,
            None => self.source.len(),
        };

        Ok(Some(self.make_token(TT::Number, length)))
    }

    fn identifier(&mut self) -> ScanResult<'s> {
        let pos = self.source.char_indices()
            .skip(1)
            .skip_while(|(_, c)| c.is_ascii_alphanumeric() || *c == '_')
            .next()
            .unwrap()
            .0;

        let span = &self.source[..pos];
        let token_type = self.identifier_type(span);

        Ok(Some(self.make_token(token_type, pos)))
    }

    fn identifier_type(&mut self, span: &str) -> TokenType {
        let mut span_chars = span.chars();
        let chars = span_chars.by_ref();

        match (chars.next().unwrap_or_default(), chars.as_str()) {
            ('a', "nd") => TT::And,
            ('c', "lass") => TT::Class,
            ('e', "lse") => TT::Else,
            ('f', _) => match (chars.next().unwrap_or_default(), chars.as_str()) {
                ('a', "lse") => TT::False,
                ('o', "r") => TT::For,
                ('u', "n") => TT::Fun,
                _ => TT::Identifier,
            }
            ('i', "f") => TT::If,
            ('n', "il") => TT::Nil,
            ('o', "r") => TT::Or,
            ('p', "rint") => TT::Print,
            ('r', "eturn") => TT::Return,
            ('s', "uper") => TT::Super,
            ('t', _) => match (chars.next().unwrap_or_default(), chars.as_str()) {
                ('h', "is") => TT::This,
                ('r', "ue") => TT::True,
                _ => TT::Identifier,
            }
            ('v', "ar") => TT::Var,
            ('w', "hile") => TT::While,
            _ => TT::Identifier
        }
    }

    fn advance(&mut self, offset: usize) {
        self.source = &self.source[offset..];
    }

}
