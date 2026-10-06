use super::patch_12_0_0_struct_shapes::assert_shape;
use wow_ui_sim::c_api::c_cooldown_viewer::CooldownViewerCooldown;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn cooldown_viewer_parent_identity_category_and_snapshots() {
    let env = WowLuaEnv::new().unwrap();
    for (id, category, spell) in [(701, 0, 19750), (702, 1, 642)] {
        env.state().borrow_mut().cooldown_viewer_cooldowns.insert(
            id,
            CooldownViewerCooldown {
                cooldown_id: id,
                category,
                spell_id: Some(spell),
                spell_category_id: Some(99),
                override_spell_id: Some(19751),
                override_tooltip_spell_id: Some(19752),
                equip_slot: Some(13),
                buff_slot: Some(2),
                linked_spell_ids: vec![19750, 642],
                self_aura: true,
                has_aura: true,
                charges: true,
                is_known: true,
                is_invisible: false,
                flags: 1,
                ..Default::default()
            },
        );
    }
    assert_shape(
        &env,
        "CooldownViewerDocumentation.lua",
        "CooldownViewerCooldown",
        "return C_CooldownViewer.GetCooldownViewerCooldownInfo(701)",
    );
    env.exec(r#"
        local first = C_CooldownViewer.GetCooldownViewerCooldownInfo(701)
        local second = C_CooldownViewer.GetCooldownViewerCooldownInfo(702)
        assert(first.cooldownID == 701 and first.category == Enum.CooldownViewerCategory.Essential)
        assert(second.cooldownID == 702 and second.category == Enum.CooldownViewerCategory.Utility)
        assert(first.spellID == 19750 and second.spellID == 642)
        assert(first.spellCategoryID == 99 and first.equipSlot == 13 and first.buffSlot == 2)
        assert(first.overrideSpellID == 19751 and first.overrideTooltipSpellID == 19752)
        assert(first.selfAura and first.hasAura and first.charges and first.isKnown and not first.isInvisible)
        first.linkedSpellIDs[1] = -1; first.category = -1
        assert(C_CooldownViewer.GetCooldownViewerCooldownInfo(701).linkedSpellIDs[1] == 19750)
        assert(second.linkedSpellIDs[1] == 19750 and second.category == 1)
        assert(C_CooldownViewer.GetCooldownViewerCooldownInfo(9999) == nil)
    "#).unwrap();
}

#[test]
fn spell_cooldown_recovery_and_gcd_follow_cast_producer() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local empty = C_Spell.GetSpellCooldown(19750)
        assert(empty.timeUntilEndOfStartRecovery == nil and empty.isOnGCD == nil)
        UpdateCount = 0
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('SPELL_UPDATE_COOLDOWN')
        frame:SetScript('OnEvent', function()
            UpdateCount = UpdateCount + 1
            EventCooldown = C_Spell.GetSpellCooldown(19750)
        end)
        CastSpellByID(19750)
        assert(UpdateCount == 1, 'real cast producer must publish cooldown update')
        assert(EventCooldown.isOnGCD == true)
        assert(type(EventCooldown.timeUntilEndOfStartRecovery) == 'number')
        assert(EventCooldown.timeUntilEndOfStartRecovery > 0 and EventCooldown.timeUntilEndOfStartRecovery <= 1.5)
        BeforeRecovery = EventCooldown.timeUntilEndOfStartRecovery
    "#).unwrap();
    assert_shape(
        &env,
        "SpellSharedDocumentation.lua",
        "SpellCooldownInfo",
        "return C_Spell.GetSpellCooldown(19750)",
    );
    // Deterministic host-clock progression, no wall-clock sleeps.
    env.state().borrow_mut().start_time -= std::time::Duration::from_millis(500);
    env.exec(
        r#"
        local next = C_Spell.GetSpellCooldown(19750)
        assert(next.isOnGCD == true and next.timeUntilEndOfStartRecovery < BeforeRecovery - 0.49)
        assert(next.timeUntilEndOfStartRecovery > 0)
        next.isOnGCD = false
        assert(EventCooldown.isOnGCD == true and C_Spell.GetSpellCooldown(19750).isOnGCD == true)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().start_time -= std::time::Duration::from_secs(2);
    env.exec(
        r#"
        local expired = C_Spell.GetSpellCooldown(19750)
        assert(expired.isOnGCD == false and expired.timeUntilEndOfStartRecovery == nil)
        assert(expired.duration == 0 and expired.isActive == false)
        CastSpellByID(642)
        assert(UpdateCount == 2 and EventCooldown.isOnGCD == false)
        assert(EventCooldown.timeUntilEndOfStartRecovery == nil)
        local spellOnly = C_Spell.GetSpellCooldown(642)
        assert(spellOnly.duration == 300 and spellOnly.isActive == true)
        assert(spellOnly.isOnGCD == false and spellOnly.timeUntilEndOfStartRecovery == nil)
    "#,
    )
    .unwrap();
}
