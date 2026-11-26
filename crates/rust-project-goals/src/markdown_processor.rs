use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::config::GoalsConfig;
use crate::{re, team};

/// Pure, stateless markdown processing logic
pub struct MarkdownProcessor {
    config: GoalsConfig,
}

/// Mutable state that gets passed around during processing
#[derive(Default)]
pub struct MarkdownProcessorState {
    /// Cache of username -> display name mappings to avoid repeated API calls
    pub display_names_cache: BTreeMap<String, Rc<String>>,
}

impl MarkdownProcessor {
    pub fn new(config: GoalsConfig) -> Self {
        MarkdownProcessor { config }
    }

    /// Process markdown content with all linking transformations
    pub fn process_markdown(
        &self,
        content: &str,
        state: &mut MarkdownProcessorState,
    ) -> anyhow::Result<String> {
        let mut content = content.to_string();
        content = self.link_users(content, state)?;
        content = self.link_teams(content)?; // stateless
        content = self.linkify(content)?; // stateless
        content = self.insert_links(content)?; // stateless

        // This forces pulldown-cmark to treat it as a HTML code block
        // instead of Markdown. Without this, it inserts spurious
        // `<p>` tags in as well as creating nested code blocks
        // (because it treats an indented section of code as a
        // markdown code block)
        let content = content.replace("<pre>", "\n\n<pre>");

        Ok(content)
    }

    /// Replace @username with [Display Name][] and add link definitions
    fn link_users(
        &self,
        content: String,
        state: &mut MarkdownProcessorState,
    ) -> anyhow::Result<String> {
        // TODO: using a regex to pick out the usernames has risk for false positives.
        //
        // E.g. it was picking up the field projection syntax
        // (`foo.@bar.@baz`) as github usernames and either finding
        // those and using their names or returning gibberish that
        // made the underlying comment indistinguishable.
        //
        // This is a pretty messy situation, but I think the best
        // option is to only look for GH usernames in non-code
        // portions of the Markdown content.
        //
        // As a workaround, we're only rendering the Project members'
        // usernames so far and leaving everything else as-is.
        let usernames: BTreeSet<String> = re::USERNAME
            .find_iter(&content)
            .map(|m| m.as_str().to_string())
            .filter(|username| !self.config.ignore_users.contains(username))
            .collect();

        let mut content = content;
        for username in &usernames {
            if let Ok(display_name) = self.get_display_name(username, state) {
                content = content.replace(username, &format!("[{display_name}][]"));
            }
        }

        // Add link definitions
        content.push_str("\n\n");
        for username in &usernames {
            if let Ok(display_name) = self.get_display_name(username, state) {
                content.push_str(&format!(
                    "[{}]: https://github.com/{}\n",
                    display_name,
                    &username[1..] // Remove @ prefix
                ));
            }
        }

        Ok(content)
    }

    /// Add team link definitions
    fn link_teams(&self, mut content: String) -> anyhow::Result<String> {
        content.push_str("\n\n");
        for team in team::get_team_names().map_err(|e| anyhow::anyhow!("{e}"))? {
            content.push_str(&format!("{team}: {}\n", team.url()));
        }
        Ok(content)
    }

    /// Apply linkifier patterns to expand [pattern][] -> [pattern](url)
    fn linkify(&self, content: String) -> anyhow::Result<String> {
        let mut content = content;
        for (regex, url_template) in &self.config.linkifiers {
            content = regex
                .replace_all(&content, |captures: &regex::Captures<'_>| -> String {
                    // The capture should be [pattern] format
                    assert!(captures[0].starts_with("[") && captures[0].ends_with("]"));

                    let mut result = String::new();
                    result.push_str(&captures[0]); // [pattern]
                    result.push('(');
                    captures.expand(url_template, &mut result);
                    result.push(')');
                    result
                })
                .to_string();
        }
        Ok(content)
    }

    /// Insert configured link definitions
    fn insert_links(&self, mut content: String) -> anyhow::Result<String> {
        content.push_str("\n\n");
        for (name, url) in &self.config.links {
            content.push_str(&format!("[{name}]: {url}\n"));
        }
        Ok(content)
    }

    /// Get display name for a username, using cache and fallbacks
    fn get_display_name(
        &self,
        username: &str,
        state: &mut MarkdownProcessorState,
    ) -> anyhow::Result<Rc<String>> {
        // Check cache first
        if let Some(display_name) = state.display_names_cache.get(username) {
            return Ok(display_name.clone());
        }

        // Check configured overrides
        if let Some(configured_name) = self.config.users.get(username) {
            let display_name = Rc::new(configured_name.clone());
            state
                .display_names_cache
                .insert(username.to_string(), display_name.clone());
            return Ok(display_name);
        }

        // Try to get name from rust teams repo
        let display_name =
            match team::get_person_data(username).map_err(|e| anyhow::anyhow!("{e}"))? {
                Some(person) => person.data.name.clone(),
                None => {
                    anyhow::bail!("Failed to load user info for {username}");
                }
            };

        let display_name_rc = Rc::new(display_name);
        state
            .display_names_cache
            .insert(username.to_string(), display_name_rc.clone());
        Ok(display_name_rc)
    }
}
