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
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(UI_FRAME_MANAGER_DEFAULTS_LUA)?;
    Ok(())
}
