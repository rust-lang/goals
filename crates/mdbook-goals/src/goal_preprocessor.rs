use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use chrono::{self, Datelike};
use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use regex::Regex;
use rust_project_goals::config::{Configuration, GoalsConfig};
use rust_project_goals::format_champions::format_champions;
use rust_project_goals::format_team_ask::format_team_asks;
use rust_project_goals::format_team_support::format_team_support;
use rust_project_goals::markdown_processor::{MarkdownProcessor, MarkdownProcessorState};
use rust_project_goals::util;
use rust_project_goals_cli::Order;

use rust_project_goals::spanned::Spanned;
use rust_project_goals::{
    goal::{self, GoalDocument, GoalSize, RoadmapDocument, TeamAsk, TeamInvolvement},
    re,
    team::TeamName,
};

/// Extension trait to convert `spanned::Result<T>` into `anyhow::Result<T>`.
trait IntoAnyhow<T> {
    fn into_anyhow(self) -> anyhow::Result<T>;
}

impl<T> IntoAnyhow<T> for rust_project_goals::spanned::Result<T> {
    fn into_anyhow(self) -> anyhow::Result<T> {
        self.map_err(|e| anyhow::anyhow!("{e}"))
    }
}

/// Load goals configuration from book.toml using clean serde deserialization
fn load_goals_config_from_book_toml(ctx: &PreprocessorContext) -> anyhow::Result<GoalsConfig> {
    // Find book.toml in the source directory
    let book_toml_path = ctx.root.join("book.toml");
    if book_toml_path.exists() {
        GoalsConfig::from_book_toml(book_toml_path)
    } else {
        Ok(GoalsConfig::default())
    }
}

pub struct GoalPreprocessor;

impl Preprocessor for GoalPreprocessor {
    fn name(&self) -> &str {
        "goals"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> anyhow::Result<Book> {
        let mut this = GoalPreprocessorWithContext::new(ctx)?;
        for item in &mut book.items {
            this.process_book_item(item)?;
        }
        Ok(book)
    }
}

pub struct GoalPreprocessorWithContext<'c> {
    ctx: &'c PreprocessorContext,
    markdown_processor: MarkdownProcessor,
    processor_state: MarkdownProcessorState,
    goal_document_map: BTreeMap<PathBuf, Arc<Vec<GoalDocument>>>,
    roadmap_document_map: BTreeMap<PathBuf, Arc<Vec<RoadmapDocument>>>,
    milestone_issues_cache:
        BTreeMap<String, Arc<Vec<rust_project_goals::gh::issues::ExistingGithubIssue>>>,
}

/// Returns the chapter's path, or an error if it has no path.
fn chapter_path<'a>(chapter: &'a Chapter, directive: &str) -> anyhow::Result<&'a Path> {
    chapter
        .path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("found `{directive}` but chapter has no path"))
}

impl<'c> GoalPreprocessorWithContext<'c> {
    pub fn new(ctx: &'c PreprocessorContext) -> anyhow::Result<Self> {
        // Extract goals configuration using clean parsing
        let goals_config = load_goals_config_from_book_toml(ctx)?;

        // Create the shared markdown processor
        let markdown_processor = MarkdownProcessor::new(goals_config);

        Ok(GoalPreprocessorWithContext {
            ctx,
            markdown_processor,
            processor_state: MarkdownProcessorState::default(),
            goal_document_map: Default::default(),
            roadmap_document_map: Default::default(),
            milestone_issues_cache: Default::default(),
        })
    }

    fn process_book_item(&mut self, book_item: &mut BookItem) -> anyhow::Result<()> {
        match book_item {
            BookItem::Chapter(chapter) => {
                self.inject_metadata_rows(chapter)?;
                self.replace_champions(chapter)?;
                self.replace_roadmaps(chapter)?;
                self.replace_roadmap_chapters(chapter)?;
                self.replace_team_asks(chapter)?;
                self.replace_valid_team_asks(chapter)?;
                self.replace_goal_lists(chapter)?;
                self.replace_roadmap_goal_rows(chapter)?;
                self.replace_goal_chapters(chapter)?;
                self.replace_goal_count(chapter)?;
                self.replace_roadmap_goal_count(chapter)?;
                self.replace_reports(chapter)?;
                chapter.content = self
                    .markdown_processor
                    .process_markdown(&chapter.content, &mut self.processor_state)?;

                for sub_item in &mut chapter.sub_items {
                    self.process_book_item(sub_item)?;
                }

                Ok(())
            }

            BookItem::Separator => Ok(()),

            BookItem::PartTitle(_) => Ok(()),
        }
    }

    fn replace_goal_count(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        if !re::GOALS_COUNT.is_match(&chapter.content) {
            return Ok(());
        }

        let chapter_path = chapter_path(chapter, "(((#GOALS)))")?;

        let goals = self.goal_documents(chapter_path)?;

        let count = goals
            .iter()
            .filter(|g| g.metadata.status.is_not_not_accepted())
            .count();

        chapter.content = re::GOALS_COUNT
            .replace_all(&chapter.content, &count.to_string())
            .to_string();

        Ok(())
    }

    fn replace_roadmap_goal_count(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        if !re::ROADMAP_GOALS_COUNT.is_match(&chapter.content) {
            return Ok(());
        }

        let chapter_path = chapter_path(chapter, "(((#ROADMAP_GOALS)))")?;

        let goals = self.goal_documents(chapter_path)?;

        let count = goals
            .iter()
            .filter(|g| g.all_roadmaps().is_some() && g.metadata.status.is_not_not_accepted())
            .count();

        chapter.content = re::ROADMAP_GOALS_COUNT
            .replace_all(&chapter.content, &count.to_string())
            .to_string();

        Ok(())
    }

    fn replace_goal_lists(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        // Handle filtered roadmap goals first (more specific pattern)
        self.replace_roadmap_goal_lists_filtered(chapter)?;

        // Handle unfiltered roadmap goals
        self.replace_goal_lists_helper(chapter, &re::ROADMAP_GOALS_LIST, |goal, _capture| {
            goal.all_roadmaps().is_some() && goal.metadata.status.content.is_not_not_accepted()
        })?;

        self.replace_goal_lists_helper(chapter, &re::OTHER_GOALS_LIST, |goal, _capture| {
            goal.all_roadmaps().is_empty() && goal.metadata.status.content.is_not_not_accepted()
        })?;
        self.replace_goal_lists_helper(chapter, &re::GOALS_LIST, |goal, _capture| {
            goal.metadata.status.content.is_not_not_accepted()
        })?;
        self.replace_goal_lists_helper(chapter, &re::GOALS_NOT_ACCEPTED_LIST, |goal, _capture| {
            !goal.metadata.status.content.is_not_not_accepted()
        })?;

        // Handle sized goal lists (Large, Medium, Small)
        self.replace_sized_goal_list(chapter, &re::LARGE_GOALS_LIST, GoalSize::Large)?;
        self.replace_sized_goal_list(chapter, &re::MEDIUM_GOALS_LIST, GoalSize::Medium)?;
        self.replace_sized_goal_list(chapter, &re::SMALL_GOALS_LIST, GoalSize::Small)?;

        // Handle filtered highlight goal lists
        self.replace_highlight_goal_lists_filtered(chapter)?;

        // Handle highlight tables
        self.replace_highlight_tables(chapter)?;

        // Handle highlight credits (people lists)
        self.replace_highlight_credits(chapter)?;

        // Handle filtered lists of goals with needs
        self.replace_goals_with_needs_lists_filtered(chapter)?;

        // Handle funding summary table
        self.replace_funding_table(chapter)?;

        // Handle roadmap-filtered funding table
        self.replace_funding_table_filtered(chapter)?;

        // Handle grouped funding table
        self.replace_funding_table_grouped(chapter)?;

        // Handle funding table grouped by goal (alias for grouped)
        self.replace_funding_table_grouped_by_goal(chapter)?;

        // Handle funding table grouped by contact
        self.replace_funding_table_grouped_by_contact(chapter)?;

        // Handle funding legend
        self.replace_funding_legend(chapter)?;

        Ok(())
    }

    fn replace_roadmap_goal_lists_filtered(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        self.replace_goal_lists_helper(
            chapter,
            &re::ROADMAP_GOALS_LIST_FILTERED,
            |goal, capture| {
                let filter_value = capture.unwrap().trim(); // Safe because this regex always has a capture
                goal.metadata.status.content.is_not_not_accepted()
                    && goal.matches_roadmap_theme(filter_value)
            },
        )
    }

    /// Replace `| (((ROADMAP ROWS: <name>))) |` with table rows for matching goals.
    /// Generates only row content (no headers), allowing roadmap authors to
    /// manually control the table structure and add future goal rows.
    fn replace_roadmap_goal_rows(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        loop {
            let Some(m) = re::ROADMAP_ROWS_FILTERED.find(&chapter.content) else {
                return Ok(());
            };
            let range = m.range();

            let chapter_path = chapter_path(chapter, "(((ROADMAP ROWS: ...)))")?;

            let filter_value = re::ROADMAP_ROWS_FILTERED
                .captures(&chapter.content[range.clone()])
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().trim())
                .unwrap(); // Safe: regex always has a capture group

            let goals = self.goal_documents(chapter_path)?;
            let mut filtered_goals: Vec<&goal::GoalDocument> = goals
                .iter()
                .filter(|g| {
                    g.metadata.status.content.is_not_not_accepted()
                        && g.matches_roadmap_theme(filter_value)
                })
                .collect();

            filtered_goals.sort_by_key(|g| g.metadata.title.to_lowercase());

            let output = goal::format_roadmap_goal_rows(&filtered_goals, filter_value);

            chapter.content.replace_range(range, output.trim_end());
        }
    }

    fn replace_highlight_goal_lists_filtered(
        &mut self,
        chapter: &mut Chapter,
    ) -> anyhow::Result<()> {
        self.replace_themed_goal_list(
            chapter,
            &re::HIGHLIGHT_GOALS_LIST_FILTERED,
            "(((HIGHLIGHT GOALS: ...)))",
            |g| &g.metadata.highlight,
        )
    }

    fn replace_highlight_tables(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        loop {
            let Some(m) = re::HIGHLIGHT_TABLE_FILTERED.find(&chapter.content) else {
                return Ok(());
            };
            let range = m.range();

            let chapter_path = chapter_path(chapter, "(((HIGHLIGHT TABLE: ...)))")?;

            let filter_value = re::HIGHLIGHT_TABLE_FILTERED
                .captures(&chapter.content[range.clone()])
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().trim())
                .unwrap();

            let goals = self.goal_documents(chapter_path)?;
            let mut filtered_goals: Vec<&GoalDocument> = goals
                .iter()
                .filter(|g| {
                    g.metadata.status.content.is_not_not_accepted()
                        && g.metadata.highlight.contains(filter_value)
                })
                .collect();

            filtered_goals.sort_by_key(|g| g.metadata.title.to_lowercase());

            let output = goal::format_highlight_table(&filtered_goals);

            chapter.content.replace_range(range, output.trim_end());
        }
    }

    fn replace_highlight_credits(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        loop {
            let Some(m) = re::HIGHLIGHT_CREDITS_FILTERED.find(&chapter.content) else {
                return Ok(());
            };
            let range = m.range();

            let chapter_path = chapter_path(chapter, "(((HIGHLIGHT CREDITS: ...)))")?;

            let filter_value = re::HIGHLIGHT_CREDITS_FILTERED
                .captures(&chapter.content[range.clone()])
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().trim())
                .unwrap();

            let goals = self.goal_documents(chapter_path)?;
            let mut filtered_goals: Vec<&GoalDocument> = goals
                .iter()
                .filter(|g| {
                    g.metadata.status.content.is_not_not_accepted()
                        && g.metadata.highlight.contains(filter_value)
                })
                .collect();

            filtered_goals.sort_by_key(|g| g.metadata.title.to_lowercase());

            let output = goal::format_highlight_credits(&filtered_goals);

            chapter.content.replace_range(range, &output);
        }
    }

    fn replace_goals_with_needs_lists_filtered(
        &mut self,
        chapter: &mut Chapter,
    ) -> anyhow::Result<()> {
        loop {
            let Some(m) = re::GOALS_WITH_NEEDS_LIST_FILTERED.find(&chapter.content) else {
                return Ok(());
            };
            let range = m.range();

            let chapter_path = chapter_path(chapter, "(((GOALS WITH NEEDS: ...)))")?;

            let capture_value = re::GOALS_WITH_NEEDS_LIST_FILTERED
                .captures(&chapter.content[range.clone()])
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().trim())
                .unwrap();

            let goals = self.goal_documents(chapter_path)?;

            let mut filtered_goals: Vec<&GoalDocument> = if capture_value == "Funding" {
                goals
                    .iter()
                    .filter(|g| {
                        g.metadata.status.content.is_not_not_accepted() && g.needs_funding()
                    })
                    .collect()
            } else if capture_value == "Contributor" {
                goals
                    .iter()
                    .filter(|g| {
                        g.metadata.status.content.is_not_not_accepted()
                            && (g.needs_contributor() || g.metadata.is_help_wanted())
                    })
                    .collect()
            } else {
                goals
                    .iter()
                    .filter(|g| {
                        g.metadata.status.content.is_not_not_accepted()
                            && g.metadata.needs.contains(capture_value)
                    })
                    .collect()
            };

            filtered_goals.sort_by_key(|g| g.metadata.title.to_lowercase());

            let heading_re = Regex::new(r"(?m)^(#+)\s").unwrap();
            let preceding = &chapter.content[..range.start];
            let context_level = heading_re
                .find_iter(preceding)
                .last()
                .map(|m| m.as_str().trim().len())
                .unwrap_or(1);

            let output = if capture_value == "Funding" {
                goal::format_funding_goal_sections(&filtered_goals, context_level + 1)
                    .into_anyhow()?
            } else if capture_value == "Contributor" {
                goal::format_help_wanted_goal_sections(&filtered_goals, context_level + 1)
                    .into_anyhow()?
            } else {
                goal::format_highlight_goal_sections(&filtered_goals, context_level + 1)
                    .into_anyhow()?
            };

            chapter.content.replace_range(range, &output);
        }
    }

    fn replace_funding_table(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        let Some(m) = re::FUNDING_TABLE.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();

        let chapter_path = chapter_path(chapter, "(((FUNDING TABLE)))")?;
        let goals = self.goal_documents(chapter_path)?;

        let mut filtered_goals: Vec<&GoalDocument> = goals
            .iter()
            .filter(|g| g.metadata.status.content.is_not_not_accepted() && g.needs_funding())
            .collect();

        filtered_goals.sort_by_key(|g| g.metadata.title.to_lowercase());

        let output = goal::format_funding_table(&filtered_goals);
        chapter.content.replace_range(range, &output);
        Ok(())
    }

    fn replace_funding_table_filtered(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        let Some(m) = re::FUNDING_TABLE_FILTERED.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();

        let chapter_path = chapter_path(chapter, "(((FUNDING TABLE: ...)))")?;

        let filter_value = re::FUNDING_TABLE_FILTERED
            .captures(&chapter.content[range.clone()])
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim())
            .unwrap();
        let filter_value = filter_value.to_string();

        let goals = self.goal_documents(chapter_path)?;

        let mut filtered_goals: Vec<&GoalDocument> = goals
            .iter()
            .filter(|g| {
                g.metadata.status.content.is_not_not_accepted()
                    && g.needs_funding()
                    && g.matches_roadmap_theme(&filter_value)
            })
            .collect();

        filtered_goals.sort_by_key(|g| g.metadata.title.to_lowercase());

        let output = goal::format_funding_table(&filtered_goals);
        chapter.content.replace_range(range, &output);
        Ok(())
    }

    fn replace_funding_table_grouped(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        let Some(m) = re::FUNDING_TABLE_GROUPED.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();

        let chapter_path = chapter_path(chapter, "(((FUNDING TABLE GROUPED)))")?;
        let goals = self.goal_documents(chapter_path)?;
        let roadmaps = self.roadmap_documents(chapter_path)?;

        let mut filtered_goals: Vec<&GoalDocument> = goals
            .iter()
            .filter(|g| g.metadata.status.content.is_not_not_accepted() && g.needs_funding())
            .collect();

        filtered_goals.sort_by_key(|g| g.metadata.title.to_lowercase());

        let roadmap_refs: Vec<&goal::RoadmapDocument> = roadmaps.iter().collect();
        let output = goal::format_funding_table_grouped(&filtered_goals, &roadmap_refs);
        chapter.content.replace_range(range, &output);
        Ok(())
    }

    fn replace_funding_table_grouped_by_goal(
        &mut self,
        chapter: &mut Chapter,
    ) -> anyhow::Result<()> {
        let Some(m) = re::FUNDING_TABLE_GROUPED_BY_GOAL.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();

        let chapter_path = chapter_path(chapter, "(((FUNDING TABLE GROUPED BY GOAL)))")?;
        let goals = self.goal_documents(chapter_path)?;
        let roadmaps = self.roadmap_documents(chapter_path)?;

        let mut filtered_goals: Vec<&GoalDocument> = goals
            .iter()
            .filter(|g| g.metadata.status.content.is_not_not_accepted() && g.needs_funding())
            .collect();

        filtered_goals.sort_by_key(|g| &g.metadata.title);

        let roadmap_refs: Vec<&goal::RoadmapDocument> = roadmaps.iter().collect();
        let output = goal::format_funding_table_grouped(&filtered_goals, &roadmap_refs);
        chapter.content.replace_range(range, &output);
        Ok(())
    }

    fn replace_funding_table_grouped_by_contact(
        &mut self,
        chapter: &mut Chapter,
    ) -> anyhow::Result<()> {
        let Some(m) = re::FUNDING_TABLE_GROUPED_BY_CONTACT.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();

        let chapter_path = chapter_path(chapter, "(((FUNDING TABLE GROUPED BY CONTACT)))")?;
        let goals = self.goal_documents(chapter_path)?;

        let mut filtered_goals: Vec<&GoalDocument> = goals
            .iter()
            .filter(|g| g.metadata.status.content.is_not_not_accepted() && g.needs_funding())
            .collect();

        filtered_goals.sort_by_key(|g| &g.metadata.title);

        let output = goal::format_funding_table_grouped_by_contact(&filtered_goals);
        chapter.content.replace_range(range, &output);
        Ok(())
    }

    fn replace_funding_legend(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        let Some(m) = re::FUNDING_LEGEND.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();
        let output = goal::format_funding_legend();
        chapter.content.replace_range(range, &output);
        Ok(())
    }

    /// Shared helper for replacing themed goal list directives (HIGHLIGHT GOALS, GOALS WITH NEEDS).
    /// Filters goals by a `Themes` field extracted via `get_themes`, then formats as heading sections.
    fn replace_themed_goal_list(
        &mut self,
        chapter: &mut Chapter,
        regex: &Regex,
        directive_name: &str,
        get_themes: impl Fn(&GoalDocument) -> &goal::Themes,
    ) -> anyhow::Result<()> {
        loop {
            let Some(m) = regex.find(&chapter.content) else {
                return Ok(());
            };
            let range = m.range();

            let chapter_path = chapter_path(chapter, directive_name)?;

            let capture_value = regex
                .captures(&chapter.content[range.clone()])
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().trim())
                .unwrap(); // Safe: regex always has a capture group

            let goals = self.goal_documents(chapter_path)?;
            let mut filtered_goals: Vec<&GoalDocument> = goals
                .iter()
                .filter(|g| {
                    g.metadata.status.content.is_not_not_accepted()
                        && get_themes(g).contains(capture_value)
                })
                .collect();

            filtered_goals.sort_by_key(|g| g.metadata.title.to_lowercase());

            // Search backwards from the directive for the most recent heading to
            // determine the level at which to emit goal sub-headings.
            let heading_re = Regex::new(r"(?m)^(#+)\s").unwrap();
            let preceding = &chapter.content[..range.start];
            let context_level = heading_re
                .find_iter(preceding)
                .last()
                .map(|m| m.as_str().trim().len())
                .unwrap_or(1);

            let output = goal::format_highlight_goal_sections(&filtered_goals, context_level + 1)
                .into_anyhow()?;

            chapter.content.replace_range(range, &output);
        }
    }

    /// Replace sized goal list markers (LARGE GOALS, MEDIUM GOALS, SMALL GOALS)
    /// with tables grouped by primary team.
    fn replace_sized_goal_list(
        &mut self,
        chapter: &mut Chapter,
        regex: &Regex,
        size: GoalSize,
    ) -> anyhow::Result<()> {
        let Some(m) = regex.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();

        let chapter_path = chapter_path(chapter, &format!("{regex}"))?;

        let goals = self.goal_documents(chapter_path)?;
        let goals_with_status: Vec<&GoalDocument> = goals
            .iter()
            .filter(|g| g.metadata.status.content.is_not_not_accepted())
            .collect();

        let output = goal::format_sized_goal_table(&goals_with_status, size).into_anyhow()?;
        chapter.content.replace_range(range, &output);

        Ok(())
    }

    /// Replace `(((GOAL CHAPTERS)))` marker by creating subchapters for each goal,
    /// without rendering a table. The marker itself is removed from the content.
    fn replace_goal_chapters(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        let Some(m) = re::GOAL_CHAPTERS.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();

        let chapter_path = chapter_path(chapter, "(((GOAL CHAPTERS)))")?;

        // Don't create subchapters for README files
        if chapter_path.file_stem() == Some("README".as_ref()) {
            chapter.content.replace_range(range, "");
            return Ok(());
        }

        let goals = self.goal_documents(chapter_path)?;
        let mut goals_with_status: Vec<&GoalDocument> = goals
            .iter()
            .filter(|g| g.metadata.status.content.is_not_not_accepted())
            .collect();

        goals_with_status.sort_by_key(|g| g.metadata.title.to_lowercase());

        // Create subchapters for each goal
        let mut parent_names = chapter.parent_names.clone();
        parent_names.push(chapter.name.clone());
        for (goal, index) in goals_with_status.iter().zip(0..) {
            let content = std::fs::read_to_string(&goal.path)
                .with_context(|| format!("reading `{}`", goal.path.display()))?;
            let path = goal.path.strip_prefix(&self.ctx.config.book.src).unwrap();
            let mut new_chapter = Chapter::new(
                &goal.title_with_status(),
                content,
                path,
                parent_names.clone(),
            );

            if let Some(mut number) = chapter.number.clone() {
                number.push(index + 1);
                new_chapter.number = Some(number);
            }

            chapter.sub_items.push(BookItem::Chapter(new_chapter));
        }

        // Remove the marker from the content
        chapter.content.replace_range(range, "");

        Ok(())
    }

    /// Replace `(((ROADMAP CHAPTERS)))` marker by creating subchapters for each roadmap document.
    /// The marker itself is removed from the content.
    fn replace_roadmap_chapters(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        let Some(m) = re::ROADMAP_CHAPTERS.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();

        let chapter_path = chapter_path(chapter, "(((ROADMAP CHAPTERS)))")?;

        let roadmaps = self.roadmap_documents(chapter_path)?;
        let mut sorted_roadmaps: Vec<&RoadmapDocument> = roadmaps.iter().collect();
        sorted_roadmaps.sort_by_key(|r| r.title.to_lowercase());

        // Create subchapters for each roadmap
        let mut parent_names = chapter.parent_names.clone();
        parent_names.push(chapter.name.clone());
        for (roadmap, index) in sorted_roadmaps.iter().zip(0..) {
            let content = std::fs::read_to_string(&roadmap.path)
                .with_context(|| format!("reading `{}`", roadmap.path.display()))?;
            let path = roadmap
                .path
                .strip_prefix(&self.ctx.config.book.src)
                .unwrap();
            let mut new_chapter =
                Chapter::new(&roadmap.title.content, content, path, parent_names.clone());

            if let Some(mut number) = chapter.number.clone() {
                number.push(index + 1);
                new_chapter.number = Some(number);
            }

            chapter.sub_items.push(BookItem::Chapter(new_chapter));
        }

        // Remove the marker from the content
        chapter.content.replace_range(range, "");

        Ok(())
    }

    fn replace_goal_lists_helper(
        &mut self,
        chapter: &mut Chapter,
        regex: &Regex,
        filter: impl Fn(&GoalDocument, Option<&str>) -> bool,
    ) -> anyhow::Result<()> {
        loop {
            let Some(m) = regex.find(&chapter.content) else {
                return Ok(());
            };
            let range = m.range();

            let chapter_path = chapter_path(chapter, &format!("{regex}"))?;

            // Extract capture group if present
            let capture_value = regex
                .captures(&chapter.content[range.clone()])
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().trim());

            // Extract out the list of goals with the given filter.
            let goals = self.goal_documents(chapter_path)?;
            let mut goals_with_status: Vec<&GoalDocument> =
                goals.iter().filter(|g| filter(g, capture_value)).collect();

            goals_with_status.sort_by_key(|g| g.metadata.title.to_lowercase());

            // Get milestone issues for progress generation
            let milestone_issues = if let Some(first_goal) = goals_with_status.first() {
                // Extract milestone from the first goal's path
                let milestone = first_goal
                    .path
                    .parent()
                    .and_then(|p| p.file_stem())
                    .and_then(|s| s.to_str());

                if let Some(milestone) = milestone {
                    match self.get_or_load_milestone_issues(milestone) {
                        Ok(issues) => Some(issues),
                        Err(e) => {
                            eprintln!(
                                "⚠️ Failed to load milestone issues for {}: {}",
                                milestone, e
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Format the list of goals and replace the `<!-- -->` comment with that.
            let output = goal::format_goal_table(
                &goals_with_status,
                milestone_issues.as_ref().map(|arc| arc.as_slice()),
            )
            .into_anyhow()?;
            chapter.content.replace_range(range, &output);
        }
    }

    /// Look for `(((CHAMPIONS)))` in the chapter content and replace it with the champions table.
    fn replace_champions(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        let Some(m) = re::CHAMPIONS.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();

        let path = chapter_path(chapter, "(((CHAMPIONS)))")?;

        let goals = self.goal_documents(path)?;
        let goal_refs: Vec<&GoalDocument> = goals.iter().collect();
        let format_champions = format_champions(&goal_refs).into_anyhow()?;
        chapter.content.replace_range(range, &format_champions);

        Ok(())
    }

    /// Look for `(((ROADMAPS)))` in the chapter content and replace it with the roadmaps table.
    fn replace_roadmaps(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        let Some(m) = re::ROADMAPS.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();

        let path = chapter_path(chapter, "(((ROADMAPS)))")?;

        let roadmaps = self.roadmap_documents(path)?;
        let roadmap_refs: Vec<&RoadmapDocument> = roadmaps.iter().collect();
        let formatted = goal::format_roadmap_table(&roadmap_refs).into_anyhow()?;
        chapter.content.replace_range(range, &formatted);

        Ok(())
    }

    /// Look for `<!-- TEAM ASKS -->` in the chapter content and replace it with the team asks.
    fn replace_team_asks(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        let Some(m) = re::TEAM_ASKS.find(&chapter.content) else {
            return Ok(());
        };
        let range = m.range();

        let path = chapter_path(chapter, "(((TEAM ASKS)))")?;

        let goals = self.goal_documents(path)?;

        // Separate goals by format
        let mut old_format_asks: Vec<&TeamAsk> = vec![];
        let mut new_format_goals: Vec<&GoalDocument> = vec![];

        for goal in goals
            .iter()
            .filter(|g| g.metadata.status.is_not_not_accepted())
        {
            match &goal.team_involvement {
                TeamInvolvement::Asks(asks) => {
                    old_format_asks.extend(asks.iter());
                }
                TeamInvolvement::Support(_) => {
                    new_format_goals.push(goal);
                }
            }
        }

        // Format both old and new format goals
        let mut formatted = String::new();

        if !old_format_asks.is_empty() {
            formatted.push_str(&format_team_asks(&old_format_asks).into_anyhow()?);
        }

        if !new_format_goals.is_empty() {
            if !formatted.is_empty() {
                formatted.push_str("\n\n");
            }
            formatted.push_str(&format_team_support(&new_format_goals).into_anyhow()?);
        }

        chapter.content.replace_range(range, &formatted);

        Ok(())
    }

    /// Look for `<!-- TEAM ASKS -->` in the chapter content and replace it with the team asks.
    fn replace_valid_team_asks(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        if !re::VALID_TEAM_ASKS.is_match(&chapter.content) {
            return Ok(());
        }
        let config = Configuration::get();
        let rows = std::iter::once(vec![
            Spanned::here("Ask".to_string()),
            Spanned::here("aka".to_string()),
            Spanned::here("Description".to_string()),
        ])
        .chain(config.team_asks.iter().map(|(name, details)| {
            vec![
                Spanned::here(format!("{name:?}")),
                Spanned::here(details.short.to_string()),
                Spanned::here(details.about.to_string()),
            ]
        }))
        .collect::<Vec<Vec<Spanned<String>>>>();
        let table = util::format_table(&rows);
        let new_content = re::VALID_TEAM_ASKS.replace_all(&chapter.content, table);
        chapter.content = new_content.to_string();
        Ok(())
    }

    /// Find the goal documents for the milestone in which this `chapter_path` resides.
    /// e.g., if invoked with `2024h2/xxx.md`, will find all goal documents in `2024h2`.
    fn goal_documents(&mut self, chapter_path: &Path) -> anyhow::Result<Arc<Vec<GoalDocument>>> {
        let Some(milestone_path) = chapter_path.parent() else {
            anyhow::bail!("cannot get goal documents from `{chapter_path:?}`")
        };

        if let Some(goals) = self.goal_document_map.get(milestone_path) {
            return Ok(goals.clone());
        }

        let goal_documents =
            goal::goals_in_dir(&self.ctx.config.book.src.join(milestone_path)).into_anyhow()?;
        let goals = Arc::new(goal_documents);
        self.goal_document_map
            .insert(milestone_path.to_path_buf(), goals.clone());
        Ok(goals)
    }

    /// Find the roadmap documents for the milestone in which this `chapter_path` resides.
    fn roadmap_documents(
        &mut self,
        chapter_path: &Path,
    ) -> anyhow::Result<Arc<Vec<RoadmapDocument>>> {
        let Some(milestone_path) = chapter_path.parent() else {
            anyhow::bail!("cannot get roadmap documents from `{chapter_path:?}`")
        };

        if let Some(roadmaps) = self.roadmap_document_map.get(milestone_path) {
            return Ok(roadmaps.clone());
        }

        let roadmap_documents =
            goal::roadmaps_in_dir(&self.ctx.config.book.src.join(milestone_path)).into_anyhow()?;
        let roadmaps = Arc::new(roadmap_documents);
        self.roadmap_document_map
            .insert(milestone_path.to_path_buf(), roadmaps.clone());
        Ok(roadmaps)
    }

    /// Get or load milestone issues, caching the result for subsequent calls.
    /// This eliminates redundant GitHub API calls within a single preprocessor run.
    fn get_or_load_milestone_issues(
        &mut self,
        milestone: &str,
    ) -> anyhow::Result<Arc<Vec<rust_project_goals::gh::issues::ExistingGithubIssue>>> {
        if let Some(cached_issues) = self.milestone_issues_cache.get(milestone) {
            eprintln!("📦 Using cached issues for milestone: {}", milestone);
            return Ok(cached_issues.clone());
        }

        eprintln!(
            "🌐 Loading issues from GitHub API for milestone: {}",
            milestone
        );
        let repository = rust_project_goals::gh::issue_id::Repository::new("rust-lang", "goals");
        let issues =
            rust_project_goals::gh::issues::list_issues_in_milestone(&repository, milestone)
                .map_err(|e| {
                    anyhow::anyhow!("Failed to load milestone issues for {}: {}", milestone, e)
                })?;

        eprintln!(
            "✅ Loaded {} issues for milestone: {}",
            issues.len(),
            milestone
        );
        let issues = Arc::new(issues);
        self.milestone_issues_cache
            .insert(milestone.to_string(), issues.clone());
        Ok(issues)
    }

    /// Find the end of a markdown table (first line that doesn't start with |).
    /// Returns the byte offset where new rows should be inserted.
    fn find_markdown_table_end(content: &str) -> Option<usize> {
        let lines: Vec<&str> = content.lines().collect();

        // Find first line starting with |
        let table_start = lines
            .iter()
            .position(|line| line.trim_start().starts_with('|'))?;

        // Find first line after table_start that doesn't start with |
        let table_end = lines[table_start..]
            .iter()
            .position(|line| !line.trim_start().starts_with('|'))
            .map(|pos| table_start + pos)
            .unwrap_or(lines.len());

        // Calculate byte offset to the start of the table_end line
        let mut offset = 0;
        for i in 0..table_end {
            if i > 0 {
                offset += 1; // newline
            }
            offset += lines[i].len();
        }

        if table_end < lines.len() {
            // There's a line after the table, insert before it
            Some(offset + 1) // +1 for the newline after the last table row
        } else {
            // Table goes to end of file, append at the end
            Some(content.len())
        }
    }

    /// Automatically inject team names and task owners into the metadata table.
    /// This replaces the need for manual placeholders and combines the logic
    /// to avoid duplicate table parsing.
    fn inject_metadata_rows(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        let Some(chapter_path) = chapter.path.as_ref() else {
            return Ok(()); // No path, nothing to inject
        };

        // Skip template files
        if chapter_path.file_name().and_then(|n| n.to_str()) == Some("TEMPLATE.md") {
            return Ok(());
        }

        // Only process files in milestone directories (like 2024h2, 2025h1, etc.)
        let Some(parent_dir) = chapter_path.parent() else {
            return Ok(());
        };

        let Some(parent_name) = parent_dir.file_name().and_then(|n| n.to_str()) else {
            return Ok(());
        };

        if !parent_name
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_digit())
        {
            return Ok(()); // Not a milestone directory, skip
        }

        // Find the goal document for this chapter
        let goals = self.goal_documents(&chapter_path)?;
        let chapter_in_context = self.ctx.config.book.src.join(chapter_path);

        if let Some(goal) = goals.iter().find(|gd| gd.path == chapter_in_context) {
            // This is a goal document — inject teams and task owners
            let team_names: Vec<String> = goal
                .teams_with_asks()
                .iter()
                .map(|team_name| team_name.name())
                .collect();

            let teams_text = if team_names.is_empty() {
                "(none)".to_string()
            } else {
                team_names.join(", ")
            };

            let task_owners: Vec<String> = goal.task_owners.iter().cloned().collect();

            let task_owners_text = if task_owners.is_empty() {
                "(none)".to_string()
            } else {
                task_owners.join(", ")
            };

            if let Some(table_end) = Self::find_markdown_table_end(&chapter.content) {
                let insertion_text = format!(
                    "| Teams            | {} |\n| Task owners      | {} |\n",
                    teams_text, task_owners_text
                );
                chapter.content.insert_str(table_end, &insertion_text);
            }
        } else if let Some(roadmap) = self
            .roadmap_documents(chapter_path)?
            .iter()
            .find(|rd| rd.path == chapter_in_context)
        {
            // This is a roadmap document — inject task owners from all constituent goals
            let theme = &roadmap.short_title.content;
            let mut all_task_owners: BTreeSet<String> = BTreeSet::new();
            for goal in goals.iter() {
                if goal.metadata.status.is_not_not_accepted() && goal.matches_roadmap_theme(theme) {
                    all_task_owners.extend(goal.task_owners.iter().cloned());
                }
            }

            let task_owners_text = if all_task_owners.is_empty() {
                "(none)".to_string()
            } else {
                all_task_owners.into_iter().collect::<Vec<_>>().join(", ")
            };

            if let Some(table_end) = Self::find_markdown_table_end(&chapter.content) {
                let insertion_text = format!("| Task owners      | {} |\n", task_owners_text);
                chapter.content.insert_str(table_end, &insertion_text);
            }
        }

        Ok(())
    }

    fn replace_reports(&mut self, chapter: &mut Chapter) -> anyhow::Result<()> {
        if !re::REPORTS.is_match(&chapter.content) {
            return Ok(());
        }

        let chapter_path = chapter_path(chapter, "(((REPORTS)))")?.to_path_buf();

        // Parse date range from the placeholder
        let date_range = if let Some(captures) = re::REPORTS.captures(&chapter.content) {
            captures.get(1).map(|m| m.as_str().trim())
        } else {
            None
        };

        // Generate list of months based on date range
        let months = self.generate_month_list(date_range)?;

        // Discover teams with champions
        let goals = self.goal_documents(&chapter_path)?;
        let mut teams_with_champions: BTreeSet<&'static TeamName> = BTreeSet::new();

        for goal in goals.iter() {
            for team_name in goal.metadata.champions.keys() {
                teams_with_champions.insert(team_name);
            }
        }

        let now = chrono::Utc::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S UTC");

        // Generate dynamic chapters instead of files
        self.generate_report_chapters(chapter, &chapter_path, &months, &teams_with_champions)?;

        let replacement = format!(
            "This section contains automatically generated reports based on the comments left in the goal tracking issues.\n\
            \n\
            These reports were last generated at {timestamp}.",
        );

        chapter.content = re::REPORTS
            .replace_all(&chapter.content, replacement)
            .to_string();

        Ok(())
    }

    fn generate_report_chapters(
        &mut self,
        parent_chapter: &mut Chapter,
        chapter_path: &Path,
        months: &[(i32, u32, &'static str)],
        teams_with_champions: &BTreeSet<&'static TeamName>,
    ) -> anyhow::Result<()> {
        // Get the milestone from the chapter path (e.g., "2025h2" from "src/2025h2/reports.md")
        let milestone = chapter_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not determine milestone from chapter path: {:?}",
                    chapter_path
                )
            })?;

        let mut parent_names = parent_chapter.parent_names.clone();
        parent_names.push(parent_chapter.name.clone());
        let mut chapter_index = 1;

        // Generate blog post chapters
        for (year, month, month_name) in months.iter().rev() {
            // Reverse to show newest first
            let blog_content = self.generate_blog_post_content(milestone, *year, *month)?;

            let chapter_name = format!("{} Blog Post", month_name);
            let virtual_path = format!("blog-post-{:04}-{:02}.md", year, month);
            let path = Path::new(&virtual_path);

            let mut blog_chapter =
                Chapter::new(&chapter_name, blog_content, path, parent_names.clone());

            if let Some(mut number) = parent_chapter.number.clone() {
                number.push(chapter_index);
                blog_chapter.number = Some(number);
                chapter_index += 1;
            }

            parent_chapter
                .sub_items
                .push(BookItem::Chapter(blog_chapter));
        }

        // Generate champion report chapters
        for team_name in teams_with_champions {
            let team_name_str = &team_name.data().name;
            // Create a team folder chapter
            let team_chapter_name = format!("{} Team Reports", team_name_str);
            let team_virtual_path = format!("{}/index.md", team_name_str);
            let team_path = Path::new(&team_virtual_path);

            let team_content = format!("# {} Team Champion Reports\n\nThis section contains champion reports for the {} team.", team_name_str, team_name_str);
            let mut team_chapter = Chapter::new(
                &team_chapter_name,
                team_content,
                team_path,
                parent_names.clone(),
            );

            if let Some(mut number) = parent_chapter.number.clone() {
                number.push(chapter_index);
                team_chapter.number = Some(number);
                chapter_index += 1;
            }

            let mut team_parent_names = parent_names.clone();
            team_parent_names.push(team_chapter_name.clone());
            let team_sub_index = 1;

            // Generate the "recent updates" report for this team

            // Reverse to show newest first
            let champion_content =
                self.generate_champion_report_content(milestone, team_name_str)?;

            let report_name = format!("Recent updates");
            let report_virtual_path = format!("{team_name_str}/recent-updates.md");
            let report_path = Path::new(&report_virtual_path);

            let mut report_chapter = Chapter::new(
                &report_name,
                champion_content,
                report_path,
                team_parent_names.clone(),
            );

            if let Some(mut number) = team_chapter.number.clone() {
                number.push(team_sub_index);
                report_chapter.number = Some(number);
            }

            team_chapter
                .sub_items
                .push(BookItem::Chapter(report_chapter));

            parent_chapter
                .sub_items
                .push(BookItem::Chapter(team_chapter));
        }

        Ok(())
    }

    fn generate_blog_post_content(
        &mut self,
        milestone: &str,
        year: i32,
        month: u32,
    ) -> anyhow::Result<String> {
        use chrono::NaiveDate;

        eprintln!(
            "📝 Generating blog post for {}-{:02} (milestone: {})",
            year, month, milestone
        );

        // Calculate start and end dates for the month
        let start_date = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| anyhow::anyhow!("Invalid date: {}-{:02}-01", year, month))?;
        // The `end_date` is an exclusive range, so this will match comments within the given `month`
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or_else(|| anyhow::anyhow!("Invalid end date calculation for {}-{:02}", year, month))?;

        // Get repository from context - assuming rust-lang/goals as default
        let repository = rust_project_goals::gh::issue_id::Repository::new("rust-lang", "goals");

        // Use cached issues for this milestone
        let issues = self.get_or_load_milestone_issues(milestone)?;

        // Use the library function with pre-loaded issues
        let content = rust_project_goals_cli::render_updates(
            &issues,
            &repository,
            milestone,
            Some(&start_date),
            Some(&end_date),
            None,
            false,
            Order::OldestFirst,
        )
        .map_err(|e| anyhow::anyhow!("Failed to generate blog post content: {}", e))?;

        Ok(content)
    }

    fn generate_champion_report_content(
        &mut self,
        milestone: &str,
        team_name: &str,
    ) -> anyhow::Result<String> {
        // Look at the updates for the last ~three months
        let end_date = chrono::Utc::now().date_naive();
        let start_date = end_date - chrono::TimeDelta::days(90);

        eprintln!(
            "👥 Generating champion report for {} team, {start_date} - {end_date} (milestone: {})",
            team_name, milestone
        );

        // Get repository from context - assuming rust-lang/goals as default
        let repository = rust_project_goals::gh::issue_id::Repository::new("rust-lang", "goals");

        // Use cached issues for this milestone
        let issues = self.get_or_load_milestone_issues(milestone)?;

        // Use the library function with team filter (team_name is already in T-teamname format)
        let content = rust_project_goals_cli::render_updates(
            &issues,
            &repository,
            milestone,
            Some(&start_date),
            Some(&end_date),
            Some(team_name),
            false,
            Order::NewestFirst,
        )
        .map_err(|e| anyhow::anyhow!("Failed to generate champion report content: {}", e))?;

        Ok(content)
    }

    fn generate_month_list(
        &self,
        date_range: Option<&str>,
    ) -> anyhow::Result<Vec<(i32, u32, &'static str)>> {
        let (start_date, end_date) = if let Some(range_str) = date_range {
            self.parse_date_range(range_str)?
        } else {
            // Default to current month only if no range specified
            let now = chrono::Utc::now();
            let start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                .ok_or_else(|| anyhow::anyhow!("Invalid current date"))?;
            (start, start)
        };

        let mut months = Vec::new();
        let mut current = start_date;

        while current <= end_date {
            let month_name = match current.month() {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => "Unknown",
            };

            months.push((current.year(), current.month(), month_name));

            // Move to next month
            if current.month() == 12 {
                current = chrono::NaiveDate::from_ymd_opt(current.year() + 1, 1, 1)
                    .ok_or_else(|| anyhow::anyhow!("Invalid date calculation"))?;
            } else {
                current = chrono::NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1)
                    .ok_or_else(|| anyhow::anyhow!("Invalid date calculation"))?;
            }
        }

        Ok(months)
    }

    fn parse_date_range(
        &self,
        range_str: &str,
    ) -> anyhow::Result<(chrono::NaiveDate, chrono::NaiveDate)> {
        // Parse format like "2025-09-01 to 2025-12-31" or "2025-09-01" (with no end date)
        let parts: Vec<&str> = range_str.split(" to ").collect();
        if parts.len() > 2 {
            anyhow::bail!("Invalid date range format: `{range_str}`. Expected: `YYYY-MM-DD to YYYY-MM-DD` or just: `YYYY-MM-DD`");
        }

        let start_date = parts[0].trim();
        let end_date = parts.get(1).map(|s| str::trim(s));

        let start_date = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .with_context(|| format!("Invalid start date: `{}`", parts[0]))?;
        let end_date = if let Some(end_date) = end_date {
            chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
                .with_context(|| format!("Invalid end date: `{}`", parts[1]))?
        } else {
            chrono::Utc::now().date_naive()
        };

        if start_date > end_date {
            anyhow::bail!("Start date must be before or equal to end date");
        }

        Ok((start_date, end_date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_markdown_table_end() {
        let content = "Some text before\n\n| Metadata | Value |\n|----------|-------|\n| Contact | @nikomatsakis |\n| Teams | (none) |\n\nSome text after";

        let result = GoalPreprocessorWithContext::find_markdown_table_end(content);
        assert!(result.is_some());

        let offset = result.unwrap();
        let (before, after) = content.split_at(offset);

        // Should split right before the blank line after the table
        assert!(before.ends_with("| Teams | (none) |\n"));
        assert!(after.starts_with("\nSome text after"));

        // Test that inserting at this offset works correctly
        let mut test_content = content.to_string();
        test_content.insert_str(offset, "| New row | value |\n");
        assert!(test_content.contains("| Teams | (none) |\n| New row | value |\n\nSome text after"));
    }

    #[test]
    fn test_reports_replacement() {
        let mut chapter = Chapter::new(
            "Test Chapter",
            "# Test\n\n(((REPORTS)))\n\nEnd".to_string(),
            "test.md",
            Vec::new(),
        );

        // Test the regex directly
        assert!(re::REPORTS.is_match(&chapter.content));

        // Test the replacement logic with a simple fixed replacement
        // (since we can't easily mock the goal documents in a unit test)
        let now = chrono::Utc::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S UTC");

        let replacement = format!(
            r#"This section contains automatically generated reports based on the comments left in the goal tracking issues.

These reports were last generated at {}.

## Blog post

These are the main blog posts that are published each month:

* [October](./blog-post-2025-10.md)
* [September](./blog-post-2025-09.md)

## Champion reports

These reports include the details only of goals for a particular team.

"#,
            timestamp
        );

        chapter.content = re::REPORTS
            .replace_all(&chapter.content, replacement)
            .to_string();

        // Check that the placeholder was replaced
        assert!(!chapter.content.contains("(((REPORTS)))"));
        assert!(chapter
            .content
            .contains("These reports were last generated at"));
        assert!(chapter.content.contains("## Blog post"));
        assert!(chapter.content.contains("## Champion reports"));
    }

    #[test]
    fn test_date_range_parsing() {
        // Test date range parsing directly
        let parts: Vec<&str> = "2025-09-01 to 2025-12-31".split(" to ").collect();
        assert_eq!(parts.len(), 2);

        let start_date = chrono::NaiveDate::parse_from_str(parts[0].trim(), "%Y-%m-%d").unwrap();
        let end_date = chrono::NaiveDate::parse_from_str(parts[1].trim(), "%Y-%m-%d").unwrap();

        assert_eq!(start_date.year(), 2025);
        assert_eq!(start_date.month(), 9);
        assert_eq!(end_date.year(), 2025);
        assert_eq!(end_date.month(), 12);

        // Test month generation logic
        let mut months = Vec::new();
        let mut current = start_date;

        while current <= end_date {
            months.push((current.year(), current.month()));

            // Move to next month
            if current.month() == 12 {
                current = chrono::NaiveDate::from_ymd_opt(current.year() + 1, 1, 1).unwrap();
            } else {
                current = chrono::NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1)
                    .unwrap();
            }
        }

        assert_eq!(months.len(), 4); // September, October, November, December
        assert_eq!(months[0], (2025, 9));
        assert_eq!(months[1], (2025, 10));
        assert_eq!(months[2], (2025, 11));
        assert_eq!(months[3], (2025, 12));
    }
}
