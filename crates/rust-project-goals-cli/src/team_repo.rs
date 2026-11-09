use std::collections::BTreeSet;
use std::fmt::Write;
use std::process::Command;

use rust_project_goals::{
    goal,
    spanned::{self, Context as _, Result},
    team,
};

pub(crate) fn generate_team_repo(
    paths: &[std::path::PathBuf],
    team_repo_path: &std::path::PathBuf,
) -> Result<()> {
    if !team_repo_path.is_dir() {
        spanned::bail_here!(
            "output path not a directory: `{}`",
            team_repo_path.display()
        );
    }

    let mut goal_documents = vec![];
    for path in paths {
        goal_documents.extend(goal::goals_in_dir(path)?);
    }

    let owners: BTreeSet<&str> = goal_documents
        .iter()
        .flat_map(|doc| doc.metadata.owner_usernames())
        .collect();

    progress_bar::init_progress_bar(owners.len() + 1);

    progress_bar::set_progress_bar_action(
        "Team file",
        progress_bar::Color::Blue,
        progress_bar::Style::Bold,
    );
    let team_file = team_file(&owners)?;
    let team_toml_file = team_repo_path.join("teams").join("goal-owners.toml");
    std::fs::write(&team_toml_file, team_file)
        .with_path_context(&team_toml_file, "writing team toml file")?;
    progress_bar::inc_progress_bar();

    // generate rudimentary people files if needed
    progress_bar::set_progress_bar_action(
        "People",
        progress_bar::Color::Blue,
        progress_bar::Style::Bold,
    );
    for owner in owners {
        ensure_person_file(owner, team_repo_path)?;
        progress_bar::inc_progress_bar();
    }

    progress_bar::finalize_progress_bar();

    Ok(())
}

fn ensure_person_file(owner: &str, team_repo_path: &std::path::PathBuf) -> Result<()> {
    let person_toml_file = team_repo_path
        .join("people")
        .join(&owner[1..])
        .with_extension("toml");

    if person_toml_file.exists() {
        return Ok(());
    }

    if team::get_person_data(owner)?.is_some() {
        return Ok(()); // person already exists
    }

    let status = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("--")
        .arg("add-person")
        .arg(&owner[1..])
        .current_dir(team_repo_path)
        .status()
        .with_str_context(format!("running `cargo run add-person` for {owner}"))?;

    if !status.success() {
        spanned::bail_here!("`cargo run add-person` failed for {owner}");
    }

    Ok(())
}

fn team_file(owners: &BTreeSet<&str>) -> Result<String> {
    let mut out = String::new();
    writeln!(out, "# Auto-generated from the goals repository")?;
    writeln!(out, "name = \"goal-owners\"")?;
    writeln!(out, "kind = \"marker-team\"")?;
    writeln!(out)?;
    writeln!(out, "[people]")?;
    writeln!(out, "leads = []")?;
    writeln!(out, "members = [")?;
    for owner in owners {
        match team::get_person_data(owner)? {
            Some(p) => writeln!(out, "    \"{}\",", p.github_username)?,
            None => writeln!(out, "    \"{}\",", &owner[1..])?,
        }
    }
    writeln!(out, "]")?;
    writeln!(out, "included-teams = []")?;
    writeln!(out)?;
    writeln!(out, "[[github]]")?;
    writeln!(out, "orgs = [\"rust-lang\"]")?;
    Ok(out)
}
