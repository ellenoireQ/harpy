use super::tokens::{Token, TokenKind};
use crate::network::method::HttpMethod;

/// Root AST node
#[derive(Debug)]
pub struct Program {
    pub blocks: Vec<Block>,
}

/// AST node
#[derive(Debug)]
pub struct Block {
    pub docs: Option<String>,
    pub method: Option<HttpMethod>,
    pub path: Option<String>,
    pub handler_name: Option<String>,
    pub body: Vec<Assignment>,
}

/// Assignment statement in a block body.
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
    Print(Box<Value>),
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
/// method        = "GET" | "POST" | "PUT" | "DELETE" | "PATCH"
/// block         = method Path block_body
/// block_body    = "main" Identifier "{" body "}" | "{" body "}"
/// body          = { assignment | return_stmt | print_stmt }
/// assignment    = Identifier "=" assign_value
/// assign_value  = String | Execute
/// return_stmt   = "return" return_value
/// return_value  = Identifier | String | Execute | Number
/// print_stmt    = "print" "(" print_value ")"
/// print_value   = Identifier | String | Execute | Number
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
        let mut blocks = Vec::new();

        // Keep parsing until input is exhausted; each iteration targets one block.
        while !self.at_end() {
            let docs = self.take_docs();

            if self.at_end() {
                break;
            }
            let method = self.parse_http_method();

            let path = if self.matches(Token::Path) {
                let path = self
                    .current()
                    .map(|tok| tok.value.clone())
                    .unwrap_or_default();
                self.advance();
                Some(path)
            } else {
                None
            };

            let mut handler_name: Option<String> = None;

            if self.matches(Token::Fn) {
                self.advance();

                let Some(name_tok) =
                    self.consume(Token::Identifier, "expected handler name after 'main'")
                else {
                    self.synchronize_block();
                    continue;
                };
                handler_name = Some(name_tok.value.clone());

                if self
                    .consume(Token::LeftBrace, "expected '{' after handler name")
                    .is_none()
                {
                    self.synchronize_block();
                    continue;
                }
            } else if self
                .consume(
                    Token::LeftBrace,
                    "expected '{' or `main <handler> {` after path",
                )
                .is_none()
            {
                self.synchronize_block();
                continue;
            }

            let body = self.parse_body();

            blocks.push(Block {
                docs,
                method,
                path,
                handler_name,
                body,
            });
        }

        Program { blocks }
    }

    fn parse_http_method(&mut self) -> Option<HttpMethod> {
        if self.matches(Token::Get) {
            self.advance();
            return Some(HttpMethod::Get);
        }

        if self.matches(Token::Post) {
            self.advance();
            return Some(HttpMethod::Post);
        }

        if self.matches(Token::Put) {
            self.advance();
            return Some(HttpMethod::Put);
        }

        if self.matches(Token::Delete) {
            self.advance();
            return Some(HttpMethod::Delete);
        }

        if self.matches(Token::Patch) {
            self.advance();
            return Some(HttpMethod::Patch);
        }

        None
    }

    /// Parses statements inside block body until `}` or end of input.
    fn parse_body(&mut self) -> Vec<Assignment> {
        let mut body = Vec::new();

        while !self.at_end() && !self.matches(Token::RightBrace) {
            if self.is_return_keyword() {
                body.extend(self.parse_return_statement());
                continue;
            }
            if self.is_print_keyword() {
                if let Some(print_stmt) = self.parse_print() {
                    body.push(print_stmt);
                }
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
            .consume(Token::RightBrace, "expected '}' to close block body")
            .is_none()
        {
            self.synchronize_block();
        }

        body
    }

    fn parse_return_statement(&mut self) -> Vec<Assignment> {
        let mut body = Vec::new();

        // Consume the `return` identifier first, then validate a return value.
        self.advance();

        if self.at_end() || self.matches(Token::RightBrace) {
            self.push_error_here("expected return value after 'return'");
            return body;
        }

        if let Some(tok) = self.current() {
            let token_value = tok.value.clone();

            match tok.token {
                Token::Execute => {
                    self.advance();
                    body.push(Assignment {
                        name: "return".to_string(),
                        value: Value::Execute(token_value),
                    })
                }
                Token::Identifier | Token::String => {
                    self.advance();
                    body.push(Assignment {
                        name: "return".to_string(),
                        value: Value::String(token_value),
                    })
                }
                _ => {
                    self.push_error_here("expected value after 'return'");
                    self.advance();
                }
            }
        }
        body
    }

    fn parse_print(&mut self) -> Option<Assignment> {
        // Current token is `print` when this function is called.
        self.advance();

        if self
            .consume(Token::LeftParent, "expected '(' after 'print'")
            .is_none()
        {
            return None;
        }

        let print_value = self.parse_print_value();

        let _ = self.consume(Token::RightParent, "expected ')' after print value");

        print_value.map(|value| Assignment {
            name: "print".to_string(),
            value,
        })
    }

    fn parse_print_value(&mut self) -> Option<Value> {
        if self.at_end() || self.matches(Token::RightParent) {
            self.push_error_here("expected value inside print(...)");
            return None;
        }

        if let Some(tok) = self.current() {
            return match tok.token {
                Token::Identifier | Token::String | Token::Number => {
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
                    self.push_error_here("expected printable value inside print(...)");
                    self.advance();
                    None
                }
            };
        }

        None
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
                Token::Print
                    if self
                        .tokens
                        .get(self.index + 1)
                        .map(|next| next.token == Token::LeftParent)
                        .unwrap_or(false) =>
                {
                    self.advance();
                    self.advance();
                    let print_value = self.parse_print_value().map(Box::new).map(Value::Print);
                    let _ = self.consume(Token::RightParent, "expected ')' after print value");
                    print_value
                }
                Token::Identifier
                    if tok.value == "print"
                        && self
                            .tokens
                            .get(self.index + 1)
                            .map(|next| next.token == Token::LeftParent)
                            .unwrap_or(false) =>
                {
                    self.advance();
                    self.advance();
                    let print_value = self.parse_print_value().map(Box::new).map(Value::Print);
                    let _ = self.consume(Token::RightParent, "expected ')' after print value");
                    print_value
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
        matches!(
            self.current(),
            Some(tok)
                if tok.token == Token::Return
                    || (tok.token == Token::Identifier && tok.value == "return")
        )
    }

    fn is_print_keyword(&self) -> bool {
        matches!(
            self.current(),
            Some(tok)
                if tok.token == Token::Print
                    || (tok.token == Token::Identifier && tok.value == "print")
        )
    }

    fn synchronize_block(&mut self) {
        // Skip tokens until a safe boundary (`}`, docs, next `GET`, or next path) is found.
        while !self.at_end() {
            if self.matches(Token::RightBrace) {
                self.advance();
                return;
            }

            if self.matches(Token::Docs) {
                return;
            }

            if self.matches(Token::Get)
                || self.matches(Token::Post)
                || self.matches(Token::Put)
                || self.matches(Token::Delete)
                || self.matches(Token::Patch)
            {
                return;
            }

            if self.matches(Token::Path) {
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
