use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
//use typort_interpreter::main_cli;
use typort_lsp::main_lsp;
use typort_parser::parse;
use typort_tyck::tyck;

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
    /// show ast
    Ast {
        /// file path
        path: PathBuf,
    },
    /// show hir
    Hir {
        /// file path
        path: PathBuf,
    },
    /// run a file
    #[command(arg_required_else_help = true)]
    Cli {
        /// file path
        path: PathBuf,
        /// main function name
        main: Option<String>,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Lsp => main_lsp().unwrap(),
        Commands::Ast { path } => {
            let text = std::fs::read_to_string(path).expect("Unable to read file");
            let ast = parse(&text, 0);
            for f in ast {
                println!("{:?}", f);
            }
        },
        Commands::Hir { path } => {
            let text = std::fs::read_to_string(path).expect("Unable to read file");
            let ast = parse(&text, 0);
            let hir = tyck(ast, Default::default()).unwrap();
            for f in hir {
                println!("{:?}", f);
            }
        },
        Commands::Cli { path, main } => todo!(),//main_cli(&path, main),
    }
}
