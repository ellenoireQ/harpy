use logos::Logos;

pub struct TokenKind {
    pub token: Token,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

pub struct LexError {
    pub value: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[token("main")]
    Fn,

    #[token("fn")]
    Function,

    #[token("GET")]
    Get,

    #[regex(r"\$[a-zA-Z0-9/_-]*")]
    Execute,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex("[0-9]+")]
    Number,

    #[regex(r#"\"([^\"\\]|\\.)*\""#)]
    String,

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

    #[token("return")]
    Return,

    #[token("print")]
    Print,

    #[regex(r"/[a-zA-Z0-9/_-]*")]
    Path,

    #[token("(")]
    LeftParent,

    #[token(")")]
    RightParent,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[regex(r"[ \t\n\f]+", logos::skip)]
    Whitespace,
}

fn line_column_at(input: &str, byte_index: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for byte in input.as_bytes().iter().take(byte_index) {
        if *byte == b'\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

pub fn generate_tokens(input: &str) -> (Vec<TokenKind>, Vec<LexError>) {
    let mut lexer = Token::lexer(input);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    while let Some(token) = lexer.next() {
        let slice = lexer.slice().to_string();
        let span = lexer.span();
        let (line, column) = line_column_at(input, span.start);

        if let Ok(token) = token {
            tokens.push(TokenKind {
                token,
                value: slice,
                line,
                column,
            });
        } else {
            errors.push(LexError {
                value: slice,
                line,
                column,
            });
        }
    }

    (tokens, errors)
}
