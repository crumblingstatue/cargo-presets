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

Presets are essentially sets of build options that cargo uses in absence of a command line flag
that sets that build option.

If a command line flag (`--target`, `--features`, `--no-default-features`) is present, it overrides
the defaults defined by the preset.

Presets are read from `.cargo/presets.toml`, similarly to how `.cargo/config.toml` is handled.
Always the most specific `.cargo/presets.toml` is used. If a `.cargo/presets.toml` file is found,
no further attempts are made to read and merge other `.cargo/presets` files.

Once found, the presets file is read as a TOML document, and parsed into a data structure that
stores fallback build options to use in absence of corresponding command line flags.

For example if `--target` or `--features` is not defined, cargo will use their corresponding fallbacks.
Any command line flag completely overrides its corresponding fallback. There is no attempt to merge a command line flag
and its corresponding fallback in any way.

However, a command line flag only overrides its corresponding fallback.
For example, if a preset defines both `target` and `features`, passing a different
`--target` flag will only override the `target` fallback. The `features` fallback will still be utilized,
unless there is also a `--features` flag present, which would override it.

# Drawbacks
[drawbacks]: #drawbacks

- This makes cargo slightly more complex.

    It should be a relatively simple addition though.
    I don't expect the implementation to add over 1000 lines.

- Cargo has to read an additional file.

    However, this shouldn't have a significant performance impact, as it's only done once,
    during cargo's initialization phase.

There should be no impact to users who don't use the presets feature.
# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

When I initially brought up this feature suggestion, one of the responses was that this feature
might have a better place in a higher level tool on top of cargo, instead of cargo itself.
However, such a higher level tool doesn't exist at this time, and it's unknown if it will
ever exist in such a way that integrates with tooling that currently invoke cargo (like Rust-analyzer).
Without being able to integrate with tooling, such a feature would lose most of its advantages, because
the user would still have to configure all their tooling manually, instead of being able to define
their preferred configuration once, centrally, and have all tooling automatically use it.

Even if such a higher level tooling existed in the future, it could write to the presets file, and
existing tooling wouldn't have to do anything special to invoke cargo with the right flags.

Another alternative is to implement this as a cargo wrapper (see <https://github.com/crumblingstatue/cargo-presets/>), which
is a drop-in replacement for cargo, intercepting its command line flags, and overwriting them.
While possible, it's unreasonable to expect users who want this feature to install and configure a cargo
wrapper, which needs to override the original cargo executable to be usable.
Cargo wrappers are also not composable, because you can only reasonably use one cargo wrapper, at least
without special careful setup of an invocation chain, which a regular user can't be expected to do.

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
