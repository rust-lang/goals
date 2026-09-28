use std::collections::{BTreeMap, BTreeSet};

use spanned::{Result, Spanned};

use crate::{goal::GoalDocument, util};

/// Format a champions table showing each champion and their goals.
pub fn format_champions(goals: &[&GoalDocument]) -> Result<String> {
    use std::fmt::Write;

    let mut output = String::new();

    // Collect champions and their goals
    let mut champion_goals: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for goal in goals {
        for champion in goal.metadata.champions.values() {
            let champion_name = champion.content.clone();
            let goal_link = format!(
                "[{}]({})",
                goal.title_with_status(),
                goal.link_path.display()
            );

            champion_goals
                .entry(champion_name)
                .or_default()
                .insert(goal_link);
        }
    }

    if champion_goals.is_empty() {
        return Ok("No champions found.".to_string());
    }

    // Create the table with one row per goal,
    // showing champion name and count only on the first row.
    let table = {
        let headings = vec![
            Spanned::here("Champion".to_string()),
            Spanned::here("#".to_string()),
            Spanned::here("Goal".to_string()),
        ];

        let rows = champion_goals.into_iter().flat_map(|(champion, goals)| {
            let goals_vec: Vec<String> = goals.into_iter().collect();
            let count = goals_vec.len();
            goals_vec
                .into_iter()
                .enumerate()
                .map(move |(i, goal_link)| {
                    if i == 0 {
                        vec![
                            Spanned::here(champion.clone()),
                            Spanned::here(count.to_string()),
                            Spanned::here(goal_link),
                        ]
                    } else {
                        vec![
                            Spanned::here(String::new()),
                            Spanned::here(String::new()),
                            Spanned::here(goal_link),
                        ]
                    }
                })
        });

        std::iter::once(headings).chain(rows).collect::<Vec<_>>()
    };

    write!(output, "{}", util::format_table(&table))?;

    Ok(output)
}
