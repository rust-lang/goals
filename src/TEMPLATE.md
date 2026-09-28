# TEMPLATE (replace with title of your goal)

> **Instructions:** Copy this template to a fresh file with a name based on your plan.
> Give it a title that names the overall theme of your goal
> (e.g., "Const Generics" or "Polonius" or "Cargo Script").
> Use the `What and why` metadata row for a more readable one-liner
> (e.g., "Permit structs/enums to be used as the value of a const generic parameter").
> Feel free to replace any text with anything, but there are placeholders
> designed to help you get started.
> See the [goal format reference](./about/goal-format.md) for full details on the format.
>
> The **contact** is the person responsible for providing updates.
>
> The **status** should be **Proposed** for new goals or **Accepted** once approved.
>
> If your goal needs a **contributor**, add a `## Help wanted` section (see below).
>
> If your goal needs **funding**, add a `## Funding` section (see below).

| Metadata         |                                                                                  |
| :--------------- | -------------------------------------------------------------------------------- |
| Contact | *must be a single GitHub username like @ghost*                                   |
| Status           | Proposed                                                                         |
| Tracking issue   | *if this is a continuing goal, add the old tracking issue, else remove the row*     |
| Zulip channel    | N/A (an existing stream can be re-used or new streams can be created on request) |

## Summary

*Short description of what you will do over the next year.*

## Motivation

### The status quo

> *Elaborate in more detail about the problem you are trying to solve. This section is making the case for why this particular problem is worth prioritizing with project bandwidth. A strong status quo section will (a) identify the target audience and (b) give specifics about the problems they are facing today. Sometimes it may be useful to start sketching out how you think those problems will be addressed by your change, as well, though it's not necessary.*

### What we propose to do about it

> *Explain your overall approach to solving the problem. Explain your design philosophy (including design axioms). Focus your discussion on what you aim to get done this year, but it is good to also give a sense for the "overall goal" you are working towards, if it extends beyond the work for this year. Team(s) should give you feedback on whether they are aligned both with your short-term and longer-term goals.*

### Work items over the next year

> *Sketch out the specific things you are trying to achieve in this goal period. This should be short and high-level -- we don't want to see the design!*

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Do the work | *owner*  |       |

> **Subgoals:** If your goal has distinct workstreams, you can break them into
> subgoals using `####` headings. Each subgoal should have a specific, actionable
> title and its own task table. For example, a goal titled "Full Const Generics"
> might have subgoals "ADT const params" and "Min generic const arguments."
>
> Subgoals can optionally have their own `| Metadata | |` table to override
> `Roadmap`, `Timespan`, or `What and why` from the parent goal.
> See the [goal format reference](./about/goal-format.md) for details.

## Team asks

> This section outlines what support you need from the Rust teams. For each team, identify the level of support you need:
>
> * Small: You only need the team to do its routine activities, or you simply need the team's approval.
>     * *Example:* Prototyping a new feature on crates.io that you hope to eventually upstream.
>     * *Example:* Conducting research that might eventually become a language feature.
>     * *Example:* A compiler change that will require a few small PRs to be reviewed.
>     * *Example:* Asking the lang team to approve a lint.
> * Medium: You need dedicated support from one person, but the rest of the team doesn't have to do much.
>     * *Example:* A compiler change that doesn't require any rearchitecting but 
>     * *Example:* Implementing a small, noncontroversial language feature.
> * Large: You need deeper review from the entire team.
>     * *Example:* Rearchitecting part of the compiler.
>     * *Example:* Implementing a complex language feature that will require design meetings.
>
> If you're not sure, leave it blank, the project goals team can help.
>
> "Small" asks require someone on the team to "second" your goal; "Medium" and "Large" asks require a dedicated champion from the team. If you don't have a second or a champion, the project goals team will help you find them, don't worry about it.

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [cargo]    |               |                                         |
| [compiler] |               |                                         |
| [infra]    |               |                                         |
| [lang]     |               |                                         |
| [libs]     |               |                                         |
| [opsem]    |               |                                         |
| [types]    |               |                                         |
| ...        | ...           | *Feel free to add rows for other teams* |

## Help wanted

> **Instructions:** If your goal needs a contributor, fill in the table below
> describing the work, the experience level needed, and the expected time investment.
> The presence of this section signals that the goal is looking for someone to do the work.
> The project goals team will try to help you find someone.
> Delete this section if your goal does not need a contributor.

| Task | Experience level | Time investment |
|------|-----------------|-----------------|
| *e.g., Implement the feature* | *e.g., Intermediate Rust* | *e.g., 3 months part-time* |

## Funding

> **Instructions:** If your goal needs funding to proceed, fill in the table below
> describing what the funding would cover and the estimated cost.
> The presence of this section signals that the goal is looking for financial support.
> Delete this section if your goal does not need funding.
>
> If you have a specific funding contact (e.g., an organization that can
> receive and administer funds), add a `Funding contact` row to the
> metadata table at the top of this file. If omitted, it defaults to the
> [Rust Funding team](https://rust-lang.org/governance/teams/launching-pad/#team-funding).

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| *e.g., Contributor (6 months, full-time)* | *e.g., $60,000* | No | |
| *e.g., Maintenance* | *e.g., $10,000* | No | |

> If you don't know the expected cost, enter `TBD`. If you do, but prefer to not have it public on the internet, enter `Ask` and the readers will be directed to ask the *Funding contact* about the cost.

## Frequently asked questions

### What do I do with this space?

*This is a good place to elaborate on your reasoning above -- for example, why did you put the design axioms in the order that you did? It's also a good place to put the answers to any questions that come up during discussion. The expectation is that this FAQ section will grow as the goal is discussed and eventually should contain a complete summary of the points raised along the way.*
