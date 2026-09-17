# Stabilize Cargo SBOM precursor

| Metadata              |                                                 |
| :--                   | :--                                             |
| Contact      | @Shnatsel                                       |
| Status                | Accepted                                        |
| Tracking issue        | [rust-lang/goals#649]              |
| Other tracking issues | [rust-lang/cargo#16565]                         |
| Zulip channel         | N/A                                             |
| [cargo] champion      | @weihanglo                                      |
| Funding contact       | @Shnatsel |


## Summary

Progress towards an MVP version of Cargo SBOM support by resolving known issues in Cargo's [SBOM precursor feature](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#sbom) and finalizing the RFC.

**Needs contributor:** This goal needs contributors to help with testing, resolving known issues in Cargo's SBOM precursor, and converting downstream tooling like cargo-cyclonedx. The work is primarily in the [rust-lang/cargo](https://github.com/rust-lang/cargo) repository.

## Motivation

[Software Bill of Materials](https://en.wikipedia.org/wiki/Software_supply_chain) is a list of project dependencies and their versions, analogous to Cargo.lock, in a format standardized across programming languages. They enable supply chain transparency and allow easily identifying dependencies with known vulnerabilities.

SBOMs are turning from a best practice to being mandatory. In the US [Executive Order 14028](https://www.nist.gov/itl/executive-order-14028-improving-nations-cybersecurity) requires the federal government to only purchase software from vendors who provide a Software Bill of Materials for each product. In the EU the [Cyber Resilience Act](https://eur-lex.europa.eu/eli/reg/2024/2847/oj) mandates that any product with "digital elements" sold in the EU must have an SBOM as part of its technical documentation; obligations for reporting vulnerabilities begin in September 2026, with full compliance required by December 2027. Many other jurisdictions have similar regulations.

### The status quo

The crucial missing piece for SBOM generation for Rust+Cargo projects is accurate reporting of the dependency tree. `cargo metadata` falls short in [multiple](https://github.com/rust-lang/cargo/issues/7754) [ways](https://github.com/rust-lang/cargo/issues/10718). This results in either false negatives or false positives in the reported dependency tree.

The [SBOM precursor](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#sbom) feature in Cargo addresses this by providing a mechanism to accurately report the dependency tree used in a given build. However, it is nightly-only, not yet widely used, and has [at least one known issue](https://github.com/rust-lang/cargo/issues/15695).

Inaccurate SBOMs lead to false positives on vulnerability scans and/or compliance issues.

### What we propose to do about it

1. Complete [RFC #3553] for this feature and get it accepted
1. Resolve the already known issue(s) in the Cargo SBOM precursor feature
1. Modify [cargo-cyclonedx](https://crates.io/crates/cargo-cyclonedx) to use the Cargo SBOM precursor as a data source, to prove that it can be used to generate a complete and accurate SBOM in an industry standard format
1. Address any issues that point 2 uncovers in the Cargo SBOM precursor feature
1. Stabilize the MVP that is sufficient to power [cargo-cyclonedx](https://crates.io/crates/cargo-cyclonedx) and [cargo-auditable](https://github.com/rust-secure-code/cargo-auditable)

### Work items over the next year

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Complete the RFC | @Shnatsel | @weihanglo can review and discuss blockers with [cargo] |
| Resolve known issues | @Shnatsel | @weihanglo can review PRs and may help with issues that would benefit from maintainer attention |
| Convert cargo-cyclonedx to use the SBOM precursor | @Shnatsel et al. | outside the Rust Project repositories, no Rust Project mentorship needed |
| Resolve newly uncovered issues | @Shnatsel |       |
| Stabilize the MVP | @Shnatsel |       |

I am in the process of applying for funding for this work, together with collaborators I'm not sure I can disclose. The amount of time we can dedicate to the project will depend on the outcome of that application. It is possible that the funding will only materialize in the second half of the year or not at all.

## Team asks

We will need:

 - Guidance to get the RFC finalized and accepted
 - A handful of 30-minute design meetings with someone on the Cargo team to guide fixing the implementation issues
 - Guidance on the stabilization process

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [cargo]    | Medium |                                         |

## Help wanted

| Task | Experience level | Time investment |
|------|-----------------|-----------------|
| Test Cargo's SBOM precursor | TBD | TBD |
| Resolve issues in the SBOM precursor | TBD | TBD |
| Convert cargo-cyclonedx to use the SBOM precursor | TBD | TBD |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor (3 months, part-time) | $40,000 | No | |

## Frequently asked questions

TODO - will fill in based on the review comments
