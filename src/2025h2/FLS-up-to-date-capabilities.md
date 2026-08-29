# Develop the capabilities to keep the FLS up to date

| Metadata             |                                    |
| :--                  | :--                                |
| Contact     | @PLeVasseur                        |
| Status               | Proposed                           |
| Tracking issue       | [rust-lang/goals#391] |
| Zulip channel        | #t-spec                            |
| [bootstrap] champion | @kobzol                            |
| [lang] champion      | @nikomatsakis                      |
| [spec] champion      | @PLeVasseur                        |


## Summary

Develop the capabilities and capacity to keep the FLS up to date with the Rust language.

## Motivation

The FLS was graciously transferred from Ferrous Systems to the Rust Project. In 2025H1, a [Project goal](https://goals.rust-lang.org/2025h1/spec-fls-publish.html) to bring the FLS in and publish a version under the auspicious of the Rust Project was successfully [completed](https://github.com/rust-lang/goals/issues/265#issuecomment-3019529070). Let's now take this to the next level by ensuring that we can keep the FLS up to date with the Rust language in a sustainable manner by developing the necessary capacity and capabilities for doing so.

### The status quo

The target audience here will be those that have a vested interest in Rust specification work and want to see movement forward in ensuring that our documents are as correct and complete as possible.

Keeping up with documentation is not always the most glamorous role for any project. And folks like @ehuss and others are doing an outstanding job at trying to keep the Reference current with actual code behavior. This is a hard job just for one document like the Reference. It will be even more difficult adding another core document like the FLS. But that doesn't necessarily mean we can't keep the FLS as up to date as is needed by its users.

### Solicitation of stakeholder involvement

The FLS is an enabler document for safety qualifying versions of the Rust compiler. The safety-critical community of Rust users actively and passively benefits from the FLS.

Active users of the FLS should be consulted on changes with a "do no harm" approach taken to ensure the capability to use the FLS for safety qualification of the Rust compiler is not disrupted. Active users consulted and contributing to maintenance will ensure the ability to achieve safety qualification from an assessor.

Passive beneficiaries of the FLS include those entities that themselves do not safety qualify a compiler, but instead because the FLS exists and can be used for that purpose, grows the safety-critical set of users of Rust. Examples of passive beneficiaries include: tool vendors, library vendors, and integrators. Those passive beneficiaries becoming involved with maintenance and enrichment of the FLS supports the capability to have safety-qualified Rust compilers. Safety-qualified Rust compilers is a prerequisite for certain levels of safety-criticality in certain industries. These passive beneficiaries contributing back to the ability to ship safety-critical Rust software is a healthy model to ensure continuity of FLS maintenance.

### The next 6 months

Explore options for developing the capability to keep the FLS updated with the Rust language, in a sustainable way, at the cadence needed by its users and stakeholders.

The outcome of the next six months is variable. The entire six months could be investigative, with some prototyping. Or we could establish a concrete cadence and capacity for updating the FLS.

### The "shiny future" we are working towards

The shiny future we are working towards is to ensure that we have the capability in place to keep the FLS updated at the pace needed by its users and stakeholders.

## Ownership and team asks

**Owner:** @PLeVasseur will champion this project goal.

| Task                               | Owner(s) or team(s)            | Notes                           |
|------------------------------------|--------------------------------|---------------------------------|
| Discussion and moral support       | ![Team][] [spec], [lang]       |                                 |
| Adjust tooling, as needed          | @PLeVasseur                    | Pete to find appropriate person |
| Standard reviews                   | ![Team][] [lang],[opsem], [types], [bootstrap] | For any process changes, document updates and/or tooling integration     |
| Continued updates for the FLS | Contributors from Ferrous Systems and others TBD               |                                 |
| Review of updates to the FLS | `t-spec` and contributors from Ferrous Systems                |                                 |

## Frequently asked questions

### Why this Project goal?

The goal is to ensure the FLS is updated sufficiently to meet the needs of its users and stakeholders.

### Can this be done in six months?

Don't know. But we need to start having the conversations with stakeholders and see where it can lead.

### Getting documentation updated is hard. Who would do that work?

This may be the biggest blocker to making this goal successful. Part of this goal will be finding people who are interested in doing the work to author and review these updates or to find the budget to hire people to do this.

The safety-critical Rust community actively and passively benefits from the FLS, so it's worthwhile to solicit engagement with the Safety-Critical Rust Consortium.

### What happens if the FLS and Reference combine as one specification document?

That's actually a potential outcome of the `t-spec` work in the future. So this may not be totally theoretical. Hopefully the processes we come up with are not so document specific and they can withstand such a merger.
