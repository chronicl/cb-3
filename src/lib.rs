mod lexer;

// Type definition for the Result that is being used by the parser. You may change it to anything
// you want
pub type ParseResult = Result<(), String>;

pub use lexer::C1Lexer;
pub use lexer::C1Token;

// You will need a re-export of your C1Parser definition. Here is an example:
// mod parser;
// pub use parser::C1Parser;

pub struct C1Parser<'a> {
    lexer: C1Lexer<'a>,
}

use C1Token::*;

// Macro for returning descriptive errors. We use a macro instead of a function
// to allow patterns as arguments.
macro_rules! next_token {
    ($this:ident, $token:pat, $error:expr) => {
        match $this.next_token() {
            Some($token) => Ok::<(), String>(()),
            Some(tok) => Err(format!(
                "Expected {:?}, got {:?}, line: {:?}",
                $error,
                tok,
                $this.lexer.current_line_number()
            )),
            None => Err(format!(
                "Expected {:?}, got EOF, line: {:?}",
                $error,
                $this.lexer.current_line_number()
            )),
        }
    };
}

impl<'a> C1Parser<'a> {
    fn new(lexer: C1Lexer) -> C1Parser {
        C1Parser { lexer }
    }

    pub fn parse(text: &str) -> ParseResult {
        let mut this = C1Parser::new(C1Lexer::new(text));
        this.program()
    }

    fn next_token(&mut self) -> Option<C1Token> {
        let token = self.lexer.current_token();
        self.eat();
        token
    }

    fn current_token(&mut self) -> Option<C1Token> {
        self.lexer.current_token()
    }

    fn peek_token(&mut self) -> Option<C1Token> {
        self.lexer.peek_token()
    }

    fn eat(&mut self) {
        self.lexer.eat();
    }

    fn err(&self, msg: &str) -> ParseResult {
        Err(format!(
            "Expect {}, line {:?}",
            msg,
            self.lexer.current_line_number()
        ))
    }

    fn program(&mut self) -> ParseResult {
        if self.current_token().is_none() {
            return Ok(());
        }
        self.functiondefinition()?;
        self.program()
    }

    fn functiondefinition(&mut self) -> ParseResult {
        self.r#type()?;
        next_token!(self, Identifier, "function name")?;
        next_token!(self, LeftParenthesis, "(")?;
        next_token!(self, RightParenthesis, ")")?;
        next_token!(self, LeftBrace, "{")?;
        self.statementlist()?;
        next_token!(self, RightBrace, "}")?;
        Ok(())
    }

    fn r#type(&mut self) -> ParseResult {
        next_token!(self, KwInt | KwFloat | KwVoid | KwBoolean, "type")?;
        Ok(())
    }

    fn functioncall(&mut self) -> ParseResult {
        next_token!(self, Identifier, "function name")?;
        next_token!(self, LeftParenthesis, "(")?;
        next_token!(self, RightParenthesis, ")")?;
        Ok(())
    }

    fn statementlist(&mut self) -> ParseResult {
        loop {
            match self.current_token() {
                Some(RightBrace) => break,
                None => self.err("expected closing bracket")?,
                _ => {}
            }
            self.block()?;
        }

        Ok(())
    }

    fn block(&mut self) -> ParseResult {
        match self.current_token() {
            Some(LeftBrace) => {
                self.eat();
                self.statementlist()?;
                next_token!(self, RightBrace, "}")?;
            }
            _ => self.statement()?,
        }

        Ok(())
    }

    fn statement(&mut self) -> ParseResult {
        match self.current_token().err(self, "expected statement")? {
            KwIf => self.ifstatement()?,
            KwReturn => {
                self.returnstatement()?;
                next_token!(self, Semicolon, ";")?;
            }
            KwPrintf => {
                self.printf()?;
                next_token!(self, Semicolon, ";")?;
            }
            Identifier => match self.peek_token() {
                Some(Assign) => {
                    self.statassignment()?;
                    next_token!(self, Semicolon, ";")?;
                }
                Some(LeftParenthesis) => {
                    self.functioncall()?;
                    next_token!(self, Semicolon, ";")?;
                }
                _ => self.err("expected assignment or function call")?,
            },
            _ => {
                self.err(&"if, return, printf, assignment or functioncall")?;
            }
        }

        Ok(())
    }

    fn printf(&mut self) -> ParseResult {
        next_token!(self, KwPrintf, "printf")?;
        next_token!(self, LeftParenthesis, "(")?;
        self.assignment()?;
        next_token!(self, RightParenthesis, ")")?;
        Ok(())
    }

    fn returnstatement(&mut self) -> ParseResult {
        next_token!(self, KwReturn, "return")?;
        if self.current_token() == Some(Semicolon) {
            return Ok(());
        }
        self.assignment()?;
        Ok(())
    }

    fn ifstatement(&mut self) -> ParseResult {
        next_token!(self, KwIf, "if")?;
        next_token!(self, LeftParenthesis, "(")?;
        self.assignment()?;
        next_token!(self, RightParenthesis, ")")?;
        self.block()?;

        Ok(())
    }

    fn statassignment(&mut self) -> ParseResult {
        next_token!(self, Identifier, "variable name")?;
        next_token!(self, Assign, "=")?;
        self.assignment()?;
        Ok(())
    }

    fn assignment(&mut self) -> ParseResult {
        match (self.current_token(), self.peek_token()) {
            (Some(Identifier), Some(Assign)) => {
                self.eat();
                self.eat();
                self.assignment()?;
            }
            _ => self.expr()?,
        }

        Ok(())
    }

    fn expr(&mut self) -> ParseResult {
        self.simpexpr()?;
        match self.current_token() {
            Some(Equal) | Some(NotEqual) | Some(Less) | Some(Greater) | Some(LessEqual)
            | Some(GreaterEqual) => {
                self.eat();
                self.simpexpr()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn simpexpr(&mut self) -> ParseResult {
        if self.current_token() == Some(Minus) {
            self.eat();
        }
        self.term()?;

        loop {
            match self.current_token() {
                Some(Plus) | Some(Minus) | Some(Or) => {
                    self.eat();
                    self.term()?;
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn term(&mut self) -> ParseResult {
        self.factor()?;

        loop {
            match self.current_token() {
                Some(Asterisk) | Some(Slash) | Some(And) => {
                    self.eat();
                    self.factor()?;
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn factor(&mut self) -> ParseResult {
        match self.current_token().err(self, "factor")? {
            Identifier if self.peek_token() == Some(LeftParenthesis) => {
                self.functioncall()?;
            }
            ConstInt | ConstFloat | ConstBoolean | Identifier => {
                self.eat();
            }
            LeftParenthesis => {
                self.eat();
                self.assignment()?;
                next_token!(self, RightParenthesis, ")")?;
            }
            _ => {
                self.err("factor")?;
            }
        }

        Ok(())
    }
}

// Some more error tooling
trait OptionC1TokenExt {
    fn err(self, parser: &mut C1Parser, msg: &str) -> Result<C1Token, String>;
}

impl OptionC1TokenExt for Option<C1Token> {
    fn err(self, parser: &mut C1Parser, msg: &str) -> Result<C1Token, String> {
        match self {
            Some(t) => Ok(t),
            None => Err(parser.err(msg).unwrap_err()),
        }
    }
}
