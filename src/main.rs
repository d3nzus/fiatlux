use clap::{ Parser, Subcommand };
use anyhow::Context;

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
    Plan {
        blueprint_name: String,
        blueprint_path: std::path::PathBuf,
    },
}

struct Blueprint {
    name: String,
    commands: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut active_blueprints: Vec<Blueprint> = Vec::new();

    match cli.command {
        Commands::Init { blueprint, project_name, dry_run } => {
            println!("Scaffolding '{project_name}' with '{blueprint}' (dry_run={dry_run})");
        }
        Commands::List => {
            if active_blueprints.is_empty() {
                println!("No usable blueprints found. Import with fiatlux plan <name> <path>");
            } else {
                println!("Listing blueprints...");
                for i in active_blueprints {
                    println!("{}", i.name);
                }
            }
        }
        Commands::Plan { blueprint_name, blueprint_path } => {
            let content = std::fs
                ::read_to_string(&blueprint_path)
                .with_context(|| format!("could not read file `{}`", blueprint_path.display()))?;

            // TODO: parse `content` into actual command lines instead of leaving it empty
            active_blueprints.push(Blueprint {
                name: blueprint_name,
                commands: Vec::new(),
            });

            println!("Loaded blueprint with {} bytes of content", content.len());
        }
    }

    Ok(())
}
