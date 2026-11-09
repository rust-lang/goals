use std::{borrow, cmp, fmt, hash};

/// Goal title.
///
/// Removes characters like backticks when sorting.
#[derive(Debug, Clone, Default)]
pub struct Title {
    /// Raw string title.
    displayable: String,

    /// Sortable title.
    sortable: String,
}
impl Title {
    /// Create a new title.
    pub fn new(displayable: String) -> Title {
        Title {
            sortable: displayable.trim().replace('`', "").to_lowercase(),
            displayable,
        }
    }

    /// Get displayable string for title.
    pub fn displayable(&self) -> &str {
        &self.displayable
    }

    /// Get sortable string for title.
    pub fn sortable(&self) -> &str {
        &self.sortable
    }

    /// Slugify title.
    pub fn slugify(&self) -> String {
        self.sortable
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Empty title?
    pub fn is_empty(&self) -> bool {
        self.sortable.is_empty()
    }
}
impl From<String> for Title {
    fn from(value: String) -> Self {
        Title::new(value)
    }
}
impl borrow::Borrow<str> for Title {
    fn borrow(&self) -> &str {
        &self.sortable
    }
}
impl PartialEq for Title {
    fn eq(&self, other: &Self) -> bool {
        self.sortable == other.sortable
    }
}
impl Eq for Title {}
impl PartialOrd for Title {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Title {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.sortable.cmp(&other.sortable)
    }
}
impl hash::Hash for Title {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.sortable.hash(state);
    }
}
impl fmt::Display for Title {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.displayable, f)
    }
}
