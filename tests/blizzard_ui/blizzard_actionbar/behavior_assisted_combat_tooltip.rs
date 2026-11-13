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
                local managerSuffix = "Blizzard_ActionBar/Mainline/AssistedCombatManager.lua"
                assert(managerSource:sub(-#managerSuffix) == managerSuffix,
                    "expected the original ActionBar owner, got " .. managerSource)

                local consumerInfo = debug.getinfo(ClassTalentButtonSpendMixin.AddTooltipInstructions, "S")
                local consumerSource = consumerInfo.source:gsub("\\", "/")
                local consumerSuffix = "Blizzard_PlayerSpells/ClassTalents/Blizzard_ClassTalentButtonTemplates.lua"
                assert(consumerSource:sub(-#consumerSuffix) == consumerSuffix,
                    "expected the selected class-talent consumer, got " .. consumerSource)

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
                    HasMassPurchase = function() return false end,
                }
                ClassTalentButtonSpendMixin.AddTooltipInstructions(button, GameTooltip)

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

#[test]
fn spellbook_rotation_hover_updates_highlight_permission() {
    with_blizzard_addon_smoke_shape(&["Blizzard_PlayerSpells"], &[], |env, loaded| {
        assert!(
            loaded.iter().any(|name| name == "Blizzard_PlayerSpells"),
            "the LoadOnDemand PlayerSpells consumer must load"
        );

        let (enabled_on_enter, disabled_on_leave): (bool, bool) = env
            .eval(
                r#"
                assert(C_AddOns.IsAddOnLoaded("Blizzard_ActionBar"),
                    "ActionBar must load before the PlayerSpells consumer")
                assert(C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells"),
                    "PlayerSpells must be runtime-loaded")

                local ownerInfo = debug.getinfo(AssistedCombatManager.SetCanHighlightSpellbookSpells, "S")
                local ownerSource = ownerInfo.source:gsub("\\", "/")
                local ownerSuffix = "Blizzard_ActionBar/Mainline/AssistedCombatManager.lua"
                assert(ownerSource:sub(-#ownerSuffix) == ownerSuffix,
                    "expected the original ActionBar owner, got " .. ownerSource)

                local consumerInfo = debug.getinfo(AssistedCombatRotationSpellFrameMixin.OnIconEnter, "S")
                local consumerSource = consumerInfo.source:gsub("\\", "/")
                local consumerSuffix = "Blizzard_PlayerSpells/SpellBook/Blizzard_SpellBookFrame.lua"
                assert(consumerSource:sub(-#consumerSuffix) == consumerSuffix,
                    "expected the selected spellbook consumer, got " .. consumerSource)

                local button = CreateFrame("Button", nil, UIParent)
                local rotationButton = {
                    Button = button,
                    tooltipAnchor = "ANCHOR_RIGHT",
                    UpdateTooltip = AssistedCombatRotationSpellFrameMixin.UpdateTooltip,
                    ShowTooltip = function() end,
                }

                AssistedCombatRotationSpellFrameMixin.OnIconEnter(rotationButton)
                local enabled = AssistedCombatManager.canHighlightSpellbookSpells == true
                AssistedCombatRotationSpellFrameMixin.OnIconLeave(rotationButton)
                local disabled = AssistedCombatManager.canHighlightSpellbookSpells == false
                return enabled, disabled
                "#,
            )
            .expect("the selected spellbook hover consumer must execute");

        assert!(
            enabled_on_enter,
            "hover enter must enable spellbook highlighting"
        );
        assert!(
            disabled_on_leave,
            "hover leave must disable spellbook highlighting"
        );
    });
}
