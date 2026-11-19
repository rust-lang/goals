use std::{
    collections::BTreeSet,
    fmt::{Display, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use regex::Regex;
use spanned::{Result, Spanned};
use walkdir::WalkDir;

pub const ARROW: &str = "↳";

pub static MILESTONE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{4}([hH][12])?$").unwrap());

/// Format a goal title cell for use in markdown tables.
/// If `subgoal_title` is Some, renders as `↳ subgoal`, otherwise as `[title](link)`.
pub fn goal_title_cell(title: &str, link: &Path, subgoal_title: Option<&str>) -> String {
    if let Some(subgoal) = subgoal_title {
        format!("{ARROW} {subgoal}")
    } else {
        format!("[{}]({})", title, link.display())
    }
}

/// Formats a table as markdown. The input should be a series of rows
/// where each row has the same number of columns.
/// The first row is the headers.
pub fn format_table(rows: &[Vec<Spanned<String>>]) -> String {
    let mut output = String::new();

    let Some((header_row, data_rows)) = rows.split_first() else {
        return String::new();
    };

    let columns = header_row.len();
    let mut widths = vec![0; columns];

    for columns in data_rows {
        for (text, col) in columns.iter().zip(0..) {
            widths[col] = widths[col].max(text.len());
        }
    }

    for (columns, row) in rows.iter().zip(0..) {
        for (text, col) in columns.iter().zip(0..) {
            output.push('|');

            write!(
                output,
                " {text:<width$} ",
                text = **text,
                width = widths[col]
            )
            .unwrap();
        }

        output.push('|');
        output.push('\n');

        // Print the `---` row after the headers
        if row == 0 {
            for width in &widths {
                output.push('|');
                write!(output, " {text:<width$} ", text = ":--", width = width).unwrap();
            }
            output.push('|');
            output.push('\n');
        }
    }

    output
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GithubUserInfo {
    pub name: Option<String>,
}

impl GithubUserInfo {
    pub fn load(login: &str) -> Result<Self> {
        Self::github_request(login)
    }

    fn github_request(login: &str) -> Result<Self> {
        in_thread(|| -> Result<_> {
            // FIXME: cache this in the target directory or something
            use reqwest::header::USER_AGENT;
            let url = format!("https://api.github.com/users/{}", &login[1..]);
            let response: GithubUserInfo = reqwest::blocking::Client::new()
                .get(&url)
                .header(USER_AGENT, "mdbook-goals/1.0")
                .send()?
                .json()?;
            Ok(response)
        })
    }
}

pub fn commas(iter: impl IntoIterator<Item: Display>) -> String {
    let mut output = String::new();
    for (elem, i) in iter.into_iter().zip(0..) {
        if i > 0 {
            write!(output, ", ").unwrap();
        }
        write!(output, "{elem}").unwrap();
    }
    output
}

/// Returns all markdown files in `directory_path` as `(absolute, relative)` pairs,
/// where `relative` is relative to `directory_path`.
pub fn markdown_files(directory_path: &Path) -> Result<Vec<(PathBuf, PathBuf)>> {
    if !directory_path.is_dir() {
        spanned::bail_here!("`{}` is not a directory", directory_path.display());
    }

    let mut files = vec![];
    for entry in WalkDir::new(directory_path) {
        let entry = entry?;

        if entry.file_type().is_file() && entry.path().extension() == Some("md".as_ref()) {
            files.push((
                entry.path().to_path_buf(),
                entry
                    .path()
                    .strip_prefix(directory_path)
                    .unwrap()
                    .to_path_buf(),
            ));
        }
    }
    Ok(files)
}

/// Returns a comma-separated list of the strings in `s` (no spaces).
pub fn comma(s: &BTreeSet<String>) -> String {
    s.iter().map(|s| &s[..]).collect::<Vec<_>>().join(",")
}

/// Runs `op` in another thread. Useful for making blocking calls to `request`
/// without making tokio upset.
pub fn in_thread<R>(op: impl FnOnce() -> R + Send) -> R
where
    R: Send,
{
    std::thread::scope(|scope| scope.spawn(op).join().unwrap())
}

/// Tracks footnotes for long text in markdown tables.
///
/// When a text value is too long to display inline in a table cell,
/// it is replaced with a numbered reference like `\*1` and the full
/// text is accumulated for rendering after the table.
pub struct Footnotes {
    entries: Vec<String>,
}

impl Footnotes {
    /// Maximum text length before converting to a footnote reference.
    const MAX_INLINE_LEN: usize = 22;

    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// If `text` is short enough, returns it as-is. Otherwise, stores a footnote
    /// and returns a reference like `\*1`.
    pub fn maybe_footnote(&mut self, text: &str, link: &Path) -> String {
        if text.len() > Self::MAX_INLINE_LEN {
            let index = self.entries.len() + 1;
            self.entries.push(format!(
                "\\*{index}: {text} ([from here]({link}))",
                link = link.display()
            ));
            format!("\\*{index}")
        } else {
            text.to_string()
        }
    }

    /// Write all accumulated footnotes to the output.
    pub fn write_to(&self, output: &mut String) -> std::fmt::Result {
        for footnote in &self.entries {
            write!(output, "\n\n{footnote}\n")?;
        }
        Ok(())
    }
}

impl Default for Footnotes {
    fn default() -> Self {
        Self::new()
    }
}
