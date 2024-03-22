use {
    crate::presets::{find_presets_file, Presets},
    presets::Preset,
    std::{
        fmt::Write as _,
        io::Write as _,
        os::unix::process::CommandExt,
        process::{Command, ExitCode},
    },
};

mod presets;

fn main() -> std::process::ExitCode {
    let mut args = std::env::args_os().skip(1);
    let first_arg = args.next();
    let mut cmd = Command::new(env!("PATH_TO_CARGO_EXEC"));
    if let Some(first) = first_arg {
        eprintln!("first arg: {first:?}");
        let cmd_meta = command_meta(first.to_str().unwrap_or(""));
        if cmd_meta.needs_config() {
            match find_presets_file() {
                Some(path) => 'found_presets: {
                    cmd.arg(first);
                    let presets = Presets::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
                    let Some(preset) = determine_preset(&presets) else {
                        break 'found_presets;
                    };
                    inject_args(&cmd_meta, preset, &mut cmd);
                }
                None => {
                    // No configuration file found
                    cmd.arg(first);
                }
            }
        } else {
            cmd.arg(first);
        }
    }
    cmd.args(args);
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
    let err = cmd.exec();
    eprintln!("Error execing cargo: {err}");
    ExitCode::FAILURE
}

fn inject_args(cmd_meta: &CommandMeta, preset: &Preset, cmd: &mut Command) {
    if cmd_meta.has_features {
        if preset.no_default_features {
            cmd.arg("--no-default-features");
        }
        if !preset.features.is_empty() {
            cmd.arg("--features");
            let mut feature_str = String::new();
            for feature in &preset.features {
                eprintln!("Injecting feature {feature}");
                write!(&mut feature_str, "{feature},").unwrap();
            }
            cmd.arg(feature_str);
        }
    }
    if cmd_meta.has_target && !preset.target.is_empty() {
        cmd.arg(format!("--target={}", preset.target));
    }
}

fn determine_preset(presets: &Presets) -> Option<&Preset> {
    let preset_arg;
    let default = if presets.default.is_empty() {
        preset_arg = std::env::args().take_while(|arg| arg == "--preset").next();
        match &preset_arg {
            Some(arg) => arg,
            None => return None,
        }
    } else {
        &presets.default
    };
    match presets.presets.get(default) {
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
