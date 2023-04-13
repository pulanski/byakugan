use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(
    author = "pulanski",
    version = "0.1.0",
    about = "Simple program to greet a person",
    long_about = "Simple program to greet a person"
)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}
