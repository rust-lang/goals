//! Generate a markdown summary for a team to review their goals.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::Path;

use regex::Regex;
use rust_project_goals::{
    goal::{GoalDocument, SupportLevel},
    spanned::Result,
    team::{get_team_name, TeamName},
    util::MILESTONE_REGEX,
};
use walkdir::WalkDir;

const BASE_URL: &str = "https://goals.rust-lang.org";

/// Generate a review summary for the given team.
pub fn review(team_name: &str, milestone: Option<&str>) -> Result<()> {
    // Validate team name
    let Some(team) = get_team_name(team_name)? else {
        rust_project_goals::spanned::bail_here!(
            "unknown team `{}`. Use a team name like `lang`, `compiler`, `cargo`, etc.",
            team_name
        );
    };

    // Find the milestone directory
    let milestone_path = if let Some(m) = milestone {
        let path = Path::new("src").join(m);
        if !path.exists() {
            rust_project_goals::spanned::bail_here!(
                "milestone directory `{}` does not exist",
                path.display()
            );
        }
        path
    } else {
        find_latest_milestone()?
    };

    let milestone_name = milestone_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Load all goals from the milestone
    let goals = rust_project_goals::goal::goals_in_dir(&milestone_path)?;
    let goals: Vec<&GoalDocument> = goals
        .iter()
        .filter(|g| g.metadata.status.content.is_not_not_accepted())
        .collect();

    // Filter to goals that involve this team
    let team_goals: Vec<&GoalDocument> = goals
        .iter()
        .filter(|g| goal_involves_team(g, team))
        .copied()
        .collect();

    if team_goals.is_empty() {
        println!(
            "# Goals review for {} team ({})\n",
            team_name, milestone_name
        );
        println!("No goals involve the {} team.", team_name);
        return Ok(());
    }

    // Generate the output
    let output = format_review(&team_goals, team, milestone_name)?;
    println!("{}", output);

    Ok(())
}

/// Check if a goal involves the given team (via asks or champions).
fn goal_involves_team(goal: &GoalDocument, team: &TeamName) -> bool {
    // Check if team has any asks
    if goal.team_involvement.teams().contains(team) {
        return true;
    }

    // Check if team has a champion
    if goal.metadata.champions.contains_key(team) {
        return true;
    }

    false
}

/// Find the latest milestone directory (e.g., "2026" or "2025h2").
fn find_latest_milestone() -> Result<std::path::PathBuf> {
    let regex = Regex::new(MILESTONE_REGEX)?;
    let mut latest: Option<(String, std::path::PathBuf)> = None;

    for entry in WalkDir::new("src").max_depth(1) {
        let entry = entry?;

        if !entry.file_type().is_dir() {
            continue;
        }

        let Some(name) = entry.file_name().to_str() else {
            continue;
        };

        if !regex.is_match(name) {
            continue;
        }

        // Simple lexicographic comparison works for our milestone format
        if latest.as_ref().map_or(true, |(n, _)| name > n.as_str()) {
            latest = Some((name.to_string(), entry.path().to_path_buf()));
        }
    }

    latest
        .map(|(_, path)| path)
        .ok_or_else(|| rust_project_goals::spanned::Error::str("no milestone directories found"))
}

/// Convert a link_path to an absolute URL.
fn goal_url(goal: &GoalDocument, milestone: &str) -> String {
    let path = goal.link_path.display().to_string();
    // Remove .md extension and use .html
    let path = path.trim_end_matches(".md");
    format!("{}/{}/{}.html", BASE_URL, milestone, path)
}

/// Format the review output.
fn format_review(
    goals: &[&GoalDocument],
    team: &'static TeamName,
    milestone: &str,
) -> Result<String> {
    let mut output = String::new();

    let team_data = team.data();
    writeln!(
        output,
        "# Goals review for {} team ({})\n",
        team_data.name, milestone
    )?;

    // Section 1: Roadmap themes affecting this team
    let roadmap_section = format_roadmap_themes(goals, team, milestone)?;
    if !roadmap_section.is_empty() {
        writeln!(output, "## Roadmaps\n")?;
        writeln!(
            output,
            "The following roadmaps include goals that involve the {} team:\n",
            team_data.name
        )?;
        write!(output, "{}", roadmap_section)?;
    }

    // Section 2: Summary table by support level
    writeln!(output, "\n## Summary by support level\n")?;
    write!(output, "{}", format_team_table(goals, team, milestone)?)?;

    // Section 3: Summary table by champion
    writeln!(output, "\n## Summary by champion\n")?;
    write!(
        output,
        "{}",
        format_by_champion_table(goals, team, milestone)?
    )?;

    // Section 4: Goal details with space for comments
    writeln!(output, "\n## Goal details\n")?;

    // Sort goals by support level (Large first)
    let mut sorted_goals: Vec<&GoalDocument> = goals.to_vec();
    sorted_goals.sort_by_key(|g| std::cmp::Reverse(get_team_support_level(g, team)));

    for goal in sorted_goals {
        // Help wanted marker
        let help_wanted = if goal.needs_contributor() || goal.metadata.is_help_wanted() {
            " ![Help Wanted][]"
        } else {
            ""
        };

        writeln!(
            output,
            "### [{}]({}){}\n",
            goal.metadata.title.content,
            goal_url(goal, milestone),
            help_wanted,
        )?;

        writeln!(output, "**Contact:** {}", goal.metadata.contacts)?;

        // Team champion for this team specifically
        let needs_champion = goal.team_involvement.as_support().map_or(false, |s| {
            s.iter().any(|s| {
                s.team == team
                    && matches!(s.support_level, SupportLevel::Large | SupportLevel::Medium)
            })
        });
        let champion = goal.metadata.champions.get(team);
        if let Some(champion) = champion {
            writeln!(output, "**Champion:** {}", champion.content)?;
        } else if needs_champion {
            writeln!(output, "**Champion:** {}", "TBD")?;
        };

        // Support level for this team
        if let Some(level) = get_team_support_level(goal, team) {
            writeln!(output, "**Support level:** {}", level)?;
        }

        // Notes for this team
        let notes = get_team_notes(goal, team);
        if !notes.is_empty() {
            writeln!(output, "**Notes:** {}", notes)?;
        }

        // Summary as blockquote
        for line in goal.summary.lines() {
            writeln!(output, "> {}", line)?;
        }
        writeln!(output)?;

        // Space for comments
        writeln!(output, "**Comments:**\n")?;
        writeln!(output, "(space for team discussion)\n")?;
    }

    Ok(output)
}

/// Format the team table with absolute URLs.
fn format_team_table(
    goals: &[&GoalDocument],
    team: &'static TeamName,
    milestone: &str,
) -> Result<String> {
    let mut output = String::new();

    // Collect entries
    let mut entries: Vec<(&GoalDocument, SupportLevel)> = goals
        .iter()
        .filter_map(|goal| get_team_support_level(goal, team).map(|level| (*goal, level)))
        .collect();

    // Sort by support level (Large first)
    entries.sort_by_key(|(_, level)| std::cmp::Reverse(*level));

    // Write the table
    writeln!(output, "| Goal | Level | Champion | Notes |")?;
    writeln!(output, "| :--- | :---- | :------- | :---- |")?;

    for (goal, level) in entries {
        let champion = goal
            .metadata
            .champions
            .get(team)
            .map(|c| c.content.as_str())
            .unwrap_or("");

        let notes = get_team_notes(goal, team);
        // Truncate long notes for the table
        let notes_display = if notes.len() > 30 {
            format!("{}...", &notes[..27])
        } else {
            notes
        };

        writeln!(
            output,
            "| [{}]({}) | {} | {} | {} |",
            goal.metadata.short_title.content,
            goal_url(goal, milestone),
            level,
            champion,
            notes_display,
        )?;
    }

    Ok(output)
}

/// Normalize a roadmap name to (display_name, slug).
/// Markdown links are already stripped at parse time by `parse_themed_rows`.
fn normalize_roadmap_name(raw: &str) -> (String, String) {
    let slug = raw.to_lowercase().replace(' ', "-");
    // Handle backticks in names like "Beyond the `&`"
    let slug = slug.replace('`', "").replace('&', "ampersand");
    (raw.to_string(), slug)
}

/// Format roadmap themes section.
fn format_roadmap_themes(
    goals: &[&GoalDocument],
    team: &'static TeamName,
    milestone: &str,
) -> Result<String> {
    let mut output = String::new();

    // Group goals by roadmap theme (normalized name)
    let mut by_roadmap: BTreeMap<(String, String), Vec<&GoalDocument>> = BTreeMap::new();

    for goal in goals {
        for roadmap in goal.all_roadmaps().iter() {
            let (name, slug) = normalize_roadmap_name(roadmap);
            by_roadmap.entry((name, slug)).or_default().push(*goal);
        }
    }

    if by_roadmap.is_empty() {
        return Ok(output);
    }

    // Load roadmap summaries from the filesystem
    let milestone_path = Path::new("src").join(milestone);

    for ((roadmap_name, slug), roadmap_goals) in &by_roadmap {
        let roadmap_file = milestone_path.join(format!("roadmap-{}.md", slug));

        // Try to read the roadmap file and extract its summary
        let summary = if roadmap_file.exists() {
            extract_summary_from_file(&roadmap_file).unwrap_or_default()
        } else {
            String::new()
        };

        let roadmap_url = format!("{}/{}/roadmap-{}.html", BASE_URL, milestone, slug);

        writeln!(
            output,
            "### [{}]({})\n",
            roadmap_name.replace('`', ""),
            roadmap_url
        )?;

        if !summary.is_empty() {
            for line in summary.lines() {
                writeln!(output, "> {}", line)?;
            }
            writeln!(output)?;
        }

        // List goals in this roadmap that involve this team
        writeln!(output, "**Goals in this theme:**\n")?;
        for goal in roadmap_goals {
            let level = get_team_support_level(goal, team)
                .map(|l| format!(" ({})", l))
                .unwrap_or_default();
            writeln!(
                output,
                "- [{}]({}){}",
                goal.metadata.short_title.content,
                goal_url(goal, milestone),
                level,
            )?;
        }
        writeln!(output)?;
    }

    Ok(output)
}

/// Extract the Summary section from a markdown file.
fn extract_summary_from_file(path: &Path) -> Option<String> {
    let sections = rust_project_goals::markwaydown::parse(path).ok()?;
    rust_project_goals::goal::extract_summary(&sections)
        .ok()
        .flatten()
}

/// Format a table grouped by champion.
fn format_by_champion_table(
    goals: &[&GoalDocument],
    team: &'static TeamName,
    milestone: &str,
) -> Result<String> {
    use std::collections::BTreeMap;

    let mut output = String::new();

    // Group goals by champion
    let mut by_champion: BTreeMap<String, Vec<(&GoalDocument, SupportLevel)>> = BTreeMap::new();

    for goal in goals {
        if let Some(level) = get_team_support_level(goal, team) {
            let champion = goal
                .metadata
                .champions
                .get(team)
                .map(|c| c.content.clone())
                .unwrap_or_else(|| "(no champion)".to_string());

            by_champion
                .entry(champion)
                .or_default()
                .push((*goal, level));
        }
    }

    // Sort each champion's goals by support level
    for goals in by_champion.values_mut() {
        goals.sort_by_key(|(_, level)| std::cmp::Reverse(*level));
    }

    // Write the table
    writeln!(output, "| Champion | Goal | Level |")?;
    writeln!(output, "| :------- | :--- | :---- |")?;

    // Sort champions: "(no champion)" last, then alphabetically
    let mut champions: Vec<_> = by_champion.keys().collect();
    champions.sort_by(|a, b| match (a.as_str(), b.as_str()) {
        ("(no champion)", _) => std::cmp::Ordering::Greater,
        (_, "(no champion)") => std::cmp::Ordering::Less,
        _ => a.cmp(b),
    });

    for champion in champions {
        let goals = &by_champion[champion];
        for (i, (goal, level)) in goals.iter().enumerate() {
            // Only show champion name on first row for that champion
            let champion_cell = if i == 0 { champion.as_str() } else { "" };

            writeln!(
                output,
                "| {} | [{}]({}) | {} |",
                champion_cell,
                goal.metadata.short_title.content,
                goal_url(goal, milestone),
                level,
            )?;
        }
    }

    Ok(output)
}

/// Get the support level for a specific team from a goal.
fn get_team_support_level(goal: &GoalDocument, team: &TeamName) -> Option<SupportLevel> {
    if let Some(supports) = goal.team_involvement.as_support() {
        supports
            .iter()
            .filter(|s| s.team == team)
            .map(|s| s.support_level)
            .max()
    } else {
        None
    }
}

/// Get notes for a specific team from a goal.
fn get_team_notes(goal: &GoalDocument, team: &TeamName) -> String {
    if let Some(supports) = goal.team_involvement.as_support() {
        supports
            .iter()
            .filter(|s| s.team == team)
            .map(|s| s.notes.as_str())
            .filter(|n| !n.is_empty())
            .collect::<Vec<_>>()
            .join("; ")
    } else {
        String::new()
    }
}
