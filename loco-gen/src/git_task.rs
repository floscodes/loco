use super::render_template;
use crate::{AppInfo, Error, GenerateResults, Result};
use git2;
use rrgen::{self, RRgen};
use serde_json::json;
use std::path::Path;
use std::str::FromStr;
use std::{clone, fmt::Display};
use toml::Value;
use toml_edit::{DocumentMut, Item, Table};

const CONFIG_FILE: &str = "Cargo.toml";

pub fn fetch_and_generate(
    rrgen: &RRgen,
    git_url: Option<&String>,
    appinfo: &AppInfo,
) -> Result<GenerateResults> {
    if let Some(git_url) = git_url {
        println!("Cloning task from git repository: {}", git_url);
        let _repo = clone_repo(git_url, Path::new("./tasks"))?;
        println!("Processing the cloned repository");
        process_repo(rrgen, git_url, appinfo)
            .map_err(|e| Error::Message(format!("Failed to process git repository: {}", e)))
    } else {
        Err(Error::Message(
            "Error while trying to generate task from git - no valid git url provided!".to_string(),
        ))
    }
}

fn clone_repo(git_url: &str, path: &Path) -> Result<()> {
    git2::Repository::clone(git_url, path)
        .map_err(|e| Error::Message(format!("Failed to clone git repository: {}", e)))?;
    Ok(())
}

fn process_repo(rrgen: &RRgen, git_url: &str, appinfo: &AppInfo) -> Result<GenerateResults> {
    let git_path = git_url
        .rsplit("/")
        .next()
        .ok_or(Error::Message("Failed to get git repo name".to_string()))?;
    let path_str = format!("./tasks/{}", git_path);
    let git_dir = Path::new(&path_str);
    let config_path = git_dir.join(CONFIG_FILE);
    println!(
        "Check if loco-task.toml exists at: {}",
        config_path.display()
    );
    // Check if the configuration file exists
    if !config_path.exists() {
        return Err(Error::Message(format!(
            "{} not found in the repository",
            CONFIG_FILE
        )));
    }
    let config_file = std::fs::read_to_string(config_path.clone())
        .map_err(|e| Error::Message(format!("Failed to read {}: {}", CONFIG_FILE, e)))?;
    println!("Parsing loco-task.toml");
    let config_toml: Value = toml::from_str(&config_file)
        .map_err(|e| Error::Message(format!("Failed to parse {}: {}", CONFIG_FILE, e)))?;
    let task_name = config_toml
        .get("package")
        .and_then(|v| v.get("name"))
        .ok_or(Error::Message(format!(
            "Package name missing in {}. Task name is required.",
            CONFIG_FILE
        )))?;
    let task_name_path_string = format!("./tasks/{}", task_name.to_string());
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
    remove_project_dep_from_cargo_toml(config_file, &config_path).map_err(|e| {
        Error::Message(format!(
            "Failed to edit Cargo.toml for updating pkg_root dependency: {}",
            e
        ))
    })?;
    let app_name = appinfo.app_name.as_str();
    println!("Rendering template files");
    render_git_task(rrgen, task_name.to_string(), app_name)
}

fn render_git_task(rrgen: &RRgen, task_name: String, app_name: &str) -> Result<GenerateResults> {
    let vars = json!(
        {
            "name": task_name,
            "pkg_name": app_name,
            "is_git_task": true
        }
    );
    render_template(rrgen, Path::new("task"), &vars)
}

// This function removes the project_root dependency from the Cargo.toml.
// This is useful, because it allows the task to be used as a dependency in other projects.
// When the task.t is being rendered, the pkg_root dependency is added to the Cargo.toml with the correct path and name.
fn remove_project_dep_from_cargo_toml(config_file: String, path: &Path) -> Result<()> {
    let new_config_file = config_file
        .lines()
        .filter(|l| !l.contains("pkg_root"))
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(path, new_config_file)
        .map_err(|e| Error::Message(format!("Failed to write updated {}: {}", CONFIG_FILE, e)))?;
    Ok(())
}

/* fn add_to_cargo_toml(task_name: &String) -> Result<()> {
    let cargo_toml_raw = std::fs::read_to_string(CONFIG_FILE)
        .map_err(|e| Error::Message(format!("Failed to read {}: {}", CONFIG_FILE, e)))?;

    let mut cargo_toml = DocumentMut::from_str(&cargo_toml_raw)
        .map_err(|e| Error::Message(format!("Failed to parse Cargo.toml: {}", e)))?;

    let deps = cargo_toml
        .entry("dependencies")
        .or_insert(Item::Table(Table::new()));

    if let Item::Table(deps_table) = deps {
        let mut dep_item = Table::new();
        dep_item["path"] = toml_edit::value(format!("./tasks/{}", task_name));
        dep_item.set_implicit(true);

        deps_table[task_name] = Item::Table(dep_item);
    }

    std::fs::write(CONFIG_FILE, cargo_toml.to_string())
        .map_err(|e| Error::Message(format!("Failed to write updated {}: {}", CONFIG_FILE, e)))?;

    Ok(())
}
 */
