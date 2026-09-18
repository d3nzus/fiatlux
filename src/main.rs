mod parser;
mod file_handler;

use clap::{ Parser as Parser, Subcommand };
use crate::file_handler::blueprints_dir;
use std::path::{ Path, PathBuf };
use anyhow::Context;

#[derive(Subcommand)]
pub enum Commands {
    /// <blueprint> <project_name> - initializes project based on blueprint
    Init {
        blueprint: String,
        project_name: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// - lists usable blueprints
    List,
    /// <blueprint_name> <path> - imports a blueprint
    Import {
        blueprint_name: String,
        source: PathBuf,
    },
}

#[derive(Parser)]
#[command(name = "fiatlux", version, about = "Made by D3nzus. Customizable project bootstrapper")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { blueprint, project_name, dry_run } => {
            println!("Scaffolding '{project_name}' with '{blueprint}' (dry_run={dry_run})");
        }
        Commands::List => {
            let dir = blueprints_dir()?;
            let mut names: Vec<String> = std::fs
                ::read_dir(&dir)
                .with_context(|| format!("could not read `{}`", dir.display()))?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("fl"))
                .filter_map(|path| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .collect();

            names.sort();

            if names.is_empty() {
                println!(
                    "No blueprints imported yet. Use `fiatlux import <name> <path>` to add one."
                );
            } else {
                println!("Available blueprints:");
                for name in names {
                    println!("  - {name}");
                }
            }
        }
        Commands::Import { blueprint_name, source } => {
            let dir = blueprints_dir()?;
            let dest = dir.join(format!("{blueprint_name}.fl"));

            std::fs
                ::copy(&source, &dest)
                .with_context(|| {
                    format!("could not copy `{}` to `{}`", source.display(), dest.display())
                })?;

            println!("Imported blueprint '{blueprint_name}' -> {}", dest.display());
        }
    }

    Ok(())
}
