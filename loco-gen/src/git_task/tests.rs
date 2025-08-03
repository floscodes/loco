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
    use super::remove_project_dep_from_cargo_toml;
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
    remove_project_dep_from_cargo_toml(config_file.to_string(), config_path).unwrap();
    let new_config = fs::read_to_string(config_path).unwrap();
    assert!(!new_config.contains("pkg_root"));
    fs::remove_file(config_path).unwrap(); // Clean up after test
}
