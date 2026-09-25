use std::{collections::BTreeSet, fs, process::Command, str::FromStr, time::SystemTime};

use chrono::NaiveDate;
use rust_project_goals_json::{GithubIssueState, Progress};
use serde::{Deserialize, Serialize};
use spanned::{Context, Error, Result};

use crate::{gh::issue_id::IssueId, re, util::comma};

use super::{issue_id::Repository, labels::GhLabel, milestone::GhMilestone};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExistingGithubIssue {
    pub number: u64,
    pub title: String,
    /// Just github username, no `@`
    pub assignees: BTreeSet<String>,
    pub comments: Vec<ExistingGithubComment>,
    pub body: String,
    pub state: GithubIssueState,
    pub labels: Vec<GhLabel>,
    pub milestone: Option<GhMilestone>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExistingGithubComment {
    /// Just github username, no `@`
    pub author: String,
    pub body: String,
    pub created_at: String,
    pub url: String,
    hidden: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct ExistingGithubIssueJson {
    title: String,
    number: u64,
    assignees: Vec<ExistingGithubAssigneeJson>,
    comments: Vec<ExistingGithubCommentJson>,
    body: String,
    state: GithubIssueState,
    labels: Vec<GhLabel>,
    milestone: Option<GhMilestone>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct ExistingGithubAssigneeJson {
    login: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct ExistingGithubCommentJson {
    body: String,
    author: ExistingGithubAuthorJson,
    #[serde(rename = "createdAt")]
    created_at: String,
    url: String,
    /// Whether a comment was marked "hidden" on the GH UI.
    #[serde(rename = "isMinimized")]
    is_minimized: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct ExistingGithubAuthorJson {
    login: String,
}

pub struct CountIssues {
    pub open: u32,
    pub closed: u32,
}

pub fn count_issues_matching_search(repository: &Repository, search: &str) -> Result<CountIssues> {
    #[derive(Deserialize)]
    struct JustState {
        state: GithubIssueState,
    }

    let output = Command::new("gh")
        .arg("-R")
        .arg(&repository.to_string())
        .arg("issue")
        .arg("list")
        .arg("-S")
        .arg(search)
        .arg("-s")
        .arg("all")
        .arg("--json")
        .arg("state")
        .output()?;

    let existing_issues: Vec<JustState> = serde_json::from_slice(&output.stdout)?;

    let mut count_issues = CountIssues { open: 0, closed: 0 };

    for issue in &existing_issues {
        match issue.state {
            GithubIssueState::Open => count_issues.open += 1,
            GithubIssueState::Closed => count_issues.closed += 1,
        }
    }

    Ok(count_issues)
}

pub fn fetch_issue(repository: &Repository, issue: u64) -> Result<ExistingGithubIssue> {
    let output = Command::new("gh")
        .arg("-R")
        .arg(&repository.to_string())
        .arg("issue")
        .arg("view")
        .arg(&format!("{issue}"))
        .arg("--json")
        .arg("title,assignees,number,comments,body,state,labels,milestone")
        .output()?;

    if !output.status.success() {
        spanned::bail_here!(
            "fetching `{}` issue {} failed: {}",
            repository.to_string(),
            issue,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let e_i: ExistingGithubIssueJson = serde_json::from_slice(&output.stdout)?;

    Ok(ExistingGithubIssue::from(e_i))
}

pub fn list_issues_in_milestone(
    repository: &Repository,
    timeframe: &str,
) -> Result<Vec<ExistingGithubIssue>> {
    // Github gets upset if a milestone name is just a number
    let milestone_name = format!("{timeframe}-goals");

    list_issues_cached(
        repository,
        &[("-m", &milestone_name)],
        Some(format!(".issues-{}.json", timeframe)),
    )
}

/// Clear the cached issues for a given milestone/timeframe
pub fn clear_milestone_issues_cache(timeframe: &str) -> Result<()> {
    let cache_file = format!(".issues-{}.json", timeframe);
    if std::path::Path::new(&cache_file).exists() {
        std::fs::remove_file(&cache_file)
            .with_str_context(format!("Failed to remove cache file {cache_file}"))?;
    }
    Ok(())
}

pub fn list_issues_cached(
    repository: &Repository,
    filter: &[(&str, &str)],
    cache_file: Option<String>,
) -> Result<Vec<ExistingGithubIssue>> {
    if let Some(ref cache_path) = cache_file {
        // Check if cache file exists and is less than 5 minutes old
        if let Ok(metadata) = fs::metadata(cache_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    if elapsed.as_secs() < 300 {
                        // 5 minutes = 300 seconds
                        // Try to read from cache
                        if let Ok(cached_data) = fs::read_to_string(cache_path) {
                            if let Ok(cached_issues) =
                                serde_json::from_str::<Vec<ExistingGithubIssue>>(&cached_data)
                            {
                                return Ok(cached_issues);
                            }
                        }
                    }
                }
            }
        }
    }

    // Cache miss or expired - fetch from GitHub
    let issues = list_issues(repository, filter)?;

    // Save to cache if cache_file is specified
    if let Some(ref cache_path) = cache_file {
        if let Ok(serialized) = serde_json::to_string_pretty(&issues) {
            let _ = fs::write(cache_path, serialized); // Ignore write errors
        }
    }

    Ok(issues)
}

pub fn list_issues(
    repository: &Repository,
    filter: &[(&str, &str)],
) -> Result<Vec<ExistingGithubIssue>> {
    let mut cmd = Command::new("gh");

    cmd.arg("-R")
        .arg(&repository.to_string())
        .arg("issue")
        .arg("list")
        .arg("-s")
        .arg("all")
        .arg("-L")
        .arg("5000");

    for (opt, val) in filter {
        cmd.arg(opt);
        cmd.arg(val);
    }

    let output = cmd
        .arg("--json")
        .arg("title,assignees,number,comments,body,state,labels,milestone")
        .output()
        .with_str_context("running github cli tool `gh`")?;

    let existing_issues: Vec<ExistingGithubIssueJson> = serde_json::from_slice(&output.stdout)?;

    Ok(existing_issues
        .into_iter()
        .map(|e_i| ExistingGithubIssue::from(e_i))
        .collect())
}

pub fn create_issue(
    repository: &Repository,
    body: &str,
    title: &str,
    labels: &[String],
    assignees: &BTreeSet<String>,
    milestone: &str,
) -> Result<IssueId> {
    let output = Command::new("gh")
        .arg("-R")
        .arg(&repository.to_string())
        .arg("issue")
        .arg("create")
        .arg("-b")
        .arg(&body)
        .arg("-t")
        .arg(&title)
        .arg("-l")
        .arg(labels.join(","))
        .arg("-a")
        .arg(comma(&assignees))
        .arg("-m")
        .arg(&milestone)
        .output()?;

    if !output.status.success() {
        return Err(Error::str(format!(
            "failed to create issue `{}`: {}",
            title,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Output in stdout looks like
    //
    // https://github.com/rust-lang/goals/issues/413}

    for line in str::from_utf8(&output.stdout)?.lines() {
        if let Some(issue_id) = IssueId::from_url(line.trim()) {
            return Ok(issue_id);
        }
    }

    Err(Error::str(format!("creating issue did not return a URL")))
}

pub fn change_title(repository: &Repository, number: u64, title: &str) -> Result<()> {
    let mut command = Command::new("gh");
    command
        .arg("-R")
        .arg(&repository.to_string())
        .arg("issue")
        .arg("edit")
        .arg(number.to_string())
        .arg("-t")
        .arg(title);

    let output = command.output()?;
    if !output.status.success() {
        Err(Error::str(format!(
            "failed to change milestone `{}`: {}",
            number,
            String::from_utf8_lossy(&output.stderr)
        )))
    } else {
        Ok(())
    }
}

pub fn change_milestone(repository: &Repository, number: u64, milestone: &str) -> Result<()> {
    let mut command = Command::new("gh");
    command
        .arg("-R")
        .arg(&repository.to_string())
        .arg("issue")
        .arg("edit")
        .arg(number.to_string())
        .arg("-m")
        .arg(milestone);

    let output = command.output()?;
    if !output.status.success() {
        Err(Error::str(format!(
            "failed to change milestone `{}`: {}",
            number,
            String::from_utf8_lossy(&output.stderr)
        )))
    } else {
        Ok(())
    }
}

pub fn create_comment(repository: &Repository, number: u64, body: &str) -> Result<()> {
    let output = Command::new("gh")
        .arg("-R")
        .arg(&repository.to_string())
        .arg("issue")
        .arg("comment")
        .arg(number.to_string())
        .arg("-b")
        .arg(body)
        .output()?;

    if !output.status.success() {
        Err(Error::str(format!(
            "failed to leave comment on issue `{}`: {}",
            number,
            String::from_utf8_lossy(&output.stderr)
        )))
    } else {
        Ok(())
    }
}

pub fn update_issue_body(repository: &Repository, number: u64, body: &str) -> Result<()> {
    let output = Command::new("gh")
        .arg("-R")
        .arg(&repository.to_string())
        .arg("issue")
        .arg("edit")
        .arg(number.to_string())
        .arg("-b")
        .arg(body)
        .output()?;

    if !output.status.success() {
        Err(Error::str(format!(
            "failed to adjust issue body on issue `{}`: {}",
            number,
            String::from_utf8_lossy(&output.stderr)
        )))
    } else {
        Ok(())
    }
}

pub fn sync_assignees(
    repository: &Repository,
    number: u64,
    remove_owners: &BTreeSet<String>,
    add_owners: &BTreeSet<String>,
) -> Result<()> {
    let mut command = Command::new("gh");
    command
        .arg("-R")
        .arg(&repository.to_string())
        .arg("issue")
        .arg("edit")
        .arg(number.to_string());

    if !remove_owners.is_empty() {
        command.arg("--remove-assignee").arg(comma(&remove_owners));
    }

    if !add_owners.is_empty() {
        command.arg("--add-assignee").arg(comma(&add_owners));
    }

    let output = command.output()?;
    if !output.status.success() {
        Err(Error::str(format!(
            "failed to sync issue assignees `{}`: {}",
            number,
            String::from_utf8_lossy(&output.stderr)
        )))
    } else {
        Ok(())
    }
}

pub fn sync_labels(
    repository: &Repository,
    number: u64,
    remove_labels: &BTreeSet<String>,
    add_labels: &BTreeSet<String>,
) -> Result<()> {
    let mut command = Command::new("gh");
    command
        .arg("-R")
        .arg(&repository.to_string())
        .arg("issue")
        .arg("edit")
        .arg(number.to_string());

    if !remove_labels.is_empty() {
        command.arg("--remove-label").arg(comma(&remove_labels));
    }

    if !add_labels.is_empty() {
        command.arg("--add-label").arg(comma(&add_labels));
    }

    let output = command.output()?;
    if !output.status.success() {
        Err(Error::str(format!(
            "failed to sync issue labels `{}`: {}",
            number,
            String::from_utf8_lossy(&output.stderr)
        )))
    } else {
        Ok(())
    }
}

pub const ROADMAP_LABEL: &str = "Roadmap Goal";

pub const LOCK_TEXT: &str = "This issue is intended for status updates only.";

pub const CONTINUING_GOAL_PREFIX: &str = "This is a continuing project goal, and the updates below this comment will be for the new period";

impl ExistingGithubIssue {
    /// We use the presence of the lock message as a heuristic
    /// that we (probably) successfully locked the issue.
    /// The github CLI doesn't let you query that directly.
    pub fn was_locked(&self) -> bool {
        self.body.contains(LOCK_TEXT)
            || self
                .comments
                .iter()
                .any(|c| c.body.trim().starts_with(LOCK_TEXT))
    }

    /// True if we have a label with the given name.
    pub fn has_label(&self, name: &str) -> bool {
        self.labels.iter().any(|label| label.name == name)
    }

    /// True if the issue has the label for a roadmap goal.
    pub fn has_roadmap_label(&self) -> bool {
        self.has_label(ROADMAP_LABEL)
    }
}

pub fn lock_issue(repository: &Repository, number: u64) -> Result<()> {
    let output = Command::new("gh")
        .arg("-R")
        .arg(&repository.to_string())
        .arg("issue")
        .arg("lock")
        .arg(number.to_string())
        .output()?;

    if !output.status.success() {
        if !output.stderr.starts_with(b"already locked") {
            return Err(Error::str(format!(
                "failed to lock issue `{}`: {}",
                number,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }

    Ok(())
}

impl ExistingGithubComment {
    /// Some comments are not actually updates we want to use in progress reports. For example,
    /// automated comments when rotating goal periods, or random comments on the tracking issues.
    /// The former are kinda possible to detect (this tool generates them in the first place) and to
    /// support filtering out the other cases, we'll just ignore comments that were marked as hidden
    /// on github.
    pub fn should_hide_from_reports(&self) -> bool {
        self.is_automated_comment() || self.hidden
    }

    /// True if this is one of the special comments that we put on issues.
    fn is_automated_comment(&self) -> bool {
        let trimmed_body = self.body.trim();
        trimmed_body.starts_with(LOCK_TEXT) || trimmed_body.starts_with(CONTINUING_GOAL_PREFIX)
    }

    pub fn created_at_date(&self) -> NaiveDate {
        NaiveDate::parse_from_str(&self.created_at, "%Y-%m-%dT%H:%M:%SZ")
            .expect("failed to parse date")
    }
}

impl From<ExistingGithubIssueJson> for ExistingGithubIssue {
    fn from(e_i: ExistingGithubIssueJson) -> Self {
        ExistingGithubIssue {
            number: e_i.number,
            title: e_i.title,
            assignees: e_i.assignees.into_iter().map(|a| a.login).collect(),
            comments: e_i
                .comments
                .into_iter()
                .map(|c| ExistingGithubComment {
                    author: format!("@{}", c.author.login),
                    body: c.body,
                    url: c.url,
                    created_at: c.created_at,
                    hidden: c.is_minimized,
                })
                .collect(),
            body: e_i.body,
            state: e_i.state,
            labels: e_i.labels,
            milestone: e_i.milestone,
        }
    }
}

/// Identify how many sub-items have been completed.
/// These can be encoded in two different ways:
///
/// * Option A, the most common, is to have checkboxes in the issue. We just count the number that are checked.
/// * Option B is to include a metadata line called "Tracked issues" that lists a search query. We count the number of open vs closed issues in that query.
///
/// Returns a tuple (completed, total) with the number of completed items and the total number of items.
pub fn checkboxes(issue: &ExistingGithubIssue) -> Progress {
    match try_checkboxes(&issue) {
        Ok(pair) => pair,
        Err(e) => Progress::Error {
            message: e.to_string(),
        },
    }
}

fn try_checkboxes(issue: &ExistingGithubIssue) -> Result<Progress> {
    let mut completed = 0;
    let mut total = 0;

    for line in issue.body.lines() {
        // Does this match TRACKED_ISSUES?
        if let Some(c) = re::TRACKED_ISSUES_QUERY.captures(line) {
            let repo = Repository::from_str(&c["repo"]).map_err(|e| Error::str(e.to_string()))?;
            let query = &c["query"];

            let CountIssues { open, closed } = count_issues_matching_search(&repo, query)?;
            completed += closed;
            total += open + closed;
            continue;
        }

        if let Some(c) = re::SEE_ALSO_QUERY.captures(line) {
            let issue_urls = c["issues"].split(&[',', ' ']).filter(|s| !s.is_empty());

            for issue_url in issue_urls {
                let c = match (
                    re::SEE_ALSO_ISSUE1.captures(issue_url),
                    re::SEE_ALSO_ISSUE2.captures(issue_url),
                ) {
                    (Some(c), _) => c,
                    (None, Some(c)) => c,
                    (None, None) => {
                        spanned::bail_here!("invalid issue URL `{issue_url}`")
                    }
                };
                let repository = Repository::new(&c["org"], &c["repo"]);
                let issue_number = c["issue"].parse::<u64>()?;
                let issue = fetch_issue(&repository, issue_number)?;
                match try_checkboxes(&issue)? {
                    Progress::Binary { is_closed } => {
                        if is_closed {
                            completed += 1;
                        }
                        total += 1;
                    }

                    Progress::Tracked {
                        completed: c,
                        total: t,
                    } => {
                        completed += c;
                        total += t;
                    }

                    Progress::Error { message } => {
                        spanned::bail_here!("error parsing {repository}#{issue_number}: {message}")
                    }
                }
            }
        }

        if re::CHECKED_CHECKBOX.is_match(line) {
            total += 1;
            completed += 1;
        } else if re::CHECKBOX.is_match(line) {
            total += 1;
        }
    }

    if total == 0 && completed == 0 {
        Ok(Progress::Binary {
            is_closed: issue.state == GithubIssueState::Closed,
        })
    } else {
        Ok(Progress::Tracked { completed, total })
    }
}
