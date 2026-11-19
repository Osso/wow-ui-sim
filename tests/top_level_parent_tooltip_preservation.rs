#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn menu_util_hook_tooltip_uses_blizzard_provider() {
    let env = tooltip_full_env_helpers::setup_full_env();
    assert_original_top_level_parent_behavior(&env);
    assert_menu_tooltip_uses_provider(&env);
}

fn assert_original_top_level_parent_behavior(env: &WowLuaEnv) {
    env.exec(
        r#"
        local alternate = CreateFrame("Frame", "TopLevelParentProbe", UIParent)
        alternate:Show()
        local alternateEventFired = false
        local originalTriggerEvent = EventRegistry.TriggerEvent
        EventRegistry.TriggerEvent = function(_, eventName)
            if eventName == "UI.AlternateTopLevelParentChanged" then
                alternateEventFired = true
            end
        end
        SetAlternateTopLevelParent(alternate)
        EventRegistry.TriggerEvent = originalTriggerEvent
        assert(alternateEventFired, "FrameUtil must publish alternate-parent changes")
        assert(GetAppropriateTopLevelParent() == alternate,
            "FrameUtil must return the alternate top-level parent")
        assert(GetAppropriateTopLevelParent(alternate) == UIParent,
            "FrameUtil must honor the excluded alternate parent")
        local child = CreateFrame("Frame", "TopLevelParentChildProbe", UIParent)
        SetAppropriateTopLevelParent(child)
        assert(child:GetParent() == alternate,
            "FrameUtil must parent the frame to the selected top-level parent")
        ClearAlternateTopLevelParent()
        assert(GetAppropriateTopLevelParent() == UIParent,
            "FrameUtil must restore UIParent after clearing the alternate parent")
        "#,
    )
    .expect("FrameUtil top-level parent behavior should use the original provider");
}

fn assert_menu_tooltip_uses_provider(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(type(GetAppropriateTooltip) == "function")
        assert(type(MenuUtil.HookTooltipScripts) == "function")
        local owner = CreateFrame("Frame", "TopLevelParentTooltipProbe", UIParent)
        local seenTooltip
        MenuUtil.HookTooltipScripts(owner, function(tooltip)
            seenTooltip = tooltip
        end)
        owner:GetScript("OnEnter")(owner)
        assert(seenTooltip == GameTooltip, "MenuUtil must use Blizzard's GameTooltip provider")
        owner:GetScript("OnLeave")(owner)
        "#,
    )
    .expect("Blizzard_Menu tooltip consumer should use the original provider");
}
