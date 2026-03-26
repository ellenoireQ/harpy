use logos::Logos;

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

pub fn generate_tokens(input: &str) {
    let mut lexer = Token::lexer(input);

    while let Some(token) = lexer.next() {
        let slice = lexer.slice();
        println!("{:?} {:?}", token, slice)
    }
}
