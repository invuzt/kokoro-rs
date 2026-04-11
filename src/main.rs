//! main.rs
//! This is the entry point for our Rust application.
//! In Rust, execution always begins in the `main` function.

mod cli; // This tells Rust to look for a file named `cli.rs` or a directory `cli/mod.rs` and include it as a module.
mod tts; // This tells Rust to include `tts/mod.rs` as a module.

// The `use` keyword brings specific items from modules into the current scope,
// similar to `import` in Python or Go.
use clap::Parser; // `Parser` is a trait (like an interface in Go) from the `clap` crate that allows structs to parse command-line arguments.
use cli::{Cli, Commands}; // Bring in our custom `Cli` struct and `Commands` enum from the `cli` module.
use std::process; // Bring in the standard library's process module to handle exiting the program.

fn main() {
    // `Cli::parse()` reads the command-line arguments passed by the user,
    // matches them against the structure we defined in `cli::Cli`, and returns an instance of `Cli`.
    // If the user passes invalid arguments or `--help`, `clap` will automatically print the error/help text and exit.
    let cli = Cli::parse();

    // `match` in Rust is a powerful version of a `switch` statement.
    // It forces you to handle *every possible variant* of the `enum`.
    // Here we are matching against the `command` field of our parsed `cli` struct.
    // We use `&cli.command` to pass a reference (borrow) rather than taking ownership.
    match &cli.command {
        Commands::Setup(args) => {
            if let Err(e) = cli::handle_setup(args) {
                eprintln!("\x1b[31mError: {}\x1b[0m", e);
                process::exit(1);
            }
        }
        // If the command is `Speak`, we extract its specific arguments into the `args` variable.
        Commands::Speak(args) => {
            // `handle_speak` returns a `Result<(), String>`.
            // `if let Err(e) = ...` is a concise way to say: "Run this function, and if it returns an Error, bind that error to `e` and run this block."
            if let Err(e) = cli::handle_speak(args) {
                // `eprintln!` prints to standard error (stderr) instead of standard output (stdout).
                // The `\x1b[31m` is an ANSI escape code to print the text in red.
                eprintln!("\x1b[31mError: {}\x1b[0m", e);
                process::exit(1); // Exit the program with a non-zero status code to indicate failure.
            }
        }
        Commands::Voices(args) => {
            if let Err(e) = cli::handle_voices(args) {
                eprintln!("\x1b[31mError: {}\x1b[0m", e);
                process::exit(1);
            }
        }
        Commands::Languages(args) => {
            if let Err(e) = cli::handle_languages(args) {
                eprintln!("\x1b[31mError: {}\x1b[0m", e);
                process::exit(1);
            }
        }
    }
}
