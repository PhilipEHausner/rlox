mod error_handling;

use crate::error_handling::ErrorHandler;
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
    println!("Hello World!");
    let file = read_file(args.file.as_str());
    ErrorHandler::init_logging().expect("Logging could not be setup.");
}

fn read_file(file: &str) -> String {
    std::fs::read_to_string(file)
        .expect("Input file could not be read.")
        .replace("\r\n", "\n")
}
