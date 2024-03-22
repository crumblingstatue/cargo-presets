use {
    crate::presets::{find_presets_file, Presets},
    presets::Preset,
    std::{
        ffi::OsString,
        fmt::Write as _,
        os::unix::process::CommandExt,
        process::{Command, ExitCode},
    },
};

mod presets;
#[cfg(test)]
mod tests;

fn strip_preset_arg(args: &mut Vec<OsString>) -> Option<String> {
    match args.iter().position(|arg| arg == "--preset") {
        Some(idx) => {
            args.remove(idx);
            if args.get(idx).is_some() {
                Some(args.remove(idx).to_string_lossy().into_owned())
            } else {
                None
            }
        }
        None => None,
    }
}

fn with_actionable_command(
    args: &mut Vec<OsString>,
    f: impl FnOnce(&CommandMeta, &mut Vec<OsString>),
) {
    if let Some(first) = args.first().and_then(|arg| arg.to_str()) {
        let cmd_meta = command_meta(first);
        if cmd_meta.needs_config() {
            f(&cmd_meta, args);
        }
    }
}

fn main() -> std::process::ExitCode {
    let mut args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let preset_arg = strip_preset_arg(&mut args);
    with_actionable_command(&mut args, |cmd_meta, args| {
        if let Some(path) = find_presets_file() {
            let presets = Presets::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
            if let Some(preset) = determine_preset(&presets, preset_arg.as_ref()) {
                inject_args(cmd_meta, preset, args);
            }
        }
    });
    let mut cmd = Command::new(env!("PATH_TO_CARGO_EXEC"));
    cmd.args(args);
    #[cfg(feature = "log_invocations")]
    log_invocation(&cmd);
    let err = cmd.exec();
    eprintln!("Error execing cargo: {err}");
    ExitCode::FAILURE
}

#[cfg(feature = "log_invocations")]
fn log_invocation(cmd: &std::process::Command) {
    use std::io::Write as _;
    let f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/cargo-preset-log.txt")
        .unwrap();
    writeln!(
        &f,
        "{:?}",
        cmd.get_args().collect::<Vec<_>>().join(" ".as_ref())
    )
    .unwrap();
}

fn inject_args(cmd_meta: &CommandMeta, preset: &Preset, args: &mut Vec<OsString>) {
    let mut additional_args: Vec<OsString> = Vec::new();
    if cmd_meta.has_features {
        if preset.no_default_features {
            additional_args.push("--no-default-features".into());
        }
        if !preset.features.is_empty() {
            additional_args.push("--features".into());
            let mut feature_str = String::new();
            for feature in &preset.features {
                eprintln!("Injecting feature {feature}");
                write!(&mut feature_str, "{feature},").unwrap();
            }
            additional_args.push(feature_str.into());
        }
    }
    if cmd_meta.has_target && !preset.target.is_empty() {
        additional_args.push(format!("--target={}", preset.target).into());
    }
    args.splice(1..1, additional_args);
}

fn determine_preset<'p>(presets: &'p Presets, preset_arg: Option<&String>) -> Option<&'p Preset> {
    let preset_id = match preset_arg {
        Some(arg) => arg,
        None => {
            if presets.default.is_empty() {
                return None;
            } else {
                &presets.default
            }
        }
    };
    eprintln!("Using preset {preset_id}");
    match presets.presets.get(preset_id) {
        Some(preset) => Some(preset),
        None => {
            eprintln!("Default preset '{}' not found in presets", presets.default);
            None
        }
    }
}

struct CommandMeta {
    has_target: bool,
    has_features: bool,
}

impl CommandMeta {
    fn needs_config(&self) -> bool {
        self.has_target | self.has_features
    }
}

fn command_meta(command: &str) -> CommandMeta {
    match command {
        "check" | "c" | "build" | "b" | "test" | "t" | "run" | "r" | "rustc" | "clippy" => {
            CommandMeta {
                has_target: true,
                has_features: true,
            }
        }
        "metadata" => CommandMeta {
            has_target: false,
            has_features: true,
        },
        _ => CommandMeta {
            has_target: false,
            has_features: false,
        },
    }
}
