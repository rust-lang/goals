# `rfc`, export RFC text

The `cargo rpg rfc` command exports a single self-contained markdown document suitable for inclusion in the [rust-lang/rfcs](https://github.com/rust-lang/rfcs) repository. It is intended for use when [preparing the RFC](./prepare_rfc.md) at the start of a goal period.

## Usage

```
cargo rpg rfc src/2026
```

The command writes to stdout. Redirect to a file or pipe through `pbcopy` / `xclip` as needed:

```
cargo rpg rfc src/2026 > /tmp/project-goals-2026.md
```

## What it does

The goal website is an mdbook with many interlinked pages — one README per timeframe, separate pages for highlights, roadmaps, individual goals, and so on. An RFC needs to be a *single document* that can be read on its own. The `rfc` command bridges this gap by assembling the mdbook content into one flat markdown file with all internal structure preserved.

At a high level, the command:

1. Runs `mdbook build` to expand directives (goal tables, counters, etc.)
2. Reads the rendered README for the timeframe as a skeleton
3. Inlines linked chapter files in place
4. Rewrites all internal `.md` links to point at the published GitHub Pages site
5. Collects and deduplicates reference link definitions

The result is a document where the README's prose is interleaved with the full content of each chapter, all headings are adjusted to nest correctly, and every link points to a stable URL.

## How it works

### Step 1: Build the book

The command runs `mdbook build`, which invokes the `mdbook-goals` preprocessor. This expands all `(((directives)))` — goal tables, goal counts, username links, etc. — producing rendered markdown files under `book/markdown/`.

For example, `src/2026/README.md` becomes `book/markdown/2026/index.md`, and `src/2026/highlights.md` becomes `book/markdown/2026/highlights.md`.

### Step 2: Read the rendered README as the skeleton

The rendered `index.md` serves as the document skeleton. It contains the full README prose with all directives already expanded, plus the **link-list lines** that reference chapter files.

A link-list line looks like this:

```markdown
- [Highlights](./2026/highlights.md)
```

These lines come from the source README's guide-level and reference-level sections. The key insight is that the mdbook preprocessor does *not* strip these lines — it passes them through as-is. This means the rendered file is both the expanded skeleton (directives resolved) and the inline map (link lines mark where to insert chapter content).

### Step 3: Inline linked files

The command scans the skeleton line by line. When it encounters a link-list line matching the pattern `- [Text](./path/to/file.md)`, it:

1. Reads the corresponding rendered file from `book/markdown/`
2. Strips the file's top-level `#` heading (since the link-list line already introduces the chapter contextually)
3. Bumps all remaining heading levels by one (`##` becomes `###`, etc.) so that chapter content nests under the README's existing heading structure
4. Replaces the link-list line with the processed content

Files that are referenced but don't exist in the rendered output produce a warning and are skipped.

### Step 4: Rewrite links

After inlining, any remaining `.md` links in the document are rewritten to point at the GitHub Pages site. For example:

| Original | Rewritten |
|----------|-----------|
| `./const-generics.md` | `https://goals.rust-lang.org/2026/const-generics.html` |
| `../about/roadmaps.md` | `https://goals.rust-lang.org/about/roadmaps.html` |

Links within the same timeframe directory get the timeframe prefix; links that already contain a `/` or reference a parent directory are treated as absolute paths from the site root.

### Step 5: Collect reference links

Markdown reference link definitions (like `[username]: https://github.com/username`) are separated from the body of both the skeleton and every inlined file. They are deduplicated by label (first definition wins) and appended as a single block at the end of the document. This keeps the RFC body clean while preserving all the reference links needed for usernames, badges, and other references.

## Backward compatibility

The inline logic is driven entirely by the presence of link-list lines in the rendered README. Older timeframes (2025h2, 2024h2) whose READMEs don't use link-list lines produce a valid RFC with just the README content — the inlining step is effectively a no-op. This means the same command works across all timeframes without special-casing.
