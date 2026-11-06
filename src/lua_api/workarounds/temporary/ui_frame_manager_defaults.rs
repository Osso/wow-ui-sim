//! Temporary UIFrameManager defaults.
//!
//! The Blizzard UIFrameManager addon owns the real XML/mixin definitions. These
//! bootstrap defaults keep early managed-frame callers working before the addon
//! has loaded, and should be removed once the load order no longer needs them.

const UI_FRAME_MANAGER_DEFAULTS_LUA: &str = r#"
if type(UIFrameManager) ~= "table" then
    UIFrameManager = {}
end
if type(UIFrameManager_ManagedFrameMixin) ~= "table" then
    UIFrameManager_ManagedFrameMixin = {}
end
local __wow_ui_frame_manager_registered_frames = {}
local __wow_ui_frame_manager_registered_frame_type_to_frames = {}
local function __wow_ui_frame_manager_ensure_state()
    if type(UIFrameManager) == "table" and UIFrameManager.registeredFrameTypeToFrames ~= __wow_ui_frame_manager_registered_frame_type_to_frames then
        UIFrameManager.registeredFrameTypeToFrames = __wow_ui_frame_manager_registered_frame_type_to_frames
    end
end
if rawget(UIFrameManager, "OnLoad") == nil then
    function UIFrameManager:OnLoad()
        __wow_ui_frame_manager_ensure_state()
        if type(self.RegisterEvent) == "function" then
            self:RegisterEvent("FRAME_MANAGER_UPDATE_ALL")
            self:RegisterEvent("FRAME_MANAGER_UPDATE_FRAME")
        end
    end
end
if rawget(UIFrameManager, "OnEvent") == nil then
    function UIFrameManager:OnEvent(event, ...)
        __wow_ui_frame_manager_ensure_state()
        if event == "FRAME_MANAGER_UPDATE_ALL" then
            for frameType, frames in pairs(__wow_ui_frame_manager_registered_frame_type_to_frames) do
                for frame in pairs(frames) do
                    frame:UpdateFrameState(C_FrameManager.GetFrameVisibilityState(frameType))
                end
            end
            return
        end
        local frameType, show = ...
        local frames = __wow_ui_frame_manager_registered_frame_type_to_frames[frameType]
        if not frames then
            return
        end
        for frame in pairs(frames) do
            frame:UpdateFrameState(show)
        end
    end
end
if rawget(UIFrameManager, "RegisterFrameForFrameType") == nil then
    function UIFrameManager:RegisterFrameForFrameType(frame, frameType)
        __wow_ui_frame_manager_ensure_state()
        if __wow_ui_frame_manager_registered_frames[frame] then
            return
        end
        local frames = __wow_ui_frame_manager_registered_frame_type_to_frames[frameType]
        if frames == nil then
            frames = {}
            __wow_ui_frame_manager_registered_frame_type_to_frames[frameType] = frames
        end
        frames[frame] = true
        __wow_ui_frame_manager_registered_frames[frame] = true
        frame:UpdateFrameState(C_FrameManager.GetFrameVisibilityState(frameType))
    end
end
if rawget(UIFrameManager_ManagedFrameMixin, "OnLoad") == nil then
    function UIFrameManager_ManagedFrameMixin:OnLoad()
        UIFrameManager:RegisterFrameForFrameType(self, self.frameType)
    end
end
if rawget(UIFrameManager_ManagedFrameMixin, "UpdateFrameState") == nil then
    function UIFrameManager_ManagedFrameMixin:UpdateFrameState(show)
        self:SetShown(show)
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(UI_FRAME_MANAGER_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    retail_namespace_fixture! {
    #[test]
    fn registers_and_updates_managed_frame_visibility() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let (shown_before, shown_after): (bool, bool) = env
            .eval(
                r#"
                C_FrameManager.GetFrameVisibilityState = function()
                    return false
                end

                local frame = CreateFrame("Frame", "TemporaryUIFrameManagerTest", UIParent)
                frame.frameType = "tutorial"
                Mixin(frame, UIFrameManager_ManagedFrameMixin)
                frame:OnLoad()
                local shownBefore = frame:IsShown()

                UIFrameManager:OnEvent("FRAME_MANAGER_UPDATE_FRAME", "tutorial", true)
                return shownBefore, frame:IsShown()
                "#,
            )
            .expect("UIFrameManager defaults should update registered frames");

        assert_eq!((shown_before, shown_after), (false, true));
    }
    }

    #[test]
    fn preserves_existing_manager_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            UIFrameManager.RegisterFrameForFrameType = function() return "existing" end
            "#,
        )
        .expect("fixture should override an existing manager member");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("UIFrameManager defaults should apply");
        }

        let result: String = env
            .eval("return UIFrameManager.RegisterFrameForFrameType()")
            .expect("existing manager member should be callable");

        assert_eq!(result, "existing");
    }
}
