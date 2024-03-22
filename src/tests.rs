use crate::{inject_args, presets::Preset, strip_preset_arg, with_actionable_command};

#[test]
fn test_inject_args() {
    let mut args = vec!["build".into(), "--preset".into(), "foo".into()];
    let preset = strip_preset_arg(&mut args);
    assert_eq!(preset, Some("foo".into()));
    with_actionable_command(&mut args, |meta, args| {
        inject_args(
            meta,
            &Preset {
                no_default_features: true,
                features: vec!["foo".into(), "bar".into()],
                target: "wasm32-unknown-unknown".into(),
            },
            args,
        );
        assert_eq!(
            args,
            &[
                "build",
                "--no-default-features",
                "--features",
                "foo,bar,",
                "--target=wasm32-unknown-unknown"
            ]
        );
    });
}
