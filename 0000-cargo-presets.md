- Feature Name: cargo_presets
- Start Date: (fill me in with today's date, YYYY-MM-DD)
- RFC PR: [rust-lang/rfcs#0000](https://github.com/rust-lang/rfcs/pull/0000)
- Rust Issue: [rust-lang/rust#0000](https://github.com/rust-lang/rust/issues/0000)

# Summary
[summary]: #summary

Add presets to cargo, which are user defined build configurations stored in a file.

# Motivation
[motivation]: #motivation

Projects often have different build configurations for different purposes.
For example an application might have different backends, or it might be buildable for the web
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
Rust-analyzer has its own configuration options for target and features, which
the user has to set independently of other tooling, or manual invocations of cargo.

There are other examples of commonly used tooling, like [Bacon](https://github.com/Canop/bacon),
which also require passing the right cargo flags to.

Conflicting cargo flags arising from forgetting to pass the exact right arguments for every tooling
and cargo itself will invalidate the current build, and can be the source of a lot of unnecessary
lengthy rebuilds.

If there was a way to define a set of build configurations (**presets**), the user would only
have to remember the name of the preset, instead of having to remember all the right flags.

Moreover, if there was a way to set a default preset, then cargo could use that information to always
include the flags defined by the preset in its invocations (unless a command line flag overrides it).
This would:
- Synchronize tooling that invokes cargo (bacon, rust-analyzer):
    They would just need to invoke cargo normally, without having to pass any additional arguments
- Synchronize tooling that reads metadata from cargo (rust-analyzer):
    Cargo would automatically use the right flags read from the preset, informing rust-analyzer
    of the current set of features, as well as the active target, etc.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

Presets are a feature of Cargo that allow defining a set of build configurations for a project.

What is a build configuration?

Let's use an application that has both *desktop* and *web* versions as an example.

Let's also say that the desktop version also has two different windowing backends, expressed
using [features](https://doc.rust-lang.org/cargo/reference/features.html):
`window-backend-1` (default) and `window-backend-2`.
Let's presume that despite `window-backend-1` being the default, `window-backend-2` works
better on your system, so you would like using it as you are working on the project.

The low level way to do this is to pass `--no-default-features --features=window-backend-2` to
cargo every time. However, this can be difficult to remember, especially for more
complex feature configurations.

The solution is to define a **preset** for your backend.

Create a TOML file named `.cargo/presets.toml` in your project's root folder:
```toml
[preset.mybackend]
default-features = false
features = ["window-backend-2"]
```

Now you can simply pass `--preset mybackend` to cargo when you want to use your preferred backend.

Let's also add the web backend:
```toml
[preset.mybackend]
default-features = false
features = ["window-backend-2"]

[preset.web]
# You can also set the target
target = "wasm32-unknown-unknown"
default-features = false
features = ["web-backend"]
```

Similarly, you can pass `--preset web` to cargo when you want to build the web version.

While this is more convenient and less error-prone than having to pass the exact flags to cargo every time,
you still have to pass which preset you want to build to cargo every time.
And what about tools like Rust-analyzer? Does the preset to use have to be configured separately
for each tooling?

The real power of presets lie in the fact that you can set a default preset:

```toml
# Now `mybackend` is the default preset
default = "mybackend"

[preset.mybackend]
default-features = false
features = ["window-backend-2"]

[preset.web]
# You can also set the target
target = "wasm32-unknown-unknown"
default-features = false
features = ["web-backend"]
```

When there is a default preset set, `cargo` will automatically use the build configuration defined
by that preset.
This works for all tooling that invokes `cargo`, including Rust-analyzer, which queries cargo
for the build configuration.

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
