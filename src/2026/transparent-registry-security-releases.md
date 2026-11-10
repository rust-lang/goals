# Transparent registry security releases

| Metadata | |
| :-- | :-- |
| Contact | @jlizen |
| Short title | Transparent registry releases |
| What and why | Make it explicit, auditable, and resolvable when a downstream registry serves releases that differ from upstream, so security fixes can be delivered without looking like a dependency confusion attack |
| Status | Proposed |
| Zulip channel | N/A (to be created on request) |
| [cargo] champion | TBD |
| [crates-io] champion | TBD |

## Summary

We will offer an official way for downstream registries to advertise special releases that are not published by maintainers, so that registries can vend security fixes while waiting on upstream to take them.
An official method allows transparency, good Cargo resolution, and a way for maintainers to take back over by issuing new releases.
Then, we will add support for "urgent" releases that bypass lockfiles to allow consumers to immediately uptake critical fixes.


## Motivation

### The status quo

Today, downstream registries can serve crate releases that upstream never published as part of [source replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html). The Cargo team canonically considers this a "dependency
confusion attack" and considers any mirror that does it as badly behaving.
But, in practice, many private downstream registries do this anyway and will continue to. Treating this as "off the beaten path" is sweeping legitimate use cases under the rug.
A frequent reason is to apply security fixes. We expect such usage to grow as the number of discovered vulnerabilities grow. Based on trends in other language ecosystems, we also expect to see new public registries form that vend security releases for the
world to consume.
Users ought to be attempting to upstream fixes, and largely do, but maintainer capacity is limited and not everybody agrees with what is fix-worthy.
For instance, registries might want to backport security fixes for versions that maintainers have no interest in supporting.

With today's support, benign divergence from upstream is difficult to distinguish from malicious, without careful review. There is
no way for a registries to prominently say "this is release is mine, not upstreams", and similarly no way for a consumer to decide
how to handle this divergence or to audit a registry against upstream.


#### Prior art

- **Maven Central CARE** ([Sonatype RFC, 2026](https://www.sonatype.com/blog/request-for-comments-care-and-maven-central)): Maven opted for central, registry-level security fixes. Under this model, pre-approved remediation actors publish emergency fixes into the existing coordinate under a reserved version form. This leaves the maintainer's next version untouched. The release author goes in release metadata rather than the version string. It involves no resolver changes.
- **Linux distributions**: Downstream packagers backport into existing package lines with distro-specific version suffixes and signed advisories. This a fairly mature, accepted model for declared divergence from upstream. We will consult with Linux distribution packagers to understand whether this goal's new features can make their lives easier.
- **Sigstore / transparency logs**: [sigstore.dev](https://www.sigstore.dev/), [Rekor](https://docs.sigstore.dev/logging/overview/). This is a public, append-only Merkle tree transparency log. This is related but distinct from the registry signing / TUF work, we can look at how these models might relate.
- **npm provenance**: [docs](https://docs.npmjs.com/generating-provenance-statements), [launch post](https://github.blog/security/supply-chain-security/introducing-npm-package-provenance/), [npm/provenance spec](https://github.com/npm/provenance). This is a system that uses Sigstore and is logged in Rekor to validate that a release came from a given source repo and committed via a specific CI workflow, with various attestations. It is interacted with via `npm publish --provenance` and `npm audit signatures` but not stored inline in the index.
- **PyPI attestations / PEP 740**: [PEP 740](https://peps.python.org/pep-0740/), [PyPI announcement](https://blog.pypi.org/posts/2024-11-14-pypi-now-supports-digital-attestations/), [index-hosted attestations spec](https://packaging.python.org/en/latest/specifications/index-hosted-attestations/). This is a similar idea to npm but hosted in the index itself, and the Simple API serves them. Probably closer to what we want for transparency logs.
- **Go checksum database**: [database](https://sum.golang.org/), [launch post](https://blog.golang.org/module-mirror-launch), [Trillian](https://github.com/google/trillian). This is along the lines of the audit tooling we are interested in: Go makes it easy to audit a mirror to see if its bytes diverge from upstream. Though, it doesn't have any concept of declared/intended divergence, so corporate users end up turning `GOSUMDB` off. There also was [a recent bypass](https://freenode.net/article/gos-checksum-database-meets-its-trust-assumptions).
- **Cargo**: [RFC 3493](https://rust-lang.github.io/rfcs/3493-precise-pre-release-cargo-update.html) lets `cargo update --precise` target a pre-release. This is the closest existing hook for resolving a non-upstream release; we likely would build on top of it.

### What we propose to do about it

We will allow registries to advertise that they support injected (security) releases (as well as propagate security releases from their upstreams).
Those security releases will be clearly marked in the index, and in consuming lockfiles.
Cargo will resolve them by default, unless the user opts out. They will respect semver, and new maintainer releases wil take precedence.
Registries that opt into using this functionality will have a canonical way to expose
a feed of registry-injected releases that they published, with metadata such as relevant advisories. Tooling will be available to audit that all registry-added releases are tagged properly and advertised in the feed.

As a later step, we will also add support for nudging Cargo to resolve certain "urgent" releases even if a lockfile is already present and pins an earlier version, to handle 0-days and other releases that consumers want to uptake as fast as possible.


Some design rules:

1. **Divergence from upstream is considered an attack unless declared.** A registry that serves releases not in its upstream must say so per release. Tooling treats undeclared divergence as a dependency confusion attack.
2. **Upstream releases always win.** A registry release never uses the maintainer's next version number. A real upstream release supersedes it on the next update.
3. **Consumers stay in control.** Resolving registry releases is opt-out. `--locked` builds never silently change. Even for the "urgent" releases, we will design either an opt-in mechanism or an implicit opt-out by using the `--locked` flag.
4. **Usage is up to registries.** Our goal adds a spec that covers how to declare and advertise registry releases. We will add Cargo support for resolving them. But, who gets to publish these releases, what goes in them, and to what extent the contents are explained to consumers is up to each registry. crates.io may never enable it.
5. **Everything is auditable.** Each registry release links to a reason (e.g., an advisory), a base version, and a producer. This al appears in a feed that a third party can compare against upstream.

Out of scope:

- crates.io publishing its own registry releases. This is a policy decision that would require a much deeper discussion, RFC, and its own goal.
- Modifying bytes of an already-published release under the same version number or any other form of opaque "dependency confusion".
- Changes to non-Cargo build tools; they should keep working unchanged and may not recognize registry releases.

### Work items over the next year

Three phases, roughly 6-9 months. The critical-release phase depends on consensus from the first two and may end up being dropped from this goal.

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Declared divergence: spec, resolver support, docs | @jlizen | |
| Feed, audit tooling, producer tooling | @jlizen | |
| Critical security releases that apply to locked versions | @jlizen | later phase |

#### Declared registry releases: spec, resolver support, docs

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Docs: Update Cargo docs to reflect current (dependency confusion) usage of source replacement | @jlizen | |
| Requirements doc; gather use cases from registry operators, Alpha-Omega, Akrites, Linux packagers | @jlizen | |
| Survey alternate registries and mirrors for how they diverge today | @jlizen | |
| RFC: Registry release declarations, Cargo resolver behavior, registry advertisement, consumer opt-out | @jlizen | may split into two |
| Cargo implementation behind unstable flag | @jlizen | |
| Reference implementation in an alternate registry or static index | @jlizen | end to end without crates.io |
| RustSec advisory DB: represent "fixed in registry X with registry release Y" | @jlizen | with RustSec maintainers |
| Rewrite Cargo source-replacement / mirroring docs for the real trust model | @jlizen | |
| Stabilization | @jlizen | |
| Docs: Guidance for registry operators who diverge from upstream | @jlizen | |

#### Feed, audit tooling, producer tooling

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Design: event feed of registry releases, signing, relation to [verifiable mirroring](./mirroring.md) | @jlizen | |
| Design: provenance and how consumers verify registry release = base + diff | @jlizen | |
| Audit tool: diff a registry against its declared upstream, flag undeclared divergence | needs owner | |
| Tooling to produce a declared security release from upstream `.crate` + diff | needs owner | |

#### Critical security releases that apply to locked versions

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Requirements from corporate users and central remediation groups | @jlizen | |
| RFC: critical class, lockfile behavior, opt-out, UX | @jlizen | |
| Cargo implementation behind unstable flag | @jlizen | |
| Stabilization | @jlizen | |

## Team asks

| Team | Support level | Notes |
| :-- | :-- | :-- |
| [cargo] | Large | Review: Resolver and lockfile semantics; source-replacement trust model and docs; RFCs and implementation |
| [crates-io] | Small | Steward the registry index spec changes so alternate registries interoperate; crates.io not expected to publish registry releases itself |

Feedback from outside the Project: RustSec maintainers, alternate registry operators (JFrog, Sonatype, Cloudsmith, kellnr), Linux distributions, Alpha-Omega, Akrites, external build tools (Bazel, Buck2, Yocto, Nix), corporate-managed downstream registries (Amazon, Microsoft, OpenAI).

## Help wanted

| Task | Experience level | Time investment |
| :-- | :-- | :-- |
| Audit tool (registry vs. upstream diff) | Intermediate Rust | 1-2 months |
| Producer tooling (package a registry patch) | Intermediate Rust | 1-2 months |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
| :-- | :-- | :-- | :-- |
| Reviewers + champions (Rust Foundation Maintainers Fund) | $10,000 | No | |

## Frequently asked questions

### Isn't this just sanctioning dependency confusion?

Not quite. Today, downstream registry operators can only really use dependency confusion attacks to apply security or other fixes.
Under this goal, we will move away from "confusion" to an explicit dependency injection, that is clearly modeled and gives consumers
more control.

### Why not just `cargo update --precise` ([RFC 3493](https://rust-lang.github.io/rfcs/3493-precise-pre-release-cargo-update.html))?

Precise updates require that every consumer know about and point-install each release. Transitive consumers won't do that. We want
`cargo update` to pick it up. This also doesn't include any clear declaration that a release is registry-injected or allow audits or 
opt-out. THe prelease machinery is a good building block, but this goal extends it.

### Who decides what releases a registry publishes?

A registry. From this goal's perspective, all users will be downstream of the Rust Project. Some expected users are: corporate security
teams running internal mirrors; a pre-approved remediation actor list for a public, shared mirror; a security team that produces fixes
that can be consumed by many mirrors.

### Does crates.io have to do anything?

There are no crates.io changes. crates.io team will review a RFC about registry spec changes that would be used by downstreams.

### Why work on critical releases (bypass lockfiles) separately / at the end?

Changing locked versions is the most surprising, complex, and potentially controversial feature in this goal. We would be happy to
accomplish everything but that and make sure that the experience of using standard security releases is good. We can evaluate
viability of the critic release component as we go.
