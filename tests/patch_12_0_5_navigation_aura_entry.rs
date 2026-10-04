#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn aura(id: i32, helpful: bool) -> AuraInfo {
    AuraInfo {
        name: if helpful {
            "Flash of Light"
        } else {
            "Corruption"
        }
        .into(),
        spell_id: if helpful { 19750 } else { 172 },
        icon: if helpful { 135987 } else { 136118 },
        duration: 47.0,
        expiration_time: 150.0,
        applications: 3,
        source_unit: "player".into(),
        is_helpful: helpful,
        is_raid: true,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: if helpful { None } else { Some("Magic".into()) },
        aura_instance_id: id,
    }
}

fn aura_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.player.buffs = vec![aura(41, true), aura(72, false)];
        sim.party_members.clear();
    }
    env.exec(
        r#"
        EntryObserver = CreateFrame('Frame')
        EntryObserver:RegisterEvent('ENCOUNTER_START')
        EntryObserver:RegisterEvent('CHALLENGE_MODE_START')
        EntryObserver:RegisterEvent('PVP_MATCH_ACTIVE')
        EntryObserver:SetScript('OnEvent', function(self, event)
            local helpful = C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL')
            local harmful = C_UnitAuras.GetAuraDataByIndex('player', 1, 'HARMFUL')
            self.helpful = helpful.auraInstanceID
            self.harmful = harmful.auraInstanceID
            self.event = event
        end)
        "#,
    )
    .unwrap();
    env
}

fn queue_ids(env: &WowLuaEnv, ids: &[i32]) {
    env.state().borrow_mut().aura_entry_ids.pending = Some(ids.to_vec());
}

fn assert_player_aura_queries(env: &WowLuaEnv, old: &[i32], new: &[i32]) {
    for id in old {
        assert!(
            env.eval::<bool>(&format!(
                "return C_UnitAuras.GetAuraDataByAuraInstanceID('player', {id}) == nil"
            ))
            .unwrap()
        );
    }
    env.exec(&format!(
        r#"
        local helpful = C_UnitAuras.GetAuraDataByAuraInstanceID('player', {})
        local harmful = C_UnitAuras.GetAuraDataByAuraInstanceID('player', {})
        assert(helpful.name == 'Flash of Light' and helpful.spellId == 19750)
        assert(harmful.name == 'Corruption' and harmful.spellId == 172)
        assert(helpful.duration == 47 and harmful.duration == 47)
        assert(helpful.expirationTime == 150 and harmful.expirationTime == 150)
        assert(helpful.applications == 3 and harmful.applications == 3)
        assert(helpful.sourceUnit == 'player' and harmful.sourceUnit == 'player')
        assert(helpful.isHelpful and harmful.isHarmful)
        assert(harmful.dispelName == 'Magic')
        local _, helpfulSlot = C_UnitAuras.GetAuraSlots('player', 'HELPFUL')
        local _, harmfulSlot = C_UnitAuras.GetAuraSlots('player', 'HARMFUL')
        assert(helpfulSlot == helpful.auraInstanceID)
        assert(harmfulSlot == harmful.auraInstanceID)
        assert(C_UnitAuras.GetAuraDataBySlot('player', helpfulSlot).spellId == 19750)
        assert(C_UnitAuras.GetAuraDataBySlot('player', harmfulSlot).spellId == 172)
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL').auraInstanceID == helpfulSlot)
        assert(C_UnitAuras.GetPlayerAuraBySpellID(19750).auraInstanceID == helpfulSlot)
        assert(EntryObserver.helpful == helpfulSlot and EntryObserver.harmful == harmfulSlot,
            'entry consumer must observe committed IDs')
        "#,
        new[0], new[1]
    ))
    .unwrap();
}

#[test]
fn navigation_trusted_results_follow_explicit_host_selection() {
    let env = WowLuaEnv::new().unwrap();
    for token in ["party1", "party2"] {
        env.state().borrow_mut().nearest_party_member_token = Some(token.into());
        assert_eq!(
            env.eval::<String>("return C_Navigation.GetNearestPartyMemberToken()")
                .unwrap(),
            token
        );
    }
    env.state().borrow_mut().nearest_party_member_token = None;
    assert!(
        env.eval::<String>("return C_Navigation.GetNearestPartyMemberToken()")
            .is_err()
    );
}

#[test]
fn navigation_addon_denial_preserves_host_selection_and_caller_taint() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().nearest_party_member_token = Some("party2".into());
    env.exec(
        r#"
        local function trusted()
            return C_Navigation.GetNearestPartyMemberToken()
        end
        local function addon()
            assert(debug.getstacktaint() == 'NavigationFixture')
            local ok, err = pcall(trusted)
            assert(not ok, 'addon must not receive selected party token')
            assert(string.find(err, 'C_Navigation.GetNearestPartyMemberToken', 1, true))
            assert(debug.getstacktaint() == 'NavigationFixture')
        end
        debug.setobjecttaint(addon, 'NavigationFixture')
        assert(issecure())
        addon()
        assert(issecure(), 'denial must restore outer caller')
        assert(C_Navigation.GetNearestPartyMemberToken() == 'party2')
        "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().nearest_party_member_token.as_deref(),
        Some("party2")
    );
}

#[test]
fn navigation_authorizes_before_reading_absent_host_selection() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local function addon()
            local ok, err = pcall(C_Navigation.GetNearestPartyMemberToken)
            assert(not ok)
            assert(string.find(err, 'addons', 1, true), 'caller denial precedes absent selection')
            assert(debug.getstacktaint() == 'NavigationFixture')
        end
        debug.setobjecttaint(addon, 'NavigationFixture')
        addon()
        assert(issecure())
        "#,
    )
    .unwrap();
}

#[test]
fn aura_entry_rekeys_before_each_entry_event_consumer_without_payload_loss() {
    let env = aura_env();
    let mut old = vec![41, 72];
    for (event, ids) in [
        ("ENCOUNTER_START", vec![907, 301]),
        ("CHALLENGE_MODE_START", vec![801, 504]),
        ("PVP_MATCH_ACTIVE", vec![605, 902]),
        ("ENCOUNTER_START", vec![1003, 412]),
    ] {
        queue_ids(&env, &ids);
        env.fire_event(event).unwrap();
        assert_player_aura_queries(&env, &old, &ids);
        assert_eq!(
            env.eval::<String>("return EntryObserver.event").unwrap(),
            event
        );
        assert!(env.state().borrow().aura_entry_ids.pending.is_none());
        env.fire_event("PLAYER_REGEN_DISABLED").unwrap();
        env.fire_on_update(0.25).unwrap();
        assert_player_aura_queries(&env, &old, &ids);
        old.extend(ids);
    }
}

#[test]
fn aura_entry_invalid_plan_is_atomic_and_does_not_deliver_entry_event() {
    let env = aura_env();
    queue_ids(&env, &[907, 301]);
    env.fire_event("ENCOUNTER_START").unwrap();
    env.exec("EntryObserver.event = nil").unwrap();
    for invalid in [
        vec![907, 100],
        vec![41, 100],
        vec![100, 100],
        vec![0, 100],
        vec![100],
    ] {
        queue_ids(&env, &invalid);
        assert!(env.fire_event("ENCOUNTER_START").is_err());
        assert_eq!(
            env.state().borrow().aura_entry_ids.pending.as_ref(),
            Some(&invalid)
        );
        assert_player_aura_queries(&env, &[41, 72], &[907, 301]);
        assert!(
            env.eval::<bool>("return EntryObserver.event == nil")
                .unwrap()
        );
    }
    env.state().borrow_mut().aura_entry_ids.pending = None;
    assert!(env.fire_event("ENCOUNTER_START").is_err());
    queue_ids(&env, &[702, 811]);
    env.fire_event("ENCOUNTER_START").unwrap();
    assert_player_aura_queries(&env, &[41, 72, 907, 301], &[702, 811]);
}

#[test]
fn aura_entry_admin_global_and_loader_dispatch_use_the_same_transition() {
    let env = aura_env();
    queue_ids(&env, &[600, 201]);
    env.exec("A_Admin.FireEvent('ENCOUNTER_START', 123, 'Fixture', 1, 5)")
        .unwrap();
    assert_player_aura_queries(&env, &[41, 72], &[600, 201]);
    queue_ids(&env, &[602, 203]);
    env.exec("FireEvent('CHALLENGE_MODE_START')").unwrap();
    assert_player_aura_queries(&env, &[41, 72, 600, 201], &[602, 203]);
    queue_ids(&env, &[604, 205]);
    wow_ui_sim::lua_api::LoaderEnv::new(&env)
        .fire_event_with_args("PVP_MATCH_ACTIVE", &[])
        .unwrap();
    assert_player_aura_queries(&env, &[41, 72, 600, 201, 602, 203], &[604, 205]);
}

#[test]
fn aura_entry_rekeys_party_helpful_and_harmful_stores_without_changing_members() {
    let env = aura_env();
    env.state().borrow_mut().party_members = vec![wow_ui_sim::lua_api::state::PartyMember {
        name: "Thrynn".into(),
        name_cached: true,
        connected: true,
        class_index: 5,
        level: 80,
        health: 700,
        health_max: 900,
        power: 200,
        power_max: 500,
        power_type: 0,
        power_type_name: "MANA".into(),
        is_leader: false,
        dead_since: None,
        buffs: vec![aura(83, true)],
        debuffs: vec![aura(94, false)],
    }];
    queue_ids(&env, &[907, 301, 802, 603]);
    env.fire_event("ENCOUNTER_START").unwrap();
    assert_player_aura_queries(&env, &[41, 72], &[907, 301]);
    env.exec(
        r#"
        assert(C_UnitAuras.GetAuraDataByAuraInstanceID('party1', 83) == nil)
        assert(C_UnitAuras.GetAuraDataByAuraInstanceID('party1', 94) == nil)
        local buff = C_UnitAuras.GetAuraDataByAuraInstanceID('party1', 802)
        local debuff = C_UnitAuras.GetAuraDataByAuraInstanceID('party1', 603)
        assert(buff.name == 'Flash of Light' and buff.duration == 47)
        assert(debuff.name == 'Corruption' and debuff.applications == 3)
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL').auraInstanceID == 802)
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HARMFUL').auraInstanceID == 603)
        "#,
    )
    .unwrap();
    let sim = env.state().borrow();
    assert_eq!(sim.party_members[0].name, "Thrynn");
    assert_eq!(sim.party_members[0].health, 700);
    assert_eq!(sim.party_members[0].power, 200);
}

#[test]
fn aura_added_after_entry_is_queryable_without_rekeying_survivors() {
    let env = aura_env();
    queue_ids(&env, &[907, 301]);
    env.fire_event("ENCOUNTER_START").unwrap();
    let mut added = aura(1111, true);
    added.name = "Post-entry aura".into();
    added.spell_id = 21562;
    env.state().borrow_mut().player.buffs.push(added);
    env.fire_on_update(0.25).unwrap();
    assert_player_aura_queries(&env, &[41, 72], &[907, 301]);
    assert_eq!(
        env.eval::<String>("return C_UnitAuras.GetAuraDataByAuraInstanceID('player', 1111).name")
            .unwrap(),
        "Post-entry aura"
    );
}
