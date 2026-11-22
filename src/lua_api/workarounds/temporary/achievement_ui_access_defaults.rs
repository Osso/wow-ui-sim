//! Temporary AchievementUI access gate defaults.
//!
//! The global stubs default `CanShowAchievementUI` to false. Blizzard
//! AchievementUI gating expects these probes to allow the panel in the
//! simulator until a real account/achievement visibility model owns them.

const ACHIEVEMENT_UI_ACCESS_DEFAULTS_LUA: &str = r#"
function HasCompletedAnyAchievement()
  return true
end

function CanShowAchievementUI()
  return true
end
"#;

const MODELED_ACHIEVEMENT_UI_ACCESS_LUA: &str = r#"
function HasCompletedAnyAchievement()
  return true
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    use crate::client_profile::{ACTIVE, ClientProfile};

    let bootstrap = if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        MODELED_ACHIEVEMENT_UI_ACCESS_LUA
    } else {
        ACHIEVEMENT_UI_ACCESS_DEFAULTS_LUA
    };
    lua.exec(bootstrap)?;
    Ok(())
}
