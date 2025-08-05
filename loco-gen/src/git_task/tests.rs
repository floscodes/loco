#[cfg(test)]
mod tests {
    use crate::git_task;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_clone_repo() {
        use git_task::clone_repo;
        use std::fs;
        use std::path::Path;

        let git_url = "https://github.com/floscodes/loco-git-task-template.git";

        clone_repo(git_url, Path::new("./tasks")).unwrap();
        fs::remove_dir_all("./tasks").unwrap(); // Clean up after test
    }

    #[test]
    #[serial]
    fn test_check_deps_table_in_config_file() {
        use git_task::check_deps_table_in_config_file;
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
        use git_task::update_project_dep_in_task_cargo_toml;
        use std::fs;
        use std::path::Path;

        // Read the original Cargo.toml
        let config_file = fs::read_to_string("./Cargo.toml").unwrap();

        update_project_dep_in_task_cargo_toml(config_file, Path::new("./"), "test_package")
            .unwrap();
        let new_config = fs::read_to_string("./Cargo.toml").unwrap();
        assert!(new_config.contains("pkg_root"));

        cleanup_cargo_toml(); // Clean up after test
    }

    // !!! IMPORTANT NOTE !!!
    // THIS TEST FUNCTION IS USED TO CREATE A TEST GIT TASK RUST LIB.
    // IT IS ONLY INTENDED FOR TESTING IN THE LOCO-GEN CRATE TO MAKE SURE THAT THE PORCESSING AND RENDERING OF A GIT TASK WORKS WITHOUT THROWING ERRORS.
    // IT IS NOT INTENDED FOR TESTING IN A LOCO PROJECT.
    #[test]
    #[serial]
    fn create_and_render_git_test_task() {
        use crate::AppInfo;
        use git_task::process_repo;
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

        // Process the repo and render the test git task
        let _result = process_repo(&rrgen, git_url, &appinfo).unwrap();

        // DO THE CLEAN-UP AFTER TESTING
        cleanup("./src/tasks", "./tests/tasks", app_file_path);
    }

    // !!! IMPORTANT NOTE !!!
    // THIS TEST FUNCTION IS USED TO CLONE A TEST GIT TASK RUST LIB.
    // IT IS ONLY INTENDED FOR TESTING IN THE LOCO-GEN CRATE TO MAKE SURE THAT THE PORCESSING AND RENDERING OF A GIT TASK WORKS WITHOUT THROWING ERRORS.
    // IT IS NOT INTENDED FOR TESTING IN A LOCO PROJECT.
    #[test]
    #[serial]
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
        // create tasks directory for testing if it doesn't exist
        let tasks_dir = "./src/tasks";
        if !Path::new(tasks_dir).exists() {
            fs::create_dir(tasks_dir).unwrap();
        }
        // create test tasks directory for testing if it doesn't exist
        let test_tasks_dir = "./tests/tasks";
        if !Path::new(test_tasks_dir).exists() {
            fs::create_dir(test_tasks_dir).unwrap();
        }

        // create app.rs for testing
        let app_file_path = "./src/app.rs";
        let mut app_file = fs::File::create(app_file_path).unwrap();

        // create mod.rs file for testing
        let mod_file_path = "./src/tasks/mod.rs";
        let _mod_file = fs::File::create(mod_file_path).unwrap();

        // create test mod file for testing
        let test_mod_file_path = "./tests/tasks/mod.rs";
        let _test_mod_file = fs::File::create(test_mod_file_path).unwrap();

        let _ = writeln!(app_file, "// tasks-inject").unwrap();

        // FETCH AND GENERATE THE GIT TASK
        git_task::fetch_and_generate(&rrgen, Some(&test_git_url.to_owned()), &appinfo).unwrap();

        // DO THE CLEAN-UP AFTER TESTING
        cleanup("./src/tasks", "./tests/tasks", app_file_path);
    }

    fn cleanup(tasks_path: &str, test_tasks_path: &str, app_file_path: &str) {
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
        cleanup_cargo_toml();
    }

    fn cleanup_cargo_toml() {
        use std::fs;
        let mut cargo_toml = fs::read_to_string("Cargo.toml")
            .unwrap()
            .lines()
            .filter(|line| !line.contains("pkg_root"))
            .collect::<Vec<&str>>()
            .join("\n");
        cargo_toml = cargo_toml
            .lines()
            .filter(|line| !line.contains("git_test_task"))
            .collect::<Vec<&str>>()
            .join("\n");
        cargo_toml = cargo_toml
            .lines()
            .filter(|line| !line.contains("loco-git-task"))
            .collect::<Vec<&str>>()
            .join("\n");
        fs::write("Cargo.toml", cargo_toml).unwrap();
    }
}
