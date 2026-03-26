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

fn main() {
    let args = Args::parse();

    if let Some(file) = args.file {
        let content = fs::read_to_string(file).expect("failed to read file");

        if args.tokens {
            generate_tokens(&content);
        }
    }

    if args.version {
        get_version();
    }
}
