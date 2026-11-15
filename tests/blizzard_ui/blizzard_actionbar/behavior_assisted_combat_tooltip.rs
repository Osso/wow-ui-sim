//! PRESERVATION: the selected PlayerSpells consumer uses the ActionBar-owned
//! AssistedCombatManager implementation after the ActionBar owner loads.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_player_spells(env: &WowLuaEnv) {
    let loaded_before: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PlayerSpells')")
        .expect("PlayerSpells load state should be available");
    assert!(!loaded_before, "PlayerSpells must begin LoadOnDemand");

    crate::common::supply_empty_forever_spellbook_outfits(env);
    env.exec("C_AddOns.LoadAddOn('Blizzard_PlayerSpells')")
        .expect("PlayerSpells should load through the addon manager");
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PlayerSpells')")
        .expect("PlayerSpells load state should be available after loading");
    assert!(loaded, "PlayerSpells must load before its consumers run");
}

fn set_spellbook_source_suffixes(env: &WowLuaEnv) {
    let frame_source_suffix = "Blizzard_PlayerSpells/SpellBook/Blizzard_SpellBookFrame.lua";
    let attic_source_suffix = match wow_ui_sim::client_profile::ACTIVE {
        wow_ui_sim::client_profile::ClientProfile::WowForever => {
            "Blizzard_PlayerSpells/Camelot/SpellBook/Blizzard_SpellBookFrame.lua"
        }
        _ => frame_source_suffix,
    };
    env.exec(&format!(
        "__wow_test_spellbook_frame_source_suffix = '{frame_source_suffix}'"
    ))
    .expect("spellbook consumer source suffix should be available to Lua assertions");
    env.exec(&format!(
        "__wow_test_spellbook_attic_source_suffix = '{attic_source_suffix}'"
    ))
    .expect("spellbook attic source suffix should be available to Lua assertions");
}

pub(super) fn with_player_spells_loaded<F>(assertions: F)
where
    F: FnOnce(&WowLuaEnv),
{
    with_blizzard_addon_smoke_shape(&["Blizzard_UIPanels_Game"], &[], |env, _| {
        load_player_spells(env);
        set_spellbook_source_suffixes(env);
        assertions(env);
    });
}

#[test]
fn class_talent_tooltip_uses_original_assisted_combat_manager() {
    with_player_spells_loaded(|env| {
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
    with_player_spells_loaded(|env| {
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
                local consumerSuffix = __wow_test_spellbook_frame_source_suffix
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

#[test]
fn spellbook_rotation_tooltip_uses_original_action_description() {
    with_player_spells_loaded(|env| {
        let (contains_description, passed_description): (bool, String) = env
            .eval(
                r#"
                assert(C_AddOns.IsAddOnLoaded("Blizzard_ActionBar"),
                    "ActionBar must load before the PlayerSpells consumer")
                assert(C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells"),
                    "PlayerSpells must be runtime-loaded")

                local ownerInfo = debug.getinfo(AssistedCombatManager.GetActionSpellDescription, "S")
                local ownerSource = ownerInfo.source:gsub("\\", "/")
                local ownerSuffix = "Blizzard_ActionBar/Mainline/AssistedCombatManager.lua"
                assert(ownerSource:sub(-#ownerSuffix) == ownerSuffix,
                    "expected the original ActionBar owner, got " .. ownerSource)

                local consumerInfo = debug.getinfo(AssistedCombatRotationSpellFrameMixin.ShowTooltip, "S")
                local consumerSource = consumerInfo.source:gsub("\\", "/")
                local consumerSuffix = __wow_test_spellbook_frame_source_suffix
                assert(consumerSource:sub(-#consumerSuffix) == consumerSuffix,
                    "expected the selected spellbook consumer, got " .. consumerSource)

                local expected = "supplied assisted-combat description"
                AssistedCombatManager.spellDescription = expected
                AssistedCombatManager.rotationSpells = {}
                local originalAddNormalLine = GameTooltip_AddNormalLine
                local passedDescription
                GameTooltip_AddNormalLine = function(tooltip, text, ...)
                    passedDescription = text
                    return originalAddNormalLine(tooltip, text, ...)
                end
                AssistedCombatRotationSpellFrameMixin.ShowTooltip()
                GameTooltip_AddNormalLine = originalAddNormalLine
                return passedDescription == expected, tostring(passedDescription)
                "#,
            )
            .expect("the selected spellbook tooltip consumer must execute");

        assert_eq!(
            contains_description, true,
            "the original tooltip consumer must pass the supplied manager description; got {passed_description:?}"
        );
        assert_eq!(passed_description, "supplied assisted-combat description");
    });
}

#[test]
fn spellbook_attic_reads_original_assisted_action_state() {
    with_player_spells_loaded(|env| {
        let (shown, spell_id): (bool, i32) = env
            .eval(
                r#"
                assert(C_AddOns.IsAddOnLoaded("Blizzard_ActionBar"),
                    "ActionBar must load before the PlayerSpells consumer")
                assert(C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells"),
                    "PlayerSpells must be runtime-loaded")

                local ownerInfo = debug.getinfo(AssistedCombatManager.HasActionSpell, "S")
                local ownerSource = ownerInfo.source:gsub("\\", "/")
                local ownerSuffix = "Blizzard_ActionBar/Mainline/AssistedCombatManager.lua"
                assert(ownerSource:sub(-#ownerSuffix) == ownerSuffix,
                    "expected the original ActionBar owner, got " .. ownerSource)

                local consumerInfo = debug.getinfo(SpellBookFrameMixin.UpdateAttic, "S")
                local consumerSource = consumerInfo.source:gsub("\\", "/")
                local consumerSuffix = __wow_test_spellbook_attic_source_suffix
                assert(consumerSource:sub(-#consumerSuffix) == consumerSuffix,
                    "expected the selected spellbook consumer, got " .. consumerSource)

                AssistedCombatManager.actionSpellID = 116
                local rotationButton = {
                    shown = false,
                    spellID = 0,
                    Show = function(self) self.shown = true end,
                    Hide = function(self) self.shown = false end,
                    SetSpellID = function(self, value) self.spellID = value end,
                }
                local settingsDropdown = {
                    ClearAllPoints = function() end,
                    SetPoint = function() end,
                }
                local frame = {
                    SettingsDropdown = settingsDropdown,
                    AssistedCombatRotationSpellFrame = rotationButton,
                    ResizeSearchBox = function() end,
                }
                SpellBookFrameMixin.UpdateAttic(frame)
                return rotationButton.shown, rotationButton.spellID
                "#,
            )
            .expect("the original spellbook attic consumer must execute");

        assert!(
            shown,
            "the supplied action spell must show the rotation button"
        );
        assert_eq!(
            spell_id, 116,
            "the consumer must use the supplied action spell id"
        );
    });
}

#[test]
fn spellbook_item_consumes_original_rotation_highlight_state() {
    with_player_spells_loaded(|env| {
        let shown: bool = env
            .eval(
                r#"
                assert(C_AddOns.IsAddOnLoaded("Blizzard_ActionBar"),
                    "ActionBar must load before the PlayerSpells consumer")
                assert(C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells"),
                    "PlayerSpells must be runtime-loaded")

                local ownerInfo = debug.getinfo(AssistedCombatManager.ShouldHighlightSpellbookSpell, "S")
                local ownerSource = ownerInfo.source:gsub("\\", "/")
                local ownerSuffix = "Blizzard_ActionBar/Mainline/AssistedCombatManager.lua"
                assert(ownerSource:sub(-#ownerSuffix) == ownerSuffix,
                    "expected the original ActionBar owner, got " .. ownerSource)

                local consumerInfo = debug.getinfo(SpellBookItemMixin.UpdateAssistedCombatState, "S")
                local consumerSource = consumerInfo.source:gsub("\\", "/")
                local consumerSuffix = "Blizzard_PlayerSpells/SpellBook/Blizzard_SpellBookItem.lua"
                assert(consumerSource:sub(-#consumerSuffix) == consumerSuffix,
                    "expected the selected spellbook item consumer, got " .. consumerSource)

                local spellID = 116
                AssistedCombatManager.useAssistedHighlight = true
                AssistedCombatManager.rotationSpells[spellID] = true
                local cover = {
                    shown = false,
                    SetShown = function(self, value) self.shown = value end,
                }
                local item = {
                    spellBookItemInfo = { spellID = spellID, isOffSpec = false },
                    Button = { AssistedCombatIconCover = cover },
                }
                SpellBookItemMixin.UpdateAssistedCombatState(item)
                return cover.shown == true
                "#,
            )
            .expect("the selected spellbook item consumer must execute");

        assert!(
            shown,
            "the original consumer must show the supplied rotation highlight"
        );
    });
}
