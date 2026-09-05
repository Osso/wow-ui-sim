//! Temporary client/build/realm defaults for startup compatibility.
//!
//! These globals describe simulated client identity and account expansion state.
//! They are compatibility defaults until the simulator has a real client/session
//! metadata model.

#[cfg(all(not(feature = "client-ptr"), feature = "retail-12-1-0"))]
const CLIENT_VERSION: &str = "12.1.0";
#[cfg(all(not(feature = "client-ptr"), not(feature = "retail-12-1-0")))]
const CLIENT_VERSION: &str = "12.0.7";

#[cfg(not(feature = "client-ptr"))]
const RETAIL_BUILD: &str = "68256";

fn client_identity() -> crate::Result<(String, String)> {
    #[cfg(feature = "client-ptr")]
    {
        let identity = crate::blizzard_ui_sync::ptr_client_identity()?;
        return Ok((identity.version, identity.build));
    }

    #[cfg(not(feature = "client-ptr"))]
    Ok((CLIENT_VERSION.to_owned(), RETAIL_BUILD.to_owned()))
}

const CLIENT_INFO_DEFAULTS_LUA: &str = r#"
if GetBuildInfo == nil then
  function GetBuildInfo()
    return "__WOW_CLIENT_VERSION__", "__WOW_CLIENT_BUILD__", "Jun 17 2026", __WOW_CLIENT_INTERFACE__, "", " "
  end
end

if GetRealmName == nil then
  function GetRealmName()
    return "SimulatedRealm"
  end
end

if GetNormalizedRealmName == nil then
  function GetNormalizedRealmName()
    return "SimulatedRealm"
  end
end

if GetRealmID == nil then
  function GetRealmID()
    return 1
  end
end

if GetExpansionLevel == nil then
  function GetExpansionLevel()
    return 10
  end
end

if GetUpgradeExpansionLevel == nil then
  function GetUpgradeExpansionLevel()
    return 80
  end
end

if IsExpansionTrial == nil then
  function IsExpansionTrial()
    return false
  end
end

if GetExpansionTrialInfo == nil then
  function GetExpansionTrialInfo()
    return false, 0
  end
end

if IsTrialAccount == nil then
  function IsTrialAccount()
    return false
  end
end

if IsRestrictedAccount == nil then
  function IsRestrictedAccount()
    return false
  end
end

if IsVeteranTrialAccount == nil then
  function IsVeteranTrialAccount()
    return false
  end
end

if IsAccountSecured == nil then
  function IsAccountSecured()
    return true
  end
end

if IsMacClient == nil then
  function IsMacClient()
    return false
  end
end

if IsWindowsClient == nil then
  function IsWindowsClient()
    return false
  end
end

if GetGraphicsAPIs == nil then
  function GetGraphicsAPIs()
    return "D3D12", "D3D11"
  end
end

if RequestTimePlayed == nil then
  function RequestTimePlayed()
  end
end

if GetClientDisplayExpansionLevel == nil then
  function GetClientDisplayExpansionLevel()
    return 10
  end
end

if GetAccountExpansionLevel == nil then
  function GetAccountExpansionLevel()
    return GetClientDisplayExpansionLevel()
  end
end

if GetMaxPlayerLevel == nil then
  function GetMaxPlayerLevel()
    return 80
  end
end

if GetMaxLevelForExpansionLevel == nil then
  function GetMaxLevelForExpansionLevel(_expansion_level)
    return GetMaxPlayerLevel()
  end
end

if GetMaxLevelForPlayerExpansion == nil then
  function GetMaxLevelForPlayerExpansion()
    return GetMaxLevelForExpansionLevel(GetAccountExpansionLevel())
  end
end

if PartialPlayTime == nil then
  function PartialPlayTime()
    return false
  end
end

if NoPlayTime == nil then
  function NoPlayTime()
    return false
  end
end

if GetBillingTimeRested == nil then
  function GetBillingTimeRested()
    return 0
  end
end

if GetReleaseTimeRemaining == nil then
  function GetReleaseTimeRemaining()
    return 0
  end
end

if GetExpansionDisplayInfo == nil then
  function GetExpansionDisplayInfo(_expansionLevel, _desiredReleaseType)
    return {
      logo = 0,
      banner = "",
      features = {},
      highResBackgroundID = 0,
      lowResBackgroundID = 0,
      textureKit = "",
      glueAmbianceSoundKit = nil,
      glueMusicSoundKit = nil,
      glueCreditsSoundKit = nil,
    }
  end
end

if GetFileStreamingStatus == nil then
  function GetFileStreamingStatus()
    return 0
  end
end

if GetBackgroundLoadingStatus == nil then
  function GetBackgroundLoadingStatus()
    return 0
  end
end

if GetWebTicket == nil then
  function GetWebTicket()
    return nil
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    let (version, build) = client_identity()?;
    let code = CLIENT_INFO_DEFAULTS_LUA
        .replace("__WOW_CLIENT_VERSION__", &version)
        .replace("__WOW_CLIENT_BUILD__", &build)
        .replace(
            "__WOW_CLIENT_INTERFACE__",
            &crate::client_profile::ACTIVE_INTERFACE_VERSION.to_string(),
        );
    lua.exec(&code)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_client_info_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let (expected_version, expected_build) = super::client_identity().unwrap();
        let expected_interface = crate::client_profile::ACTIVE_INTERFACE_VERSION;
        let script = format!(
            r#"
                local version, build, date, interface = GetBuildInfo()
                if version ~= "{}" or build ~= "{}" or date ~= "Jun 17 2026" or interface ~= {} then
                  return "build"
                end
                if GetRealmName() ~= "SimulatedRealm" or GetNormalizedRealmName() ~= "SimulatedRealm" then return "realm" end
                if GetRealmID() ~= 1 then return "realm_id" end
                if GetExpansionLevel() ~= 10 then return "expansion" end
                if GetUpgradeExpansionLevel() ~= 80 then return "upgrade_expansion" end
                if IsExpansionTrial() ~= false then return "expansion_trial" end
                local isExpansionTrial, expansionTrialRemaining = GetExpansionTrialInfo()
                if isExpansionTrial ~= false or expansionTrialRemaining ~= 0 then return "expansion_trial_info" end
                if IsTrialAccount() ~= false or IsRestrictedAccount() ~= false then return "account_restrictions" end
                if IsVeteranTrialAccount() ~= false or IsAccountSecured() ~= true then return "account_status" end
                if IsMacClient() ~= false or IsWindowsClient() ~= false then return "platform" end
                local primaryGraphicsApi, fallbackGraphicsApi = GetGraphicsAPIs()
                if primaryGraphicsApi ~= "D3D12" or fallbackGraphicsApi ~= "D3D11" then return "graphics_api" end
                if not pcall(RequestTimePlayed) then return "time_played" end
                if GetClientDisplayExpansionLevel() ~= 10 then return "client_expansion" end
                if GetAccountExpansionLevel() ~= 10 then return "account_expansion" end
                if GetMaxPlayerLevel() ~= 80 then return "max_player_level" end
                if GetMaxLevelForExpansionLevel(0) ~= GetMaxPlayerLevel() then return "max_level" end
                if GetMaxLevelForPlayerExpansion() ~= GetMaxPlayerLevel() then return "player_max_level" end
                if PartialPlayTime() ~= false then return "partial_play_time" end
                if NoPlayTime() ~= false then return "no_play_time" end
                if GetBillingTimeRested() ~= 0 then return "billing_rested" end
                if GetReleaseTimeRemaining() ~= 0 then return "release_time" end
                local info = GetExpansionDisplayInfo(10)
                if type(info) ~= "table" or info.textureKit ~= "" or type(info.features) ~= "table" then return "display_info" end
                if GetFileStreamingStatus() ~= 0 or GetBackgroundLoadingStatus() ~= 0 then return "streaming_status" end
                if GetWebTicket() ~= nil then return "web_ticket" end
                return "ok"
                "#,
            expected_version, expected_build, expected_interface
        );
        let result: String = env
            .eval(&script)
            .expect("client info defaults probe should run");

        assert_eq!(result, "ok");
    }
}
