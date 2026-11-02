use wow_ui_sim::logging;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::saved_variables::SavedVariablesManager;

pub fn configure_saved_vars(
    no_saved_vars_arg: bool,
    session_wtf_path: &std::path::Path,
) -> Option<SavedVariablesManager> {
    if should_skip_saved_vars(no_saved_vars_arg) {
        logging::println_elapsed("SavedVariables loading disabled");
        return None;
    }

    let mut saved_vars =
        SavedVariablesManager::with_storage_dir(session_wtf_path.join("SavedVariables"));
    if let Some(wtf) = wow_ui_sim::paths::default_wtf_config() {
        logging::println_elapsed(&format!(
            "WTF import source (read-only): {} @ {}/{}",
            wtf.account, wtf.realm, wtf.character
        ));
        saved_vars.set_wtf_config(wtf);
    } else {
        logging::println_elapsed("WTF config: no WoW WTF directory found");
    }
    Some(saved_vars)
}

pub fn configure_edit_mode_cache_vars(
    session_wtf_path: &std::path::Path,
) -> Option<SavedVariablesManager> {
    let mut saved_vars =
        SavedVariablesManager::with_storage_dir(session_wtf_path.join("SavedVariables"));
    if let Some(wtf) = wow_ui_sim::paths::default_wtf_config() {
        logging::println_elapsed(&format!(
            "EditMode cache source (read-only): {} @ {}/{}",
            wtf.account, wtf.realm, wtf.character
        ));
        saved_vars.set_wtf_config(wtf);
        Some(saved_vars)
    } else {
        logging::println_elapsed("EditMode cache: no WoW WTF directory found");
        None
    }
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
