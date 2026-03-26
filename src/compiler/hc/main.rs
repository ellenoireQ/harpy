mod utils;
use std::fs;

use clap::Parser;

use crate::utils::{tokens::generate_tokens, version::get_version};

#[derive(Parser, Debug)]
struct Args {
    file: Option<String>,

    /// Show tokens
    #[arg(short, long)]
    tokens: bool,

    #[arg(short, long)]
    version: bool,
}

enum RunMode {
    Tokens { file: String },
    Version,
    None,
}

fn resolve_run_mode(args: Args) -> RunMode {
    match (args.file, args.tokens, args.version) {
        (Some(file), true, false) => RunMode::Tokens { file },
        (_, _, true) => RunMode::Version,
        _ => RunMode::None,
    }
}

fn main() {
    let mode = resolve_run_mode(Args::parse());

    match mode {
        RunMode::Tokens { file } => {
            let content = fs::read_to_string(file).expect("failed to read file");
            generate_tokens(&content);
        }
        RunMode::Version => get_version(),
        RunMode::None => {}
    }
}
