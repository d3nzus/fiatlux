mod parser;

use clap::{ Parser as Parser, Subcommand };
use anyhow::Context;
use std::path::{ Path, PathBuf };

#[derive(Parser)]
#[command(name = "fiatlux", version, about = "Customizable project bootstrapper")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

fn blueprints_dir() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().context("could not determine a config directory for this OS")?;
    let dir = base.join("fiatlux").join("blueprints");
    std::fs
        ::create_dir_all(&dir)
        .with_context(|| format!("could not create blueprints dir `{}`", dir.display()))?;
    Ok(dir)
}

fn run_shell(cmd_str: &str, cwd: &Path) -> anyhow::Result<std::process::ExitStatus> {
    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("cmd")
        .args(["/C", cmd_str])
        .current_dir(cwd)
        .status()
        .with_context(|| format!("failed to run `{cmd_str}`"))?;

    #[cfg(not(target_os = "windows"))]
    let status = std::process::Command::new("sh")
        .args(["-c", cmd_str])
        .current_dir(cwd)
        .status()
        .with_context(|| format!("failed to run `{cmd_str}`"))?;

    Ok(status)
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
