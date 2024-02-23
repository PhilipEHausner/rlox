mod error_handling;
mod frontend;

use crate::error_handling::ErrorHandler;
use crate::frontend::scanner::{scan, TokenType};
use clap::Parser;

#[derive(clap::Parser, Debug)]
#[command(version, about = "A statically typed lox interpreter.")]
struct Args {
    /// The input file to parse and execute
    #[arg(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();
    let file = read_file(args.file.as_str());
    ErrorHandler::init_logging().expect("Logging could not be setup.");

    let error_handler = ErrorHandler::new(&file);
    let tokens = scan(&file, &error_handler).unwrap();

    for token in tokens.iter() {
        if token.token_type() == &TokenType::EOF {
            continue;
        }
        error_handler.report_error(
            &format!("{:?}", &token.token_type()),
            token.line_information(),
        );
    }
}

fn read_file(file: &str) -> String {
    std::fs::read_to_string(file)
        .expect("Input file could not be read.")
        .replace("\r\n", "\n")
}
