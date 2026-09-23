# Implement Verifiable Mirroring Prototype

| Metadata             |                                    |
| :--                  | :--                                |
| Contact     | @walterhpearce                     |
| Status               | Accepted                           |
| Tracking issue       | [rust-lang/goals#637] |
| Zulip channel        | [#tbd-signing]                     |
| [rustup] champion    | @djc                               |
| [cargo] champion     | @arlosi                            |
| [infra] champion     | @Mark-Simulacrum                   |
| [crates-io] champion | @LawnGnome                         |


[#tbd-signing]: https://rust-lang.zulipchat.com/#narrow/channel/417663-tbd-signing

## Summary

We aim to ship a Minimum Viable Product that provides cryptographically verified mirrors for Rustup and Cargo, specifically targeting high-traffic environments like GitHub Actions (GHA) runners on Azure. By utilizing [The Update Framework (TUF)][tuf], we will establish a secure, multi-key distribution model that reduces infrastructure costs while providing for utilizing TUF as a validating mechanism on the backend transfers for mirroring, while integrating the needed unstable features into Rustup and Cargo for implementation. Our goal is to implement a first trial pass of [RFC #3724], with modifications, allowing for mirrors of Rust releases and crates to be configurable or automatically utilized by the Rustup toolchain.

[tuf]: https://theupdateframework.io/

We plan to trial TUF validation on these mirrors in a phased approach, starting on the server-side prior to any client-side integrations. Specifically, we plan to work in the following phases:

1. Implement unstable mirror URL specifications for client software (rustup and cargo)
2. Deploy infrastructure at alternative locations for mirroring (Azure, GCP)
3. During mirror propagation, on the server side, utilize TUF to validate mirror updates
4. Upon coming to consensus on this backend validation, we will migrate the implementations to unstable client-side features

## Motivation

### The status quo
While Rustup and Cargo have basic building blocks for mirrors, they lack first-class support and robust security. Currently, artifacts are often signed with legacy keys on individual laptops, and users must trust the HTTPS endpoint of the infra providers implicitly.

For the past 2 years, we have seen multiple iterations of artifact signing exploration for the Rust Project (PKI, TheUpdateFramework). [^rustweek-2025] As we continue testing and exploring out-of-band prototypes for these solutions, we need to align on what a more concrete mirroring solution for Rust may look like. We want to investigate and implement a first-pass solution for officially mirroring both Rust releases and crates.io packages. This problem space is parallel but distinct from signing; specifically, we need to answer the question: "Assuming all artifacts can be cryptographically validated, how would official and private mirrors for Rustup and crates.io operate?". This goal is to determine what solutions are available to us, what the current hurdles in our ecosystem are, and implement a solution for mirroring Rust releases at different endpoints and the integration of this implementation into Rustup.

### What we propose to do about it

We have decided to prioritize shipping and iteration over extended theoretical discussion. This goal is to implement a working unstable mirroring solution in Rustup and cargo, allowing for fetching artifacts and verifying their integrity from the existing index and an out-of-band TUF repository. 

This approach addresses several immediate needs:
* Cost Reduction: Bandwidth for Rustup and logging for crates.io are significant costs; official mirrors on Azure can mitigate these for GHA traffic.
* Security Infrastructure: Implementing TUF for Rustup—which changes less frequently than crates.io—allows us to practice key signing ceremonies and establish rotation strategies in a controlled environment.
* Incremental Changes: Building a functional prototype allows us to test "mirror discovery" and consistency models (e.g., DNS-based mirrors or local network advertisements) in the real world.


## Technical Strategy

We propose implementing a suite of standard tools for mirroring various degrees of the rust-lang and crates.io artifacts to secondary services. We will stand up additional infrastructure in a new cloud environment (Azure, GCP) which will conduct eventually-consistent synchronization of all artifacts in the rust-lang distribution channels as well as crates.io. Additionally, we will deploy TUF repositories to validate these synchronizations using client tools. These repositories will mirror rust-lang releases and the crates.io crates artifacts to a new suite of URLs. 

We propose crafting a set of mirroring tools for these new locations, where we can then utilize TUF cryptographic verification to confirm the consistency of these mirrors. This will give us the ability to test prototype solutions of TUF and their scalability, transparent to end users. However, this approach will require implementing and deploying full mirrors for us to begin the work. 

This work can be broken up into these concrete milestones:
1. Deploy infrastructure for mirrors on Azure and GCP from AWS (Synchronization host, and storage buckets on each)
2. Implement tooling for conducting eventually-consistent mirroring to these locations
3. Implement internal TUF repositories to cryptographically verify these mirrors
4. Implement unstable features in both rustup & cargo to optionally utilize these mirrors
5. Iterate of TUF implementations used during synchronization


### Rustup

We will implement an ability to target different rust-lang.org URLs for mirrors, configurable on the client side to point at a new CDN source. 


### Cargo

We will implement experimental and unstable features for specifying default mirrors. We hope to come to a consensus solution for specifying and redirecting main crates.io artifact requests to a configured mirror.


### Verification & Security (TUF)

We will deploy an prototype TUF repository which provided signing for the rust release channels manifest and artifacts. This repository will be used strictly on the rust-lang channel manifests and files, which will allow us to self-verify our prototype mirror utilizing TUF. We believe this will allow us to begin testing the scalability of TUF for rust-lang internally, without directly impacting users.


#### Rustup TUF Repository

There are two solutions which we will compare for scalability of the rust-lang artifacts.

##### 1. Direct artifact Repository
The TUF repository which we will utilize internally will be periodically synced to the rust-lang manifests structure; parsing off of manifests.txt and iterating all channels TOML files for building out the repository. This will be accomplished by:

1. Iterate manifests.txt and build out a dataset all of channels and rust releases
2. Channel artifacts will be collected in a set of URLs and SHA256 hash pairs, which exist within the channel manifest files
3. URLS will be striped to exempt the rust-lang URL, allowing our repository targets to be the CDN path to match. (ex: 2026-02-12/cargo-1.93.1-aarch64-apple-darwin.tar.gz)
4. The SHA256 hash from the manifest and the striped path will become a target artifact in the TUF repository


##### 2. Channel Manifest Repository

This version of the TUF repository can become a standard, unmodified TUF repository of the dist channel files themselves. Instead of extracting artifact SHA256 files, we would
have this TUF repository verify all channel files specified from manifests.txt and associated channel TOML files. This is possible because all channels also verify SHA256 hashes of the dist files - giving us transitive cryptographic security. Additionally, files are not modified or updated.
1. Iterate manifests.txt and collect all channel TOML files
2. All channel TOML files will be added to the TUF target as artifact files
3. On channel updates and checks, we will verify the channel file via TUF


#### Crates.io TUF Repository

We have multiple approaches to a TUF repository for crates.io which we will experiment with.

##### 1. Merkle Proofs

@arlo-siemsen and @walterhpearce have competing implementations of a Merkle Tree Proof for verifying crates either via TUF directly or via the Cargo resolver. Both of these cases build upon utilizing TUF only for Merkle roots - we will experiment with these solutions to see if they meet our round-trip and bandwidth needs.

##### 2. Index TUF Repository

We will experiment with using classic TUF and the Merkle proofs for the index files instead of direct artifacts. This allows us to reduce our TUF footprint from n=(crates x versions) to only N=crates. This value is still growing and will continue to infinitely grow, but allows us to reduce our artifact count to a more predictable and reasonable number for experimentation.

##### 3. Direct Artifact Repository

We have decided not to explore this method, as our exponential artifact growth (250,000 and growing) and update frequency (3-5 a minute) are prohibitive to the TUF design.

### Tooling Integration
- Rustup: Will be the first tool to support the new signing and mirror selection logic.
- Cargo: We will begin design work on a configurable "mirror registry" field, allowing Cargo to fall back to alternative sources if the primary is unreachable.

- Mirroring Tool:
  - Mirror can be deployed via a new specified URL
  - Ability to sync and copy out rust-releases full mirrors (5tb) to other providers
  - Time gating mirror to N years to preserve space (or not)
  - TUF verify on mirror update

## Work items over the next year

| Task | Owner(s) | Notes |
| :--- | :--- | :--- |
| Rustup Mirror Configuration Implementation | @walterhpearce | Implement mirror configurability and redirection in Rustup. |
| TUF Implementation | @walterhpearce | Integrate TUF verification into Rustup client. |
| Azure/GCP Mirror prototype | @walterhpearce,@simulacrum | Deploy and test the first official  mirror. |
| Mirror Registry Design | @arlosi | Draft Cargo chawalterhpearcenges for mirror discovery and fallback. |
| Key Signing Ceremony | @walterhpearce | Record a demo of the multi-key signing process. |
| Sync/Mirror Agent | @walterhpearce | Develop the standardized tool for mirror synchronization. |
| RFC Update & Submission | @walterhpearce | Formalize the "Rust Project Mirroring" architecture based on prototype results. |

## Team asks

| Team | Support level | Notes |
| :--- | :--- | :--- |
| [cargo] | Medium | Support needed for registry field design and resolver consistency. |
| [rustup] | Medium | Required for integrating the prototype into the primary toolchain installer. |
| [infra] | Medium | Critical for setting up the signing pipeline and Azure deployment. |
| [crates-io] | Small | Primarily focused on potential future logging/bandwidth savings. |

[^rustweek-2025]: See a [related discussion](https://rust-lang.zulipchat.com/#narrow/channel/417663-tbd-signing/topic/.E2.9C.94.20Notes.20from.20Rust.20Week.20discussion/with/525551676).
