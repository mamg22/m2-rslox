use crate::chunk::{Chunk, OpCode};
use crate::debug;
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;

struct Parser<'s> {
    scanner: Scanner<'s>,
    previous: Option<Token<'s>>,
    current: Option<Token<'s>>,
    had_error: bool,
    panic_mode: bool,
}

enum ErrorSource {
    Current,
    Previous,
}

impl<'s> Parser<'s> {
    fn new(source: &'s str) -> Self {
        let scanner = Scanner::new(source);
        Self {
            scanner,
            previous: None,
            current: None,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn advance(&mut self) {
        self.previous = self.current.take();

        loop {
            match self.scanner.scan_token() {
                Ok(tok) => {
                    self.current = tok;
                    break;
                },
                Err(msg) => {
                    self.error_at_current(msg);
                }
            }

        }
    }

    pub fn consume(&mut self, token_type: TokenType, message: &'static str) {
        if self.current.as_ref().is_some_and(|t| t.token_type == token_type) {
            self.advance();
        }
        else {
            self.error_at_current(message);
        }
    }

    pub fn error(&mut self, message: &'static str) {
        self.error_at(ErrorSource::Previous, message);
    }

    pub fn error_at_current(&mut self, message: &'static str) {
        self.error_at(ErrorSource::Current, message);
    }

    fn error_at(&mut self, source: ErrorSource, message: &'static str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        let token = match source {
            ErrorSource::Current => self.current.as_ref(),
            ErrorSource::Previous => self.previous.as_ref(),
        };

        match token {
            Some(token) => {
                eprintln!("[line {}] Error at '{}': {}", token.line, token.span, message)
            },
            None => {
                eprintln!("[line {}] Error at end: {}", self.scanner.line, message)
            }
        }

        self.had_error = true;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    fn below(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Assignment => Self::Or,
            Self::Or => Self::And,
            Self::And => Self::Equality,
            Self::Equality => Self::Comparison,
            Self::Comparison => Self::Term,
            Self::Term => Self::Factor,
            Self::Factor => Self::Unary,
            Self::Unary => Self::Call,
            Self::Call => Self::Primary,
            Self::Primary => Self::Primary,
        }
    }
}

type ParseFn<'s> = fn(&mut Compiler<'s>) -> ();

struct ParseRule<'s> {
    prefix: Option<ParseFn<'s>>,
    infix: Option<ParseFn<'s>>,
    precedence: Precedence,
}

macro_rules! parse_rule {
    ( None, None, $prec:ident ) => {
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::$prec,
        }
    };
    ( $prefix:ident, None, $prec:ident ) => {
        ParseRule {
            prefix: Some(Compiler::$prefix),
            infix: None,
            precedence: Precedence::$prec,
        }
    };
    ( None, $infix:ident, $prec:ident ) => {
        ParseRule {
            prefix: None,
            infix: Some(Compiler::$infix),
            precedence: Precedence::$prec,
        }
    };
    ( $prefix:ident, $infix:ident, $prec:ident ) => {
        ParseRule {
            prefix: Some(Compiler::$prefix),
            infix: Some(Compiler::$infix),
            precedence: Precedence::$prec,
        }
    };
}

impl<'s> Into<ParseRule<'s>> for TokenType {
    fn into(self) -> ParseRule<'s> {
        match self {
            Self::LeftParen => parse_rule!(grouping, None, None),
            Self::Minus => parse_rule!(unary, binary, Term),
            Self::Plus => parse_rule!(None, binary, Term),
            Self::Slash => parse_rule!(None, binary, Factor),
            Self::Star => parse_rule!(None, binary, Factor),
            Self::Number => parse_rule!(number, None, None),
            Self::False | Self::True | Self::Nil => parse_rule!(literal, None, None),
            Self::Bang => parse_rule!(unary, None, None),
            Self::BangEqual | Self::EqualEqual => parse_rule!(None, binary, Equality),
            Self::Greater | Self::GreaterEqual |
            Self::Less | Self::LessEqual => parse_rule!(None, binary, Comparison),
            _ => parse_rule!(None, None, None),
        }
    }
}

pub struct Compiler<'s> {
    parser: Parser<'s>,
    compiling_chunk: Option<Chunk>,
    // Note for later chapters:
    // Hold a single scanner and a stack of (Class)Compiler contexts
}

impl<'s> Compiler<'s> {
    pub fn new(source: &'s str) -> Self {
        let mut parser = Parser::new(source);
        parser.advance();
        Self { parser, compiling_chunk: None }
    }

    pub fn compile(&mut self) -> Result<Chunk, ()> {
        self.compiling_chunk = Some(Chunk::new());

        self.expression();

        if self.parser.current.is_some() {
            self.parser.error_at_current("Expected end of expression");
        }

        self.end_compiler();

        if self.parser.had_error {
            Err(())
        }
        else {
            self.compiling_chunk.take().ok_or(())
        }
    }

    fn emit(&mut self, op: OpCode) {
        let tok = self.parser.previous.as_ref();
        let line = match tok {
            Some(tok) => tok.line,
            None => 0,
        };
        self.current_chunk().write(op, line);
    }

    fn emit_return(&mut self) {
        self.emit(OpCode::Return);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit(OpCode::Constant(constant));
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        self.current_chunk().add_constant(value)
    }

    fn end_compiler(&mut self) {
        if cfg!(feature = "debug_print_code") {
            if !self.parser.had_error {
                debug::disassemble_chunk(self.current_chunk(), "code");
            }
        }
        self.emit_return();
    }

    fn binary(&mut self) {
        let operator_type = self.parser.previous
            .as_ref().unwrap().token_type;

        let rule: ParseRule = operator_type.clone().into();

        self.parse_precedence(Precedence::below(&rule.precedence));

        match operator_type {
            TokenType::Plus => self.emit(OpCode::Add),
            TokenType::Minus => self.emit(OpCode::Substract),
            TokenType::Star => self.emit(OpCode::Multiply),
            TokenType::Slash => self.emit(OpCode::Divide),
            TokenType::BangEqual => {
                self.emit(OpCode::Equal);
                self.emit(OpCode::Not);
            },
            TokenType::EqualEqual => self.emit(OpCode::Equal),
            TokenType::Greater => self.emit(OpCode::Greater),
            TokenType::GreaterEqual => {
                self.emit(OpCode::Less);
                self.emit(OpCode::Not);
            },
            TokenType::Less => self.emit(OpCode::Less),
            TokenType::LessEqual => {
                self.emit(OpCode::Greater);
                self.emit(OpCode::Not);
            },
            _ => unreachable!(),
        }
    }

    fn literal(&mut self) {
        match self.parser.previous.as_ref().unwrap().token_type {
            TokenType::False => self.emit(OpCode::False),
            TokenType::True => self.emit(OpCode::True),
            TokenType::Nil => self.emit(OpCode::Nil),
            _ => unreachable!(),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.parser.consume(TokenType::RightParen, "Expected ')' after expression");
    }

    fn number(&mut self) {
        let value: f64 = self.parser.previous.as_ref().unwrap().span.parse().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous
            .as_ref().unwrap().token_type.to_owned();

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Bang => self.emit(OpCode::Not),
            TokenType::Minus => self.emit(OpCode::Negate),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.parser.advance();
        let tt = match self.parser.previous.as_ref() {
            Some(tok) => tok.token_type,
            None => return,
        };

        let rule: ParseRule = tt.into();

        match rule.prefix {
            Some(ref func) => {
                func(self);


                loop {
                    let current_prec: Precedence = match self.parser.current.as_ref() {
                        Some(tok) => {
                            let rule: ParseRule = tok.token_type.into();
                            rule.precedence
                        },
                        None => Precedence::None,
                    };

                    if precedence > current_prec {
                        break;
                    }
                    self.parser.advance();
                    let tt = match self.parser.previous.as_ref() {
                        Some(tok) => tok.token_type,
                        None => break,
                    };

                    let rule: ParseRule = tt.into();
                    
                    if let Some(ref func) = rule.infix {
                        func(self);
                    }
                }
            },
            None => self.parser.error("Expected expression"),
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        self.compiling_chunk.as_mut().unwrap()
    }
}