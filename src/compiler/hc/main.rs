#![allow(clippy::all)]
#![allow(non_snake_case)]

mod logs;
mod utils;
use std::fs;

use clap::Parser;

use crate::{
    logs::diagnostics::Span,
    utils::{
        parser::{Value, parse_program},
        tokens::generate_tokens,
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

            for err in lex_errors {
                let span = Span {
                    file: file.clone(),
                    line: err.line,
                    column: err.column,
                };
                let message = format!("invalid token: '{}'", err.value);
                compiler_error!(&span, &message);
            }

            match parse_program(&ctx_tok) {
                Ok(program) => {
                    for route in program.routes {
                        if let Some(docs) = route.docs {
                            println!("Docs: {}", docs);
                        }

                        println!("Route: {:?} {}", route.method, route.path);
                        for assignment in route.body {
                            match assignment.value {
                                Value::String(value) => {
                                    println!("  {} = {}", assignment.name, value)
                                }
                                Value::Execute(value) => {
                                    println!("  {} = {}", assignment.name, value)
                                }
                            }
                        }
                    }
                }
                Err(parse_errors) => {
                    for err in parse_errors {
                        let span = Span {
                            file: file.clone(),
                            line: err.line,
                            column: err.column,
                        };
                        compiler_error!(&span, &err.message);
                    }
                }
            }
        }
        RunMode::None => {
            eprintln!("error: no input files");
        }
    }
}
