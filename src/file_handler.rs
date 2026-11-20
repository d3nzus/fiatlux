use anyhow::{Context, bail};
use std::path::{Path, PathBuf};

// create fialtux/blueprints folder if not already created, when it exists, return a path to it
pub fn blueprints_dir() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().context("could not determine a config directory for this OS")?;
    // dir is path to fiatlux/blueprints folder in the computer config directory
    let dir = base.join("fiatlux").join("blueprints");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("could not create blueprints dir `{}`", dir.display()))?;
    Ok(dir)
}

// runs a single command in the working directory
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

// runs blueprint scripts in order from the generated project directory
pub fn run_scripts(scripts: &[String], cwd: &Path) -> anyhow::Result<()> {
    for script in scripts {
        let status = run_shell(script, cwd)
            .with_context(|| format!("failed to execute blueprint script `{script}`"))?;

        if !status.success() {
            bail!("blueprint script `{script}` exited with status {status}");
        }
    }

    Ok(())
}
