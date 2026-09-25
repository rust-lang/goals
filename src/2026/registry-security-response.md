# Project Goal: Registry security response: hold, withdraw, publish-time checks


| Metadata            |                                    |
| :--                 | :--                                |
| Contact         | @jlizen                     |
| Status          | Proposed                                     |
| Zulip channel   | TODO(link)        |
| [crates-io] champion     | ??                       |
| [cargo] champion | ??                           |

## Summary

We want an auditable way for crates.io administrators to **hold** (reversibly "freeze" releases for investigation, 
preventing normal fetching of bytes) and **withdraw** (remove release bytes, leaving behind a tombstone). This is necessary to 
allow a reversible way to quickly "freeze" a situation during a security investigation. It also lets us set up
auditable mechanisms to automatically hold supply chain attacks before they are released to the mirror.

We will use this capability in two ways:
1. manual admin action, via API or console, similar to current "admin delete" workflows
2. automated publish-time checks that, upon suspicious signals, place releases on hold and into a "manual review" queue

Our focus here is on building security primitives, with some simple detections wired up. Further detections and more sophisticated usage of these primitives will require separate Project Goals and/or RFCs.

## Motivation

### The status quo

crates.io today has three states:
- fully public
- public but "yanked" (installable via `Cargo.lock` coordinates, but don't resolve otherwise)
- deleted (bytes inaccessible, redacted from registry index, admin notifications are manual)

This makes life difficult from an incident response point of view. In a recent security incident, a stolen token published malware across an account of ~250 crates with hundreds of automated versions. The responder had to "wade through LLM spam of hundreds of versions" and chain together a script with ~70 yank & delete commands. In fact, one deletion failed and the related version lasted for a few extra days. According to the operator: "I kinda wish we had an intermediate step here. Deletion is only semi-reversible, but I would like to publicly nuke the account while we investigate."

Meanwhile, we recently saw a [successful supply chain attack on the arrayref crate](https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/), which is present in ~75% of Rust environments and has ~250 million downloads. 
Among other attack elements, a malicious, namesquatting crate was published to crates.io, and then a compromised 
credential cut `arrayref` over to depending on it. There are a number of deterministic signals here that are clearly suspicious: a popular crate adding a new build dependency, a popular crate taking a dependency on a typosquat, a crate bearing base64 encoded URLs in its build script and other obvious malware signs, and so on.

In fact, for the `arrayref` incident, several Rust Foundation security systems did in fact trigger (for instance, the namesquatting detection). But, no automated action is taken by default, and signal quality is not high enough to page ourselves on those signals that we did detect. Instead, we manually pulled the malicious crate 86 minutes after publication, in response to an external vulnerability report from a security researcher.

We [recently stabilized min-publish-age in cargo](https://github.com/rust-lang/cargo/pull/17335), which allows setting a "cooldown" to give time for security reports and admin action before clients uptake malicious releases.
I expect that we will soon set a default min-publish-age for cargo. This will be useful and complementary change. 
Most reasonable default cooldowns would be longer than 86 minutes, meaning the arrayref attack would have much more limited impact. 
However, this still produces a "firedrill" for security responders where our release process fails open in case of a 
delayed response. This is a concerning operational posture given that we expect the supply chain attacks to grow both 
more frequent and more sophisticated. It also relies on client-side configuration that does not extend to other build tools, or tools that override the default cargo configuration.

### What we propose to do about it

We imagine three phases of work. Each will have a RFC.

1. **A registry held/withdrawn state, matching Cargo behavior, and an API for retrieving held bytes.** First, we can add
a `status` field to the index capturing held/withdrawn state. Held bytes are located via a different registry path,
with a new `dl-unsafe` field. Security researchers and crate publishers can explicitly opt into fetching held bytes via
specific crate/version coordinates. All other usages hopefully route around those versions, or else loudly fail. docs.rs will skip building held or withdrawn crates. This requires reviews from cargo, crates.io, and docs.rs teams.
2. **Admin APIs in crates.io to hold and withdraw releases, along with other bulk admin actions.** Next, we can add some
new admin APIs in crates.io to actually trigger a hold or withdraw (including a new database table, new crates.io 
display tags, and so forth). While we are there, we can add some sensible bulk-admin-action APIs to improve our 
incident response capabilities. This will only need review from crates.io team (though advisory review from infra would be great!) and will rely on existing consensus around
authority for admin delete/yank actions.
3. **A publish-time check + manual review queue, supporting admin console** Last, we set up systems to do automated checks
for obvious supply chain attacks, and punt them to a human review queue. This RFC will involve defining a review SLA, a criteria from moving detections from "shadow" mode to actioning on release, some initial detection systems, and operational logistics. crates.io team is the only technical reviewer for this RFC, but we would look for broader consensus
across the project with an emphasis on avoiding toil for maintainers and consumers.

## Team asks

@jlizen plans to do most of the implementation work for this across all relevant systems, possibly delegating some to
contractors or Rust Foundation teammates. The team asks are for feedback on approach and reviews.

| Team | Support level | Notes |
|------|---------------|-------|
| [cargo] | Medium | Review and approve RFC 1 (registry `status` state, resolution, error and publish behavior, `dl-unsafe` web API contract); review implementation of RFC 1 |
| [crates-io] | Medium | Co-review and approve RFC 1 (registry status state, `dl-unsafe` web API); review and approve RFC 2 (manual admin capability, byte management, authorization); review implementation of RFC 2; review and approve RFC 3; review implementation of RFC 3 |
| [docs-rs] | Small | Review and approve RFC 1 with regard to changes to build conditions; review the implementation of RFC |
| [infra] | Small | Advisory consult on RFC 2 and RFC 3 |

Beyond Rust Project teams, we will also want to consult with the Rust Foundation, particularly its security team. We
also will want to consult with outside build tools (Bazel, Buck2, Yocto). Our designs should maintain security boundaries
without external build tool changes, but they could make UX improvements if they were aware of our new index states.


## Frequently asked questions

### What's the point of holding already-released malware?

Users continuously install software in CI and otherwise. Even if a version has been released, there is benefit in
holding it to prevent subsequent installs. Lowering the threshold for taking admin action via a softer mitigation will
allow faster responses. Even better, we're working towards holding software before it is even released in the first place. 

### Does this add work for maintainers?

For RFC 1 and RFC 2, this is strictly a softening of the current mitigation available (ie, delete). It's also a quality
of life improvement for incident responders via bulk actions. For RFC 3 (automatic scan / hold + manual review queue),
there is a potential for maintainer impact. The RFC will go more in depth around the risks and controls for this. In
general, the guiding principle will be, run in shadow mode for a while, make sure we have acceptable false-positive rates, and only then promote a detection to acting. This protects both maintainers from toil, and the crates.io and security teams reviewing the queue from overload. Similarly we will need to specify a SLA, an appeal mechanism,
and other nuts and bolts.

### Who decides what gets held? What is the trust model?

This concern should be decided at the registry level. For RFCs 1 and 2 (the manual hold), we can use the existing
criteria that we use for admin deletion. We also can reuse the [existing malicious crate channels](https://blog.rust-lang.org/2026/02/13/crates.io-malicious-crate-update/) (along with new tombstones in the index files).

For RFC 3 (the automatic hold), we need to hash this out still. I imagine starting narrow, with deterministic checks,
and shadow mode to judge impact, will be a good place to start.

### Why three RFCs?

We'd prefer each individual RFC to be relatively small to keep it manageable to review and find consensus. Specifically,
we are designing the registry and cargo behaviors (RFC 1), separately from the actual distributed systems work that crates.io will implement on top of it (RFC 2). We expect RFC 3 to be much more complex to find consensus on so would preserve to build the foundation for it separately.

### What are we leaving out?

The biggest thing is revocation of already-cached copies of crates. That is worth doing, but as a separate effort. We
also erred on the side of simple UX in a few other places. Details are in the RFCs.