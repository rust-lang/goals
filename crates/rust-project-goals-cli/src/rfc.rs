use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use regex::Regex;

use rust_project_goals::{
    gh::{
        issue_id::{IssueId, Repository},
        issues::{
            change_milestone, change_title, create_comment, create_issue, fetch_issue,
            list_issues_in_milestone, lock_issue, sync_assignees, sync_labels, update_issue_body,
            CONTINUING_GOAL_PREFIX, LOCK_TEXT, ROADMAP_LABEL,
        },
        labels::GhLabel,
    },
    goal::{self, GoalDocument, GoalPlan, ParsedOwners},
    spanned::{self, Context, Error, Result, Spanned},
    team::{get_person_data, TeamName},
};

fn validate_path(path: &Path) -> Result<String> {
    if !path.is_dir() {
        spanned::bail_here!("RFC path should be a directory like src/2024h2");
    };

    if path.is_absolute() {
        spanned::bail_here!("RFC path should be relative");
    }

    let timeframe = path
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .ok_or_else(|| Error::str(format!("invalid path `{}`", path.display())))?;

    Ok(timeframe.to_string())
}

pub fn generate_comment(path: &Path) -> Result<()> {
    let _ = validate_path(path)?;
    let goal_documents = goal::goals_in_dir(path)?;
    let teams_with_asks = teams_with_asks(&goal_documents);

    for team_name in teams_with_asks {
        let team_data = team_name.data();

        println!("\n## {}\n", team_data.name);

        let (leads, members): (Vec<_>, Vec<_>) = team_data.members.iter().partition(|m| m.is_lead);

        for lead in leads {
            println!("* [ ] @{} (required, lead)", lead.github);
        }

        for member in members {
            println!("* [ ] {} (optional)", member.github);
        }
    }

    Ok(())
}

/// Split markdown content into body text and reference link definitions.
/// Reference links are lines matching `[label]: URL`.
fn separate_reference_links(text: &str) -> (String, Vec<String>) {
    let ref_link_re = Regex::new(r"^\[([^\]]+)\]:\s+\S").unwrap();
    let mut body_lines = Vec::new();
    let mut ref_links = Vec::new();

    for line in text.lines() {
        if ref_link_re.is_match(line) {
            ref_links.push(line.to_string());
        } else {
            body_lines.push(line);
        }
    }

    // Trim trailing blank lines from body
    while body_lines.last().map_or(false, |l| l.trim().is_empty()) {
        body_lines.pop();
    }

    (body_lines.join("\n"), ref_links)
}

/// Rewrite `.md` links to GitHub Pages URLs.
fn rewrite_md_links(text: &str, timeframe: &str) -> String {
    let link_re = Regex::new(r"\]\((\./)?([^)#]*?)\.md(#[^)]*)?\)").unwrap();

    link_re
        .replace_all(text, |caps: &regex::Captures| {
            let path_stem = &caps[2];
            let fragment = caps.get(3).map_or("", |m| m.as_str());

            let clean_path = path_stem.trim_start_matches("../");
            if clean_path.contains('/') || path_stem.starts_with("../") {
                format!("](https://goals.rust-lang.org/{clean_path}.html{fragment})")
            } else {
                format!("](https://goals.rust-lang.org/{timeframe}/{clean_path}.html{fragment})")
            }
        })
        .to_string()
}

/// Rewrite .md URLs inside reference link definitions to GitHub Pages URLs.
fn rewrite_ref_link_urls(ref_links: &mut BTreeMap<String, String>, timeframe: &str) {
    let md_url_re = Regex::new(r"^(\[[^\]]+\]:\s+)(\./)?(\S*?)\.md(\s.*)?$").unwrap();
    for def in ref_links.values_mut() {
        let def_clone = def.clone();
        if let Some(caps) = md_url_re.captures(&def_clone) {
            let prefix = &caps[1];
            let path_stem = &caps[3];
            let rest = caps.get(4).map_or("", |m| m.as_str());
            let clean_path = path_stem.trim_start_matches("../");
            if clean_path.contains('/') || path_stem.starts_with("../") {
                *def = format!("{prefix}https://goals.rust-lang.org/{clean_path}.html{rest}");
            } else {
                *def = format!(
                    "{prefix}https://goals.rust-lang.org/{timeframe}/{clean_path}.html{rest}"
                );
            }
        }
    }
}

/// Inline a rendered markdown file: strip the top heading and adjust all
/// remaining heading levels relative to the insertion context.
///
/// `context_level` is the heading level at the point of insertion in the
/// parent document. Headings in the inlined file are shifted so that the
/// file's top heading level maps to `context_level + 1`.
fn inline_rendered_file(text: &str, context_level: usize) -> String {
    let heading_re = Regex::new(r"^(#+)\s").unwrap();
    let mut lines = text.lines();

    // Find and consume the top-level heading, recording its level
    let mut top_level = None;
    for line in &mut lines {
        if let Some(caps) = heading_re.captures(line) {
            top_level = Some(caps[1].len());
            break;
        }
    }

    let top_level = top_level.unwrap_or(1);
    // delta: how much to shift remaining headings
    // e.g. if top heading was # (1) and context is # (1), delta = 1 - 1 = 0
    // e.g. if top heading was # (1) and context is ## (2), delta = 2 - 1 = 1
    let delta: isize = context_level as isize - top_level as isize;

    let mut result = Vec::new();
    for line in lines {
        if let Some(caps) = heading_re.captures(line) {
            let old_level = caps[1].len() as isize;
            let new_level = (old_level + delta).max(1) as usize;
            let rest = &line[caps[1].len()..];
            result.push(format!("{}{}", "#".repeat(new_level), rest));
        } else {
            result.push(line.to_string());
        }
    }

    // Trim leading blank lines
    while result.first().map_or(false, |l| l.trim().is_empty()) {
        result.remove(0);
    }

    result.join("\n")
}

pub fn generate_rfc(path: &Path) -> Result<()> {
    let timeframe = &validate_path(path)?;

    // Run mdbook build to expand (((directives)))
    Command::new("mdbook").arg("build").status()?;

    let book_dir = PathBuf::from("book/markdown").join(timeframe);
    if !book_dir.exists() {
        spanned::bail_here!("no markdown generated at {}", book_dir.display());
    }

    // Read the rendered README (index.md) as our skeleton
    let index_path = book_dir.join("index.md");
    let index_text = std::fs::read_to_string(&index_path)
        .with_path_context(&index_path, "reading rendered index.md")?;

    // Separate reference links from the index
    let (index_body, index_ref_links) = separate_reference_links(&index_text);

    // Collect all reference link definitions for deduplication
    let ref_label_re = Regex::new(r"^\[([^\]]+)\]:\s").unwrap();
    let mut all_ref_links: BTreeMap<String, String> = BTreeMap::new();
    for ref_link in &index_ref_links {
        if let Some(caps) = ref_label_re.captures(ref_link) {
            let label = caps[1].to_string();
            all_ref_links
                .entry(label)
                .or_insert_with(|| ref_link.clone());
        }
    }

    // Pattern for link-list lines: `- [Text](./relative/path.md)`
    // Captures the relative path (e.g., `./highlights.md` or `./2026/highlights.md`)
    let link_line_re = Regex::new(r"^- \[[^\]]+\]\((\./[^)]+\.md)\)\s*$").unwrap();
    let heading_re = Regex::new(r"^(#+)\s").unwrap();

    // Process the index body line by line, inlining linked files
    let mut current_heading_level: usize = 1;
    let mut output = String::new();
    for line in index_body.lines() {
        // Track the current heading level
        if let Some(caps) = heading_re.captures(line) {
            current_heading_level = caps[1].len();
        }

        if let Some(caps) = link_line_re.captures(line) {
            let rel_path = Path::new(&caps[1]);
            let chapter_path = book_dir.join(rel_path);
            if !chapter_path.exists() {
                eprintln!(
                    "warning: linked file not found: {}, skipping",
                    chapter_path.display()
                );
                continue;
            }

            let chapter_text = std::fs::read_to_string(&chapter_path)
                .with_path_context(&chapter_path, "reading linked file")?;

            // Collect reference links from the chapter
            let (chapter_body, chapter_ref_links) = separate_reference_links(&chapter_text);
            for ref_link in &chapter_ref_links {
                if let Some(caps) = ref_label_re.captures(ref_link) {
                    let label = caps[1].to_string();
                    all_ref_links
                        .entry(label)
                        .or_insert_with(|| ref_link.clone());
                }
            }

            // Inline: strip top heading, adjust remaining headings relative to context
            let inlined = inline_rendered_file(&chapter_body, current_heading_level);
            output.push_str(&inlined);
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    // Rewrite .md links to GitHub Pages URLs
    let rewritten = rewrite_md_links(&output, timeframe);

    // Rewrite .md URLs inside reference link definitions
    rewrite_ref_link_urls(&mut all_ref_links, timeframe);

    // Assemble final output: body + reference links
    let mut final_output = rewritten;
    if !final_output.ends_with('\n') {
        final_output.push('\n');
    }
    final_output.push('\n');
    for ref_link in all_ref_links.values() {
        final_output.push_str(ref_link);
        final_output.push('\n');
    }

    print!("{final_output}");

    Ok(())
}

pub fn generate_issues(
    repository: &Repository,
    path: &Path,
    commit: bool,
    sleep: u64,
) -> Result<()> {
    // Verify the `gh` client is installed to compute which actions need to be taken in the repo.
    let sanity_check = Command::new("gh").arg("--version").output();
    if sanity_check.is_err() {
        spanned::bail_here!(
            "The github `gh` client is missing and needs to be installed and configured with a token."
        );
    }

    // Hacky but works: we loop because after creating the issue, we sometimes have additional sync to do,
    // and it's easier this way.
    let mut iteration_count = 0;
    const MAX_ITERATIONS: usize = 10;

    loop {
        iteration_count += 1;
        if iteration_count > MAX_ITERATIONS {
            spanned::bail_here!(
                "Fixed point iteration failed to converge after {} iterations. \
                 This may indicate duplicate tracking issues assigned to multiple goals.",
                MAX_ITERATIONS
            );
        }
        let timeframe = validate_path(path)?;

        let mut goal_documents = goal::goals_in_dir(path)?;
        goal_documents.retain(|gd| gd.is_not_not_accepted());

        let teams_with_asks = teams_with_asks(&goal_documents);
        let mut actions = initialize_labels(repository, &teams_with_asks)?;
        actions.extend(initialize_issues(repository, &timeframe, &goal_documents)?);

        if actions.is_empty() {
            return Ok(());
        }

        if commit {
            progress_bar::init_progress_bar(actions.len());
            progress_bar::set_progress_bar_action(
                "Executing",
                progress_bar::Color::Blue,
                progress_bar::Style::Bold,
            );
            let mut success = 0;
            for action in actions.into_iter() {
                progress_bar::print_progress_bar_info(
                    "Action",
                    &format!("{}", action),
                    progress_bar::Color::Green,
                    progress_bar::Style::Bold,
                );
                if let Err(e) = action.execute(repository, &timeframe) {
                    progress_bar::print_progress_bar_info(
                        "Error",
                        &format!("{}", e),
                        progress_bar::Color::Red,
                        progress_bar::Style::Bold,
                    );
                } else {
                    success += 1;
                }
                progress_bar::inc_progress_bar();

                std::thread::sleep(Duration::from_millis(sleep));
            }
            progress_bar::finalize_progress_bar();
            if success == 0 {
                spanned::bail_here!("all actions failed, aborting")
            }

            // Clear the cached issues since we just modified them
            if let Err(e) = rust_project_goals::gh::issues::clear_milestone_issues_cache(&timeframe)
            {
                eprintln!("Warning: Failed to clear issues cache: {}", e);
            }
        } else {
            eprintln!("Actions to be executed:");
            for action in &actions {
                eprintln!("* {action}");
            }
            eprintln!("");
            eprintln!("Use `--commit` to execute the actions.");
            return Ok(());
        }

        eprintln!("Waiting for github commands to propagate.");
        std::thread::sleep(Duration::from_millis(1000));
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GithubIssue<'doc> {
    pub title: String,
    pub assignees: BTreeSet<String>,
    pub body: String,
    pub labels: Vec<String>,
    pub tracking_issue: Option<&'doc IssueId>,
    pub goal_document: &'doc GoalDocument,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum GithubAction<'doc> {
    CreateLabel {
        label: GhLabel,
    },

    CreateIssue {
        issue: GithubIssue<'doc>,
    },

    ChangeTitle {
        number: u64,
        title: String,
    },

    ChangeMilestone {
        number: u64,
        milestone: String,
    },

    Comment {
        number: u64,
        body: String,
    },

    UpdateIssueBody {
        number: u64,
        body: String,
    },

    // We intentionally do not sync the issue *text*, because it may have been edited.
    SyncAssignees {
        number: u64,
        remove_owners: BTreeSet<String>,
        add_owners: BTreeSet<String>,
    },

    SyncLabels {
        number: u64,
        remove_labels: BTreeSet<String>,
        add_labels: BTreeSet<String>,
    },

    LockIssue {
        number: u64,
    },
}

/// Initializes the required `T-<team>` labels on the repository.
/// Warns if the labels are found with wrong color.
fn initialize_labels(
    repository: &Repository,
    teams_with_asks: &BTreeSet<&TeamName>,
) -> Result<BTreeSet<GithubAction<'static>>> {
    const TEAM_LABEL_COLOR: &str = "bfd4f2";

    let mut desired_labels: BTreeSet<_> = teams_with_asks
        .iter()
        .map(|team| {
            let label_name = team.gh_label();

            GhLabel {
                name: label_name,
                color: TEAM_LABEL_COLOR.to_string(),
            }
        })
        .collect();

    desired_labels.insert(GhLabel {
        name: "C-tracking-issue".to_string(),
        color: "f5f1fd".to_string(),
    });

    desired_labels.insert(GhLabel {
        name: ROADMAP_LABEL.to_string(),
        color: "5319E7".to_string(),
    });

    for existing_label in GhLabel::list(repository)? {
        desired_labels.remove(&existing_label);
    }

    Ok(desired_labels
        .into_iter()
        .map(|label| GithubAction::CreateLabel { label })
        .collect())
}

/// Initializes the required `T-<team>` labels on the repository.
/// Warns if the labels are found with wrong color.
fn initialize_issues<'doc>(
    repository: &Repository,
    timeframe: &str,
    goal_documents: &'doc [GoalDocument],
) -> Result<BTreeSet<GithubAction<'doc>>> {
    // the set of issues we want to exist
    let desired_issues: BTreeSet<GithubIssue> = goal_documents
        .iter()
        .map(|goal_document| issue(timeframe, goal_document))
        .collect::<Result<_>>()?;

    // Check for duplicate tracking issues
    let mut tracking_issue_counts = std::collections::HashMap::new();
    for issue in &desired_issues {
        if let Some(tracking_issue) = issue.tracking_issue {
            let count = tracking_issue_counts
                .entry(tracking_issue.number)
                .or_insert(0);
            *count += 1;
        }
    }

    for (issue_number, count) in tracking_issue_counts {
        if count > 1 {
            let goals_with_issue: Vec<_> = desired_issues
                .iter()
                .filter(|issue| issue.tracking_issue.map(|ti| ti.number) == Some(issue_number))
                .map(|issue| issue.goal_document.path.display().to_string())
                .collect();

            spanned::bail_here!(
                "Tracking issue #{} is assigned to {} goals: {}. \
                 Each tracking issue can only be assigned to one goal.",
                issue_number,
                count,
                goals_with_issue.join(", ")
            );
        }
    }

    // the list of existing issues in the target milestone
    let milestone_issues = list_issues_in_milestone(repository, timeframe)?;

    let mut actions = BTreeSet::new();

    // Go through each of the issues we want to exist (derived from the goals defined in the target folder)
    for desired_issue in desired_issues {
        // Check if we already created a tracking issue...
        //
        let existing_issue = if let Some(tracking_issue) = desired_issue.tracking_issue {
            // a. We first check if there is a declared tracking issue in the markdown file.
            // If so, check if we've already loaded its data.
            if let Some(issue) = milestone_issues
                .iter()
                .find(|issue| issue.number == tracking_issue.number)
            {
                // If so, reuse it to avoid latency.
                Some(issue.clone())
            } else {
                // If not, load its information from the repository by number.
                let existing_issue =
                    fetch_issue(repository, tracking_issue.number).map_err(|e| {
                        e.wrap_str(Spanned::here(format!(
                            "error while fetching declared tracking issue {} for goal {}",
                            tracking_issue.number,
                            desired_issue.goal_document.path.display(),
                        )))
                    })?;
                Some(existing_issue)
            }
        } else {
            // b. If the markdown does not have a declared tracking issue, then we can search through
            // the issues in the milestone for one with the correct title.
            // We could also do a fresh GH query for an issue with the desired title
            // but that is slower.
            //
            // This addresses a kind of awkward gap in our handling-- when a new project goal
            // is created, we first create an issue for it, then do a loop and execute again.
            // This second time, we will find the issue with the known title, get its
            // number, and put that number into the markdown.
            milestone_issues
                .iter()
                .find(|issue| issue.title == desired_issue.title)
                .cloned()
        };

        match existing_issue {
            Some(existing_issue) => {
                if existing_issue.assignees != desired_issue.assignees {
                    actions.insert(GithubAction::SyncAssignees {
                        number: existing_issue.number,
                        remove_owners: existing_issue
                            .assignees
                            .difference(&desired_issue.assignees)
                            .cloned()
                            .collect(),
                        add_owners: desired_issue
                            .assignees
                            .difference(&existing_issue.assignees)
                            .cloned()
                            .collect(),
                    });
                }

                // Compare labels - convert existing issue labels to strings for comparison
                let existing_label_names: BTreeSet<String> = existing_issue
                    .labels
                    .iter()
                    .map(|label| label.name.clone())
                    .collect();
                let desired_label_names: BTreeSet<String> = desired_issue
                    .labels
                    .iter()
                    .cloned()
                    .chain(existing_issue.labels.iter().filter_map(|label| {
                        matches!(
                            label.name.as_str(),
                            "ex-2025h2"
                                | "Flagship Goal"
                                | "R-every-4-weeks"
                                | "R-every-2-weeks"
                                | "R-every-week"
                        )
                        .then(|| label.name.clone())
                    }))
                    .collect();

                if existing_label_names != desired_label_names {
                    actions.insert(GithubAction::SyncLabels {
                        number: existing_issue.number,
                        remove_labels: existing_label_names
                            .difference(&desired_label_names)
                            .cloned()
                            .collect(),
                        add_labels: desired_label_names
                            .difference(&existing_label_names)
                            .cloned()
                            .collect(),
                    });
                }

                if existing_issue.title != desired_issue.title {
                    actions.insert(GithubAction::ChangeTitle {
                        number: existing_issue.number,
                        title: desired_issue.title,
                    });
                }

                if existing_issue.milestone.as_ref().map(|m| m.title.as_str()) != Some(timeframe) {
                    actions.insert(GithubAction::ChangeMilestone {
                        number: existing_issue.number,
                        milestone: timeframe.to_string(),
                    });
                    actions.insert(GithubAction::Comment {
                        number: existing_issue.number,
                        body: format!("{CONTINUING_GOAL_PREFIX} {timeframe}",),
                    });
                }

                if !existing_issue.was_locked() {
                    actions.insert(GithubAction::LockIssue {
                        number: existing_issue.number,
                    });
                    actions.insert(GithubAction::Comment {
                        number: existing_issue.number,
                        body: LOCK_TEXT.to_string(),
                    });
                }

                let link_text = goal_document_link(timeframe, &desired_issue.goal_document);
                if !existing_issue.body.contains(&link_text) {
                    // Let's update the tracking issue to the new goal description, while keeping
                    // the old text in case we need it. It's surprisingly hard to get out of GH
                    // otherwise.
                    let body = format!(
                        "{desired_body}\n---\nNote: we have updated the body to match the \
                         {timeframe} goal. Your original text is preserved below. \
                         <details>\n{existing_body}\n</details>",
                        desired_body = desired_issue.body,
                        existing_body = existing_issue.body,
                    );
                    actions.insert(GithubAction::UpdateIssueBody {
                        number: existing_issue.number,
                        body,
                    });
                }
            }

            None => {
                actions.insert(GithubAction::CreateIssue {
                    issue: desired_issue,
                });
            }
        }
    }

    Ok(actions)
}

fn issue<'doc>(timeframe: &str, document: &'doc GoalDocument) -> Result<GithubIssue<'doc>> {
    let mut assignees = BTreeSet::default();
    for username in document.metadata.owner_usernames() {
        if let Some(data) = get_person_data(username)? {
            assignees.insert(data.github_username.clone());
        }
    }

    let mut labels = vec!["C-tracking-issue".to_string()];
    if document.all_roadmaps().is_some() {
        labels.push("Roadmap Goal".to_string());
    }
    for team in document.teams_with_asks() {
        labels.push(team.gh_label());
    }

    Ok(GithubIssue {
        title: document.metadata.title.to_string(),
        assignees,
        body: issue_text(timeframe, document)?,
        labels,
        tracking_issue: document.metadata.tracking_issue.as_ref(),
        goal_document: document,
    })
}

fn goal_document_link(timeframe: &str, document: &GoalDocument) -> String {
    let goal_file = document.link_path.file_stem().unwrap().to_str().unwrap();
    format!("[{timeframe}/{goal_file}](https://goals.rust-lang.org/{timeframe}/{goal_file}.html)")
}

fn issue_text(timeframe: &str, document: &GoalDocument) -> Result<String> {
    let mut tasks = vec![];
    for goal_plan in &document.goal_plans {
        tasks.extend(task_items(goal_plan)?);
    }

    let teams = document
        .teams_with_asks()
        .iter()
        .map(|team| team.name_and_link())
        .collect::<Vec<_>>();

    Ok(format!(
        r##"
| Metadata         | |
| --------         | --- |
| Contact          | {contact} |
| Team(s)          | {teams} |
| Goal document    | {goaldocument} |

## Summary

{summary}

## Tasks and status

{tasks}

[Team]: https://img.shields.io/badge/Team%20ask-red
"##,
        contact = &document.metadata.owner_usernames().join(", "),
        teams = teams.join(", "),
        summary = document.summary,
        tasks = tasks.join("\n"),
        goaldocument = goal_document_link(timeframe, document),
    ))
}

fn task_items(goal_plan: &GoalPlan) -> Result<Vec<String>> {
    use std::fmt::Write;

    let mut tasks = vec![];

    if let Some(title) = &goal_plan.subgoal {
        tasks.push(format!("### {}", **title));
    }

    for plan_item in &goal_plan.plan_items {
        let mut description = format!(
            "* {box} {text}",
            box = if plan_item.is_complete() { "[x]" } else { "[ ]" },
            text = plan_item.text.content
        );

        if let Some(parsed_owners) = plan_item.parse_owners()? {
            match parsed_owners {
                ParsedOwners::TeamAsks(asks) => {
                    let teams: Vec<String> = asks.iter().map(|ask| ask.name_and_link()).collect();

                    write!(description, " ({} ![Team][])", teams.join(", "))?;
                }

                ParsedOwners::Usernames(usernames) => {
                    write!(description, " ({})", usernames.join(", "))?;
                }
            }
        }

        tasks.push(description);
    }

    Ok(tasks)
}

fn teams_with_asks(goal_documents: &[GoalDocument]) -> BTreeSet<&'static TeamName> {
    goal_documents
        .iter()
        .flat_map(|g| g.team_involvement.teams())
        .collect()
}

impl Display for GithubAction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GithubAction::CreateLabel {
                label: GhLabel { name, color },
            } => {
                write!(f, "create label `{}` with color `{}`", name, color)
            }
            GithubAction::CreateIssue { issue } => {
                write!(f, "create issue \"{}\"", issue.title,)
            }
            GithubAction::ChangeMilestone { number, milestone } => {
                write!(f, "update issue #{} milestone to \"{}\"", number, milestone)
            }
            GithubAction::ChangeTitle { number, title } => {
                write!(f, "update issue #{} title to \"{}\"", number, title)
            }
            GithubAction::Comment { number, body } => {
                write!(f, "post comment on issue #{}: \"{}\"", number, body)
            }
            GithubAction::UpdateIssueBody { number, body: _ } => {
                write!(f, "update the body on issue #{} for new milestone", number)
            }
            GithubAction::SyncAssignees {
                number,
                remove_owners,
                add_owners,
            } => {
                write!(
                    f,
                    "sync issue #{} assignees ({})",
                    number,
                    remove_owners
                        .iter()
                        .map(|s| format!("-{}", s))
                        .chain(add_owners.iter().map(|s| format!("+{}", s)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            GithubAction::SyncLabels {
                number,
                remove_labels,
                add_labels,
            } => {
                write!(
                    f,
                    "sync issue #{} labels ({})",
                    number,
                    remove_labels
                        .iter()
                        .map(|s| format!("-{}", s))
                        .chain(add_labels.iter().map(|s| format!("+{}", s)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            GithubAction::LockIssue { number } => {
                write!(f, "lock issue #{}", number)
            }
        }
    }
}

impl GithubAction<'_> {
    pub fn execute(self, repository: &Repository, timeframe: &str) -> Result<()> {
        match self {
            GithubAction::CreateLabel { label } => {
                label.create(repository)?;
                Ok(())
            }

            GithubAction::CreateIssue {
                issue:
                    GithubIssue {
                        title,
                        assignees,
                        body,
                        labels,
                        tracking_issue: _,
                        goal_document,
                    },
            } => {
                let issue_id =
                    create_issue(repository, &body, &title, &labels, &assignees, timeframe)?;

                goal_document.link_issue(issue_id)?;

                Ok(())
            }

            GithubAction::ChangeMilestone { number, milestone } => {
                change_milestone(repository, number, &milestone)?;
                Ok(())
            }

            GithubAction::ChangeTitle { number, title } => {
                change_title(repository, number, &title)?;
                Ok(())
            }

            GithubAction::Comment { number, body } => {
                create_comment(repository, number, &body)?;
                Ok(())
            }

            GithubAction::UpdateIssueBody { number, body } => {
                update_issue_body(repository, number, &body)?;
                Ok(())
            }

            GithubAction::SyncAssignees {
                number,
                remove_owners,
                add_owners,
            } => {
                // NOTE: Swallow errors here because sometimes people are not present in the org.
                // We don't want to stop everything for that.
                sync_assignees(repository, number, &remove_owners, &add_owners)?;
                Ok(())
            }

            GithubAction::SyncLabels {
                number,
                remove_labels,
                add_labels,
            } => {
                sync_labels(repository, number, &remove_labels, &add_labels)?;
                Ok(())
            }

            GithubAction::LockIssue { number } => lock_issue(repository, number),
        }
    }
}
