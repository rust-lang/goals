use std::process::Command;

use serde::{Deserialize, Serialize};

use super::issue_id::Repository;
use spanned::{Error, Result};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct GhLabel {
    pub name: String,
    pub color: String,
}

impl GhLabel {
    pub fn list(repository: &Repository) -> Result<Vec<GhLabel>> {
        let mut limit = 128;

        loop {
            let output = Command::new("gh")
                .arg("-R")
                .arg(repository.to_string())
                .arg("label")
                .arg("list")
                .arg("--json")
                .arg("name,color")
                .arg("-L")
                .arg(format!("{limit}"))
                .output()?;

            let labels: Vec<GhLabel> = serde_json::from_slice(&output.stdout)?;
            if labels.len() >= limit {
                // If we got exactly as many as we asked for,
                // we might be missing some.
                limit *= 2;
                continue;
            }

            return Ok(labels);
        }
    }

    pub fn create(&self, repository: &Repository) -> Result<()> {
        let output = Command::new("gh")
            .arg("-R")
            .arg(repository.to_string())
            .arg("label")
            .arg("create")
            .arg(&self.name)
            .arg("--color")
            .arg(&self.color)
            .arg("--force")
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(Error::str(format!(
                "failed to create label `{}`: {}",
                self.name,
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }
}
