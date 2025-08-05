use super::{render_template, CARGO_TOML};
use crate::{AppInfo, Error, GenerateResults, Result};
use git2;
use rrgen::{self, RRgen};
use serde_json::json;
use std::fs;
use std::path::Path;
use toml::Value;

pub fn clone_repo(git_url: &str, path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| Error::Message(format!("Failed to create tasks directory: {}", e)))?;
    }
    // Get the name of the git repository to use as the task name
    let task_name = git_url
        .rsplit('/')
        .next()
        .ok_or(Error::Message(
            "Failed to get git repo name. Maybe no valid GIT URL has been provided!".to_string(),
        ))?
        .trim_end_matches(".git")
        .to_string();

    let task_path = path.join(&task_name);
    if task_path.exists() {
        return Err(Error::Message(format!(
            "Task directory {} already exists. Please remove it before cloning.",
            task_path.display()
        )));
    }

    // Create the task directory if it does not exist
    fs::create_dir_all(&task_path)
        .map_err(|e| Error::Message(format!("Failed to create task directory: {}", e)))?;

    // Create a temporary directory for cloning the repository

    println!("Cloning git repository");
    let temp_dir_str = &format!("./{}/{}-temp", path.display(), task_name);
    let temp_dir = Path::new(&temp_dir_str);
    git2::Repository::clone(git_url, temp_dir)
        .map_err(|e| Error::Message(format!("Failed to clone git repository: {}", e)))?;
    fs::rename(temp_dir, &task_path).map_err(|e| {
        Error::Message(format!(
            "Failed to move cloned files to task directory: {}",
            e
        ))
    })?;
    println!("Successfully generated git task in {}", task_path.display());
    Ok(())
}

// This function processes the cloned git repository.
// It checks if the Cargo.toml file exists, parses it, renames the directory
// to the task name, and renders the template files.
pub fn process_repo(rrgen: &RRgen, git_url: &str, appinfo: &AppInfo) -> Result<GenerateResults> {
    let git_path = git_url
        .rsplit("/")
        .next()
        .ok_or(Error::Message("Failed to get git repo name".to_string()))?
        .trim_end_matches(".git");
    println!("Processing git repository: {}", git_path);
    let path_str = format!("./src/tasks/{}", git_path);
    let git_dir = Path::new(&path_str);
    let config_path = git_dir.join(CARGO_TOML);
    println!("Check if Cargo.toml exists at: {}", git_dir.display());
    // Check if the configuration file exists
    if !config_path.exists() {
        return Err(Error::Message(format!(
            "{} not found in the repository",
            CARGO_TOML
        )));
    }
    let mut config_file = std::fs::read_to_string(config_path.clone()).map_err(|e| {
        Error::Message(format!(
            "Failed to read {} in task root directory: {}",
            CARGO_TOML, e
        ))
    })?;
    // Check if the dependencies table exists in the configuration file.
    // If not, add it.
    config_file = check_deps_table_in_config_file(config_file);
    let config_toml: Value = toml::from_str(&config_file)
        .map_err(|e| Error::Message(format!("Failed to parse {}: {}", CARGO_TOML, e)))?;

    // the task name is the package name in the Cargo.toml file
    // If the package name is missing, return an error.
    // The directory will be renamed to the package name later on.
    let task_name = config_toml
        .get("package")
        .and_then(|v| v.get("name"))
        .ok_or(Error::Message(format!(
            "Package name missing in {}. Task name is required.",
            CARGO_TOML
        )))?
        .as_str()
        .ok_or(Error::Message(format!(
            "Package name in {} is not a string",
            CARGO_TOML
        )))?;
    println!("Adding package root dependency to Cargo.toml");
    add_deps_to_root_cargo_toml(task_name)
        .map_err(|e| Error::Message(format!("Failed to update {}: {}", CARGO_TOML, e)))?;
    let task_name_path_string = format!("./src/tasks/{}", task_name.to_string());
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
    println!("Rendering template files");
    render_git_task(rrgen, task_name.to_string(), app_name)
}

pub fn render_git_task(
    rrgen: &RRgen,
    task_name: String,
    app_name: &str,
) -> Result<GenerateResults> {
    let vars = json!(
        {
            "name": task_name,
            "pkg_name": app_name,
            "is_git_task": true
        }
    );
    // Check if mod.rs exists in the tasks directory
    let task_path = Path::new("./src/tasks");
    let task_mod_path = task_path.join("mod.rs");
    if !task_path.exists() {
        fs::create_dir(task_path)
            .map_err(|e| Error::Message(format!("Failed to create tasks directory: {}", e)))?;
    }
    if !task_mod_path.exists() {
        fs::File::create(&task_mod_path)
            .map_err(|e| Error::Message(format!("Failed to create mod.rs: {}", e)))?;
    }
    render_template(rrgen, Path::new("task"), &vars)
}

pub fn add_deps_to_root_cargo_toml(task_name: &str) -> Result<()> {
    let root_config_file = fs::read_to_string(Path::new(CARGO_TOML)).map_err(|e| {
        Error::Message(format!(
            "Failed to read {} in project root: {}",
            CARGO_TOML, e
        ))
    })?;
    let parts = root_config_file.split("[dependencies]").collect::<Vec<_>>();
    let mut new_parts = Vec::new();
    new_parts.push(parts[0].to_string());
    new_parts.push("[dependencies]".to_string());
    new_parts.push(format!(
        r#"{} = {{ path = "./src/tasks/{}" }}"#,
        task_name, task_name
    ));
    new_parts.push(parts[1].to_string());
    fs::write(CARGO_TOML, new_parts.join("\n"))
        .map_err(|e| Error::Message(format!("Failed to write {}: {}", CARGO_TOML, e)))?;
    Ok(())
}

// This function removes the project_root dependency from the Cargo.toml.
// This is useful, because it allows the task to be used as a dependency in other projects.
// When the task.t is being rendered, the pkg_root dependency is added to the Cargo.toml with the correct path and name.
pub fn update_project_dep_in_task_cargo_toml(
    config_file: String,
    path: &Path,
    app_name: &str,
) -> Result<()> {
    let mut new_config_file = config_file
        .lines()
        .filter(|l| !l.contains("pkg_root"))
        .collect::<Vec<_>>()
        .join("\n");
    let parts = new_config_file.split("[dependencies]").collect::<Vec<_>>();
    let mut new_parts = Vec::new();
    new_parts.push(parts[0].to_string());
    new_parts.push("[dependencies]".to_string());
    new_parts.push(format!(
        r#"pkg_root = {{ package = "{}", path = "../../../" }}"#,
        app_name
    ));
    new_parts.push(parts[1].to_string());

    new_config_file = new_parts.join("\n");

    fs::write(path.join(CARGO_TOML), new_config_file).map_err(|e| {
        Error::Message(format!(
            "Cannot write in {}/{}: {}",
            path.display(),
            CARGO_TOML,
            e
        ))
    })?;
    Ok(())
}

// This function checks if the dependencies table exists in the configuration file.
// If it does not exist, it adds it.
// If it does exist, it returns the original configuration file.
// This is useful to ensure that the dependencies table is always present in the Cargo.toml.
pub fn check_deps_table_in_config_file(config_file: String) -> String {
    let mut new_config_file = String::new();
    if config_file.contains("[dependencies]") {
        config_file
    } else {
        new_config_file.push_str("\n[dependencies]\n");
        new_config_file
    }
}
