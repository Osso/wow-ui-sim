//! Named T6 service gaps; never an arbitrary C_* member factory.
//! Retire each default with the backing system named beside its definition.

use crate::client_profile::{ACTIVE, ClientProfile};

const MODERN_SERVICE_CAPABILITIES: &str = r#"
C_ContentTracking = C_ContentTracking or {}
-- Missing system: collectable-source tracking capability/entitlement policy.
if rawget(C_ContentTracking, "GetCollectableSourceTrackingEnabled") == nil then
    function C_ContentTracking.GetCollectableSourceTrackingEnabled()
        return false
    end
end

-- Missing system: authenticated voice-service login session.
-- A simulated connection status or active channel is not login evidence.
if rawget(C_VoiceChat, "IsLoggedIn") == nil then
    function C_VoiceChat.IsLoggedIn()
        return false
    end
end
"#;

const CORPSE_LOCATION: &str = r#"
C_DeathInfo = C_DeathInfo or {}
-- Missing system: corpse location keyed by UI map, independent of player location.
if rawget(C_DeathInfo, "GetCorpseMapPosition") == nil then
    function C_DeathInfo.GetCorpseMapPosition(_uiMapID)
        return nil
    end
end
"#;

const PREFERRED_LOCALES: &str = r#"
-- Missing system: preferred account locale ID/name catalog.
if rawget(C_AccountServices, "GetPreferredLocales") == nil then
    function C_AccountServices.GetPreferredLocales()
        return {}, {}
    end
end
"#;

const ACTIVE_VOICE_PROVIDER: &str = r#"
-- Missing system: authoritative active voice-provider selection.
-- Legacy is a presentation default from the selected profile enum, not login state.
if rawget(C_VoiceChat, "GetActiveVoiceProviderID") == nil then
    function C_VoiceChat.GetActiveVoiceProviderID()
        return Enum.VoiceProviderID.Legacy
    end
end
"#;

const FOREVER_DISCORD_CAPABILITY: &str = r#"
-- Missing system: Discord-specific voice capability/settings, not service enablement.
if rawget(C_Discord, "IsVoiceEnabled") == nil then
    function C_Discord.IsVoiceEnabled()
        return false
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    match ACTIVE {
        ClientProfile::Retail => lua.exec(MODERN_SERVICE_CAPABILITIES)?,
        ClientProfile::Ptr => {
            lua.exec(MODERN_SERVICE_CAPABILITIES)?;
            // User-approved simulator default, not evidence of native behavior.
            lua.exec(ACTIVE_VOICE_PROVIDER)?;
        }
        ClientProfile::Era | ClientProfile::Anniversary => lua.exec(CORPSE_LOCATION)?,
        ClientProfile::WowForever => {
            lua.exec(MODERN_SERVICE_CAPABILITIES)?;
            lua.exec(PREFERRED_LOCALES)?;
            lua.exec(FOREVER_DISCORD_CAPABILITY)?;
            lua.exec(ACTIVE_VOICE_PROVIDER)?;
        }
        ClientProfile::Mists | ClientProfile::Wrath => {}
    }
    Ok(())
}
