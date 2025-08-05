#[test]
fn test_clone_repo() {
    use super::clone_repo;
    use std::fs;
    use std::path::Path;

    let git_url = "https://github.com/loco-rs/loco";
    let path = Path::new("./tasks");
    clone_repo(git_url, path).unwrap();
    fs::remove_dir_all(path).unwrap(); // Clean up after test
}

#[test]
fn test_check_deps_table_in_config_file() {
    use super::check_deps_table_in_config_file;
    let config_file_with_deps = r#"
        [package]
        name = "test_package"
        version = "0.1.0"
        edition = "2021"

        [dependencies]
        "#;
    let result = check_deps_table_in_config_file(config_file_with_deps.to_string());
    assert!(result.contains("[dependencies]"));

    let config_file_without_deps = r#"
        [package]
        name = "test_package"
        version = "0.1.0"
        edition = "2021"
        "#;
    let result = check_deps_table_in_config_file(config_file_without_deps.to_string());
    assert!(result.contains("[dependencies]"));
}

#[test]
fn test_remove_project_dep_from_cargo_toml() {
    use super::update_project_dep_in_task_cargo_toml;
    use std::fs;
    use std::path::Path;

    let config_file = r#"
        [package]
        name = "test_package"
        version = "0.1.0"
        edition = "2021"

        [dependencies]
        pkg_root = { path = "./" }
        "#;

    let config_path = Path::new("./Cargo_test.toml");
    update_project_dep_in_task_cargo_toml(config_file.to_string(), config_path, "test_package")
        .unwrap();
    let new_config = fs::read_to_string(config_path).unwrap();
    assert!(new_config.contains("pkg_root"));
    fs::remove_file(config_path).unwrap(); // Clean up after test
}

// !!! IMPORTANT NOTE !!!
// THIS TEST FUNCTION IS USED TO CREATE A TEST GIT TASK RUST LIB.
// IT IS ONLY INTENDED FOR TESTING IN THE LOCO-GEN CRATE TO MAKE SURE THAT THE PORCESSING AND RENDERING OF A GIT TASK WORKS WITHOUT THROWING ERRORS.
// IT IS NOT INTENDED FOR TESTING IN A LOCO PROJECT.
#[test]
fn create_and_render_git_test_task() {
    use super::process_repo;
    use crate::AppInfo;
    use std::fs;
    use std::io::Write;
    use std::path::Path;
    use std::process::Command;

    _ = Command::new("cargo")
        .args(["new", "./src/tasks/git_test_task", "--lib"])
        .status()
        .unwrap();
    let rrgen = crate::RRgen::with_working_dir(".");
    let appinfo = AppInfo {
        app_name: "test_app".to_string(),
    };
    // create mod.rs file for testing
    let mod_file_path = "./src/tasks/mod.rs";
    let _mod_file = fs::File::create(mod_file_path).unwrap();
    // create tasks directory for testing if it doesn't exist
    let test_tasks_dir = "./tests/tasks";
    if !Path::new(test_tasks_dir).exists() {
        fs::create_dir(test_tasks_dir).unwrap();
    }
    // create test mod file for testing
    let test_mod_file_path = "./tests/tasks/mod.rs";
    let _test_mod_file = fs::File::create(test_mod_file_path).unwrap();
    // create app.rs for testing
    let app_file_path = "./src/app.rs";
    let mut app_file = fs::File::create(app_file_path).unwrap();
    let _ = writeln!(app_file, "// tasks-inject").unwrap();
    let git_url = "/git_test_task.git";

    // Save the original Cargo.toml to restore it after the test
    let cargo_toml = fs::read_to_string("./Cargo.toml").unwrap();

    // Process the repo and render the test git task
    let _result = process_repo(&rrgen, git_url, &appinfo).unwrap();

    // DO THE CLEAN-UP AFTER TESTING
    cleanup("./src/tasks", "./tests/tasks", app_file_path, &cargo_toml);
}

// !!! IMPORTANT NOTE !!!
// THIS TEST FUNCTION IS USED TO CLONE A TEST GIT TASK RUST LIB.
// IT IS ONLY INTENDED FOR TESTING IN THE LOCO-GEN CRATE TO MAKE SURE THAT THE PORCESSING AND RENDERING OF A GIT TASK WORKS WITHOUT THROWING ERRORS.
// IT IS NOT INTENDED FOR TESTING IN A LOCO PROJECT.
#[test]
fn clone_and_render_git_test_task() {
    use crate::AppInfo;
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    let test_git_url = "https://github.com/floscodes/loco-git-task-template.git";
    let rrgen = crate::RRgen::with_working_dir(".");
    let appinfo = AppInfo {
        app_name: "test_app".to_string(),
    };

    super::fetch_and_generate(&rrgen, Some(&test_git_url.to_owned()), &appinfo).unwrap();

    // create mod.rs file for testing
    let mod_file_path = "./src/tasks/mod.rs";
    let _mod_file = fs::File::create(mod_file_path).unwrap();
    // create tasks directory for testing if it doesn't exist
    let test_tasks_dir = "./tests/tasks";
    if !Path::new(test_tasks_dir).exists() {
        fs::create_dir(test_tasks_dir).unwrap();
    }
    // create test mod file for testing
    let test_mod_file_path = "./tests/tasks/mod.rs";
    let _test_mod_file = fs::File::create(test_mod_file_path).unwrap();
    // create app.rs for testing
    let app_file_path = "./src/app.rs";
    let mut app_file = fs::File::create(app_file_path).unwrap();
    let _ = writeln!(app_file, "// tasks-inject").unwrap();

    // Save the original Cargo.toml to restore it after the test
    let cargo_toml = fs::read_to_string("./Cargo.toml").unwrap();

    // DO THE CLEAN-UP AFTER TESTING
    cleanup("./src/tasks", "./tests/tasks", app_file_path, &cargo_toml);
}

fn cleanup(tasks_path: &str, test_tasks_path: &str, app_file_path: &str, cargo_toml_string: &str) {
    use std::path::Path;

    // Clean up the tasks directory
    if Path::new(tasks_path).exists() {
        std::fs::remove_dir_all(tasks_path).unwrap();
    }
    // Clean up the test tasks directory
    if Path::new(test_tasks_path).exists() {
        std::fs::remove_dir_all(test_tasks_path).unwrap();
    }
    // Clean up the app file
    if Path::new(app_file_path).exists() {
        std::fs::remove_file(app_file_path).unwrap();
    }

    // Restore the original Cargo.toml
    let cargo_toml_path = Path::new("./Cargo.toml");
    if cargo_toml_path.exists() {
        std::fs::write(cargo_toml_path, cargo_toml_string).unwrap();
    }
}
