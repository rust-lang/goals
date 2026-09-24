use std::path::{Path, PathBuf};
use std::sync::Arc;

use handlebars::{
    Context, DirectorySourceOptions, Handlebars, Helper, HelperDef, HelperResult, Output,
    RenderContext, RenderErrorReason,
};
use rust_project_goals::config::GoalsConfig;
use rust_project_goals::gh::issues::ExistingGithubComment;
use rust_project_goals::markdown_processor::{MarkdownProcessor, MarkdownProcessorState};
use serde::Serialize;

use rust_project_goals::spanned::Result;
use rust_project_goals_json::Progress;

pub struct Templates<'h> {
    reg: Handlebars<'h>,
}

impl<'h> Templates<'h> {
    pub fn new() -> Result<Self> {
        let templates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates");

        // Load config from book.toml using clean approach
        let book_toml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../book.toml");
        let goals_config = if book_toml_path.exists() {
            GoalsConfig::from_book_toml(book_toml_path).map_err(|e| {
                rust_project_goals::spanned::Error::str(format!(
                    "Failed to load goals config: {}",
                    e
                ))
            })?
        } else {
            GoalsConfig::default()
        };

        Self::from_templates_dir(&templates, goals_config)
    }

    pub fn from_templates_dir(
        dir_path: impl AsRef<Path>,
        goals_config: GoalsConfig,
    ) -> Result<Self> {
        let dir_path = dir_path.as_ref();
        let mut reg = Handlebars::new();

        reg.set_strict_mode(true);

        reg.register_templates_directory(dir_path, DirectorySourceOptions::default())?;
        assert!(reg.get_template("updates").is_some());

        // Create the shared markdown processor
        let markdown_processor = MarkdownProcessor::new(goals_config);
        let processor_arc = Arc::new(markdown_processor);

        // Register custom helper with processor
        let markdown_helper = MarkdownToHtmlHelper::new(processor_arc);
        reg.register_helper("markdown_to_html", Box::new(markdown_helper));
        reg.register_helper("is_complete", Box::new(is_complete));

        Ok(Templates { reg })
    }
}

/// Custom handlebars helper that processes markdown with linking
pub struct MarkdownToHtmlHelper {
    processor: Arc<MarkdownProcessor>,
}

impl MarkdownToHtmlHelper {
    pub fn new(processor: Arc<MarkdownProcessor>) -> Self {
        Self { processor }
    }
}

impl HelperDef for MarkdownToHtmlHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        if let Some(md) = h.param(0).and_then(|v| v.value().as_str()) {
            // Create fresh state for this template invocation
            let mut local_state = MarkdownProcessorState::default();

            // Process markdown with linking
            let processed = self
                .processor
                .process_markdown(md, &mut local_state)
                .map_err(|e| {
                    RenderErrorReason::Other(format!("Markdown processing failed: {}", e))
                })?;

            // Convert to HTML
            let html = comrak::markdown_to_html(&processed, &comrak::ComrakOptions::default());
            out.write(&html)?;
        }
        Ok(())
    }
}

handlebars::handlebars_helper!(is_complete: |p: Progress| match p {
    Progress::Binary { is_closed } => is_closed,
    Progress::Tracked { completed, total } => completed == total,
    Progress::Error { .. } => false,
});

/// The parameters expected by the `updates.md` template.
#[derive(Serialize, Debug)]
pub struct Updates {
    pub milestone: String,
    pub roadmap_goals_by_theme: Vec<ThemeSection>,
    pub other_goals: Vec<UpdatesGoal>,
    pub goal_count: usize,
    pub roadmap_goal_count: usize,
}

#[derive(Serialize, Debug)]
pub struct ThemeSection {
    pub theme_name: String,
    pub goals: Vec<UpdatesGoal>,
}

impl Updates {
    pub fn new(
        milestone: String,
        roadmap_goals: Vec<UpdatesGoal>,
        other_goals: Vec<UpdatesGoal>,
    ) -> Self {
        // Group roadmap goals by theme
        let mut themes_map: std::collections::BTreeMap<String, Vec<UpdatesGoal>> =
            std::collections::BTreeMap::new();

        for goal in roadmap_goals.iter() {
            if goal.theme.is_empty() {
                themes_map
                    .entry("Other".to_string())
                    .or_default()
                    .push(goal.clone());
            } else {
                for theme in &goal.theme {
                    themes_map
                        .entry(theme.clone())
                        .or_default()
                        .push(goal.clone());
                }
            }
        }

        let roadmap_goals_by_theme: Vec<ThemeSection> = themes_map
            .into_iter()
            .map(|(theme_name, goals)| ThemeSection { theme_name, goals })
            .collect();

        Updates {
            milestone,
            roadmap_goal_count: roadmap_goals.len(),
            goal_count: roadmap_goals.len() + other_goals.len(),
            roadmap_goals_by_theme,
            other_goals,
        }
    }
    pub fn render(self) -> Result<String> {
        let templates = Templates::new()?;
        Ok(templates.reg.render("updates", &self)?)
    }
}

/// Part of the parameters expected by the `updates.md` template.
#[derive(Serialize, Debug, Clone)]
pub struct UpdatesGoal {
    /// Title of the tracking issue
    pub title: String,

    /// Tracking issue number on the project goals repository
    pub issue_number: u64,

    /// Comma-separated list of assignees
    pub issue_assignees: String,

    /// URL of the tracking issue
    pub issue_url: String,

    /// Link text for the issue (e.g., "rust-lang/goals#123")
    pub issue_link_text: String,

    /// True if the issue is closed.
    pub is_closed: bool,

    /// True if there are "help wanted" comments OR the TL;DR includes a help wanted request.
    pub has_help_wanted: bool,

    /// If there are comments that include ["help wanted"](`rust_project_goals::re::HELP_WANTED`)
    /// comments, those comments are included here.
    pub help_wanted: Vec<HelpWanted>,

    /// Markdown with update text (bullet list)
    pub comments: Vec<ExistingGithubComment>,

    /// The "<details>" summary, a prettified version of comments.len().
    pub details_summary: String,

    /// Progress towards the goal
    pub progress: Progress,

    /// TL;DR comment (if any, empty string if none)
    pub tldr: Option<String>,

    /// If this goal needs to be separated from its following sibling by an empty line.
    pub needs_separator: bool,

    /// Themes for roadmap goals (e.g., "Beyond the `&`", "Unblocking dormant traits", etc.)
    /// Empty for non-roadmap goals
    pub theme: Vec<String>,

    /// Contact for the goal
    pub contact: String,

    /// Team champions for this goal (e.g., "T-lang (nikomatsakis), T-compiler (jackh726)")
    pub team_champions: String,

    /// Task owners for this goal (individual contributors)
    pub task_owners: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct HelpWanted {
    pub text: String,
}
