//! Cached addon preparation for the offline login policy, not a native API.

use crate::lua_api::WowLuaEnv;
use crate::screen::ScreenKind;

pub(crate) fn prepare_character_addons(env: &WowLuaEnv) -> crate::Result<()> {
    let glue_loaded = env
        .state()
        .borrow()
        .addons
        .iter()
        .any(|addon| addon.folder_name == "Blizzard_GlueXML" && addon.loaded);
    if !glue_loaded {
        return Ok(());
    }

    // Initial Login discovery intentionally omits these roots. Supply the
    // original character UI before publishing authentication, without replaying
    // startup or selecting a screen on behalf of GlueParent.
    let roots = super::excluded_addons_for_screen(ScreenKind::Login);
    let addons = super::discover_blizzard_addon_closure_for_screen(
        &crate::client_profile::blizzard_ui_addons_dir(),
        ScreenKind::CharacterCreate,
        roots,
    );
    if !addons
        .iter()
        .any(|(name, _)| name == "Blizzard_CharacterCreate")
    {
        return Err(crate::Error::Other(
            "offline login requires cached Blizzard_CharacterCreate".into(),
        ));
    }
    for (name, path) in addons {
        let result = super::load_addon(&env.loader_env(), &path)
            .map_err(|error| crate::Error::Other(format!("login addon {name}: {error}")))?;
        if !result.warnings.is_empty() || !result.missing_requirements.is_empty() {
            return Err(crate::Error::Other(format!(
                "login addon {name}: warnings={:?}, missing requirements={:?}",
                result.warnings, result.missing_requirements
            )));
        }
    }
    // Reuse only the character-create post-load step that initial Login could
    // not apply before these frames existed. Do not replay all startup patches.
    crate::lua_api::workarounds::patch_character_create_defaults(env);
    Ok(())
}
