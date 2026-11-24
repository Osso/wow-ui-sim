//! Temporary camera and tutorial defaults for profiles that still need shims.

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const CAMERA_TUTORIAL_DEFAULTS_LUA: &str = r#"
if GetCameraFOVDefaults == nil then
  function GetCameraFOVDefaults()
    return 0, 30, 110
  end
end

if GetTutorialsEnabled == nil then
  function GetTutorialsEnabled()
    return false
  end
end

if IsTutorialFlagged == nil then
  function IsTutorialFlagged()
    return false
  end
end

for _, __wow_camera_verb in ipairs({
  "MoveViewOutStart", "MoveViewOutStop",
  "MoveViewInStart", "MoveViewInStop",
  "MoveViewLeftStart", "MoveViewLeftStop",
  "MoveViewRightStart", "MoveViewRightStop",
  "MoveViewUpStart", "MoveViewUpStop",
  "MoveViewDownStart", "MoveViewDownStop",
}) do
  if _G[__wow_camera_verb] == nil then
    _G[__wow_camera_verb] = function()
    end
  end
end
"#;

/// Retail/Forever keep only the camera movement verbs, which have no modeled consumer.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const CAMERA_TUTORIAL_DEFAULTS_LUA: &str = r#"
for _, __wow_camera_verb in ipairs({
  "MoveViewOutStart", "MoveViewOutStop",
  "MoveViewInStart", "MoveViewInStop",
  "MoveViewLeftStart", "MoveViewLeftStop",
  "MoveViewRightStart", "MoveViewRightStop",
  "MoveViewUpStart", "MoveViewUpStop",
  "MoveViewDownStart", "MoveViewDownStop",
}) do
  if _G[__wow_camera_verb] == nil then
    _G[__wow_camera_verb] = function()
    end
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CAMERA_TUTORIAL_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_camera_tutorial_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let result: String = env
            .eval(
                r#"
                local currentFov, minFov, maxFov = GetCameraFOVDefaults()
                if currentFov ~= 0 or minFov ~= 30 or maxFov ~= 110 then return "fov" end
                if GetTutorialsEnabled() ~= false then return "tutorials" end
                if IsTutorialFlagged(1) ~= false then return "tutorial_flagged" end
                for _, name in ipairs({
                    "MoveViewOutStart", "MoveViewOutStop",
                    "MoveViewInStart", "MoveViewInStop",
                    "MoveViewLeftStart", "MoveViewLeftStop",
                    "MoveViewRightStart", "MoveViewRightStop",
                    "MoveViewUpStart", "MoveViewUpStop",
                    "MoveViewDownStart", "MoveViewDownStop",
                }) do
                    if type(_G[name]) ~= "function" then return name .. "_missing" end
                    if pcall(_G[name], 1.0, 0, true) ~= true then return name .. "_call" end
                end
                return "ok"
                "#,
            )
            .expect("camera/tutorial defaults probe should run");
        assert_eq!(result, "ok");
    }
}
