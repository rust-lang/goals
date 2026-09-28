//! Formatting for the new team support format (2026+).
//!
//! This module formats team support entries into tables grouped by team,
//! sorted by support level (Small → Medium → Large).

use std::collections::BTreeSet;
use std::path::PathBuf;

use spanned::{Result, Spanned};

use crate::{
    goal::{GoalDocument, Status, SupportLevel},
    team::TeamName,
    util,
};

/// Data needed to format a goal's support entry
struct GoalSupportData<'g> {
    goal_title: &'g str,
    subgoal_title: Option<&'g str>,
    link: &'g PathBuf,
    status: Status,
    support_level: SupportLevel,
    notes: &'g str,
    champion: Option<&'g str>,
}

impl<'g> GoalSupportData<'g> {
    fn goal_title_cell(&self) -> String {
        let title = self.subgoal_title.unwrap_or(self.goal_title);
        let title = self.status.decorate_title(title);
        util::goal_title_cell(
            &title,
            self.link,
            self.subgoal_title.map(|_| title.as_str()),
        )
    }
}

/// Format team support entries into tables, one per team.
///
/// Output looks like:
///
/// ```ignore
/// #### Compiler team
///
/// | Goal | Level | Champion | Notes |
/// | :--- | :---- | :------- | :---- |
/// | [Foo](foo.md) | Small | | |
/// | [Bar](bar.md) | Medium | @person | Need complex reviews |
/// | ↳ Subgoal | Medium | | Dedicated work |
/// ```
pub fn format_team_support(goals: &[&GoalDocument]) -> Result<String> {
    use std::fmt::Write;

    let mut output = String::new();

    // Collect all teams across all goals
    let all_teams: BTreeSet<&TeamName> = goals
        .iter()
        .flat_map(|g| g.team_involvement.teams())
        .collect();

    for team_name in all_teams {
        let team_data = team_name.data();
        write!(output, "\n#### {} team\n", team_data.name)?;

        let table_output = format_team_support_for_team(goals, team_name)?;
        write!(output, "{}", table_output)?;
    }

    Ok(output)
}

/// Format team support entries for a single team.
///
/// Output is just the table (no header), suitable for embedding in other contexts.
pub fn format_team_support_for_team(
    goals: &[&GoalDocument],
    team_name: &'static TeamName,
) -> Result<String> {
    use std::fmt::Write;

    let mut output = String::new();
    let mut footnotes = util::Footnotes::new();

    // Collect support entries for this team from all goals
    let mut entries: Vec<GoalSupportData> = vec![];

    for goal in goals {
        // Get champion for this team if any
        let champion = goal
            .metadata
            .champions
            .get(team_name)
            .map(|c| c.content.as_str());

        if let Some(supports) = goal.team_involvement.as_support() {
            for support in supports.iter().filter(|s| s.team == team_name) {
                entries.push(GoalSupportData {
                    goal_title: &goal.metadata.title,
                    subgoal_title: support.subgoal.as_ref().map(|s| s.content.as_str()),
                    link: &goal.link_path,
                    status: *goal.metadata.status,
                    support_level: support.support_level,
                    notes: &support.notes,
                    champion,
                });
            }
        }
    }

    // Sort by support level (Large first, then Medium, Small)
    // Within same level, maintain original order (by goal title implicitly)
    entries.sort_by_key(|e| std::cmp::Reverse(e.support_level));

    // Build the table
    let mut table: Vec<Vec<Spanned<String>>> = vec![vec![
        Spanned::here("Goal".to_string()),
        Spanned::here("Level".to_string()),
        Spanned::here("Champion".to_string()),
        Spanned::here("Notes".to_string()),
    ]];

    // Track which goals we've seen to avoid repeating goal title for subgoals
    let mut seen_goals: BTreeSet<&PathBuf> = BTreeSet::new();

    for entry in &entries {
        let goal_cell = if entry.subgoal_title.is_some() {
            // This is a subgoal row
            entry.goal_title_cell()
        } else if seen_goals.contains(entry.link) {
            // We've already shown this goal, skip the title
            String::new()
        } else {
            seen_goals.insert(entry.link);
            entry.goal_title_cell()
        };

        let champion_cell = entry.champion.unwrap_or("").to_string();

        let notes_cell = footnotes.maybe_footnote(entry.notes, entry.link);

        table.push(vec![
            Spanned::here(goal_cell),
            Spanned::here(entry.support_level.to_string()),
            Spanned::here(champion_cell),
            Spanned::here(notes_cell),
        ]);
    }

    write!(output, "{}", util::format_table(&table))?;

    footnotes.write_to(&mut output)?;

    Ok(output)
}
