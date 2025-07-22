use super::render_template;
use crate::{Error, GenerateResults, Result};
use git2;
use rrgen::{self, GenResult, RRgen};
use std::path::PathBuf;

pub fn fetch(rrgen: &RRgen, git_url: Option<&String>) -> Result<GenerateResults> {
    if let Some(git_url) = git_url {
        let repo = git2::Repository::clone(git_url, "./task")
            .map_err(|e| Error::Message(format!("Failed to clone git repository: {}", e)))?;
    }
    let name = String::from("task");
    Ok(GenerateResults {
        rrgen: vec![rrgen::GenResult::Skipped],
        local_templates: vec![PathBuf::from(name)],
    })
}
