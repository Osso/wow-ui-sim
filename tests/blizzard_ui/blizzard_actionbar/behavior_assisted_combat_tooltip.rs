//! PRESERVATION: the selected PlayerSpells consumer uses the ActionBar-owned
//! AssistedCombatManager implementation after the ActionBar owner loads.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

#[test]
fn class_talent_tooltip_uses_original_assisted_combat_manager() {
    with_blizzard_addon_smoke_shape(&["Blizzard_PlayerSpells"], &[], |env, loaded| {
        assert!(
            loaded.iter().any(|name| name == "Blizzard_PlayerSpells"),
            "the LoadOnDemand PlayerSpells consumer must load"
        );

        let (found_line, expected, actual): (bool, String, String) = env
            .eval(
                r#"
                assert(C_AddOns.IsAddOnLoaded("Blizzard_ActionBar"),
                    "ActionBar must load before the PlayerSpells consumer")
                assert(C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells"),
                    "PlayerSpells must be runtime-loaded")

                local managerInfo = debug.getinfo(AssistedCombatManager.AddSpellTooltipLine, "S")
                local managerSource = managerInfo.source:gsub("\\", "/")
                assert(managerSource:find("Blizzard_ActionBar/Mainline/AssistedCombatManager.lua", 1, true),
                    "expected the original ActionBar owner, got " .. managerSource)
                assert(managerInfo.linedefined == 339,
                    "expected original owner line 339, got " .. tostring(managerInfo.linedefined))

                local consumerInfo = debug.getinfo(ClassTalentButtonSpendMixin.AddTooltipInstructions, "S")
                local consumerSource = consumerInfo.source:gsub("\\", "/")
                assert(consumerSource:find("Blizzard_PlayerSpells/ClassTalents/Blizzard_ClassTalentButtonTemplates.lua", 1, true),
                    "expected the selected class-talent consumer, got " .. consumerSource)
                assert(consumerInfo.linedefined == 209,
                    "expected original consumer line 209, got " .. tostring(consumerInfo.linedefined))

                local spellID = 116
                AssistedCombatManager.useAssistedHighlight = true
                AssistedCombatManager.rotationSpells[spellID] = true
                GameTooltip:ClearLines()

                local button = {
                    GetActionBarStatus = function() return ActionButtonUtil.ActionBarActionStatus.NotMissing end,
                    GetSpellID = function() return spellID end,
                    GetOverriddenSpellID = function() return nil end,
                    CanPurchaseRank = function() return false end,
                    CanRefundRank = function() return false end,
                    CanCascadeRepurchaseRanks = function() return false end,
                    IsGhosted = function() return false end,
                }
                ClassTalentButtonSpendMixin:AddTooltipInstructions(button, GameTooltip)

                local expected = C_ActionBar.HasAssistedCombatActionButtons()
                    and ASSISTED_COMBAT_SPELL_INCLUDED
                    or ASSISTED_COMBAT_HIGHLIGHT_SPELL_INCLUDED
                local actual = ""
                for index = 1, GameTooltip:NumLines() do
                    local line = _G["GameTooltipTextLeft" .. index]
                    if line and line:GetText() == expected then
                        actual = line:GetText()
                        break
                    end
                end
                return actual == expected, expected, actual
                "#,
            )
            .expect("the selected class-talent tooltip consumer must execute");

        assert_eq!(
            found_line, true,
            "the original consumer must add the assisted-combat line for a supplied rotation spell; expected {expected:?}, found {actual:?}"
        );
        assert_eq!(actual, expected);
    });
}
