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
}
