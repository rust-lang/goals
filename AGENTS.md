# Instructions for goals work

This file provides instructions for contributing to the `goals` repository.  It is intended to be self-contained: everything needed to work effectively in this repository is documented here or cited with a precise path.

## Repository overview

This repository hosts the Rust project's **goal proposals** -- a bottom-up process where contributors propose goals, Rust teams review them, and accepted goals are tracked to completion.  The repository is published as an mdBook site at <https://goals.rust-lang.org/>.

There are two kinds of content:

- **Goal documents** (`src/<milestone>/<name>.md`) -- individual proposals with metadata, motivation, work items, and team asks.

- **Roadmap documents** (`src/<milestone>/roadmap-<theme>.md`) -- narrative pages that group related goals under a unifying theme.

Milestone directories follow the pattern `YYYY` or `YYYYhN` (e.g., `2024h2`, `2025h1`, `2026`).  Starting with 2026, goals are annual rather than semiannual.

## Repository structure

```
src/                    mdBook source
  SUMMARY.md            Book table of contents (manually maintained)
  TEMPLATE.md           Template for new goal proposals
  ROADMAP_TEMPLATE.md   Template for new roadmap documents
  2026/                 Current goal period
    README.md           Overview page (uses preprocessor directives)
    *.md                Goal and roadmap documents
  2025h2/, 2025h1/, ... Prior periods
  about/, how_to/       Explanatory pages
  admin/                Admin/operational docs
crates/
  rust-project-goals/       Core library (parsing, formatting, GitHub API)
  rust-project-goals-cli/   CLI tool (`cargo rpg`)
  mdbook-goals/             mdBook preprocessor
  rust-project-goals-json/  External JSON API types
templates/              Handlebars templates for generated content
book.toml               mdBook and preprocessor configuration
rust-project-goals.toml Configuration for valid team ask types
justfile                Task runner (just check, just serve, just build)
```

## Essential commands

A Cargo alias is defined in `.cargo/config.toml`:

```
cargo rpg <command>   # shorthand for: cargo run -q --bin rust-project-goals-cli -- <command>
```

The commands you need most often:

| Command                   | Action                                                  |
|---------------------------|---------------------------------------------------------|
| `cargo check --workspace` | Verify all Rust code compiles                           |
| `cargo rpg check`         | Validate all goal and roadmap documents parse correctly |
| `cargo test --workspace`  | Run unit tests                                          |
| `just check`              | Same as `cargo rpg check`                               |
| `just serve`              | Build the book and serve it locally (needs `GH_TOKEN`)  |
| `just build`              | Build the mdBook site (needs `GH_TOKEN`)                |

**Before submitting any change**, at minimum run:

1. `cargo check --workspace`
2. `cargo rpg check`

Both of these run in CI (via `.github/workflows/compile.yml` and `.github/workflows/check.yml`).

Note: `cargo rpg check` validates all milestone directories.  A failure in a preexisting file (not one you touched) is not your problem, but you should be aware of it and not mistake it for a regression you introduced.

## Goal document format

Goal documents live at `src/<milestone>/<name>.md`.  The canonical template is `src/TEMPLATE.md`.  Here is the structure:

### Title

The first line must be a level-1 heading (`#`).  This becomes the goal's title.

### Metadata table

Immediately after the title, a two-column markdown table with the header `| Metadata | |`.  The rows below are recognized.  Rows marked "parsed" are extracted by the parser and validated; rows marked "conventional" are used by humans and the preprocessor but the parser does not reject typos in their names.

| Row name                | Required | Kind         | Notes                                                                           |
|-------------------------|----------|--------------|---------------------------------------------------------------------------------|
| `Contact`      | Yes      | Parsed       | Single GitHub `@username`                                                       |
| `Status`                | Yes      | Parsed       | `Proposed`, `Invited`, `Accepted`, `Proposed for mentorship`, or `Not accepted` |
| `Short title`           | No       | Parsed       | Alternate short title for tables; defaults to the `#` heading                   |
| `Tracking issue`        | Note [1] | Parsed       | Must contain `rust-lang/goals#NNN` if present                                   |
| `Other tracking issues` | No       | Conventional | For issues in other repos; use `org/repo#NNN` format                            |
| `Zulip channel`         | No       | Conventional | Link to Zulip stream                                                            |
| `Help wanted`           | No       | Conventional | Informational                                                                   |
| `Roadmap`               | No       | Parsed       | Theme name matching a `roadmap-*.md` short title; repeatable                    |
| `Highlight`             | No       | Parsed       | Theme name for highlight grouping; repeatable                                   |
| `[team] champion`       | Note [2] | Parsed       | Champion from a team, e.g., `[lang] champion`                                   |

[1] Required when Status is `Accepted`; blank for `Proposed` goals.

[2] Required when the corresponding team ask is Medium or Large.  The team name must be a valid Rust team (validated against the `rust-lang/team` repository at runtime).

**Legacy note:** The row name `Flagship` is accepted as a synonym for `Roadmap` for backward compatibility with pre-2026 documents.

**Auto-injected rows:** The mdBook preprocessor automatically appends `Teams` and `Task owners` rows during rendering.  Do not add these manually.

### Required sections

Every goal document must contain:

- `## Summary` -- a summary.
- `## Motivation` -- contains subsections such as `### The status quo` and `### What we propose to do about it`.
- A work items table (under `### Work items over the next year`): `| Task | Owner(s) | Notes |`.
- `## Team asks` -- a table with columns `| Team | Support level | Notes |`.
- `## Frequently asked questions` -- a FAQ.

### Team asks table

```markdown
| Team       | Support level | Notes                          |
| ---------- | ------------- | ------------------------------ |
| [compiler] | Small         |                                |
| [lang]     | Medium        | Design meeting needed          |
| [types]    | Large         | Soundness review               |
| [libs]     | Vibes         |                                |
```

Team names use markdown link-reference syntax: `[cargo]`, `[compiler]`, `[lang]`, `[libs]`, `[opsem]`, `[types]`, etc.  These are resolved to team pages by the preprocessor.

Valid support levels:

| Level    | Meaning                                                |
|----------|--------------------------------------------------------|
| `Vibes`  | No action needed; just want to know the team likes it  |
| `Small`  | Only routine activities (e.g., reviewing a few PRs)    |
| `Medium` | Dedicated support from one person; requires a champion |
| `Large`  | Deeper review from the full team; requires a champion  |

**Pre-2026 format:** Milestones before 2026 use a different team asks structure.  The section is named `## Ownership and team asks` and the table has columns `| Task | Owner(s) or team(s) | Notes |`.  Rows that represent team asks (as opposed to personal work items) are marked with `![Team][]` before the team name.  When editing pre-2026 goal files, preserve the existing format.

### Other tracking issue format

When referencing tracking issues in other repositories, prefer the explicit `org/repo#NNN` format (e.g., `rust-lang/rust#31844`).  A bare `#NNN` is ambiguous because it resolves to this repository.

## Roadmap document format

Roadmap documents must be named `roadmap-<theme-slug>.md` and live alongside goal documents in the milestone directory.  The structural template is `src/ROADMAP_TEMPLATE.md`, but note that the template is currently out of date: it claims roadmaps have no metadata table, whereas all actual roadmap files do.  Use an existing roadmap in `src/2026/` (e.g., `roadmap-beyond-the-ampersand.md`) as a model.

Roadmaps have a simpler metadata table:

| Row name           | Required | Notes                                           |
|--------------------|----------|-------------------------------------------------|
| `Short title`      | No       | Used to match goals' `Roadmap` rows             |
| `What and why`     | Yes      | One-line summary                                |
| `Contact` | Yes      | `@username` or "TBD"                            |
| `Application area` | No       | Category (repeatable); e.g., "Network services" |

Roadmaps must contain `## Summary`, `## Motivation` (with `### The status quo`, `### What we are shooting for`, `### Key use cases`, `### Design axioms`), `## 2026 goals` (containing a `(((ROADMAP GOALS: Theme name)))` directive), and `## Frequently asked questions`.

Goals reference roadmaps by adding a `| Roadmap | Theme name |` row to their metadata table, where "Theme name" matches the roadmap's `Short title` (or its `#` heading if no short title is set).  A goal may belong to multiple roadmaps.

## mdBook preprocessor directives

The preprocessor replaces `(((DIRECTIVE)))` markers in markdown files with generated content.  The most common directives:

| Directive                      | Generates                                           |
|--------------------------------|-----------------------------------------------------|
| `(((GOALS)))`                  | Table of all accepted/proposed goals                |
| `(((ROADMAP GOALS)))`          | Table of goals tagged with any roadmap              |
| `(((ROADMAP GOALS: Theme)))`   | Table filtered to a specific roadmap theme          |
| `(((OTHER GOALS)))`            | Table of goals not in any roadmap                   |
| `(((LARGE GOALS)))`            | Goals with Large support asks                       |
| `(((MEDIUM GOALS)))`           | Goals with Medium support asks (and not Large ones) |
| `(((SMALL GOALS)))`            | Goals with Small/Vibes asks only                    |
| `(((HIGHLIGHT GOALS: Theme)))` | Sections for goals with a matching Highlight        |
| `(((TEAM ASKS)))`              | Team ask tables                                     |
| `(((CHAMPIONS)))`              | Champions table                                     |
| `(((GOAL CHAPTERS)))`          | Creates subchapters for each goal                   |
| `(((ROADMAP CHAPTERS)))`       | Creates subchapters for each roadmap                |
| `(((ROADMAPS)))`               | Table of all roadmap documents                      |
| `(((ROADMAPS: Area)))`         | Roadmaps filtered by application area               |
| `(((APPLICATION AREAS)))`      | Application areas and their roadmaps                |
| `(((#GOALS)))`                 | Count of goals                                      |
| `(((#ROADMAP GOALS)))`         | Count of roadmap-tagged goals                       |
| `(((VALID TEAM ASKS)))`        | Table of valid team ask types                       |
| `(((REPORTS)))`                | Blog post and champion report subchapters           |
| `(((GOALS NOT ACCEPTED)))`     | Table of not-accepted goals                         |

**Legacy synonyms:** For backward compatibility with pre-2026 files, `FLAGSHIP GOALS` is accepted wherever `ROADMAP GOALS` appears (including the filtered and count variants).

Individual goal files are **not** listed in `SUMMARY.md`.  They are discovered automatically by scanning the milestone directory.  The `(((GOAL CHAPTERS)))` directive creates book subchapters for them dynamically.

## Crate architecture

- **`rust-project-goals`** (library): Core parsing (`goal.rs`, `markwaydown.rs`), formatting (`format_team_ask.rs`, `format_team_support.rs`, `format_champions.rs`), markdown link/team processing (`markdown_processor.rs`), GitHub API (`gh/`), configuration (`config.rs`), regex patterns (`re.rs`), team data (`team.rs`), and utilities (`util.rs`).

- **`rust-project-goals-cli`**: The `cargo rpg` CLI.  Entry point is `crates/rust-project-goals-cli/src/main.rs`.

- **`mdbook-goals`**: The mdBook preprocessor.  The core logic is in `crates/mdbook-goals/src/goal_preprocessor.rs`.

- **`rust-project-goals-json`**: Small crate defining the external JSON API types for tracking issue data.

Team names are validated at runtime against data fetched from the `rust-lang/team` repository (via the `rust_team_data` crate).

## Conventions for contributing

### Goal documents

When adding or editing a goal document:

1. Copy `src/TEMPLATE.md` to `src/<milestone>/<name>.md`.

2. Fill in all required metadata rows (`Contact`, `Status`).

3. Include all required sections.

4. Use `org/repo#NNN` format for external tracking issue references, not bare `#NNN`.

5. Reference teams as `[team-name]` in the team asks table.

6. Run `cargo rpg check` to validate the document parses.

7. The preprocessor and book build require a `GH_TOKEN` environment variable for some operations (e.g., fetching team data, milestone issues).  For offline work, `cargo rpg check` and `cargo check --workspace` are sufficient.

### Roadmap documents

When adding or editing a roadmap document:

1. Use an existing roadmap in `src/2026/` as a model (e.g., `roadmap-beyond-the-ampersand.md`).  The file must be named `roadmap-<theme-slug>.md`.

2. Include the metadata table with at least `What and why` and `Contact`.

3. Include all required sections (see the roadmap format section above).

4. Ensure the `(((ROADMAP GOALS: Theme name)))` directive uses the exact theme name that goals will reference in their `Roadmap` metadata rows.

5. Run `cargo rpg check` to validate the document parses.

## Rust code conventions

- All crates use Rust edition 2021.
- No `rustfmt.toml` or `clippy.toml` exists; use default `rustfmt` and `clippy` settings.
- The dev profile uses `opt-level = 1`.
- Prefer the module-per-file layout (`src/name.rs`) over the directory layout (`src/name/mod.rs`).

## CI checks

Three GitHub Actions workflows are defined.  The first two run on pushes and PRs to `main`; the third runs only on pushes to `main` (and on a cron schedule):

| Workflow           | File              | What it runs                    | Trigger            |
| ------------------ | ----------------- | ------------------------------- | ------------------ |
| Validate markdown  | `check.yml`       | `just check`                    | push, PR           |
| Compile Rust code  | `compile.yml`     | `cargo check --workspace`       | push, PR           |
| Deploy mdBook      | `mdbook.yml`      | `just build` + deploy to Pages  | push, cron, manual |

The first two are the gate checks for pull requests.  The `just check` command is equivalent to `cargo rpg check` (see the commands table above).
