use wow_ui_sim::logging;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::saved_variables::SavedVariablesManager;

pub fn configure_saved_vars(
    no_saved_vars_arg: bool,
    session_wtf_path: &std::path::Path,
    explicit_session: bool,
) -> wow_ui_sim::Result<Option<SavedVariablesManager>> {
    if should_skip_saved_vars(no_saved_vars_arg) {
        logging::println_elapsed("SavedVariables loading disabled");
        return Ok(None);
    }

    let mut saved_vars = if explicit_session {
        SavedVariablesManager::with_session_wtf_path(session_wtf_path)
    } else {
        SavedVariablesManager::with_storage_dir(
            wow_ui_sim::saved_variables::saved_variables_storage_dir(session_wtf_path, false),
        )
    };
    if let Some(wtf) = wow_ui_sim::paths::try_default_wtf_config()? {
        logging::println_elapsed(&format!(
            "WTF import source (read-only): {} @ {}/{}",
            wtf.account, wtf.realm, wtf.character
        ));
        saved_vars.set_wtf_config(wtf);
    } else {
        logging::println_elapsed("WTF config: no WoW WTF directory found");
    }
    Ok(Some(saved_vars))
}

pub fn configure_edit_mode_cache_vars(
    session_wtf_path: &std::path::Path,
    explicit: bool,
) -> wow_ui_sim::Result<Option<SavedVariablesManager>> {
    let mut saved_vars = SavedVariablesManager::with_storage_dir(
        wow_ui_sim::saved_variables::saved_variables_storage_dir(session_wtf_path, explicit),
    );
    if let Some(wtf) = wow_ui_sim::paths::try_default_wtf_config()? {
        logging::println_elapsed(&format!(
            "EditMode cache source (read-only): {} @ {}/{}",
            wtf.account, wtf.realm, wtf.character
        ));
        saved_vars.set_wtf_config(wtf);
        Ok(Some(saved_vars))
    } else {
        logging::println_elapsed("EditMode cache: no WoW WTF directory found");
        Ok(None)
    }
}

/// Seed explicit file-selection identity before any addon chooses its profile.
pub fn configure_character_identity(
    env: &WowLuaEnv,
    source: Option<&SavedVariablesManager>,
) -> wow_ui_sim::Result<()> {
    if std::env::var_os("WOW_SIM_WTF_CHARACTER").is_none_or(|value| value.is_empty()) {
        return Ok(());
    }
    let config = source
        .and_then(SavedVariablesManager::wtf_config)
        .ok_or_else(|| {
            wow_ui_sim::Error::Other(
                "explicit character selection requires a WTF import source".into(),
            )
        })?;
    let (first_name, surname) = selected_name(&config.character);
    let mut state = env.state().borrow_mut();
    state.player.name = first_name.to_string();
    state.player.surname = surname.map(str::to_string);
    state.player.realm_name = Some(config.realm.clone());
    Ok(())
}

fn selected_name(character: &str) -> (&str, Option<&str>) {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::WowForever
        && let Some((first, surname)) = character.split_once('-')
        && !first.is_empty()
        && !surname.is_empty()
    {
        return (first, Some(surname));
    }
    (character, None)
}

pub fn load_keybindings_from_wtf(env: &WowLuaEnv, saved_vars: Option<&SavedVariablesManager>) {
    let Some(config) = saved_vars.and_then(SavedVariablesManager::wtf_config) else {
        return;
    };
    match wow_ui_sim::keybinding_cache::load_wtf_account_bindings(config) {
        Ok(entries) => {
            let mut state = env.state().borrow_mut();
            wow_ui_sim::keybinding_cache::apply_bindings_cache(&mut state.keybindings, &entries);
            logging::println_elapsed(&format!("Loaded {} keybinding(s) from WTF", entries.len()));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            logging::println_elapsed(&format!("Failed to load keybindings from WTF: {error}"))
        }
    }
}

#[cfg(feature = "gui")]
pub(super) fn configure_imported_ui_scale(env: &WowLuaEnv) {
    if let Some(path) = explicit_import_wtf_path() {
        configure_ui_scale_from_wtf(env, &path);
    }
}

#[cfg(feature = "gui")]
pub(super) fn apply_session_ui_scale(env: &WowLuaEnv) {
    if explicit_import_wtf_path().is_some() {
        apply_current_ui_scale(env);
    }
}

#[cfg(feature = "gui")]
fn explicit_import_wtf_path() -> Option<std::path::PathBuf> {
    std::env::var_os("WOW_SIM_WTF_PATH")
        .filter(|path| !path.is_empty())
        .map(std::path::PathBuf::from)
}

#[cfg(feature = "gui")]
fn configure_ui_scale_from_wtf(env: &WowLuaEnv, path: &std::path::Path) {
    if let Ok(bytes) = std::fs::read(path.join("Config.wtf")) {
        let config = String::from_utf8_lossy(&bytes);
        let state = env.state().borrow();
        for name in ["useUiScale", "uiScale"] {
            if let Some(value) = state.cvars.get_default(name) {
                state.cvars.set_imported(name, &value);
            }
        }
        for line in config.trim_start_matches('\u{feff}').lines() {
            let mut fields = line.split_ascii_whitespace();
            let (Some(command), Some(name), Some(value), None) =
                (fields.next(), fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            if !command.eq_ignore_ascii_case("SET") {
                continue;
            }
            let Some(value) = value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
            else {
                continue;
            };
            if (name.eq_ignore_ascii_case("useUiScale") && matches!(value, "0" | "1"))
                || (name.eq_ignore_ascii_case("uiScale") && parse_ui_scale(value).is_some())
            {
                state.cvars.set_imported(name, value);
            }
        }
    }
    apply_current_ui_scale(env);
}

#[cfg(feature = "gui")]
fn apply_current_ui_scale(env: &WowLuaEnv) {
    let scale = {
        let state = env.state().borrow();
        if !state.cvars.get_bool("useUiScale") {
            return;
        }
        state
            .cvars
            .get("uiScale")
            .as_deref()
            .and_then(parse_ui_scale)
    };
    if let Some(scale) = scale {
        wow_ui_sim::startup::apply_ui_scale(env, scale);
    }
}

#[cfg(feature = "gui")]
fn parse_ui_scale(value: &str) -> Option<f32> {
    value
        .parse::<f32>()
        .ok()
        .filter(|scale| scale.is_finite() && *scale > 0.0)
}

fn should_skip_saved_vars(no_saved_vars_arg: bool) -> bool {
    no_saved_vars_arg
        || std::env::var("WOW_SIM_NO_SAVED_VARS")
            .map(|value| value == "1")
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::selected_name;

    #[cfg(feature = "gui")]
    #[test]
    fn window_size_imports_manual_ui_scale_without_persisting_it() {
        let copy = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let config = copy.path().join("Config.wtf");
        let contents =
            "\u{feff}SET UseUiScale \"1\"\r\nset UIScale \"0.80\"\r\nSET unrelated \"example\"\r\n";
        std::fs::write(&config, contents).unwrap();
        let env = super::WowLuaEnv::with_session_wtf_path(runtime.path().to_path_buf()).unwrap();

        super::configure_ui_scale_from_wtf(&env, copy.path());
        let (scale, cvar, enabled, unrelated): (f32, String, String, bool) = env
            .eval(
                "return UIParent:GetEffectiveScale(), C_CVar.GetCVar('uiScale'), \
                 C_CVar.GetCVar('useUiScale'), C_CVar.GetCVar('unrelated') == nil",
            )
            .unwrap();
        assert_eq!((scale, cvar.as_str(), enabled.as_str()), (0.8, "0.80", "1"));
        assert!(unrelated);
        assert!(!runtime.path().join("cvars.json").exists());

        env.exec("assert(C_CVar.SetCVar('uiScale', '0.9'))")
            .unwrap();
        let stored: serde_json::Value =
            serde_json::from_slice(&std::fs::read(runtime.path().join("cvars.json")).unwrap())
                .unwrap();
        assert_eq!(stored, serde_json::json!({"uiscale": "0.9"}));
        assert_eq!(
            env.eval::<f32>("return UIParent:GetEffectiveScale()")
                .unwrap(),
            0.9
        );
        assert_eq!(std::fs::read_to_string(config).unwrap(), contents);
    }

    #[cfg(feature = "gui")]
    #[test]
    fn window_size_uses_automatic_ui_scale_without_a_manual_setting() {
        let copy = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        std::fs::write(
            copy.path().join("Config.wtf"),
            "SET GxMaximize \"0\"\nSET GxWindowedResolution \"3774x2113\"\n",
        )
        .unwrap();
        let persisted_path = runtime.path().join("cvars.json");
        let persisted = r#"{"useuiscale":"1","uiscale":"0.8"}"#;
        std::fs::write(&persisted_path, persisted).unwrap();
        let env = super::WowLuaEnv::with_session_wtf_path(runtime.path().to_path_buf()).unwrap();
        super::configure_ui_scale_from_wtf(&env, copy.path());
        assert_eq!(
            env.eval::<(f32, String, bool)>(
                "return UIParent:GetEffectiveScale(), C_CVar.GetCVar('useUiScale'), \
                 C_CVar.GetCVar('uiScale') == C_CVar.GetCVarDefault('uiScale')",
            )
            .unwrap(),
            (1.0, "0".into(), true)
        );
        assert_eq!(std::fs::read_to_string(persisted_path).unwrap(), persisted);
    }

    #[test]
    fn selected_name_preserves_empty_parts_and_profile_specific_hyphens() {
        for name in ["", "Ada", "-Family", "Ada-"] {
            assert_eq!(selected_name(name), (name, None));
        }
        if wow_ui_sim::client_profile::ACTIVE
            == wow_ui_sim::client_profile::ClientProfile::WowForever
        {
            assert_eq!(selected_name("Ada-Family"), ("Ada", Some("Family")));
            assert_eq!(
                selected_name("Ada-Family-Second"),
                ("Ada", Some("Family-Second"))
            );
        } else {
            assert_eq!(selected_name("Ada-Family"), ("Ada-Family", None));
        }
    }
}
