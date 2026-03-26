use logos::Logos;

use crate::utils::tokens;

pub struct TokenKind {
    pub token: Token,
    pub value: String,
}

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[token("main")]
    Fn,

    #[token("GET")]
    Get,

    #[regex(r"\$[a-zA-Z0-9/_-]*")]
    Execute,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex("[0-9]+")]
    Number,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("=")]
    Equal,

    #[regex(r"#.*", logos::skip, allow_greedy = true)]
    Comment,

    #[regex(r"##[^\n]*", allow_greedy = true)]
    Docs,

    #[regex(r"/[a-zA-Z0-9/_-]*")]
    Path,

    #[regex(r"[ \t\n\f]+", logos::skip)]
    Whitespace,
}

pub fn generate_tokens(input: &str) -> Vec<TokenKind> {
    let mut lexer = Token::lexer(input);
    let mut tokens = Vec::new();

    while let Some(token) = lexer.next() {
        let slice = lexer.slice().to_string();

        if let Ok(token) = token {
            tokens.push(TokenKind {
                token,
                value: slice,
            });
        }
    }
    tokens
}
