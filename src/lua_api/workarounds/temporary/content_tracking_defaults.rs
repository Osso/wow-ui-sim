//! Temporary ContentTrackingUtil defaults.
//!
//! Blizzard_ContentTracking replaces this table when it loads. These defaults
//! keep earlier FrameXML callers from failing when content tracking is absent.

const CONTENT_TRACKING_DEFAULTS_LUA: &str = r#"
if type(ContentTrackingUtil) ~= "table" then
  ContentTrackingUtil = {}
end

if rawget(ContentTrackingUtil, "IsTrackingModifierDown") == nil then
  function ContentTrackingUtil.IsTrackingModifierDown() return false end
end
if rawget(ContentTrackingUtil, "IsContentTrackingEnabled") == nil then
  function ContentTrackingUtil.IsContentTrackingEnabled() return false end
end
if rawget(ContentTrackingUtil, "RegisterTrackableElement") == nil then
  function ContentTrackingUtil.RegisterTrackableElement() end
end
if rawget(ContentTrackingUtil, "UnregisterTrackableElement") == nil then
  function ContentTrackingUtil.UnregisterTrackableElement() end
end
if rawget(ContentTrackingUtil, "ProcessChatLink") == nil then
  function ContentTrackingUtil.ProcessChatLink() return false end
end
if rawget(ContentTrackingUtil, "GetTrackingMapInfoByEncounterID") == nil then
  function ContentTrackingUtil.GetTrackingMapInfoByEncounterID() return nil end
end
if rawget(ContentTrackingUtil, "IsContentTrackedInEncounter") == nil then
  function ContentTrackingUtil.IsContentTrackedInEncounter() return false end
end
if rawget(ContentTrackingUtil, "OpenMapToTrackable") == nil then
  function ContentTrackingUtil.OpenMapToTrackable() return false end
end
if rawget(ContentTrackingUtil, "DisplayTrackingError") == nil then
  function ContentTrackingUtil.DisplayTrackingError() end
end
if rawget(ContentTrackingUtil, "MakeCombinedID") == nil then
  function ContentTrackingUtil.MakeCombinedID(trackableType, trackableID)
    return (trackableID or 0) * 1000 + (trackableType or 0)
  end
end
if rawget(ContentTrackingUtil, "SplitCombinedID") == nil then
  function ContentTrackingUtil.SplitCombinedID(combinedID)
    if type(combinedID) ~= "number" then
      return nil, nil
    end
    return combinedID % 1000, math.floor(combinedID / 1000)
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CONTENT_TRACKING_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_content_tracking_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(ContentTrackingUtil) ~= "table" then return "missing_table" end
                if ContentTrackingUtil.IsTrackingModifierDown() ~= false then return "modifier" end
                if ContentTrackingUtil.IsContentTrackingEnabled() ~= false then return "enabled" end
                if ContentTrackingUtil.ProcessChatLink(1, 2) ~= false then return "chat_link" end
                if ContentTrackingUtil.GetTrackingMapInfoByEncounterID(42) ~= nil then return "map_info" end
                if ContentTrackingUtil.IsContentTrackedInEncounter(42) ~= false then return "encounter" end
                if ContentTrackingUtil.OpenMapToTrackable(1, 2) ~= false then return "open_map" end
                local combined = ContentTrackingUtil.MakeCombinedID(17, 42)
                local trackableType, trackableID = ContentTrackingUtil.SplitCombinedID(combined)
                if combined ~= 42017 then return "combined" end
                if trackableType ~= 17 or trackableID ~= 42 then return "split" end
                return "ok"
                "#,
            )
            .expect("content tracking defaults probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_content_tracking_util_functions() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            ContentTrackingUtil = {
              IsContentTrackingEnabled = function() return true end,
              MakeCombinedID = function() return 99 end,
            }
            "#,
        )
        .expect("fixture should install existing content tracking functions");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("content tracking defaults should apply");
        }

        let result: (bool, i32, bool) = env
            .eval(
                r#"
                return ContentTrackingUtil.IsContentTrackingEnabled(),
                       ContentTrackingUtil.MakeCombinedID(1, 2),
                       type(ContentTrackingUtil.SplitCombinedID) == "function"
                "#,
            )
            .expect("content tracking preservation probe should run");

        assert_eq!(result, (true, 99, true));
    }
}
