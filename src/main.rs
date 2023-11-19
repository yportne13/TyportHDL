use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use typort_interpreter::main_cli;
use typort_lsp::main_lsp;

/// A HDL
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "typort")]
#[command(about = "A HDL", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// for language server
    Lsp,
    /// run a file
    #[command(arg_required_else_help = true)]
    Cli {
        /// file path
        path: PathBuf,
        /// main function name
        main: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Lsp => main_lsp().await,
        Commands::Cli { path, main } => main_cli(&path, main),
    }
}
