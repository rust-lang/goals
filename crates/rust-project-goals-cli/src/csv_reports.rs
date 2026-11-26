use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use rust_project_goals::{
    gh::issue_id::Repository,
    goal,
    spanned::{Result, Spanned},
    team::TeamName,
};

use crate::CsvReports;

pub fn csv(repository: &Repository, cmd: &CsvReports) -> Result<()> {
    match cmd {
        CsvReports::Champions { milestone } => champions(repository, milestone)?,
    }
    Ok(())
}

struct ChampionRow {
    title: String,
    url: String,
    contacts: String,
    champions: BTreeMap<&'static TeamName, Spanned<String>>,
    teams_with_asks: BTreeSet<&'static TeamName>,
}

fn champions(repository: &Repository, milestone: &str) -> Result<()> {
    let mut milestone_path = PathBuf::from("src");
    milestone_path.push(milestone);

    let goal_documents = goal::goals_in_dir(&milestone_path)?;

    let all_teams: BTreeSet<&TeamName> = goal_documents
        .iter()
        .flat_map(|d| d.teams_with_asks())
        .collect();

    let rows: Vec<ChampionRow> = goal_documents
        .iter()
        .map(|doc| ChampionRow {
            title: doc.metadata.title.to_string(),
            url: format!(
                "https://github.com/{org}/{repo}/blob/main/{path}",
                org = repository.org,
                repo = repository.repo,
                path = doc.path.display()
            ),
            contacts: doc.metadata.contacts.clone(),
            champions: doc.metadata.champions.clone(),
            teams_with_asks: doc.teams_with_asks(),
        })
        .collect();

    // Write header row
    write_csv_row(|cell| {
        cell.write_cell("Title");
        cell.write_cell("Contact(s)");
        for team in &all_teams {
            cell.write_cell(&format!("{team}"));
        }
        cell.write_cell("URL");
    });

    // Write data rows
    for row in &rows {
        write_csv_row(|cell| {
            cell.write_cell(&row.title);
            cell.write_cell(&row.contacts);

            for team in &all_teams {
                if row.teams_with_asks.contains(team) {
                    // Team has an ask - check if there's a champion
                    if let Some(champion) = row.champions.get(team) {
                        cell.write_cell(champion);
                    } else {
                        cell.write_cell("!");
                    }
                } else {
                    // Team has no ask for this goal
                    cell.write_cell("-");
                }
            }

            cell.write_cell(&row.url);
        });
    }

    Ok(())
}

trait WriteCell {
    fn write_cell(&mut self, s: &str);
}

impl WriteCell for String {
    fn write_cell(&mut self, s: &str) {
        let mut s = s.replace(r#"""#, r#"\""#);
        s = s.replace("\n", "\\n");

        if !self.is_empty() {
            self.push(',');
        }
        self.push('"');
        self.push_str(&s);
        self.push('"');
    }
}

fn write_csv_row(op: impl FnOnce(&mut dyn WriteCell)) {
    let mut s = String::new();
    op(&mut s);
    println!("{s}");
}
