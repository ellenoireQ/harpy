#![allow(clippy::all)]
#![allow(non_snake_case)]

mod logs;
mod network;
mod utils;
use std::fs;

use clap::Parser;

use crate::{
    logs::diagnostics::Span,
    utils::{
        codegen::generate_rust_code,
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

    /// Output file for generated code
    #[arg(short, long, default_value = "generated.rs")]
    output: String,
}

enum RunMode {
    Tokens { file: String },
    Version,
    Compile { file: String, output: String },
    None,
}

fn resolve_run_mode(args: &Args) -> RunMode {
    match (&args.file, args.tokens, args.version) {
        (Some(file), true, false) => RunMode::Tokens { file: file.clone() },
        (_, _, true) => RunMode::Version,
        (Some(file), false, false) => RunMode::Compile {
            file: file.clone(),
            output: args.output.clone(),
        },
        _ => RunMode::None,
    }
}

fn main() {
    let mode = resolve_run_mode(&Args::parse());

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
        RunMode::Compile { file, output } => {
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
                    // Print AST info
                    for block in &program.blocks {
                        if let Some(ref docs) = block.docs {
                            println!("Docs: {}", docs);
                        }

                        println!("Block: {:?} {}", block.method, block.path);
                        for assignment in &block.body {
                            match &assignment.value {
                                Value::String(value) => {
                                    println!("  {} = {}", assignment.name, value)
                                }
                                Value::Execute(value) => {
                                    println!("  {} = {}", assignment.name, value)
                                }
                            }
                        }
                    }

                    // Generate Rust code
                    let rust_code = generate_rust_code(&program);
                    fs::write(&output, &rust_code).expect("failed to write output file");
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
