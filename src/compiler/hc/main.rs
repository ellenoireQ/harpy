#![allow(clippy::all)]

mod logs;
mod utils;
use std::fs;

use clap::Parser;

use crate::{
    logs::diagnostics::Span,
    utils::{
        tokens::{Token, TokenKind, generate_tokens},
        version::get_version,
    },
};

#[derive(Parser, Debug)]
struct Args {
    /// Input file to compile or run
    /// Example: hc main.hp
    file: Option<String>,

    /// Show tokens
    #[arg(short, long)]
    tokens: bool,

    /// Print version
    #[arg(short, long)]
    version: bool,
}

enum RunMode {
    Tokens { file: String },
    Version,
    Compile { file: String },
    None,
}

fn resolve_run_mode(args: Args) -> RunMode {
    match (args.file, args.tokens, args.version) {
        (Some(file), true, false) => RunMode::Tokens { file },
        (_, _, true) => RunMode::Version,
        (Some(file), false, false) => RunMode::Compile { file },
        _ => RunMode::None,
    }
}

fn main() {
    let mode = resolve_run_mode(Args::parse());

    match mode {
        RunMode::Tokens { file } => {
            let content = fs::read_to_string(&file).expect("failed to read file");
            let (ctx_tok, lex_errors) = generate_tokens(&content);
            for token in ctx_tok {
                println!("{:?} => {:?}", token.token, token.value)
            }

            for err in lex_errors {
                let span = Span {
                    file: file.clone(),
                    line: err.line,
                    column: err.column,
                };
                let message = format!("invalid token: '{}'", err.value);
                compiler_error!(&span, &message);
            }
        }
        RunMode::Version => get_version(),
        RunMode::Compile { file } => {
            let content = fs::read_to_string(&file).expect("failed to read file");
            let (ctx_tok, lex_errors) = generate_tokens(&content);
            let mut matched = false;

            for err in lex_errors {
                let span = Span {
                    file: file.clone(),
                    line: err.line,
                    column: err.column,
                };
                let message = format!("invalid token: '{}'", err.value);
                compiler_error!(&span, &message);
            }

            let mut i = 0;

            // We started from zero
            // Started from searching Docs, the docs is optional if not have it DO skipped
            // if docs zero increased i by 1 and catched by Token::Get using ctx_tok.get(i)
            // after that Token::Path continuing by increasing the index ctx_tox.get(i + 1)
            // This logic support to scan what is happen inside { }
            while i < ctx_tok.len() {
                if let TokenKind {
                    token: Token::Docs, ..
                } = &ctx_tok[i]
                {
                    i += 1;
                    continue;
                }
                if let Some(TokenKind {
                    token: Token::Get, ..
                }) = ctx_tok.get(i)
                {
                    if let Some(TokenKind {
                        token: Token::Path,
                        value: path,
                        ..
                    }) = ctx_tok.get(i + 1)
                    {
                        let mut j = i + 2;
                        let mut has_left_brace = false;
                        let mut has_right_brace = false;

                        while j < ctx_tok.len() {
                            if let TokenKind {
                                token: Token::LeftBrace,
                                ..
                            } = &ctx_tok[j]
                            {
                                has_left_brace = true;
                                let mut k = j + 1;
                                while k < ctx_tok.len() {
                                    if let TokenKind {
                                        token: Token::RightBrace,
                                        ..
                                    } = &ctx_tok[k]
                                    {
                                        has_right_brace = true;
                                        println!("GET: {}", path);
                                        matched = true;

                                        println!("Body:");
                                        for window in ctx_tok[j + 1..k].windows(3) {
                                            if let [name_tok, equal_tok, value_tok] = window {
                                                if name_tok.token == Token::Identifier
                                                    && equal_tok.token == Token::Equal
                                                    && (value_tok.token == Token::String
                                                        || value_tok.token == Token::Execute)
                                                {
                                                    println!(
                                                        "Variable {} = {}",
                                                        name_tok.value, value_tok.value
                                                    );
                                                }
                                            }
                                        }
                                        for body in &ctx_tok[j + 1..k] {
                                            println!("{:?} {:?}", body.token, body.value);
                                        }

                                        break;
                                    }

                                    k += 1;
                                }

                                break;
                            }

                            j += 1;
                        }

                        if !has_left_brace {
                            let span = Span {
                                file: file.clone(),
                                line: ctx_tok[i + 1].line,
                                column: ctx_tok[i + 1].column,
                            };
                            compiler_error!(&span, "expected '{' after route path");
                        } else if !has_right_brace {
                            let span = Span {
                                file: file.clone(),
                                line: ctx_tok[i + 1].line,
                                column: ctx_tok[i + 1].column,
                            };
                            compiler_error!(&span, "expected '}' to close route body");
                        }
                    } else {
                        let span = Span {
                            file: file.clone(),
                            line: ctx_tok[i].line,
                            column: ctx_tok[i].column,
                        };
                        compiler_error!(&span, "expected route path after GET");
                    }
                }

                i += 1;
            }

            if !matched {
                // Do nothing
            }
        }
        RunMode::None => {
            eprintln!("error: no input files");
        }
    }
}
