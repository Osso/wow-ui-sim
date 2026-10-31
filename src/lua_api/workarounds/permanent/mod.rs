//! Permanent compatibility workarounds for intentionally unsupported UI scope.
//!
//! These are not WoW API implementations. They are explicit simulator
//! compromises kept out of the generic bootstrap/C API surface.

mod achievement_display;
mod flash_client_icon;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    achievement_display::apply_bootstrap(lua)?;
    flash_client_icon::apply_bootstrap(lua)
}

pub(crate) fn reapply_achievement_display_after_blizzard_load(env: &crate::lua_api::WowLuaEnv) {
    achievement_display::reapply_after_blizzard_load(env);
}
