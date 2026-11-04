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
        local _, _, _, x = GameTooltip:GetLeftLine(1):GetPoint(2)
        assert(math.abs(x - 30) < 0.1, "default icon reserves 12 + 8 on the left")
        local _, _, _, x3 = GameTooltip:GetLeftLine(3):GetPoint(2)
        assert(math.abs(x3 - 32) < 0.1, "overlaid icons reserve the maximum, not the sum")
        "#,
    )
    .unwrap();
}
