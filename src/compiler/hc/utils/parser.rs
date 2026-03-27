use super::tokens::{Token, TokenKind};

/// Root AST node
#[derive(Debug)]
pub struct Program {
    pub routes: Vec<Route>,
}

/// AST node
#[derive(Debug)]
pub struct Route {
    pub docs: Option<String>,
    pub method: HttpMethod,
    pub path: String,
    pub body: Vec<Assignment>,
}

#[derive(Debug)]
pub enum HttpMethod {
    Get,
}

/// Assignment statement in a route body.
#[derive(Debug)]
pub struct Assignment {
    pub name: String,
    pub value: Value,
}

/// Value kinds accepted on the right side of assignment.
#[derive(Debug)]
pub enum Value {
    String(String),
    Execute(String),
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

/// Parses token stream into [`Program`].
///
/// Returns `Ok(Program)` when no parse errors are collected,
/// otherwise returns all accumulated errors so callers can display
/// multiple diagnostics in one pass.
pub fn parse_program(tokens: &[TokenKind]) -> Result<Program, Vec<ParseError>> {
    let mut parser = ParserState::new(tokens);
    let program = parser.parse_program();

    if parser.errors.is_empty() {
        Ok(program)
    } else {
        Err(parser.errors)
    }
}

struct ParserState<'a> {
    tokens: &'a [TokenKind],
    index: usize,
    errors: Vec<ParseError>,
}

/// # ParserState
/// The dragon wants to be defeated with a piece of magic skill.
/// The knight does not yet have the magic skill to defeat the dragon,
/// the knight becomes confused and calls someone who has the skill
/// to beat the dragon.
///
/// The knight brings the 'Harpy' and asks it to defeat the dragon.
///
/// The knight 'won'.
///
/// ===================================================================
/// Harpy Lang
///
/// ```
/// docs          = Docs
/// route         = "GET" Path route_block
/// route_block   = "main" Identifier "{" body "}" | "{" body "}"
/// body          = { assignment | return_stmt }
/// assignment    = Identifier "=" assign_value
/// assign_value  = String | Execute
/// return_stmt   = "return" return_value
/// return_value  = Identifier | String | Execute | Number
/// ```
///
/// Example:
///
/// ```
/// GET /X
/// main handler {
///   l = $get_any
///   return l
/// }
/// ```
///
/// Notes:
/// - `docs` is optional and comes from one `##...` line emitted by the lexer as
///   `Token::Docs`.
/// - Assignment values are intentionally strict (`String` or `Execute`) for now.
impl<'a> ParserState<'a> {
    /// Creates a parser cursor positioned at the first token.
    fn new(tokens: &'a [TokenKind]) -> Self {
        Self {
            tokens,
            index: 0,
            errors: Vec::new(),
        }
    }

    fn parse_program(&mut self) -> Program {
        let mut routes = Vec::new();

        // Keep parsing until input is exhausted; each iteration targets one route.
        while !self.at_end() {
            let docs = self.take_docs();

            if self.at_end() {
                break;
            }

            if !self.matches(Token::Get) {
                self.push_error_here("expected 'GET' or docs line");
                self.advance();
                continue;
            }

            let method = HttpMethod::Get;
            self.advance();

            let Some(path_tok) = self.consume(Token::Path, "expected route path after GET") else {
                self.synchronize_route();
                continue;
            };
            let path = path_tok.value.clone();

            if self.matches(Token::Fn) {
                self.advance();

                if self
                    .consume(Token::Identifier, "expected handler name after 'main'")
                    .is_none()
                {
                    self.synchronize_route();
                    continue;
                }

                if self
                    .consume(Token::LeftBrace, "expected '{' after handler name")
                    .is_none()
                {
                    self.synchronize_route();
                    continue;
                }
            } else if self
                .consume(
                    Token::LeftBrace,
                    "expected '{' or `main <handler> {` after route path",
                )
                .is_none()
            {
                self.synchronize_route();
                continue;
            }

            let body = self.parse_body();

            routes.push(Route {
                docs,
                method,
                path,
                body,
            });
        }

        Program { routes }
    }

    /// Parses statements inside route body until `}` or end of input.
    fn parse_body(&mut self) -> Vec<Assignment> {
        let mut body = Vec::new();

        while !self.at_end() && !self.matches(Token::RightBrace) {
            if self.is_return_keyword() {
                self.parse_return_statement();
                continue;
            }

            let Some(name_tok) = self.consume(Token::Identifier, "expected variable name") else {
                self.advance();
                continue;
            };

            if self
                .consume(Token::Equal, "expected '=' after variable name")
                .is_none()
            {
                self.advance();
                continue;
            }

            let Some(value) = self.parse_value() else {
                self.advance();
                continue;
            };

            body.push(Assignment {
                name: name_tok.value.clone(),
                value,
            });
        }

        if self
            .consume(Token::RightBrace, "expected '}' to close route body")
            .is_none()
        {
            self.synchronize_route();
        }

        body
    }

    fn parse_return_statement(&mut self) {
        // Consume the `return` identifier first, then validate a return value.
        self.advance();

        if self.at_end() || self.matches(Token::RightBrace) {
            self.push_error_here("expected return value after 'return'");
            return;
        }

        if let Some(tok) = self.current() {
            match tok.token {
                Token::Identifier
                | Token::String
                | Token::Execute
                | Token::Number
                | Token::Path => {
                    self.advance();
                }
                _ => {
                    self.push_error_here("expected value after 'return'");
                    self.advance();
                }
            }
        }
    }

    fn parse_value(&mut self) -> Option<Value> {
        if let Some(tok) = self.current() {
            return match tok.token {
                Token::String => {
                    let value = Value::String(tok.value.clone());
                    self.advance();
                    Some(value)
                }
                Token::Execute => {
                    let value = Value::Execute(tok.value.clone());
                    self.advance();
                    Some(value)
                }
                _ => {
                    self.push_error_here("expected string or execute value");
                    None
                }
            };
        }

        self.push_error_eof("expected value but found end of input");
        None
    }

    fn take_docs(&mut self) -> Option<String> {
        let mut docs = Vec::new();
        while let Some(tok) = self.current() {
            if tok.token == Token::Docs {
                docs.push(tok.value.clone());
                self.advance();
            } else {
                break;
            }
        }
        if docs.is_empty() {
            None
        } else {
            Some(docs.join("\n"))
        }
    }

    fn is_return_keyword(&self) -> bool {
        matches!(self.current(), Some(tok) if tok.token == Token::Identifier && tok.value == "return")
    }

    fn synchronize_route(&mut self) {
        // Skip tokens until a safe boundary (`}` or next `GET`) is found.
        while !self.at_end() {
            if self.matches(Token::RightBrace) {
                self.advance();
                return;
            }

            if self.matches(Token::Get) {
                return;
            }

            self.advance();
        }
    }

    fn consume(&mut self, expected: Token, message: &str) -> Option<&'a TokenKind> {
        if self.matches(expected) {
            let tok = self.current();
            self.advance();
            tok
        } else {
            self.push_error_here(message);
            None
        }
    }

    fn matches(&self, expected: Token) -> bool {
        matches!(self.current(), Some(tok) if tok.token == expected)
    }

    fn current(&self) -> Option<&'a TokenKind> {
        self.tokens.get(self.index)
    }

    fn at_end(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn advance(&mut self) {
        if !self.at_end() {
            self.index += 1;
        }
    }

    fn push_error_here(&mut self, message: &str) {
        if let Some(tok) = self.current() {
            self.errors.push(ParseError {
                message: message.to_string(),
                line: tok.line,
                column: tok.column,
            });
        } else {
            self.push_error_eof(message);
        }
    }

    fn push_error_eof(&mut self, message: &str) {
        let (line, column) = self
            .tokens
            .last()
            .map(|tok| (tok.line, tok.column))
            .unwrap_or((1, 1));

        self.errors.push(ParseError {
            message: message.to_string(),
            line,
            column,
        });
    }
}
