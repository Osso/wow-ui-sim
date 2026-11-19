#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

#[test]
fn menu_util_hook_tooltip_uses_blizzard_provider() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.exec(
        r#"
        assert(type(GetAppropriateTooltip) == "function")
        assert(type(MenuUtil.HookTooltipScripts) == "function")
        local alternate = CreateFrame("Frame", "TopLevelParentProbe", UIParent)
        alternate:Show()
        SetAlternateTopLevelParent(alternate)
        assert(GetAppropriateTopLevelParent() == alternate,
            "FrameUtil must return the alternate top-level parent")
        local child = CreateFrame("Frame", "TopLevelParentChildProbe", UIParent)
        SetAppropriateTopLevelParent(child)
        assert(child:GetParent() == alternate,
            "FrameUtil must parent the frame to the selected top-level parent")
        ClearAlternateTopLevelParent()
        assert(GetAppropriateTopLevelParent() == UIParent,
            "FrameUtil must restore UIParent after clearing the alternate parent")

        local owner = CreateFrame("Frame", "TopLevelParentTooltipProbe", UIParent)
        local seenTooltip
        MenuUtil.HookTooltipScripts(owner, function(tooltip)
            seenTooltip = tooltip
        end)
        owner:GetScript("OnEnter")(owner)
        assert(seenTooltip == GameTooltip, "MenuUtil must use Blizzard's GameTooltip provider")
        assert(GameTooltip:GetOwner() == owner, "MenuUtil must assign the owner to the tooltip")
        owner:GetScript("OnLeave")(owner)
        "#,
    )
    .expect("Blizzard_Menu tooltip consumer should use the original provider");
}
