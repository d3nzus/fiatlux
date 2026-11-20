/*
cli.rs contains cli logic using clap crate
*/

use crate::blueprint::{generate_fs, parse_source};
use crate::file_handler::{blueprints_dir, run_scripts};
use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use std::path::{Component, Path, PathBuf};

// the CLI struct inherits from clap::Parser, which provides the command line parsing functionality
#[derive(Parser)]
#[command(
    name = "fiatlux",
    version,
    about = "Customizable project bootstrapper by D3nzus"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

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

pub fn match_command() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            blueprint,
            project_name,
            dry_run,
        } => {
            let project_name_path = Path::new(&project_name);
            if !matches!(
                project_name_path.components().next(),
                Some(Component::Normal(_))
            ) || project_name_path.components().count() != 1
            {
                bail!("project name must be a single folder name");
            }

            let dir = blueprints_dir()?;
            let blueprint_path = dir.join(format!("{blueprint}.yaml"));
            let source = std::fs::read_to_string(&blueprint_path).with_context(|| {
                format!(
                    "could not read blueprint `{blueprint}` at `{}`",
                    blueprint_path.display()
                )
            })?;

            // parse blueprint
            let blueprint_def = parse_source(&source).with_context(|| {
                format!("could not parse blueprint `{}`", blueprint_path.display())
            })?;

            let destination = std::env::current_dir()
                .context("could not determine the current working directory")?
                .join(&project_name);

            if dry_run {
                println!(
                    "Would scaffold '{project_name}' with '{blueprint}' at {}",
                    destination.display()
                );
                return Ok(());
            }

            std::fs::create_dir(&destination).with_context(|| {
                format!(
                    "could not create project directory `{}`",
                    destination.display()
                )
            })?;
            println!("Created project directory `{}`", destination.display());

            generate_fs(&blueprint_def.directories, &destination).with_context(|| {
                format!("could not generate project in `{}`", destination.display())
            })?;
            println!("Generated project files in `{}`", destination.display());

            run_scripts(&blueprint_def.scripts, &destination).with_context(|| {
                format!(
                    "could not run blueprint scripts in `{}`",
                    destination.display()
                )
            })?;
            println!("Successfully ran blueprint scripts");

            println!("Scaffolded '{project_name}' at {}", destination.display());
        }
        Commands::List => {
            let dir = blueprints_dir()?;

            // read all files in the blueprints directory, filter for .yaml files, and collect their names
            let mut names: Vec<String> = std::fs::read_dir(&dir)
                .with_context(|| format!("could not read `{}`", dir.display()))?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("yaml"))
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
        Commands::Import {
            blueprint_name,
            source,
        } => {
            let dir = blueprints_dir()?;
            let dest = dir.join(format!("{blueprint_name}.yaml"));

            std::fs::copy(&source, &dest).with_context(|| {
                format!(
                    "could not copy `{}` to `{}`",
                    source.display(),
                    dest.display()
                )
            })?;

            println!(
                "Imported blueprint '{blueprint_name}' -> {}",
                dest.display()
            );
        }
    }

    Ok(())
}
