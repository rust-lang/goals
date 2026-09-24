use chrono::{Datelike, NaiveDate};
use regex::Regex;
use rust_project_goals::re::{HELP_WANTED, TLDR};
use rust_project_goals::spanned::Result;
use rust_project_goals::util::{comma, MILESTONE_REGEX};
use rust_project_goals::{goal, spanned, team};
use rust_project_goals_json::GithubIssueState;
use std::path::PathBuf;

mod templates;
use rust_project_goals::gh::issues::ExistingGithubIssue;
use rust_project_goals::gh::{
    issue_id::{IssueId, Repository},
    issues::{checkboxes, ExistingGithubComment},
};
use templates::{HelpWanted, UpdatesGoal};

#[derive(Copy, Clone, Default)]
/// Order in which GitHub comments for each goal are displayed.
pub enum Order {
    #[default]
    /// Chronological order: the oldest comments show up first.
    /// Mirrors the order on the corresponding GitHub issue.
    OldestFirst,

    /// Reverse chronological order: the most recent comments will show up first.
    #[allow(unused)]
    NewestFirst,
}

/// Library function that renders updates as a string without side effects.
/// This is suitable for use from the mdbook preprocessor.
pub fn render_updates(
    cached_issues: &[ExistingGithubIssue],
    repository: &Repository,
    milestone: &str,
    start_date: Option<&NaiveDate>,
    end_date: Option<&NaiveDate>,
    with_champion_from: Option<&str>,
    use_progress_bar: bool,
    comment_order: Order,
) -> Result<String> {
    let milestone_re = Regex::new(MILESTONE_REGEX).unwrap();
    if !milestone_re.is_match(milestone) {
        spanned::bail_here!(
            "the milestone `{}` does not follow the `$year$semester` format, where $semester is `h1` or `h2`",
            milestone,
        );
    }

    let issues = cached_issues;

    // Load goal documents to extract theme information and optionally filter by team
    let mut milestone_path = PathBuf::from("src");
    milestone_path.push(milestone);
    let goal_documents = goal::goals_in_dir(&milestone_path)?;

    // Create a mapping from issue numbers to themes for roadmap goals
    let issue_themes: std::collections::HashMap<u64, Vec<String>> = {
        let mut map: std::collections::HashMap<u64, Vec<String>> = std::collections::HashMap::new();
        for doc in &goal_documents {
            if let Some(tracking_issue) = doc.metadata.tracking_issue.as_ref() {
                for theme in doc.all_roadmaps().iter() {
                    map.entry(tracking_issue.number)
                        .or_default()
                        .push(theme.trim().to_string());
                }
            }
        }
        map
    };

    // Create mappings for ownership information
    let issue_contact: std::collections::HashMap<u64, String> = goal_documents
        .iter()
        .filter_map(|doc| {
            doc.metadata
                .tracking_issue
                .as_ref()
                .map(|issue| (issue.number, doc.contact_for_goal_list()))
        })
        .collect();

    let issue_team_champions: std::collections::HashMap<u64, String> = goal_documents
        .iter()
        .filter_map(|doc| {
            doc.metadata.tracking_issue.as_ref().map(|issue| {
                let teams = doc.team_involvement.teams();

                let team_champions: Vec<String> = teams
                    .into_iter()
                    .filter_map(|team| {
                        doc.metadata
                            .champions
                            .get(team)
                            .map(|champion| format!("{} ({})", team, champion.content))
                    })
                    .collect();

                let team_champions_str = if team_champions.is_empty() {
                    "(none)".to_string()
                } else {
                    team_champions.join(", ")
                };

                (issue.number, team_champions_str)
            })
        })
        .collect();

    let issue_task_owners: std::collections::HashMap<u64, String> = goal_documents
        .iter()
        .filter_map(|doc| {
            doc.metadata.tracking_issue.as_ref().map(|issue| {
                let task_owners_str = if doc.task_owners.is_empty() {
                    "(none)".to_string()
                } else {
                    doc.task_owners
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                (issue.number, task_owners_str)
            })
        })
        .collect();

    // Filter issues by champion team if specified
    let filtered_issues: Vec<ExistingGithubIssue> = if let Some(team_name) = with_champion_from {
        let team_name = team::get_team_name(team_name)?
            .ok_or_else(|| spanned::Error::str(format!("unknown team: {}", team_name)))?;

        // Create a set of issue numbers for goals that have a champion from the specified team
        let champion_issue_numbers: std::collections::HashSet<u64> = goal_documents
            .iter()
            .filter(|doc| doc.metadata.champions.contains_key(team_name))
            .filter_map(|doc| {
                doc.metadata
                    .tracking_issue
                    .as_ref()
                    .map(|issue| issue.number)
            })
            .collect();

        issues
            .iter()
            .filter(|issue| champion_issue_numbers.contains(&issue.number))
            .cloned()
            .collect()
    } else {
        issues.to_vec()
    };

    let filter = Filter {
        start_date: match start_date {
            Some(date) => date.clone(),
            None => default_start_date(),
        },
        end_date,
    };

    if use_progress_bar {
        progress_bar::init_progress_bar(filtered_issues.len());
        progress_bar::set_progress_bar_action(
            "Executing",
            progress_bar::Color::Blue,
            progress_bar::Style::Bold,
        );
    }

    let roadmap_goals = prepare_goals(
        repository,
        &filtered_issues,
        &filter,
        true,
        use_progress_bar,
        comment_order,
        &issue_themes,
        &issue_contact,
        &issue_team_champions,
        &issue_task_owners,
    )?;
    let other_goals = prepare_goals(
        repository,
        &filtered_issues,
        &filter,
        false,
        use_progress_bar,
        comment_order,
        &issue_themes,
        &issue_contact,
        &issue_team_champions,
        &issue_task_owners,
    )?;
    let updates = templates::Updates::new(milestone.to_string(), roadmap_goals, other_goals);

    if use_progress_bar {
        progress_bar::finalize_progress_bar();
    }

    // Render the output using handlebars and return it
    updates.render()
}

fn prepare_goals(
    repository: &Repository,
    issues: &[ExistingGithubIssue],
    filter: &Filter<'_>,
    roadmap: bool,
    use_progress_bar: bool,
    comment_order: Order,
    issue_themes: &std::collections::HashMap<u64, Vec<String>>,
    issue_contact: &std::collections::HashMap<u64, String>,
    issue_team_champions: &std::collections::HashMap<u64, String>,
    issue_task_owners: &std::collections::HashMap<u64, String>,
) -> Result<Vec<UpdatesGoal>> {
    let mut result = vec![];
    // We process roadmap and regular goals in two passes, and capture comments differently for roadmap goals.
    for issue in issues {
        if roadmap != issue.has_roadmap_label() {
            continue;
        }

        let issue_id = IssueId {
            repository: repository.clone(),
            number: issue.number,
        };

        let title = &issue.title;

        if use_progress_bar {
            progress_bar::print_progress_bar_info(
                &format!("Issue #{number}", number = issue.number),
                title,
                progress_bar::Color::Green,
                progress_bar::Style::Bold,
            );
        }

        let progress = checkboxes(&issue);

        let mut comments = issue.comments.clone();
        comments.sort_by_key(|c| c.created_at.clone());
        comments.retain(|c| !c.should_hide_from_reports() && filter.matches(c));

        // We got the comments in the chronological order. Reverse it if desired.
        if matches!(comment_order, Order::NewestFirst) {
            comments.reverse();
        }

        // Prettify the comments' timestamp after using it for sorting.
        for comment in comments.iter_mut() {
            comment.created_at = format!("{}", comment.created_at_date());
        }

        let tldr = tldr(&issue_id, &mut comments)?;

        let (has_help_wanted, help_wanted) = help_wanted(&issue_id, tldr.as_deref(), &comments)?;

        let details_summary = match comments.len() {
            0 => String::from("No detailed updates available."),
            1 => String::from("1 detailed update available."),
            len => format!("{len} detailed updates available."),
        };
        result.push(UpdatesGoal {
            title: title.clone(),
            issue_number: issue.number,
            issue_assignees: comma(&issue.assignees),
            issue_url: issue_id.url(),
            issue_link_text: format!("rust-lang/goals#{}", issue.number),
            progress,
            has_help_wanted,
            help_wanted,
            is_closed: issue.state == GithubIssueState::Closed,
            details_summary,
            comments,
            tldr,
            needs_separator: true, // updated after sorting
            theme: issue_themes.get(&issue.number).cloned().unwrap_or_default(),
            contact: issue_contact
                .get(&issue.number)
                .cloned()
                .unwrap_or_else(|| "(unknown)".to_string()),
            team_champions: issue_team_champions
                .get(&issue.number)
                .cloned()
                .unwrap_or_else(|| "(none)".to_string()),
            task_owners: issue_task_owners
                .get(&issue.number)
                .cloned()
                .unwrap_or_else(|| "(none)".to_string()),
        });

        if use_progress_bar {
            progress_bar::inc_progress_bar();
        }
    }

    // Updates are in a random order, sort them.
    result.sort_by_cached_key(|update| update.title.to_lowercase());

    // Mark the last entry as not needing a separator from its following sibling, it has none.
    if let Some(last) = result.last_mut() {
        last.needs_separator = false;
    }

    Ok(result)
}

/// Search for a TL;DR comment. If one is found, remove it and return the text.
fn tldr(_issue_id: &IssueId, comments: &mut Vec<ExistingGithubComment>) -> Result<Option<String>> {
    // `comments` are sorted by creation date in an ascending order, so we look for the most recent
    // TL;DR comment from the end.
    let Some(index) = comments.iter().rposition(|c| c.body.starts_with(TLDR)) else {
        return Ok(None);
    };

    let comment = comments.remove(index);
    Ok(Some(comment.body[TLDR.len()..].trim().to_string()))
}

/// Search for comments that talk about help being wanted and extract that
fn help_wanted(
    _issue_id: &IssueId,
    tldr: Option<&str>,
    comments: &[ExistingGithubComment],
) -> Result<(bool, Vec<HelpWanted>)> {
    use std::fmt::Write;

    let mut help_wanted = vec![];

    let tldr_has_help_wanted = tldr
        .unwrap_or("")
        .lines()
        .any(|line| HELP_WANTED.is_match(line));

    for comment in comments {
        let mut lines = comment.body.split('\n').peekable();

        // Look for a line that says "Help wanted" at the front.
        // Then extract the rest of that line along with subsequent lines until we find a blank line.
        while lines.peek().is_some() {
            while let Some(line) = lines.next() {
                if let Some(c) = HELP_WANTED.captures(line) {
                    let text = c["text"].trim().to_string();
                    if !text.is_empty() {
                        help_wanted.push(HelpWanted { text });
                        break;
                    }
                }
            }

            while let Some(line) = lines.next() {
                if line.trim().is_empty() {
                    break;
                } else {
                    let last = help_wanted.len() - 1;
                    writeln!(&mut help_wanted[last].text, "{line}")?;
                }
            }
        }
    }

    Ok((tldr_has_help_wanted || !help_wanted.is_empty(), help_wanted))
}

struct Filter<'f> {
    start_date: NaiveDate,
    end_date: Option<&'f NaiveDate>,
}

impl Filter<'_> {
    fn matches(&self, comment: &ExistingGithubComment) -> bool {
        let date = comment.created_at_date();

        date >= self.start_date
            && match self.end_date {
                Some(end_date) => date < *end_date,
                None => true,
            }
    }
}

fn default_start_date() -> NaiveDate {
    let date = chrono::Utc::now().date_naive();
    let start_of_month = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap();
    start_of_month - chrono::Duration::days(7)
}
