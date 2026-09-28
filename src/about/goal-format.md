# Goal format

Each project goal is a single markdown file with a standard structure: a metadata table, prose sections explaining the motivation, work items with task ownership, and team asks. This page documents the format in detail.

## Goal sections

### Metadata table

Every goal begins with a `# Title` heading followed by a metadata table:

```markdown
| Metadata         |                          |
|:-----------------|--------------------------|
| Contact | @username                |
| Status           | Proposed                 |
```

The recognized fields are:

| Field | Required? | Notes |
|-------|-----------|-------|
| **Contact** | Yes | A single GitHub username like `@ghost`. This person is responsible for driving the goal and providing status updates. |
| **Status** | Yes | One of `Proposed`, `Accepted`, `Completed`, `Discontinued`, or `Not accepted`. |
| **Short title** | No | A shorter display name. Defaults to the `#` heading if omitted. |
| **What and why** | No | A readable one-liner used in roadmap table cells. If omitted, the first sentence of the Summary section is used instead. |
| **Tracking issue** | If Accepted | Must reference an issue in the goals repository, e.g. `rust-lang/goals#274`. Required for accepted goals; leave blank or omit for proposed goals. |
| **Other tracking issues** | No | Additional issue references in other repositories, e.g. `rust-lang/rust#44874`. |
| **Zulip channel** | No | A link to the relevant Zulip stream for discussion. |
| **Roadmap** | No | The name of a roadmap theme this goal belongs to, e.g. `Rust for Linux`. Can appear multiple times if the goal spans several roadmaps. |
| **Highlight** | No | A category name for the highlights page. Can appear multiple times. |
| **Timespan** | No | Overrides the default goal period, e.g. `2026-2027` for multi-year goals. |
| **Funding contact** | No | Freeform text (may include markdown links) identifying who to contact about funding this goal. Defaults to the [Rust Funding team](https://rust-lang.org/governance/teams/launching-pad/#team-funding) if omitted. Only relevant for goals with a `## Funding` section. |
| **\[team\] champion** | No | The champion for a specific team, e.g. `[lang] champion \| @someone`. Medium and Large team asks require a champion. |
| **Teams** | *Auto-injected* | Filled in automatically from team asks. Do not add this row yourself. |
| **Task owners** | *Auto-injected* | Filled in automatically from work item tables. Do not add this row yourself. |

**Multiple-value fields:** `Roadmap` and `Highlight` support multiple values by repeating the row:

```markdown
| Roadmap          | Rust for Linux           |
| Roadmap          | Beyond the &             |
```

### Summary

The `## Summary` section should be one or two sentences describing what the goal will accomplish. Keep it concise — the first sentence is used as a fallback for the "What and why" text in roadmap tables if no explicit `What and why` metadata is provided.

### Motivation

The `## Motivation` section makes the case for why this goal is worth pursuing. See [Goal motivations](./motivation.md) for detailed guidance. Common subsections:

- **The status quo** — who is affected and what problems they face today
- **What we propose to do about it** — the approach and design philosophy
- **Design axioms** (optional) — prioritized principles guiding tradeoffs
- **The "shiny future" we are working towards** (optional) — longer-term context

### Work items and subgoals

The `## Work items over the next year` section describes the concrete work to be done, organized as task tables.

**Task tables** list specific work items with ownership:

```markdown
| Task                     | Owner(s)      | Notes |
|--------------------------|---------------|-------|
| Publish and merge RFC    | @username     |       |
| Implement the feature    | @alice, @bob  |       |
| Write documentation      | ![Help Wanted][] |    |
```

- **Owner(s)** should be GitHub usernames prefixed with `@`
- Use `![Help Wanted][]` for tasks that need a volunteer

**Subgoals** use `####` headings within the work items section to break out distinct workstreams. Each subgoal gets its own task table and an optional prose description. See the FAQ below for guidance on [when to use subgoals](#when-should-i-use-subgoals).

For example, the [Full Const Generics](../2026/const-generics.md) goal uses subgoals to separate two independent workstreams:

```markdown
### Work items over the next year

#### ADT const params

Support structs, tuples, arrays in const generics.

| Task                                        | Owner(s) | Notes |
|---------------------------------------------|----------|-------|
| Publish and merge `adt_const_params` RFC    | @BoxyUwU |       |
| Model `adt_const_params` in a-mir-formality | @BoxyUwU |       |
| Stabilize `adt_const_params`                | @BoxyUwU |       |

#### Min generic const arguments

Support associated constants and generic parameters in expressions.

| Task                                           | Owner(s) | Notes |
|------------------------------------------------|----------|-------|
| Finish `min_generic_const_args` implementation | @BoxyUwU |       |
| Prototype "full" generic const args            | @BoxyUwU |       |
```

You can also have a task table at the top level *and* subgoals — tasks before the first `####` heading belong to the goal as a whole.

**Subgoal metadata:** Each subgoal can optionally include its own metadata table to override or extend the parent goal's metadata:

| Field | Effect |
|-------|--------|
| **Roadmap** | Additional roadmap theme(s), combined with the parent goal's themes |
| **Timespan** | Overrides the parent goal's timespan for this subgoal |
| **What and why** | Overrides the parent goal's description for this subgoal |

```markdown
#### My subgoal

| Metadata    |                     |
|-------------|---------------------|
| Roadmap     | Some other roadmap  |
| What and why| Specific description for this workstream |

| Task        | Owner(s) | Notes |
|-------------|----------|-------|
| Do the work | @owner   |       |
```

If no subgoal `What and why` is provided, the first sentence of the subgoal's prose description is used as a fallback.

### Team asks

The `## Team asks` section specifies what support the goal needs from Rust teams:

```markdown
| Team       | Support level | Notes                          |
|------------|---------------|--------------------------------|
| [lang]     | Large         | Stabilization decisions        |
| [compiler] | Small         | Reviews                        |
```

**Support levels:**

- **Small** — routine activities only: approvals, small PR reviews, lint decisions
- **Medium** — dedicated support from one person on the team
- **Large** — deeper review from the entire team, design meetings, rearchitecting

Medium and Large asks require a **champion** from the team, declared via a `[team] champion` row in the metadata table. If you don't have a champion yet, the goals team will help you find one.

See [Team asks](./team_asks.md) for the full list of recognized ask types.

### Help wanted

The `## Help wanted` section is optional. Include it if your goal needs a contributor to do the work. It contains a table with three columns:

```markdown
## Help wanted

| Task | Experience level | Time investment |
|------|-----------------|-----------------|
| Implement the feature | Intermediate Rust | 3 months part-time |
| Write documentation | Beginner | 2 weeks |
```

The presence of this section signals that the goal is looking for someone to step up. Goals that already have all the contributors they need should omit this section entirely. The project goals team will help connect goals with potential contributors.

### Funding

The `## Funding` section is optional. Include it if your goal needs financial support to proceed. It contains the following table:

```markdown
## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor (6 months, full-time) | $60,000 | No | |
| Maintenance | $10,000 | Partial | Acme Corp |
```

The presence of this section signals that the goal is looking for funding. Goals that do not need funding should omit this section entirely. The table rows describe what the funding would cover (purpose), the estimated cost, the current funding status, and any sponsors. Funded values are:

- `No` — not yet funded
- `Partial` — partially funded or in discussion
- `Full` — fully funded

To specify who potential funders should contact, add a `Funding contact` row to the metadata table at the top of the file. This can be freeform text with markdown links, e.g. `[Trifecta Tech Foundation](https://trifectatech.org/)`. If omitted, it defaults to the [Rust Funding team](https://rust-lang.org/governance/teams/launching-pad/#team-funding).

### Frequently asked questions

The `## Frequently asked questions` section is a place for elaboration and discussion. Use it to explain design decisions, answer questions raised during review, and summarize points from the goal discussion. This section typically grows over time as the goal is discussed.

### File naming

Goal files live in the milestone directory (e.g. `src/2026/`) and should have a descriptive kebab-case name: `src/2026/const-generics.md`, `src/2026/cargo-semver-checks.md`. The filename does not need to match the title exactly but should be recognizable.

## Frequently asked questions

### How should I name my goal?

Choose names carefully — they appear in tables, summaries, and roadmaps throughout the site.

- **Goal title** (the `#` heading) should name the overall theme:
  *"Full Const Generics"*, *"Arbitrary Self Types"*, *"Stabilize and model Polonius Alpha"*

- **Subgoal titles** (`####` headings under work items) should identify specific actionable deliverables:
  *"ADT const params"*, *"Min generic const arguments"*, *"Stabilize polonius alpha"*

- **"What and why"** metadata gives a more readable description for roadmap tables:
  *"Permit structs/enums to be used as the value of a const generic parameter"*

The goal title is the brand; subgoals are the deliverables; "What and why" is the elevator pitch.

### When should I use subgoals?

Use `####` subgoals when:

- **You have distinct pieces of work** that you want tracked and cited separately. Subgoals appear as separate entries in roadmap tables and can have their own metadata, so if different workstreams belong on different roadmaps, subgoals let you express that.

- **Your goal has a broad, multi-year theme** and you want to identify the precise actions you'll be taking *this year*. For example, a goal titled "Full Const Generics" is a long-running effort — the subgoals "ADT const params" and "Min generic const arguments" name exactly what's being delivered in the current goal period.

If your goal has a single workstream with a straightforward set of tasks, a flat task table under `## Work items over the next year` is simpler and perfectly fine.
