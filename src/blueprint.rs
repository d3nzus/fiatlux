use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
pub struct BlueprintDef {
    #[serde(default)]
    pub directories: Vec<FsEntry>,
    #[serde(default)]
    pub scripts: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FsEntry {
    File(String),
    Dir(HashMap<String, Vec<FsEntry>>),
}

pub fn parse_source(source: &str) -> anyhow::Result<BlueprintDef> {
    serde_yaml::from_str(source).context("failed to parse blueprint YAML")
}

pub fn generate_fs(entries: &[FsEntry], root: &Path) -> anyhow::Result<()> {
    for entry in entries {
        match entry {
            FsEntry::File(name) => {
                let file_name = if name.contains('.') {
                    name.clone()
                } else {
                    format!("{name}.txt")
                };
                let file_path = root.join(&file_name);
                std::fs::File::create(&file_path)
                    .with_context(|| format!("could not create file `{}`", file_path.display()))?;
            }
            FsEntry::Dir(map) => {
                for (name, children) in map {
                    let dir_path = root.join(name);
                    std::fs::create_dir_all(&dir_path)
                        .with_context(|| format!("could not create dir `{}`", dir_path.display()))?;
                    generate_fs(children, &dir_path)?;
                }
            }
        }
    }
    Ok(())
}