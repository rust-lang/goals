use crate::re::{GITHUB_ISSUE_URL, REPOSITORY, TRACKING_ISSUE};
use std::fmt::Display;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Repository {
    /// Something like `rust-lang`
    pub org: String,

    /// Something like `goals`
    pub repo: String,
}

impl Repository {
    pub fn new(org: &(impl Display + ?Sized), repo: &(impl Display + ?Sized)) -> Self {
        Self {
            org: org.to_string(),
            repo: repo.to_string(),
        }
    }
}

impl std::fmt::Display for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Repository { org, repo } = self;
        write!(f, "{org}/{repo}")
    }
}

impl std::str::FromStr for Repository {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(c) = REPOSITORY.captures(s) else {
            anyhow::bail!("invalid repository `{s}`");
        };

        Ok(Repository::new(&c[1], &c[2]))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct IssueId {
    pub repository: Repository,

    /// Something like `22`
    pub number: u64,
}

impl IssueId {
    pub fn new(repository: Repository, number: u64) -> Self {
        Self { repository, number }
    }

    pub fn url(&self) -> String {
        let IssueId {
            repository: Repository { org, repo },
            number,
        } = self;
        format!("https://github.com/{org}/{repo}/issues/{number}")
    }

    pub fn from_url(url: &str) -> Option<Self> {
        let c = GITHUB_ISSUE_URL.captures(url)?;

        let repository = Repository::new(&c[1], &c[2]);
        let number = c[3].parse().unwrap();

        Some(IssueId::new(repository, number))
    }
}

impl std::fmt::Debug for IssueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl std::fmt::Display for IssueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let IssueId {
            repository: Repository { org, repo },
            number,
        } = self;
        write!(f, "[{org}/{repo}#{number}]")
    }
}

impl std::str::FromStr for IssueId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(c) = TRACKING_ISSUE.captures(s) else {
            anyhow::bail!("invalid issue-id");
        };

        Ok(IssueId::new(Repository::new(&c[1], &c[2]), c[3].parse()?))
    }
}
