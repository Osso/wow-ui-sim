//! Per-environment click-binding profile used by secure unit interactions.
//! The initial unmodified left/right assignments are simulator policy, not native-probed defaults.

const PROFILE_LUA: &str = r#"
do
    local interaction = Enum.ClickBindingType.Interaction
    local target = Enum.ClickBindingInteraction.Target
    local menu = Enum.ClickBindingInteraction.OpenContextMenu

    local function defaults()
        return {
            {type = interaction, actionID = target, button = "LeftButton", modifiers = 0},
            {type = interaction, actionID = menu, button = "RightButton", modifiers = 0},
        }
    end

    local profile = defaults()
    local function copy_info(info)
        return {
            type = info.type,
            actionID = info.actionID,
            button = info.button,
            modifiers = info.modifiers,
        }
    end

    local function find_binding(button, modifiers)
        for _, info in ipairs(profile) do
            if info.button == button and info.modifiers == modifiers then
                return info
            end
        end
    end

    function C_ClickBindings.GetBindingType(button, modifiers)
        local info = find_binding(button, modifiers)
        return info and info.type or Enum.ClickBindingType.None
    end

    function C_ClickBindings.GetEffectiveInteractionButton(button, modifiers)
        local info = find_binding(button, modifiers)
        if info and info.type == interaction then
            if info.actionID == target then
                return "LeftButton"
            elseif info.actionID == menu then
                return "RightButton"
            end
        end
        return button
    end

    function C_ClickBindings.GetProfileInfo()
        local result = {}
        for index, info in ipairs(profile) do
            result[index] = copy_info(info)
        end
        return result
    end

    function C_ClickBindings.SetProfileByInfo(infos)
        assert(type(infos) == "table", "ClickBindingInfo vector must be a table")
        local next_profile = {}
        for index, info in ipairs(infos) do
            assert(type(info) == "table" and type(info.type) == "number"
                and type(info.actionID) == "number" and type(info.button) == "string"
                and type(info.modifiers) == "number", "invalid ClickBindingInfo")
            next_profile[index] = copy_info(info)
        end
        profile = next_profile
    end

    function C_ClickBindings.ResetCurrentProfile()
        profile = defaults()
    end
end
"#;

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PROFILE_LUA)?;
    Ok(())
}
