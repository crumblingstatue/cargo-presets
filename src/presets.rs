use {
    std::{collections::HashMap, path::PathBuf},
    toml_edit::{Item, Value},
};

pub fn find_presets_file() -> Option<PathBuf> {
    let mut dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("{e}");
            return None;
        }
    };
    loop {
        let preset_file = dir.join(".cargo/presets.toml");
        if preset_file.exists() {
            return Some(preset_file);
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => return None,
        }
    }
}

pub struct Presets {
    pub default: String,
    pub presets: HashMap<String, Preset>,
}

#[derive(Default)]
pub struct Preset {
    pub no_default_features: bool,
    pub features: Vec<String>,
    pub target: String,
}

impl Presets {
    pub fn parse(toml: &str) -> Result<Self, toml_edit::TomlError> {
        let mut default = String::new();
        let mut presets = HashMap::new();
        let doc = toml_edit::ImDocument::parse(toml)?;
        match doc.get("preset") {
            Some(Item::Table(tbl)) => {
                for (k, v) in tbl.into_iter() {
                    let preset = Preset::from_toml_item(v);
                    presets.insert(k.to_owned(), preset);
                }
            }
            other => {
                eprintln!("Invalid value for 'preset': {other:?}");
            }
        }
        match doc.get("default") {
            Some(Item::Value(Value::String(name))) => {
                name.value().clone_into(&mut default);
            }
            other => {
                eprintln!("Invalid value for 'default': {other:?}");
            }
        }
        Ok(Self { default, presets })
    }
}

impl Preset {
    fn from_toml_item(item: &Item) -> Self {
        let mut preset = Self::default();
        match item.get("features") {
            Some(Item::Value(Value::Array(arr))) => {
                for val in arr.iter() {
                    preset.features.push(val.as_str().unwrap().to_owned())
                }
            }
            other => {
                eprintln!("Invalid value for 'features': {other:?}");
            }
        }
        match item.get("default-features") {
            Some(Item::Value(Value::Boolean(bool))) => {
                preset.no_default_features = !bool.value();
            }
            other => {
                eprintln!("Invalid value for 'default-features': {other:?}");
            }
        }
        match item.get("target") {
            Some(Item::Value(Value::String(str))) => {
                preset.target.clone_from(str.value());
            }
            other => {
                eprintln!("Invalid value for 'target': {other:?}");
            }
        }
        preset
    }
}
