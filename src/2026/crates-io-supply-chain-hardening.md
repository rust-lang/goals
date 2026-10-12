# crates.io hardening against supply chain attacks

| Metadata | |
| :-- | :-- |
| Contact | @jlizen |
| Status | Proposed |
| Zulip channel | N/A |
| [crates-io] champion | @Turbo87 |
| [cargo] champion | @eh2406 |
| [docs-rs] champion | @syphar |
| [infra] champion | @ubiratansoares |
| [social-media] champion | @m-ou-se |

## Summary

We plan to speed up and improve efficacy of manual security responses in the crates.io ecosystem by adding support for temporarily quarantining distribution of specific crate versions. Then, we will build automated detection systems that freeze extremely suspicious crates for human evaluation. This automation will improve time to mitigation and also reduce operator load by allowing manual review on less urgent timelines.

## Motivation

### The status quo

Registry administrators see supply chain attacks increasing in both volume and sophistication. Today, we have a tight-knit crates.io security response team that reacts quickly, but our mitigations are mostly manual and often destructive enough to delay response. Recent attacks have highlighted gaps in our toolkit that we want to address.

In a recent security incident, a stolen token published malware across an account of ~250 crates with hundreds of automated versions. The responder had to "wade through LLM spam of hundreds of versions" and chain together a script with ~70 yank & delete commands. In fact, one deletion failed and the related version lasted for a few extra days. According to the operator: "I kinda wish we had an intermediate step here. Deletion is only semi-reversible, but I would like to publicly nuke the account while we investigate."

Most language ecosystems have recently experienced supply chain attacks that compromised significant infrastructure. Attacks are becoming more sophisticated, using two-stage payloads, hiding primary attacks in poisoned dependencies, and manipulating the resolver to increase delivery ([arrayref in August 2026](https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/)). We also see new attacks such as [spearphishing high-impact individuals](https://blog.rust-lang.org/2026/09/17/targeted-attacks/). Other projects face similar attacks, such as [Django in October 2026](https://frankwiles.com/posts/i-got-targeted/).


Our current security responses are fairly effective, and we have largely avoided broad impact. `arrayref` was the worst attack we know of to date, and it was live for about 90 minutes, with little evidence of further spread through compromised environments.

#### Our current tools

We already have quite a few detection systems running offline. (In fact, during the `arrayref` incident, several Rust Foundation security systems did trigger, e.g., the namesquatting detection.) They largely run as scheduled jobs. Some send notifications, but many are prone to false positives, so their results must be reviewed manually.

We are also building client-side hardening via `min-publish-age`. It adds Cargo-side cooldowns to allow time for extra scrutiny. This could be enabled by default and relieve some of the urgency of security responses, at least for some of our consumers. However, it still leaves registry administrators in a position of "press this button in time or else there is an incident", which is still a psychologically stressful operator role.

Crates.io and other registries have three package lifecycle states: published, yanked, and deleted. Yanking a package marks it as undesirable while keeping it accessible, and individual tools may prevent users from downloading it (e.g., cargo avoids resolving yanked versions, but still allows downloading them if they appear in an existing Cargo.lock file). This means yanking is a reversible operation, albeit less safe. In contrast, deleting a package makes it inaccessible by redacting it from the registry index, which makes it safer but destructive.

Deletion is only partially reversible for two reasons: it loses download stats and history, which significantly impacts the perceived legitimacy of the project, and it frees the crate name for alternative ownership after an initial 24 hour period. Restoring deletes only means restoring crate versions in the index and making crate bytes accessible. Furthermore, deletion has a poor auditability story. Deletions show up in a registry's git index as removals of version lines, but this record is lost when the index is squashed. crates.io admins currently add notifications manually to a Zulip channel, but these are not easily discoverable by consumers outside of the Rust Project.

Stepping back, our current workflows are insufficient for operators because the more impactful mitigation (deletion) forces them to take destructive action under time pressure. This is stressful and delays response. It also currently requires chaining many individual actions (yanks and deletes of specific versions) into a bulk operation, which is error-prone and can partially fail. In practice, operators end up making direct database queries rather than scripting via admin APIs, which is also risky under pressure due to the potentially large blast radius.

#### Peer approaches

Our peers at PyPI have built [a quarantine state](https://blog.pypi.org/posts/2024-12-30-quarantine/) that explicitly *freezes* package bytes and stops serving them, even if the specific package version appears in a lockfile. This is the primitive we were missing in the incident mentioned earlier.

PyPI maintainers are also [discussing automated detection and hold systems](https://github.com/python/peps/pull/5070). [npm has similar systems](https://github.com/orgs/community/discussions/203413), though its current implementation has faced criticism for its impact on maintainers. Similar systems are available downstream of Maven Central through the paid product [Firewall](https://help.sonatype.com/en/firewall-quarantine.html).

## What we propose to do about it

We propose adding an intermediate *quarantined* state that bridges the gap between yank and deletion, and a *withdrawn* state that is a fully reversible, auditable delete. This prevents access of crate bytes and avoids resolving withheld versions, but does not wipe ownership identity and usage data. This is preferable to the "yanked" state because it prevents usage in new builds even if a version appears in an existing Cargo.lock. It is preferable to deletion because it can be used used with less friction but similar efficacy due to reversibility.

We will move our highest-confidence detection systems from being passive to active, automatically quarantining clearly suspicious releases for human review. This gives our operators room to review flagged crates in depth, on more reasonable timelines, rather than needing to make high-pressure decisions as quickly as possible. As part of this, we will validate our detections against real-world crates.io events to evaluate false positives and efficacy, and we will test new detection systems that appear promising.

### Work items over the next year

We expect three phases of work, spanning 4-6 months.

I (@jlizen) am open to collaborating with others on subtasks - the second phase, offline testing, in particular would benefit from more
hands coming up with detection algorithms. For now, I am listing myself for all tasks as I will anchor them in absence of other participants.

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Manual quarantine support for crates.io and Cargo | @jlizen | |
| Offline testing for supply chain attack detection systems | @jlizen | building on systems created by @LawnGnome and @walterhpearce |
| Publish-time enforcement and related governance | @jlizen | |

#### Manual quarantine support crates.io and Cargo

Expected to run roughly October 2026 - November 2026.

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| RFC to add withheld/quarantined state, Cargo support | @jlizen | [RFC 4016] |
| Cargo implementation of the RFC | @jlizen | |
| Withheld/quarantine support in docs.rs, crates-index-diff, related tools | @jlizen | backend + frontend for docs.rs |
| crates.io backend support for withheld/quarantined state | @jlizen | database + CDN side |
| crates.io admin API support for withheld/quarantined state | @jlizen | crates.io API service |
| Bulk operations on crates.io admin APIs | @jlizen | atomic actions with crate-wide scope and owner-wide scope |
| crates.io frontend support for withheld/quarantined state | @jlizen | quarantine reasons + warnings |
| User-facing crates.io policy documentation for withheld/quarantined state | @jlizen | updates to https://crates.io/policies |
| Blog announcement of new quarantine supprt/policies | @jlizen | |

#### Offline testing for supply chain attack detection systems

Expected to run roughly December 2026 - February 2027, though testing and data analysis will be ongoing.

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Write a design on "shadow mode" testing system | @jlizen | Shadow mode = dry run the crates.io event feed through detection algorithms. Design will include signs of success and ways of sending synthetic attack traffic |
| Implement testing systems | @jlizen | stub out a detection system |
| Wire up existing detection systems against testing system | @jlizen | Supported by @walterhpearce and @LawnGnome, they might do some implementation too |
| Select and build 1-3 additional experimental detection systems | | Supported by @walterhpearce and @LawnGnome, they might do some implementation too |
| Research and implement further detection systems | | |
| Operate systems for at least one month and gather data | @jlizen | |
| Write up technical analysis of data that includes recommendations which systems to enable | @jlizen | |
| Semi-technical blog post discussing high level findings | @jlizen | |

#### Publish-time enforcement and related governance

Expected to run roughly January - March 2027. Design/discussions will start in parallel to offline testing (preceding subgoal).

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Rough draft of policies practices around automatic quarantine | @jlizen | Focused on publish-time, but could include post-publish automation as well |
| Cargo/registry spec RFC on unreleased state | @jlizen | Addresses use cases like publishing release trains against unreleased crates and forensic builds |
| Async and/or sync discussion with interested community | @jlizen | Build consensus around policy and practice for automated enforcement strategies |
| RFC on automated crates.io quarantine / manual review systems | @jlizen | Both system and policy/governance design |
| Implement crates.io platform support for running pre-publish detections | @jlizen | |
| Wire up initial detection systems | @jlizen | |
| Semi-technical blog post announcing new capabilities and discussing system design | @jlizen | |
| Blog case study on 1-3 successful mitigations | @jlizen | |

## Team asks

| Team | Support level | Notes |
| :-- | :-- | :-- |
| [crates-io] | Large | Review RFC on registry spec support for quarantine; review PRs on manual admin APIs, byte management, authorization; review offline testing design and related PRs; give input on detection systems; give input on policy/governance/operations for automated quarantine; review RFC on automated quarantine on crates.io + related implementation |
| [cargo] | Medium | Review RFC on cargo/index support for quarantine + related implementation; review RFC on auto-quarantine + handling for quarantined publishes, related implementation |
| [docs-rs] | Small | Review + approve changes to docs.rs to avoid building quarantined crates + display badges |
| [infra] | Small | Advisory consult on crates.io implementation of quarantine and automated detection systems |
| [social-media] | Small | Advice + edits for blog posts |

Beyond Rust Project teams, we will also want to consult with the Rust Foundation, particularly its security team. We also will want to consult with outside build tools (Bazel, Buck2, Yocto). Our designs should maintain security boundaries without external build tool changes, but they could make UX improvements if they were aware of our new index states.

## Help wanted

| Task | Experience level | Time investment |
| :-- | :-- | :-- |
| Add new detection systems | Flexible | 2 weeks - 2 months |

During phase two, offline testing for supply chain attack detection systems, we will be wiring up a stub and then a small assortment of further detections. There is an opportunity to dry run many more approaches for detection while we are at it. For instance, social dependency graph analysis, scanning for certain patterns in source, and more.

Glad to collaborate with others on this stage in particular (though probably other parts too). This would be a good opportunity for any of:
- data scientist
- applied scientist
- distributed systems engineer
- security engineer

## Frequently asked questions

### What's the point of holding already-released malware?

Users continuously install software in CI and otherwise. Even if a version has been released, there is benefit in holding it to prevent subsequent installs. Lowering the threshold for taking admin action via a softer mitigation will allow faster responses. Even better, we're working towards holding software before it is even released in the first place.

### What about malware that is already in local Cargo caches?

Today, yanked and deleted crates persist in local Cargo caches even after registry administrators take action on them upstream. This gap will be closed separately from this goal by the ongoing [Verifiable Mirroring work](https://goals.rust-lang.org/2026/mirroring.html). At a high level, the verifiable mirror will have cheap access to a cryptographically verifiable view of the latest upstream registry index, allowing it to invalidate caches when a crate's index file changes. This will apply out of the box to the existing states (yanked and deleted), as well as the new lifecycle states added by this goal (quarantined and withdrawn).

### Does this add work for maintainers?

Quarantine support is strictly a softening of the current mitigation available (i.e. deletion). It's also a quality-of-life improvement for incident responders via bulk actions.

Publish-time detections and automated holds could affect maintainers. The RFC will go into more depth on the risks and controls. In general, the guiding principle will be to run detections in shadow mode for a while, make sure we have acceptable false-positive rates, focus on serious signals that are obviously suspicious, and only then enable automated holds. This protects maintainers from toil, and the crates.io and security teams reviewing the queue from overload. We will also need to specify an SLA, an appeal mechanism, and other nuts and bolts.

I'm confident that we can find an imperfect solution that everybody is comfortable with, even if it doesn't cover all possible threats that we want to handle. We can keep iterating once we land a base set of systems.

### Who decides what gets held? What is the trust model?

This should be decided at the registry level. For manual quarantine, we can use the existing criteria for admin deletion. We can also reuse the [existing malicious crate channels](https://blog.rust-lang.org/2026/02/13/crates.io-malicious-crate-update/), along with new tombstones in the index files.

For automatic holds, we still need to hash this out. I imagine starting narrow, with deterministic checks and shadow mode to judge impact, will be a good place to start. We will need to balance Project values around shared decision-making and transparency with the cat-and-mouse nature of detection efficacy. I suspect we will ultimately land on a model where a small delegated group within a security-related project team is responsible for approving new detections, using publicly agreed-upon criteria that include data, values, etc.

### Why split this up?

We'd prefer each individual RFC to be relatively small to keep it manageable to review and reach consensus. Specifically, we are designing the quarantine registry spec and Cargo behaviors separately from the distributed systems work that crates.io will implement on top of them.

We build offline testing systems early so we can start testing detection approaches against dry-run data, since we will need strong data to bring to the later RFCs to argue for enabling our first couple of detection systems. We are also separating the Cargo/registry spec's baseline support for publish-time holds from the policies around detections that actually apply such holds, and from the crates.io-side implementation of those policies.

### What are we leaving out?

We aren't trying to build a whole lot of detection systems, or the best detection systems. We mostly want to have at least one detection system, however simple, that is reliable enough to turn on. It will be easier to discuss further systems once we have the base platforms in place.

We also are not going deep into crowdsourcing reports of bad releases. Right now, we have a relatively crude process involving email intake. One could imagine crowdsourcing flags [like PyPI has discussed](https://blog.pypi.org/posts/2024-12-30-quarantine/#future-improvement-automation), for instance via cargo-vet. But that deserves its own discussion and Project Goal. This one is already fairly large :)

## Funding

Funding will go to team reviewers and champions. Excess funds will be contributed to the Rust Foundation Maintainer Fund to support ongoing maintenance of related features.

| Purpose | Cost | Funded | Sponsor(s) |
| :-- | :-- | :-- | :-- |
| Reviewers + champions | $10,000 | No | [Devolutions](https://devolutions.net/) |
