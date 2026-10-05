use crate::lua_api::AddonInfo;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

const ADDONS_TXT_ENV: &str = "WOW_SIM_ADDONS_TXT";

pub fn local_addons_txt_path() -> PathBuf {
    if let Some(path) = std::env::var_os(ADDONS_TXT_ENV) {
        return PathBuf::from(path);
    }

    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("wow-sim")
        .join("AddOns.txt")
}

pub fn read_addon_enable_overrides(path: &Path) -> io::Result<HashMap<String, bool>> {
    let text = std::fs::read_to_string(path)?;
    Ok(parse_addons_txt(&text))
}

pub fn write_addon_enable_overrides(path: &Path, addons: &[AddonInfo]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format_addons_txt(addons))
}

pub fn save_local_addon_enable_overrides(addons: &[AddonInfo]) -> io::Result<()> {
    write_addon_enable_overrides(&local_addons_txt_path(), addons)
}

fn parse_addons_txt(text: &str) -> HashMap<String, bool> {
    let mut states = HashMap::new();
    for line in text.lines() {
        if let Some((name, state)) = line.split_once(':') {
            states.insert(name.trim().to_string(), state.trim() == "enabled");
        }
    }
    states
}

fn format_addons_txt(addons: &[AddonInfo]) -> String {
    let mut entries: Vec<_> = addons
        .iter()
        .filter(|addon| should_persist_addon(addon))
        .map(|addon| {
            let state = if addon.enabled { "enabled" } else { "disabled" };
            (addon.folder_name.as_str(), state)
        })
        .collect();
    entries.sort_by_key(|(name, _)| name.to_ascii_lowercase());

    let mut text = String::new();
    for (name, state) in entries {
        text.push_str(name);
        text.push_str(": ");
        text.push_str(state);
        text.push('\n');
    }
    text
}

fn should_persist_addon(addon: &AddonInfo) -> bool {
    addon.folder_name != "__BuiltIn"
        && !crate::blizzard_ui_sync::is_builtin_addon_folder(&addon.folder_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addon(name: &str, enabled: bool) -> AddonInfo {
        AddonInfo {
            folder_name: name.to_string(),
            enabled,
            ..Default::default()
        }
    }

    #[test]
    fn read_addon_enable_overrides_parses_addons_txt() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("AddOns.txt");
        std::fs::write(
            &path,
            "EnabledAddon: enabled\nDisabledAddon: disabled\nMalformed\n",
        )
        .expect("write AddOns.txt");

        let overrides = read_addon_enable_overrides(&path).expect("read AddOns.txt");

        assert_eq!(overrides.get("EnabledAddon"), Some(&true));
        assert_eq!(overrides.get("DisabledAddon"), Some(&false));
        assert_eq!(overrides.get("Malformed"), None);
    }

    #[test]
    fn write_addon_enable_overrides_writes_local_addons_txt() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("Account").join("AddOns.txt");

        write_addon_enable_overrides(
            &path,
            &[
                addon("Blizzard_ActionBar", false),
                addon("AddonB", false),
                addon("__BuiltIn", false),
                addon("AddonA", true),
            ],
        )
        .expect("write AddOns.txt");

        let text = std::fs::read_to_string(path).expect("read written AddOns.txt");
        assert_eq!(text, "AddonA: enabled\nAddonB: disabled\n");
    }
}
