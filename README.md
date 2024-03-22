# Presets for cargo

This repo contains an (upcoming) RFC for presets, which is a feature for
defining reusable build configurations which integrate well with tooling like
rust-analyzer and others.

It also contains an implementation of the RFC using a cargo wrapper executable.
The implementation currently only supports unix
(it uses [CommandExt::exec](https://doc.rust-lang.org/nightly/std/os/unix/process/trait.CommandExt.html#tymethod.exec)).

What do presets look like?
Presets files are read from `.cargo/presets`, similarly to `.cargo/config`.
Here is an example presets file I'm using for my project, [simple kana input](https://github.com/crumblingstatue/simplekanainput):
```toml
# Optional default preset
# If set, and no `--preset` argument is given to cargo, it will use this preset by default.
default = "my-preferred"

# Presets

# The SFML backend has better behavior on my system, but it's harder to build.
# I made the `eframe` backend the default, so users can build the project easily,
# but I personally prefer using the `sfml` backend.
[preset.my-preferred]
default-features = false
features = ["ipc", "backend-sfml"]

# The web version (https://crumblingstatue.github.io/simplekanainput/)
# If I just want to build it, I can build it using `cargo build --preset web`.
# If I want to hack on it for a longer period, I can set the default preset to this,
# and all tooling like `cargo`, `rust-analyzer` and `bacon` will use this configuration,
# without having to pass the right flags.
[preset.web]
default-features = false
target = "wasm32-unknown-unknown"
features = ["backend-eframe"]
```

## Building and using the wrapper
The cargo wrapper needs to know the location to the original cargo executable, in order
to be able to run it.
It reads this from the `PATH_TO_CARGO_EXEC` environment variable at build time,
so you need to set this environment variable before building.

To use it, first build the wrapper, then place it into your local bin folder as `cargo`.

For example, I use it as `$HOME/bin/cargo`.

`$HOME/bin` is in my `PATH` before `/usr/bin/`,
so `$HOME/bin/cargo` will be used instead of `/usr/bin/cargo`.