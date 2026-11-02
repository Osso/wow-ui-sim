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

fn should_skip_saved_vars(no_saved_vars_arg: bool) -> bool {
    no_saved_vars_arg
        || std::env::var("WOW_SIM_NO_SAVED_VARS")
            .map(|value| value == "1")
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::selected_name;

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
