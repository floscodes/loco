use super::render_template;
use crate::{AppInfo, Error, GenerateResults, Result};
use git2;
use rrgen::{self, RRgen};
use serde_json::json;
use std::path::Path;
use std::str::FromStr;
use toml::Value;
use toml_edit::{Document, DocumentMut, Item, Table};

const CONFIG_FILE: &str = "loco-task.toml";

pub fn fetch_and_generate(
    rrgen: &RRgen,
    git_url: Option<&String>,
    appinfo: &AppInfo,
) -> Result<GenerateResults> {
    if let Some(git_url) = git_url {
        println!("Fetching task from git repository: {}", git_url);
        let _repo = git2::Repository::clone(git_url, "./task")
            .map_err(|e| Error::Message(format!("Failed to clone git repository: {}", e)))?;
        let git_path = git_url
            .rsplit("/")
            .next()
            .ok_or(Error::Message("Failed to get git repo name".to_string()))?;
        let path_str = format!("./task/{}", git_path);
        let git_dir = Path::new(&path_str);
        let config_path = git_dir.join(CONFIG_FILE);
        println!(
            "Check if loco-task.toml exists at: {}",
            config_path.display()
        );
        // Check if the configuration file exists
        if !config_path.exists() {
            return Err(Error::Message(format!(
                "Configuration file '{}' not found in the repository",
                CONFIG_FILE
            )));
        }
        let config_file = std::fs::read_to_string(config_path.clone())
            .map_err(|e| Error::Message(format!("Failed to read loco-task.toml: {}", e)))?;
        println!("Parsing loco-task.toml");
        let config_toml: Value = toml::from_str(&config_file)
            .map_err(|e| Error::Message(format!("Failed to parse loco-task.toml: {}", e)))?;
        let task_name = config_toml
            .get("loco-task")
            .and_then(|v| v.get("name"))
            .ok_or(Error::Message(
                "Task name not found in loco-task.toml. Task name is required.".to_string(),
            ))?;
        let task_name_path_string = format!("./task/{}", task_name.to_string());
        println!(
            "Renaming git directory to task name: {}",
            task_name_path_string
        );
        let renamed_git_dir = Path::new(&task_name_path_string);
        std::fs::rename(git_dir, renamed_git_dir).map_err(|e| {
            Error::Message(format!(
                "Failed to rename git directory to task name: {}",
                e
            ))
        })?;
        let app_name = appinfo.app_name.as_str();

        println!("Adding required dependencies to Cargo.toml if needed");
        fetch_and_clone_deps(&renamed_git_dir).map_err(|e| {
            Error::Message(format!("Failed to fetch and clone dependencies: {}", e))
        })?;
        println!("Rendering templates");
        render_git_task(rrgen, &renamed_git_dir, task_name.to_string(), app_name)
    } else {
        Err(Error::Message(
            "Error while trying to generate task from git - no valid git url provided!".to_string(),
        ))
    }
}

fn render_git_task(
    rrgen: &RRgen,
    git_dir: &Path,
    task_name: String,
    app_name: &str,
) -> Result<GenerateResults> {
    let vars = json!(
        {
            "name": task_name,
            "pkg_name": app_name
        }
    );
    render_template(rrgen, git_dir, &vars)
}

fn fetch_and_clone_deps(git_dir: &Path) -> Result<()> {
    let task_config_raw = std::fs::read_to_string(git_dir.join(CONFIG_FILE))
        .map_err(|e| Error::Message(format!("Failed to read task config file: {}", e)))?;
    let cargo_toml_raw = std::fs::read_to_string(Path::new("Cargo.toml"))
        .map_err(|e| Error::Message(format!("Failed to read Cargo.toml: {}", e)))?;
    let task_config = Document::parse(task_config_raw)
        .map_err(|e| Error::Message(format!("Failed to parse task config file TOML: {}", e)))?;
    let mut cargo_toml = DocumentMut::from_str(&cargo_toml_raw)
        .map_err(|e| Error::Message(format!("Failed to parse Cargo.toml: {}", e)))?;

    if let Some(deps) = task_config.get("dependencies") {
        if let Some(deps) = deps.as_table() {
            let entry = cargo_toml
                .entry("dependencies")
                .or_insert(Item::Table(Table::new()));
            if let Item::Table(table) = entry {
                table.extend(deps.clone());
            }
        }
    }

    std::fs::write("Cargo.toml", cargo_toml.to_string())
        .map_err(|e| Error::Message(format!("Failed to write updated Cargo.toml: {}", e)))?;

    Ok(())
}
