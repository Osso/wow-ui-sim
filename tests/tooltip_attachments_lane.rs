#![cfg(all(
    feature = "gui",
    any(feature = "client-retail", feature = "client-ptr")
))]

mod tooltip_full_env_helpers;

use tooltip_full_env_helpers::setup_full_env;

#[test]
fn blizzard_tooltip_typed_icons_attach_to_their_existing_rows() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = Enum.TooltipDataLineType.GemSocket, leftText = "Socket", gemIcon = 134400 },
            { type = Enum.TooltipDataLineType.LearnableSpell, leftText = "Spell", spellIcon = 134401 },
            { type = Enum.TooltipDataLineType.AzeriteEssenceSlot, leftText = "Essence", slotType = "Major", essenceIcon = 134402 },
            { type = Enum.TooltipDataLineType.QuestObjective, leftText = "Objective", completed = true },
        } } }))
        assert(GameTooltip:NumLines() == 4, "icons must not append text rows")
        "#,
    )
    .unwrap();
}

#[test]
fn tooltip_attachment_cap_is_per_tooltip_and_clear_resets_it() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        GameTooltip:AddTexture(134400, { width = 200 })
        assert(GameTooltip:NumLines() == 0, "an empty tooltip has no attachment row")
        GameTooltip:AddLine("First")
        for i = 1, 29 do GameTooltip:AddTexture(134400) end
        GameTooltip:AddLine("Second")
        GameTooltip:AddTexture(134400)
        GameTooltip:AddTexture(134400, { width = 200 })
        GameTooltip:Show()
        GameTooltip:ClearLines()
        GameTooltip:AddLine("Fresh")
        GameTooltip:AddTexture(134400, { width = 50 })
        GameTooltip:Show()
        "#,
    )
    .unwrap();
}
