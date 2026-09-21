#![cfg(feature = "aura-containers")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn player_controlled_or_group_member_classifies_native_token_families() {
    let env = WowLuaEnv::new().unwrap();
    // The native contract classifies tokens, not whether their units exist.
    env.state().borrow_mut().party_group_active = false;
    env.state().borrow_mut().party_members.clear();
    env.exec(
        r#"
        assert(type(UnitIsPlayerControlledOrGroupMember) == 'function')
        assert(__secureenv.UnitIsPlayerControlledOrGroupMember == UnitIsPlayerControlledOrGroupMember)
        for _, unit in ipairs({ 'party1', 'partypet1', 'raid1', 'raidpet1' }) do
            assert(not UnitExists(unit), 'fixture requires absent group unit: ' .. unit)
            assert(UnitIsPlayerControlledOrGroupMember(unit) == true, unit)
        end
        for _, unit in ipairs({
            "player", "pet", "vehicle", "party1", "party4",
            "partypet1", "partypet4", "raid1", "raid40", "raidpet1", "raidpet40"
        }) do
            assert(UnitIsPlayerControlledOrGroupMember(unit) == true, unit)
        end
        "#,
    )
    .expect("native controlled/group token families do not require populated units");
}

#[test]
fn player_controlled_or_group_member_rejects_nonfamily_and_invalid_indices() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, unit in ipairs({
            "target", "focus", "mouseover", "npc", "boss1", "arena1", "arenapet1",
            "party0", "party5", "partypet0", "partypet5", "raid0", "raid41",
            "raidpet0", "raidpet41", "party", "raidpet", "party-1", "raid1target", ""
        }) do
            assert(UnitIsPlayerControlledOrGroupMember(unit) == false, unit)
        end
        "#,
    )
    .expect("nonfamily tokens and out-of-range group indices are not classified as members");
}

#[test]
fn player_controlled_or_group_member_supports_secure_aura_identity_filtering() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec_maybe_secure(
                    r#"
                    local aura = { spellId = 19750, isHelpful = true, isHarmful = false }
                    assert(C_Secrets.GetSpellAuraSecrecy(aura.spellId) ~= Enum.SecrecyLevel.NeverSecret,
                        'fixture must reach the controlled/group predicate, not the secrecy exemption')
                    for _, unit in ipairs({ "player", "pet", "party1", "partypet1", "raid1", "raidpet1" }) do
                        assert(AuraContainerUtil.CanApplyIdentityCandidateFilters(unit, aura), unit)
                    end
                    assert(UnitIsPlayerControlledOrGroupMember("target") == false)
                    "#,
                    true,
                )
                .expect("real aura filtering permits helpful auras for native controlled/group tokens");
            },
        );
    });
}
