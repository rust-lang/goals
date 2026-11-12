# Modeling downstream registry divergence

| Metadata | |
| :-- | :-- |
| Contact | @jlizen |
| Short title | Downstream registry divergence |
| What and why | Downstream registries already serve releases upstream never published, mostly for security fixes; document that reality and give it a declared, auditable, resolvable shape so it is distinguishable from a dependency confusion attack |
| Status | Proposed |
| Zulip channel | N/A (to be created on request) |
| [cargo] champion | TBD |

Ecosystem security champion: TBD. (Team is still being chartered, [ref](https://github.com/rust-lang/leadership-council/issues/352).)

## Summary

Cargo's stable source replacement functionality is under-documented and powerful. It is being used in many ways, including some not considered when the feature was last documented. We will inventory how it is being used and abused, and update the documentation to describe what tangible risks to look out for under "use at your own risk". Many companies rely on source replacement to emergency-replace open source code when a vulnerability is found in it. Given that this is already technically possible, we will propose an RFC for a recommended way to do so, with Cargo support, that can be distinguished from a malicious dependency confusion attack.


## Motivation

### The status quo

Today, downstream registries can serve crate releases that upstream never published as part of [source replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html). The Cargo team canonically considers this a "dependency
confusion attack" and considers any mirror that does it as badly behaving.
But, in practice, many private downstream registries do this anyway and will continue to. Treating this as "off the beaten path" is sweeping legitimate use cases under the rug.
A frequent reason is to apply security fixes. We expect such usage to grow as the number of discovered vulnerabilities grows. Based on trends in other language ecosystems, we also expect to see new public registries form that vend security releases for the
world to consume.
Users ought to be attempting to upstream fixes, and largely do, but maintainer capacity is limited and not everybody agrees with what is fix-worthy.
For instance, registries might want to backport security fixes for versions that maintainers have no interest in supporting.

With today's support, there is no way for a registry to prominently say "this release is mine, not upstream's", and similarly no way for a consumer to decide
how to handle this divergence or to audit a registry against upstream. Benign divergence from upstream is difficult to distinguish from malicious, without careful review.
We call this type of release a "registry release", where a registry serves a release that does not originate with upstream and may need different handling and auditability by consumers.


#### Prior art

Prior art falls into three buckets: downstream publication of fixes, provenance, and mirror auditing.

- **Maven Central CARE** ([Sonatype RFC, 2026](https://www.sonatype.com/blog/request-for-comments-care-and-maven-central)): Maven Central has proposed central, registry-level security fixes. Under this model, pre-approved remediation actors publish emergency fixes into the existing coordinate under a reserved version form. This leaves the maintainer's next version untouched. The release author goes in release metadata rather than the version string. It involves no resolver changes.
- **Linux distributions**: Some distributions, such as Fedora, Ubuntu, and Debian, vendor crates into a local, offline source replacement directory and patch them under the upstream version number, for example to remove non-Linux dependencies. Their own versioning and signed advisories live at the package level (the RPM or deb release), not in the crate version Cargo sees. In Ubuntu and Debian the vendored crates also ship as installable `librust-*-dev` packages, so they reach end users in source form. This is a mature, accepted model for divergence from upstream, but it is declared outside of Cargo. Other distributions may differ and serve more conventionally modeled registries with patched releases. We will consult with Linux distribution packagers to understand whether this goal's new features can make their lives easier.
- **Sigstore / transparency logs**: [sigstore.dev](https://www.sigstore.dev/), [Rekor](https://docs.sigstore.dev/logging/overview/). This is a public, append-only Merkle tree transparency log, and a model for the feed of registry releases: a registry cannot quietly add a release and later deny it. This is related but distinct from the registry signing / TUF work, we can look at how these models might relate.
- **npm provenance**: [docs](https://docs.npmjs.com/generating-provenance-statements), [launch post](https://github.blog/security/supply-chain-security/introducing-npm-package-provenance/), [npm/provenance spec](https://github.com/npm/provenance). This is a system that uses Sigstore and is logged in Rekor to validate that a release came from a given source repo and committed via a specific CI workflow, with various attestations. It is interacted with via `npm publish --provenance` and `npm audit signatures` but not stored inline in the index.
- **PyPI attestations / PEP 740**: [PEP 740](https://peps.python.org/pep-0740/), [PyPI announcement](https://blog.pypi.org/posts/2024-11-14-pypi-now-supports-digital-attestations/), [index-hosted attestations spec](https://packaging.python.org/en/latest/specifications/index-hosted-attestations/). This is a similar idea to npm but hosted in the index itself, and the Simple API serves them. Probably closer to what we want for declaring a release's origin in the index itself.
- **Go checksum database**: [database](https://sum.golang.org/), [launch post](https://blog.golang.org/module-mirror-launch), [Trillian](https://github.com/google/trillian). This is along the lines of the audit tooling we are interested in: Go makes it easy to audit a mirror to see if its bytes diverge from upstream. Though, it doesn't have any concept of declared/intended divergence, so corporate users end up turning `GOSUMDB` off. There also was [a recent bypass](https://freenode.net/article/gos-checksum-database-meets-its-trust-assumptions).
- **Package URL (PURL)**: [spec](https://github.com/package-url/purl-spec). A standard identifier for a package release across ecosystems (`pkg:cargo/tokio@1.48.0`), used by SBOM and vulnerability tooling. A registry release needs a PURL distinct from the upstream release it is based on, e.g. via a distinct version or a `repository_url` qualifier, so SBOMs and advisory matching don't report it as upstream. AboutCode's VulnerableCode [models fixes](https://github.com/aboutcode-org/vulnerablecode/blob/9e1aeb7dba1a4c951ea1c7739dad59c13167e4d0/vulnerabilities/models.py#L1812) as code changes tied to a base package version by PURL.
- **Hatch environment sources** ([docs](https://hatch.pypa.io/latest/config/environment/sources/)): Hatch lets a project redirect a named dependency to a local path, git ref, URL, or another index. This is client-side, like Cargo's `[patch]` and `[source]`, not registry-side. Two relevant pieces: `HATCH_NO_SOURCES` turns off all redirection so CI can check that the published metadata resolves by itself, and `dep show sources` lists each redirection and the dependencies it affects.
- **Cargo**: [RFC 3493](https://rust-lang.github.io/rfcs/3493-precise-pre-release-cargo-update.html) lets `cargo update --precise` target a pre-release. This is the closest existing hook for resolving a non-upstream release; we likely would build on top of it.
- **Central Patches draft RFC** ([HackMD](https://hackmd.io/GvCnMrbWSAuUo2fR8E2E-g), @celinval): one exploration of the Cargo side of this goal, written before a full inventory of cargo source replacement usage. Registries reserve a pre-release tag for their own releases, Cargo resolves them within existing requirements, and consumers control uptake per registry. It is a starting point for discussion. The RFC will start from the problem and alternatives, and this draft will be rewritten or replaced based on what the real-world usage inventory finds.

### What we propose to do about it

We will start by documenting how source replacement is used today and what risks come with it.

We will then allow registries to advertise that they publish registry releases (as well as propagate registry releases from their upstreams). Those releases will be clearly marked in the index. Cargo will resolve them by default, unless the user opts out, including a way to take them on top of an existing lockfile. They will respect semver, and new maintainer releases will take precedence.
Registries that opt into this functionality will have a canonical way to expose a feed of registry releases they published, with metadata such as relevant advisories. Tooling will be available to audit that all registry releases are tagged properly and advertised in the feed.

Some design rules:

1. **Divergence from upstream is considered an attack unless declared.** A registry that serves releases not in its upstream must say so per release. Serving a subset of upstream's crates and versions is not divergence; a partial mirror is still a valid mirror. Any change to a crate version it does serve, including index metadata or bytes, is divergence. This applies to local vendored sources too, such as a `cargo vendor` directory in a distro build, and tooling such as `cargo vendor` and `cargo-auditable` should be able to record that divergence.
2. **Upstream releases always win.** A registry release never uses the maintainer's next version number. A real upstream release supersedes it on the next update.
3. **Consumers stay in control.** Resolving registry releases is opt-out. Lockfiles are respected unless the consumer opts in to taking registry releases on top of them.
4. **Usage is up to registries.** Our goal adds a spec that covers how to declare and advertise registry releases. We will add Cargo support for resolving them. But, who gets to publish these releases, what goes in them, and to what extent the contents are explained to consumers is up to each registry. crates.io may never enable it.
5. **Everything is auditable.** Each registry release links to a reason (e.g., an advisory), a base version, and a producer. This all appears in a feed that a third party can compare against upstream.

Out of scope:

- crates.io publishing its own registry releases. This is a policy decision that would require a much deeper discussion, RFC, and its own goal.
- Supporting modified bytes or metadata under an upstream version number in a registry served to others. The recommended path is a registry release that is clearly differentiated from the upstream version.
- Changes to non-Cargo build tools; they should keep working unchanged and may not recognize registry releases.

### Work items over the next year

This goal is co-authored by @jlizen and @celinval. @jlizen leads the inventory (survey, requirements, initial docs); the RFC and Cargo implementation are shared.

Two phases, roughly 3-6 months.

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Declared registry releases: spec, resolver support, docs | @jlizen, @celinval | |
| Feed, audit tooling, producer tooling | @jlizen, @CAD97 | |

#### Declared registry releases: spec, resolver support, docs

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Survey alternate registries and mirrors for how they diverge today | @jlizen | including behaviors that amount to dependency confusion in practice |
| Docs: describe how source replacement is used today and its risks; define terms (mirror, downstream registry, registry release) | @jlizen | |
| Requirements doc; gather use cases from registry operators, Alpha-Omega, Akrites, Linux packagers, Rust contracting shops (Ferrous Systems, Mainmatter) | @jlizen | |
| Requirements from the TUF / [verifiable mirroring](./mirroring.md) work: registry releases must fit the signing model | @jlizen | with @arlosi |
| RFC: Registry release declarations, Cargo resolver behavior, registry advertisement, consumer opt-out, lockfile interaction | @jlizen, @celinval | may split into two; informed by the [Central Patches draft](https://hackmd.io/GvCnMrbWSAuUo2fR8E2E-g) |
| Cargo implementation behind unstable flag | @jlizen, @celinval | |
| Reference implementation in an alternate registry or static index | @jlizen, @celinval | end to end without crates.io |
| RustSec advisory DB: represent "fixed in registry X with registry release Y" | @jlizen | with RustSec maintainers |
| Docs: update source-replacement / mirroring docs for registry releases | @jlizen | |
| Docs: Guidance for registry operators who diverge from upstream | @jlizen | |
| Stabilization | @jlizen, @celinval | |

#### Feed, audit tooling, producer tooling

| Task | Owner(s) | Notes |
| :-- | :-- | :-- |
| Design: event feed of registry releases, signing, relation to [verifiable mirroring](./mirroring.md) | @jlizen | |
| Design: provenance and how consumers verify registry release = base + diff | @jlizen | |
| Audit tool: diff a registry against its declared upstream, flag undeclared divergence | needs owner | |
| Tooling to produce a declared registry release from upstream `.crate` + diff | @CAD97 | |
| Record modified vendored crates as divergent in `cargo vendor` and `cargo-auditable`, so SBOMs don't report them as the upstream version | @CAD97 | `cargo-auditable` is maintained outside Cargo; coordinate with its maintainers |
| Represent registry releases and modified vendored crates with a distinct [PURL](https://github.com/package-url/purl-spec) in SBOM tooling | needs owner | coordinate with PURL spec maintainers (AboutCode) |
| Blog post: how downstream registries diverge today, the new declared model, and how to audit a mirror | @jlizen | after modeling + audit tooling land |

## Team asks

| Team | Support level | Notes |
| :-- | :-- | :-- |
| [cargo] | Large | Review: Resolver and lockfile semantics; source-replacement trust model and docs; RFCs and implementation |
| [crates-io] | Small | Steward the registry index spec changes so alternate registries interoperate; crates.io not expected to publish registry releases itself |
| [social-media] | Small | Advice + edits for blog post |

We also expect a Small review from the proposed ecosystem security team once it exists.

Feedback from outside the Project: RustSec maintainers, alternate registry operators (JFrog, Sonatype, Cloudsmith, kellnr), Linux distributions (Fedora, Canonical/Ubuntu, Debian), Alpha-Omega, Akrites, Rust contracting shops (Ferrous Systems, Mainmatter), external build tools (Bazel, Buck2, Yocto, Nix), corporate-managed downstream registries (Amazon, Microsoft, OpenAI), corporate-managed downstream package vendors (Google, Meta?, etc).

## Help wanted

| Task | Experience level | Time investment |
| :-- | :-- | :-- |
| Audit tool (registry vs. upstream diff) | Intermediate Rust | 1-2 months |
| Distinct PURLs for registry releases and modified vendored crates in SBOM tooling | Intermediate | 2-4 weeks |

## Frequently asked questions

### Isn't this just sanctioning dependency confusion?

Not quite. Today, downstream registry operators can only apply security or other fixes by doing what looks like a dependency confusion attack.
Under this goal, we will move away from "confusion" to an explicit dependency injection, that is clearly modeled and gives consumers
more control.

### Is a mirror that serves only some of crates.io diverging?

No. A mirror that serves a subset of upstream's crates and versions is a valid mirror. Any change to a crate version it serves, including index metadata or bytes, is divergence. Companies often vet a subset, some mirrors carry only the top N most-downloaded crates, and large mirrors may one day carry only popular crates and defer the rest to upstream. The spec should let a mirror declare "subset of upstream" separately from "publishes registry releases", and the audit tooling should treat missing crates differently from changed or added ones.

### Why not just `cargo update --precise` ([RFC 3493](https://rust-lang.github.io/rfcs/3493-precise-pre-release-cargo-update.html))?

Precise updates require that every consumer know about and point-install each release. Transitive consumers won't do that. We want
`cargo update` to pick it up. RFC 3493 also doesn't include any clear declaration that a release is registry-injected or allow audits or 
opt-out. The pre-release machinery is a good building block, but this goal extends it.

### Why not use `[patch.crates-io]` in each project?

`[patch]` requires every consumer to opt in, crate by crate, and to keep the list current as versions change. That often isn't practical
for organizations with many independent teams. Switching to a registry should be a one-line change in `config.toml`. The fix also has
to be hosted somewhere: a git URL is a muddy provenance story, and many locked-down build environments only allow a private registry.
Security boundaries are best enforced at the registry, with client configuration as a supplement.

### Who decides what releases a registry publishes?

A registry. From this goal's perspective, all users will be downstream of the Rust Project. Some expected users are: corporate security
teams running internal mirrors; a pre-approved remediation actor list for a public, shared mirror; a security team that produces fixes
that can be consumed by many mirrors.

### Does crates.io have to do anything?

No, besides review (optional) registry spec changes to ensure that there is one format. We don't expect crates.io to use this feature
in the short term, certainly not without a separate RFC/goal.

### How does this relate to TUF / verifiable mirroring?

They are complementary. Signing proves that bytes came from the registry that served them. This goal makes a registry declare which of those bytes are not upstream's. Depending on how signing lands, undeclared divergence may become impossible rather than merely detectable, which is the outcome we want. We will gather requirements from that work during the inventory so registry releases fit the signing model.

### Does this change lockfile behavior?

No. Lockfiles are respected by default. The RFC will include an opt-in way to take registry releases on top of an existing lockfile.

### What if a new upstream release doesn't include the registry's fix?

TBD, but an initial opinion: consumers who update would lose the fix, since upstream releases take precedence, so handling this is the registry's job. It can publish a registry release on top of the new upstream version, or hold back the new upstream version until it has checked it. Whether Cargo or the audit tooling should help (for example, by flagging an upstream release that supersedes a registry release, or carrying an advisory forward) is an open question for the RFC.

### Does this change semver?

Maybe. Registry releases need a version shape that sorts correctly, does not use the maintainer's next version, and is recognizable as not upstream's. Candidates include a reserved pre-release tag, a post-release identifier, build metadata, or a fourth version component. Each has costs for older Cargo versions and for tools outside Cargo. The inventory includes talking to the semver maintainers and crates.io about this. The RFC will weigh the options rather than assume one.

### Does this cover distro packaging and `cargo vendor` directories?

Yes. Patching a vendored crate is divergence, whether or not the vendored source reaches end users. We will consult packagers during the inventory, and add a way for `cargo vendor` and `cargo-auditable` to record a vendored crate as modified, so SBOMs and advisory tracking don't report it as the upstream version. Whether these builds should also use the registry release version scheme is an open question for the RFC.
