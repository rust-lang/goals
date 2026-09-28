# Experimental language specification

| Metadata         |                                              |
| :--              | :--                                          |
| Contact | @jackh726                                    |
| Status           | Discontinued                                 |
| Tracking issue   | [rust-lang/goals#627]           |
| Zulip channel    | Likely a combination of #t-types and #t-spec |
| [lang] champion  | @joshtriplett                                |
| [types] champion | @jackh726                                    |

## Summary

Create an experimental/nightly version of a language specification, based on a branch of the Rust Reference with the addition of stability markers and corresponding tooling. Develop and refine processes around it. (This could theoretically use the Reference directly, but for experimentation purposes it likely makes sense to make it a separate repository, depending on how we can best implement and experiment with this new process without gating on its adoption.)

Do an N=1 case study using the types team as the exemplar, on what it would look like to integrate this with the processes of various teams, with extensive documentation to facilitate scaling this to other teams of domain experts within the project. This case study would include:
- The handling of proposed changes that fall within the purview of that team, where the types team would review the proposed content.
- The handling of changes *from* that team, which would be reviewed by other members of the team. This includes ensuring that prospective language changes arising from the work of the team get flagged as needing documentation changes, and the documentation occurs contemporaneously with the language changes.
- How and when to merge text with unstability markers (whether "unstable text" or "unstable Rust feature"), and what process to follow to review and remove the markers so that text is considered stable.

As a case study, we can use current work on const generics, RTN, or const traits. All of those would heavily overlap with the purview of the types team. (Lang would make design calls on the feel of the language, but the detailed semantics and recommendations typically fall under the types team.)

This goal may potentially align with separate work to improve lang processes, but that work would not be part of this project goal.

This goal will collaborate with the goal to integrate a-mir-formality into language specifications, though neither goal will block on the other.

## Motivation

### The status quo

Currently, there is no established process for teams of domain experts within the Project to document prospective language changes within their domain, or to check in at an early point about prospective changes within their domain that may have a language-level impact requiring documentation.

Often, such changes may take place primarily on the language side, and end up with documentation as a subsequent step beginning *after* the language change has been decided and largely completed. Yet, the documentation would potentially serve to facilitate the change; since such documentation will be needed anyway, the documentation could potentially precede the change.

Conversely, proposed changes within a given domain typically go through review cycles with non-domain-experts, rather than being systematically reviewed and approved by domain experts.

### What we propose to do about it

Experiment with, and iteratively refine, processes for documenting changes within a domain, and processes for reviewing such changes.

Work with the types team as an N=1 case study of a Project team. Use one or more specific language improvements (e.g. const generics, RTN, or const traits) as a case study for larger changes to review, with the active cooperation of one of the leads for one of those language improvements.

This experiment will necessarily involve a fast, iterative process that's amenable to delegation. Conducting the experiment may involve creating a nightly repository for integrating Reference work, in order to facilitate this iterative approach.

Part of this process includes the process for stabilizing for removing the instability markers from the text and declaring it stable.

One output of this goal will be clear periodic reporting of "how the experiment is going".

### Work items over the next year

> *Sketch out the specific things you are trying to achieve in this goal period. This should be short and high-level -- we don't want to see the design!*

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Develop and iterate on process | @jackh726, @joshtriplett | Incremental, iterative. |
| Implement experimental process within types team | @jackh726 | |
| Provide clear process documentation | @joshtriplett, @jackh726 | |
| Work with new process as part of shipping a language feature | Various | Varies by the language feature. |
| Start conversations with other domain-expert teams | @joshtriplett, @jackh726 | For instance, wg-const-eval, compiler, opsem |
| Provide experimental reference repository | @joshtriplett, @jackh726 | |
| Develop tooling/infrastructure as needed | @joshtriplett, @jackh726 | |

## Team asks

| Team       | Support level | Notes                                                                            |
| ---------- | ------------- | -------------------------------------------------------------------------------- |
| [types]    | Medium        | Evaluate potential changes to (experimental) reference in routine team decisions |
| [lang]     | Medium        |                                                                                  |
| [spec]     | Small         | General discussion on how this may align with other efforts to specify Rust.     |

This goal will also coordinate discussions with lang-docs in a non-blocking fashion, to ensure that different sources of Rust documentation are clearly delineated in a manner that avoids confusion, both in source repositories and in any rendered versions.

## Frequently asked questions

### Will this experiment take a year?

Hopefully not. If the experiment iterates to a successful conclusion successfully earlier than that, we could work with additional Rust teams to adopt it further.

### What is the distinction between the types of contributions? What are some examples?

Some changes originate from elsewhere within the Rust Project or the Rust community, and fall within the purview of a team of domain experts. For instance, a proposed change to the Rust type system (whether standalone or as part of a larger change) would fall within the purview of the types team. For such a change, the types team would review and approve the documentation of the proposed change.

Some changes originate from *within* a team of domain experts. For instance, the types team has been steadily working to make the Rust type system simpler and more robust, while maximally preserving compatibility (bolstered by crater runs). Such work typically involves changes to the language, and such changes require corresponding documentation. For such changes, the types team would review and approve the documentation of the proposed change; since the types team may also have been the on to *write* the change, this means the process must take particular care to maintain the constraint that the review and the author must be different people.

In both cases, the lang team would also benefit from such documentation to precisely explain the change being proposed.
