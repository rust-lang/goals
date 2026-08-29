# Stabilize FLS Release Cadence

| Metadata         |                                                                                                                |
| :--              | :--                                                                                                            |
| Contact | @PLeVasseur                                                                                                    |
| Status           | Accepted                                                                                                       |
| What and why     | Predictable FLS releases within six weeks of each Rust stable release, so safety assessors have a current spec |
| Roadmap          | Safety-Critical Rust                                                                                           |
| Tracking issue   | [rust-lang/goals#651]                                                                             |
| Zulip channel    | [#t-lang/fls](https://rust-lang.zulipchat.com/#narrow/channel/520710-t-lang.2Ffls)                             |
| [fls] champion   | @PLeVasseur                                                                                                    |


## Summary

Establish a predictable release cadence for the FLS, with each version published within six weeks of the corresponding Rust stable release.

## Motivation

### The status quo

The FLS serves as one specification for the Rust language. For it to be useful as a reference, it needs to track stable Rust releases in a timely manner.

In 2025H2, the [FLS capabilities goal][fls-2025h2] explored options for sustainable FLS maintenance and established an FLS team. That exploratory work identified a six-week release cadence as achievable. This goal formalizes that cadence and demonstrates it in practice.

[fls-2025h2]: https://goals.rust-lang.org/2025h2/FLS-up-to-date-capabilities.html

### What we propose to do about it

We will establish a six-week release cadence: FLS version N will be published within six weeks of Rust stable version N. This means FLS N is available before Rust N+1 ships, which aligns with Rust's release train and leaves adequate time for specification work while remaining timely.

This involves setting up the tooling and processes to sustain this cadence, then demonstrating it works by shipping on schedule for the releases in this goal period.

#### Design axioms

- **Predictability**: Users can plan around a known schedule. FLS N arrives before Rust N+1 ships.
- **Timeliness**: Six weeks balances thoroughness with relevance; the specification remains current.
- **Sustainability**: The cadence is achievable with existing FLS team capacity, avoiding burnout or quality degradation.
- **Process clarity**: Working on releases sequentially keeps the scope of each release well-defined.
- **Ecosystem alignment**: The cadence mirrors Rust's release train, making it easy to communicate and reason about.

#### The "shiny future" we are working towards

A predictable FLS release cadence is foundational infrastructure for broader goals:

- **Safety-critical adoption**: Organizations pursuing certification (ISO 26262, IEC 61508, DO-178C) require specification documents that track the language version they are certifying against. Predictable FLS releases make it feasible to plan certification timelines around known specification availability.
- **Tighter integration with Rust releases**: Once the six-week cadence is proven, we can explore starting specification work earlier by tracking Beta release note issues, giving a 12-week window. This may eventually enable same-week or same-day FLS releases aligned with Rust stable.
- **Tooling and automation**: A consistent cadence creates opportunities for automated checks, diff generation, and integration with other Rust documentation infrastructure. One advantage of such automation could be generation of the FLS changelog, which at the moment is a manual process.

#### Phased approach

**Phase 1: Stable release notes (this goal period)**

We will target starting from when the release notes are made available for the Stable Rust compiler release. In 2025 this is how we operated and would give us a six week cycle maximum to produce FLS versions.

A benefit of this approach is that we are working on each FLS release in sequential fashion, able to keep the items to complete clear.

**Phase 2: Beta release note issues (future)**

Once we've demonstrated the ability to do releases on a consistent cadence from the Stable compiler releases, we will investigate having a 12-week lookback by starting from the release note issues opened when the Beta release of the compiler is cut.

There are benefits in examining the Beta release notes issues. For one, this will help with having more than the Rust Reference writers looking at them, to allow for questions to be raised and potentially improve the rigor of release notes. The FLS Team will become better integrated into the earlier process that the Reference writers undertake.

When we begin having 12-week lookback to Beta, it's possible we'll have multiple releases being worked on at a time. We'll likely need to maintain a way of having the N and N-1 items not be mixed together in an FLS version, before it's tagged for release. This is a solvable problem, but will require more process and enforced discipline.

### Work items over the next year

| Task                            | Owner(s)    | Notes |
| ------------------------------- | ----------- | ----- |
| Overall coordination            | @PLeVasseur |       |
| FLS authoring and review        | ![Team][] [fls] |       |
| Ship FLS 1.93 through 1.101     | [Team][] [fls] | Each within 6 weeks of corresponding Rust stable |
| Investigate work of Beta release issues lookback | @PLeVasseur | Learn how Reference writers approach, apply best practices |

## Team asks

| Team   | Support level | Notes |
| ------ | ------------- | ----- |
| [spec] | Small         | Alignment on release cadence goal |
| [fls]  | Large        | Core work of authoring and releasing FLS versions on schedule |

## Frequently asked questions

### Why six weeks instead of same-day releases?

Specification work takes time. Six weeks provides a reasonable window to incorporate changes from a Rust release while still being timely. It also aligns neatly with Rust's release train: FLS N is ready before Rust N+1 ships.

### What if a Rust release has unusually large specification impact?

We may occasionally slip past six weeks for releases with major language changes. The goal is to establish six weeks as the norm, not as an absolute guarantee. If slips become common, we will revisit the cadence.

### Does this require additional team members?

No. This goal is scoped to be achievable with existing FLS team capacity.
