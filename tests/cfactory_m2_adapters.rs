//! M2 generated-documentation contracts and supplied-state adapter behavior.
#![cfg(not(feature = "client-wrath"))]

use crate::cfactory_support::documented_env;
use wow_ui_sim::lua_api::state::EquippedItem;

#[test]
fn cfactory_m2_offhand_weapon_uses_equipment_identity() {
    let env = documented_env("PaperDollInfoDocumentation.lua", "OffhandHasWeapon");
    for (item_id, expected) in [(None, false), (Some(221165), true), (Some(133503), false)] {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.player.equipped_items.remove(&17);
        if let Some(item_id) = item_id {
            sim.player.equipped_items.insert(
                17,
                EquippedItem {
                    item_id,
                    enchant_id: 0,
                    gem_ids: [0; 3],
                    applied_transmog: None,
                    owned_identity: None,
                },
            );
        }
        drop(sim);
        let actual: bool = env
            .eval("return C_PaperDollInfo.OffhandHasWeapon()")
            .unwrap();
        assert_eq!(actual, expected, "offhand {item_id:?}");
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn cfactory_m2_known_spell_or_book_uses_existing_membership() {
    let env = documented_env("SpellBookDocumentation.lua", "IsSpellKnownOrInSpellBook");
    env.exec(r#"
        local spellID
        for i = 1, C_SpellBook.GetNumSpellBookSkillLines() do
            local line = C_SpellBook.GetSpellBookSkillLineInfo(i)
            if line.numSpellBookItems > 0 then
                local info = C_SpellBook.GetSpellBookItemInfo(line.itemIndexOffset + 1, Enum.SpellBookSpellBank.Player)
                spellID = info.spellID
                break
            end
        end
        assert(spellID, 'SETUP: seeded spellbook')
        assert(C_SpellBook.IsSpellKnownOrInSpellBook(spellID))
        assert(not C_SpellBook.IsSpellKnownOrInSpellBook(4000000000))
    "#).unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn cfactory_m2_forever_keyring_capability_documented_boolean() {
    let env = documented_env("ActionBarFrameDocumentation.lua", "ShouldShowKeyring");
    env.exec("assert(type(C_ActionBar.ShouldShowKeyring()) == 'boolean')")
        .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn cfactory_m2_forever_unregister_removes_only_selected_button() {
    let env = documented_env(
        "ActionBarFrameDocumentation.lua",
        "UnregisterActionUIButton",
    );
    env.exec(
        r#"
        FirstButton = CreateFrame('CheckButton')
        OtherButton = CreateFrame('CheckButton')
        C_ActionBar.RegisterActionUIButton(FirstButton, 1)
        C_ActionBar.RegisterActionUIButton(OtherButton, 2)
        C_ActionBar.UnregisterActionUIButton(FirstButton)
    "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().action_ui_buttons.len(), 1);
    env.exec("C_ActionBar.UnregisterActionUIButton(OtherButton)")
        .unwrap();
    assert!(env.state().borrow().action_ui_buttons.is_empty());
}

#[cfg(feature = "client-wowforever")]
#[test]
fn cfactory_m2_forever_cooldown_viewer_reports_supplied_availability() {
    let env = documented_env(
        "CooldownViewerDocumentation.lua",
        "IsCooldownViewerAvailable",
    );
    env.exec(
        r#"
        local available, reason = C_CooldownViewer.IsCooldownViewerAvailable()
        assert(available == false and type(reason) == 'string')
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .cooldown_viewer_availability
        .is_available = true;
    env.state()
        .borrow_mut()
        .cooldown_viewer_availability
        .failure_reason = "Host supplied".into();
    env.exec(
        r#"
        local available, reason = C_CooldownViewer.IsCooldownViewerAvailable()
        assert(available == true and reason == 'Host supplied')
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn cfactory_m2_forever_earned_talent_point_documented_boolean() {
    let env = documented_env(
        "SpecializationInfoDocumentation.lua",
        "HasPlayerEarnedATalentPoint",
    );
    env.exec("assert(type(C_SpecializationInfo.HasPlayerEarnedATalentPoint()) == 'boolean')")
        .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn cfactory_m2_forever_item_condition_documented_boolean() {
    let env = documented_env("SpellDocumentation.lua", "TargetSpellChecksItemCondition");
    env.exec("assert(type(C_Spell.TargetSpellChecksItemCondition()) == 'boolean')")
        .unwrap();
}
