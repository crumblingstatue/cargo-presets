- Feature Name: cargo_presets
- Start Date: (fill me in with today's date, YYYY-MM-DD)
- RFC PR: [rust-lang/rfcs#0000](https://github.com/rust-lang/rfcs/pull/0000)
- Rust Issue: [rust-lang/rust#0000](https://github.com/rust-lang/rust/issues/0000)

# Summary
[summary]: #summary

Add presets to cargo, which are named sets of user defined build configurations for different purposes.

# Motivation
[motivation]: #motivation

Projects often have different build configurations for different purposes.
For example a project might have different backends, or it might be buildable for the web
using WebAssembly.

Currently, if someone wants to build such a specific build configuration, they always have to provide
the right flags to cargo for every invocation.

Here is an example of what those flags might look like:
- Web build configuration
    - `--target=wasm32-unknown-unknown`
    - `--default-features=false` (there might be features that need to be disabled on the web)
    - `--features=web,specific,features`
- Configuration for an alternate backend
    - `--default-features=false` (disable the default backend)
    - `--features=alternate-backend`

These flags also have to be set independently for every piece of tooling, most notably Rust-analyzer.
Rust-analyzer for example has its own configuration options for target and features, which
the user has to set independently of other tooling, or manual invocations of cargo.

There are other examples of commonly used tooling, like [Bacon](https://github.com/Canop/bacon),
which would also require passing the right cargo flags to.

Conflicting cargo flags arising from forgetting to pass the exact right arguments for every tooling
and cargo itself will invalidate the current build, and can be the source of a lot of unnecessary
lengthy rebuilds.

If there was a way to define a set of build configurations (**presets**), the user would only
have to remember the name of the preset, instead of having to remember all the right flags.

Moreover, if there was a way to set a default preset, then cargo could use that information to always
include the flags defined by the preset in its invocations.
This would:
- Synchronize tooling that invokes cargo (bacon, rust-analyzer):
    They would just need to invoke cargo normally, without having to pass any additional arguments
- Synchronize tooling that reads metadata from cargo (rust-analyzer):
    Cargo would automatically use the right flags read from the preset, informing rust-analyzer
    of the current set of features, as well as the active target, etc.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

Presets are a feature of Cargo that allow defining a set of build configurations for a project.
For example, a project might have a web version, or different backends.
Without presets, one would have to pass the right flags to Cargo for each invocation.
An example might be `cargo build --default-features=false --features=web,specific,features` for the web version.

Explain the proposal as if it was already included in the language and you were teaching it to another Rust programmer. That generally means:

- Introducing new named concepts.
- Explaining the feature largely in terms of examples.
- Explaining how Rust programmers should *think* about the feature, and how it should impact the way they use Rust. It should explain the impact as concretely as possible.
- If applicable, provide sample error messages, deprecation warnings, or migration guidance.
- If applicable, describe the differences between teaching this to existing Rust programmers and new Rust programmers.
- Discuss how this impacts the ability to read, understand, and maintain Rust code. Code is read and modified far more often than written; will the proposed feature make code easier to maintain?

For implementation-oriented RFCs (e.g. for compiler internals), this section should focus on how compiler contributors should think about the change, and give examples of its concrete impact. For policy RFCs, this section should provide an example-driven introduction to the policy, and explain its impact in concrete terms.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

This is the technical portion of the RFC. Explain the design in sufficient detail that:

- Its interaction with other features is clear.
- It is reasonably clear how the feature would be implemented.
- Corner cases are dissected by example.

The section should return to the examples given in the previous section, and explain more fully how the detailed proposal makes those examples work.

# Drawbacks
[drawbacks]: #drawbacks

This makes cargo slightly more complex, and has to read an additional file.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

- Why is this design the best in the space of possible designs?
- What other designs have been considered and what is the rationale for not choosing them?
- What is the impact of not doing this?
- If this is a language proposal, could this be done in a library or macro instead? Does the proposed change make Rust code easier or harder to read, understand, and maintain?

# Prior art
[prior-art]: #prior-art

Build systems that have a configure step (for example CMake), allow generating different configurations,
which can be independently build without having to respecify all the options.
IDEs can even collect information about these configuration, and present a dropdown menu with
selectable configurations. Qt Creator for example supports this using CMake.
Rust-analyzer could potentially support a similar dropdown menu as well in editors that allow such a thing.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

- Could this be a part of `.cargo/config.toml`? Would it make sense for it to be?

# Future possibilities
[future-possibilities]: #future-possibilities

- Allow more options than just `default-features`, `features`, and `target` (any good candidates?)
