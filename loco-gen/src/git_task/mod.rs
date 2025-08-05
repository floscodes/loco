use super::render_template;
use crate::{AppInfo, Error, GenerateResults, Result};
use rrgen::{self, RRgen};
use std::path::Path;

mod tests;
mod utils;

use utils::*;

pub(self) const CARGO_TOML: &str = "Cargo.toml";

pub fn fetch_and_generate(
    rrgen: &RRgen,
    git_url: Option<&String>,
    appinfo: &AppInfo,
) -> Result<GenerateResults> {
    if let Some(git_url) = git_url {
        println!("Cloning task from git repository: {}", git_url);
        let _repo = clone_repo(git_url, Path::new("./src/tasks"))?;
        println!("Processing the cloned repository");
        process_repo(rrgen, git_url, appinfo)
            .map_err(|e| Error::Message(format!("Failed to process git repository: {}", e)))
    } else {
        Err(Error::Message(
            "Error while trying to generate task from git - no valid git url provided!".to_string(),
        ))
    }
}
