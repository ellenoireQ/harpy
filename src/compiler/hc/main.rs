mod utils;
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

    if args.tokens {
        generate_tokens();
    }
    if args.version {
        get_version();
    }
}
