//! Named M1 conditions, not native realm/profile policy.
//!
//! Retire each default when its named backing system is authoritative.

use crate::client_profile::{ACTIVE, ClientProfile};

const RANGED_SLOT_CAPABILITY: &str = r#"
-- Missing system: ranged-slot display capability state.
-- False preserves the former factory's false-y branch, not native slot policy.
if rawget(C_PaperDollInfo, "IsRangedSlotShown") == nil then
    function C_PaperDollInfo.IsRangedSlotShown()
        return false
    end
end
"#;

const HARDCORE_REALM_CAPABILITY: &str = r#"
-- Missing system: realm game-rules state and its Hardcore activation policy.
-- Do not equate an arbitrary IsGameRuleActive key with native realm policy.
if rawget(C_GameRules, "IsHardcoreActive") == nil then
    function C_GameRules.IsHardcoreActive()
        return false
    end
end
"#;

const DAY_NIGHT_CLOCK_CAPABILITY: &str = r#"
-- Missing system: authoritative day/night clock and DIEL_CYCLE_CHANGED feed.
-- False is a fixed presentation default, not a calendar-time calculation.
if rawget(C_DateAndTime, "IsDayTime") == nil then
    function C_DateAndTime.IsDayTime()
        return false
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(RANGED_SLOT_CAPABILITY)?;
    if matches!(
        ACTIVE,
        ClientProfile::Mists | ClientProfile::Era | ClientProfile::Anniversary
    ) {
        lua.exec(HARDCORE_REALM_CAPABILITY)?;
    }
    if ACTIVE == ClientProfile::WowForever {
        lua.exec(DAY_NIGHT_CLOCK_CAPABILITY)?;
    }
    Ok(())
}
