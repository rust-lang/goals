# Modeling downstream registry divergence

| Metadata | |
| :-- | :-- |
| Contact | @jlizen |
| Short title | Downstream registry divergence |
| What and why | Distinguish intended source replacement in downstream registries from supply chain attacks |
| Status | Proposed |
| Zulip channel | N/A (to be created on request) |
| [cargo] champion | @weihanglo |
| [crates-io] champion | TBD |
| Funding contact  | [Software Stewardship Lab](https://stewardshiplab.org/contact/)        |

## Summary

Cargo's stable source replacement functionality is under-documented and powerful. It is being used in many ways, including some not considered when the feature was last documented. We will inventory how it is being used and abused, and update the documentation to describe what tangible risks to look out for under "use at your own risk". Many companies rely on source replacement to emergency-replace open source code when a vulnerability is found in it. Given that this is already technically possible, we will propose a RFC, for a recommended way to do so, with Cargo support, that can be distinguished from a malicious dependency confusion attack. Outside of Cargo and the Rust Project, we will experiment with supporting tooling such as for auditing registries for undeclared divergence, and sample implementation of a self-hosted dowstream registry that serves registry releases.

## Motivation

### The status quo

Today, downstream registries can serve crate releases that upstream never published as part of [source replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html). By "downstream registry" we mean any source Cargo uses in place of an upstream registry (usually crates.io). Most are Cargo registries, such as a private mirror or a caching proxy, but local directory sources such as `cargo vendor` output count too.

The Cargo team canonically considers source replacement a "dependency
confusion attack" and considers any mirror that does it as badly behaving. 
But, in practice, many private downstream registries do this anyway and will continue to. Treating this as "off the beaten path" is sweeping legitimate use cases under the rug.
A frequent reason is to apply security fixes.
Users ought to be attempting to upstream fixes, and largely do, but maintainer capacity is limited and not everybody agrees with what is fix-worthy.
For instance, registries might want to backport security fixes for versions that maintainers have no interest in supporting.

With today's support, there is no way for a registry to prominently say "this release is mine, not upstream's", and similarly no way for a consumer to decide
how to handle this divergence or to audit a registry against upstream. Benign divergence from upstream is difficult to distinguish from malicious, without careful review.
We call this type of release a "registry release", where a registry serves a release that does not originate with upstream and may need different handling and auditability by consumers.


#### Prior art

Downstream publication of fixes:
- **Maven Central CARE** ([Sonatype RFC, 2026](https://www.sonatype.com/blog/request-for-comments-care-and-maven-central)): Maven Central has proposed central, registry-level security fixes. Under this model, pre-approved remediation actors publish emergency fixes into the existing coordinate under a reserved version form. This leaves the maintainer's next version untouched. The release author goes in release metadata rather than the version string. It involves no resolver changes.
- **Linux distributions**: Some distributions, such as Fedora, Ubuntu, and Debian, vendor crates into a local, offline source replacement directory and patch them under the upstream version number, for example to remove non-Linux dependencies. Their own versioning and signed advisories live at the package level (the RPM or deb release), not in the crate version Cargo sees. In Ubuntu and Debian the vendored crates also ship as installable `librust-*-dev` packages, so they reach end users in source form. This is a mature, accepted model for divergence from upstream, but it is declared outside of Cargo. We will consult with Linux distribution packagers to understand whether this goal's new features can make their lives easier.
- **Cargo**: [RFC 3493](https://rust-lang.github.io/rfcs/3493-precise-pre-release-cargo-update.html) lets `cargo update --precise` target a pre-release. This is the closest existing hook for resolving a non-upstream release; we likely would build on top of it.
- **Hatch environment sources** ([docs](https://hatch.pypa.io/latest/config/environment/sources/)): Hatch lets a project redirect a named dependency to a local path, git ref, URL, or another index. This is client-side, like Cargo's `[patch]` and `[source]`, not registry-side. Two relevant pieces: `HATCH_NO_SOURCES` turns off all redirection so CI can check that the published metadata resolves by itself, and `dep show sources` lists each redirection and the dependencies it affects.

Provenance logging of registry releases:
- **Sigstore / transparency logs**: [sigstore.dev](https://www.sigstore.dev/), [Rekor](https://docs.sigstore.dev/logging/overview/). This is a public, append-only Merkle tree transparency log, and a model for the feed of registry releases: a registry cannot quietly add a release and later deny it. This is related but distinct from the registry signing / TUF work.
- **npm provenance**: [docs](https://docs.npmjs.com/generating-provenance-statements), [launch post](https://github.blog/security/supply-chain-security/introducing-npm-package-provenance/), [npm/provenance spec](https://github.com/npm/provenance). This is a system that uses Sigstore and is logged in Rekor to validate that a release came from a given source repo and committed via a specific CI workflow, with various attestations. It is interacted with via `npm publish --provenance` and `npm audit signatures` but not stored inline in the index.
- **PyPI attestations / PEP 740**: [PEP 740](https://peps.python.org/pep-0740/), [PyPI announcement](https://blog.pypi.org/posts/2024-11-14-pypi-now-supports-digital-attestations/), [index-hosted attestations spec](https://packaging.python.org/en/latest/specifications/index-hosted-attestations/). This is a similar idea to npm but hosted in the index itself, and the Simple API serves them. Probably closer to what we want for declaring a release's origin in the index itself.
- **Package URL (PURL)**: [spec](https://github.com/package-url/purl-spec). A standard identifier for a package release across ecosystems (`pkg:cargo/tokio@1.48.0`), used by SBOM and vulnerability tooling. A registry release needs a PURL distinct from the upstream release it is based on, e.g. via a distinct version or a `repository_url` qualifier, so SBOMs and advisory matching don't report it as upstream. AboutCode's VulnerableCode [models fixes](https://github.com/aboutcode-org/vulnerablecode/blob/9e1aeb7dba1a4c951ea1c7739dad59c13167e4d0/vulnerabilities/models.py#L1812) as code changes tied to a base package version by PURL.

Mirror auditing:
- **Go checksum database**: [database](https://sum.golang.org/), [launch post](https://blog.golang.org/module-mirror-launch), [Trillian](https://github.com/google/trillian). This is along the lines of the audit tooling we are interested in: Go makes it easy to audit a mirror to see if its bytes diverge from upstream. Though, it doesn't have any concept of declared/intended divergence, so corporate users end up turning `GOSUMDB` off. There also was [a recent bypass](https://freenode.net/article/gos-checksum-database-meets-its-trust-assumptions).

Other research:
- **Registry release alternatives survey** ([HackMD](https://hackmd.io/GvCnMrbWSAuUo2fR8E2E-g), @celinval): an in-progress survey of how Cargo could model registry releases, evaluated against a shared set of use cases and constraints (lockfile stability, opt-out, downstream pass-through, semver ordering, older Cargo versions). Options include a reserved pre-release tag on the next version (currently leading), a tag on the base version, build metadata, a new semver component, serving registry releases outside the index, and extending `[patch]` to accept registry coordinates.

### What we propose to do about it

We will start by documenting how source replacement is used today and what risks come with it.

Then, we will propose a recommended way to add releases via downstream registries, including tooling support in Cargo and/or the Cargo
registry spec, via RFC. We'll implement the RFC's approach as experimental flags.

With experimental support in place in the Rust toolchain, we will experiment with supporting tooling outside of Cargo: audit tooling to track whether a downstream registry declares all divergence from upstream, tools for packaging patches for offline installs / `cargo-auditable`, integrations with metadata consumers like [ecosyste.ms](ecosyste.ms), and explorations of reachability analysis 
for CVEs that consider modified security releases.

In addition to client-side tooling, we will contribute a sample implementation of a self-hosted downstream registry that serves registry releases, which will live in an ecosyste.ms-managed repository. Using this code, ecosyste.ms will host its own downstreamregistry of crates.io, as an extension of its pre-existing [caching crates.io proxy](https://proxy.ecosyste.ms). To kick the tires on the new registry release support, the ecosyste.ms downstream will be initially backfilled with a range of releases backpatching security-related fixes from historical CVEs.

Out of scope:

- crates.io publishing its own registry releases. This is a policy decision that would require a much deeper discussion, RFC, and its own goal.

### Work items over the next year

Three phases, roughly 4-8 months.

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Declared registry releases: spec, resolver support, docs | @jlizen, @celinval | |
| Feed, audit tooling, producer tooling | @CAD97, vladh@ | |
| Self-hosted downstream registry implementation | @srckln, @andrew, @vladh |

#### Declared registry releases: spec, resolver support, docs

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Survey alternate registries and mirrors for how they diverge today | @jlizen | including behaviors that amount to dependency confusion in practice |
| Cargo docs PR: describe how source replacement is used today and its risks; define terms (mirror, downstream registry, registry release) | @jlizen | |
| Requirements doc; gather use cases and constraints for prospective consumers and registry operators | @jlizen | |
| Requirements from the TUF / verifiable work; make sure approach co-exists | @jlizen | with @arlosi |
| Survey Cargo-side alternatives for modeling registry releases | @celinval | in progress; [HackMD](https://hackmd.io/GvCnMrbWSAuUo2fR8E2E-g) |
| RFC: Registry release declarations, Cargo support, registry spec changes | @jlizen, @celinval |  |
| Cargo implementation behind unstable flag | @jlizen, @celinval | |
| Simple reference implementation in an alternate registry or static index | @jlizen, @celinval | |
| Docs: update source-replacement / mirroring docs for registry releases | @jlizen | |
| Docs: Guidance for registry operators who diverge from upstream | @jlizen | |
| Blog post: overview of new registry release support, what's coming next | @jlizen | |

#### Feed, audit tooling, producer tooling

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Design: provenance and how consumers verify registry release = base + diff | @CAD97, @vladh | |
| Tooling to produce a declared registry release from upstream `.crate` + diff | @CAD97 | |
| Record modified vendored crates `cargo vendor` and `cargo-auditable`, so SBOMs don't report them as the upstream version | @CAD97 |  |
| Audit tool: diff a registry against its declared upstream, flag undeclared divergence | @vladh | |
| Improvements to reachability analysis for CVEs with downstream registry releases | @vladh | |
| Represent registry releases and modified vendored crates with a distinct [PURL](https://github.com/package-url/purl-spec) in SBOM tooling | @vladh | |
| Blog post: discussion of the producer/consumer tooling, some ways people are using it, how to audit a mirror | @vladh, @CAD97 |  |

#### Self-hosted downstream registry implementation
| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Design: self-hosted downstream that serves registry releases | @smrckln | High level, focuses on interfaces + architecture |
| Design: event feed of registry releases, signing | @andrew | |
| Application code for self-hosted downstream registry | @smrckln | |
| Infrastructure code for self-hosted downstream registry | @andrew | |
| Prepare a series of registry releases to fix historical CVEs on old versions | @vladh | |
| Launch ecosyste.ms-managed downstream registry serving the historical CVE fixes | @andrew | |
| Integrate event feed from downstream registry with ecosyste.ms | @andrew | |
| Blog post: ecosyste.ms-hosted downstream, what we learned along the way, next steps | @andrew, @smrckln | |


## Team asks

| Team | Support level | Notes |
| :-- | :-- | :-- |
| [cargo] | Large | Review: Resolver and lockfile semantics; source-replacement trust model and docs; RFCs and implementation |
| [crates-io] | Medium | Advise on approach and steward any registry index spec changes so alternate registries interoperate; crates.io not expected to publish registry releases itself |
| [social-media] | Small | Advice + edits for blog post |

We also expect a Small review from the proposed ecosystem security team [once it exists](https://github.com/rust-lang/leadership-council/issues/352).

Feedback from outside the Project: RustSec maintainers, alternate registry operators (JFrog, Sonatype, Cloudsmith, kellnr), Linux distributions (Fedora, Canonical/Ubuntu, Debian), Git forge (GitLab, GitHub), ecosyste.ms, Alpha-Omega, Akrites, OpenSSF Security Software Repos working group, Rust contracting shops (Ferrous Systems, Mainmatter), external build tools (Bazel, Buck2, Yocto, Nix), corporate-managed downstream registries (Amazon, Microsoft, OpenAI), corporate-managed downstream package vendors (Google, Meta, etc).

## Frequently asked questions

### Isn't this just sanctioning dependency confusion?

Not quite. Today, downstream registry operators can only apply security or other fixes by doing what looks like a dependency confusion attack.
Under this goal, we will move away from "confusion" to an explicit dependency injection, that is clearly modeled and gives consumers
more control.

### Is a mirror that serves only some of crates.io diverging?

No. A mirror that serves a subset of upstream's crates and versions is a valid mirror. Any change to a crate version it serves, including index metadata or bytes, is divergence. Companies often vet a subset, and some mirrors carry only the top N most-downloaded crates. The spec should let a mirror declare "subset of upstream" separately from "publishes registry releases", and the audit tooling should treat missing crates differently from changed or added ones.

### Why not just `cargo update --precise` ([RFC 3493](https://rust-lang.github.io/rfcs/3493-precise-pre-release-cargo-update.html))?

Precise updates require that every consumer know about and point-install each release. Transitive consumers won't do that. We want
`cargo update` to pick it up. RFC 3493 also doesn't include any clear declaration that a release is registry-injected or allow audits or 
opt-out. The pre-release machinery is a good building block, but this goal extends it.

### Why not use `[patch.crates-io]` in each project?

Today, `[patch]` points a crate at a git URL or local path. Every consumer has to opt in, crate by crate, and keep the list current as versions change, whether in `Cargo.toml` or in `.cargo/config.toml`. That often isn't practical for organizations with many independent teams; switching to a registry should be a one-line change in `config.toml`. A git URL is also a muddy provenance story, and many locked-down build environments only allow a private registry.

### Who decides what releases a registry publishes?

A registry. From this goal's perspective, all users will be downstream of the Rust Project. Some expected users are: corporate security
teams running internal mirrors; caching proxies such as Haven; Linux packagers. Registries may opt to accept releases from other trusted parties, but that is a registry-by-registry decision.

### Does this hurt maintainer autonomy?

No; we don't know the solution yet, but we will be discussing this with a variety of maintainers during research tasks and we expect that any viable approach will need to be respectful of people's ownership of their projects. We shouldn't incentivize downstream registries bundling releases using larger and larger stacks of patches that they don't upstream. We expect that any Cargo resolver changes would prioritize newer semver-compatible releases from maintainers over existing registries releass, unles a registry explicitly packages further registry release on top of newer upstream.

### Does crates.io have to do anything?

No, besides review (optional) registry spec changes to ensure that there is one format. We don't expect crates.io to use this feature
in the short term, certainly not without a separate RFC/goal.

### How does this relate to TUF / verifiable mirroring?

They are complementary. Signing proves that bytes came from the registry that served them. This goal makes a registry declare which of those bytes are not upstream's. Depending on how signing lands, undeclared divergence may become impossible rather than merely detectable, which is the outcome we want. We will gather requirements from that work during the inventory so registry releases fit the signing model.

### Does this change lockfile behavior?

Needs to be discussed in the RFC, but almost certainly not. There might be an opt-in way to take registry releases in lockfiles.

### Does this cover distro packaging and `cargo vendor` directories?

To some extent. We want to add nicer support for distro packagers that want clearer provenance of their patched dependencies, for instance with `cargo-auditable`.

That said, we expect that normative guidance will distinguish between undeclared divergence in dependencies served directly to end users, versus only used for bundling binaries. Linux packagers applying patches when bundling binaries, is not a dependency confusion attack in the same way as serving modified releases via a mirror that claims to match upstream.


## Funding

Funding will cover vladh@'s contributions on audit tooling, reachability analysis, and preparing registry releases to fix historical CVEs for the ecosyste.ms-hosted downstream registry. Fiscal sponsorship is provided by the [Software Stewardship Lab](https://stewardshiplab.org).

| Purpose | Cost | Funded | Sponsor(s) |
| :-- | :-- | :-- | :-- |
| vladh@'s contributions | $30,000 | No | TBD |
