#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;
#[cfg(feature = "retail-12-0-7")]
use wow_ui_sim::lua_api::state::AuraInfo;

const LUA_FIXTURES: &str = r#"
        Sound = {unitToken='player', spellID=12345, soundFileID=98765}
        function SoundProbe()
            local function check(...)
                assert(select('#', ...) == 1)
                local id = ...
                assert(type(id) == 'number' and not issecretvalue(id))
                return id
            end
            return check(C_UnitAuras.AddPrivateAuraAppliedSound(Sound))
        end
        function Reject(query, message, ...)
            local ok, err = pcall(query, ...)
            assert(not ok and type(err) == 'string' and not issecretvalue(err))
            assert(err:find(message, 1, true), err)
        end
        function CheckAsset(query, asset, expected, ...)
            local function check(...)
                assert(select('#', ...) == 1)
                local result = ...
                assert(result == expected and not issecretvalue(result))
            end
            check(query(asset, ...))
        end
    "#;

fn environment() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("audit slice environment");
    env.exec(LUA_FIXTURES).unwrap();
    env
}

fn addon(env: &WowLuaEnv, body: &str) {
    env.exec(&format!(
        "local function probe() assert(debug.getstacktaint() == 'SliceAddon'); {body}; assert(debug.getstacktaint() == 'SliceAddon') end; debug.setobjecttaint(probe, 'SliceAddon'); probe(); assert(issecure())"
    )).expect("stamped addon closure with secure outer recovery");
}

fn install_secrets(env: &WowLuaEnv) {
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
    }
    // Secrets are created before entering tainted Lua.
    env.exec(
        r#"
        assert(issecure())
        SecretID = secretwrap(136243)
        SecretUnit = secretwrap('player')
        SecretExtra = secretwrap(false)
        SecretSound = secretwrap(Sound)
        SecretPath = secretwrap('Interface/AddOns/Slice/Missing.ogg')
        collectgarbage('collect')
    "#,
    )
    .unwrap();
}

#[test]
fn b24_epoch_permission_matrix_has_32_contexts_and_atomic_denial() {
    // Bits: secure caller, encounter, M+, PvP, player combat.
    for bits in 0u8..32 {
        let env = environment();
        let secure = bits & 1 != 0;
        let encounter = bits & 2 != 0;
        let mythic = bits & 4 != 0;
        let pvp = bits & 8 != 0;
        let combat = bits & 16 != 0;
        {
            let mut sim = env.state().borrow_mut();
            sim.world.encounter_in_progress = encounter;
            sim.mythic_plus.is_active = mythic;
            sim.private_aura_sound_registrations.pvp_match_active = pvp;
            sim.player.in_combat = combat;
            assert!(
                sim.private_aura_sound_registrations
                    .registrations
                    .is_empty()
            );
        }
        let mythic_restricted = mythic && (!cfg!(feature = "retail-12-0-7") || combat);
        // INFERRED: independent encounter/PvP restrictions still take precedence.
        let allowed = secure || !(encounter || pvp || mythic_restricted);
        let probe = if allowed {
            "MatrixID = SoundProbe()"
        } else {
            "Reject(C_UnitAuras.AddPrivateAuraAppliedSound, 'insecure registration denied', Sound)"
        };
        if secure {
            env.exec(probe).unwrap();
        } else {
            addon(&env, probe);
        }
        let sim = env.state().borrow();
        let sounds = &sim.private_aura_sound_registrations;
        assert_eq!(
            sounds.registrations.len(),
            usize::from(allowed),
            "bits={bits}"
        );
        assert_eq!(sounds.live_ids.len(), usize::from(allowed), "bits={bits}");
        assert_eq!(
            sounds.next_id,
            Some(if allowed { 2 } else { 1 }),
            "bits={bits}"
        );
        if allowed {
            let record = &sounds.registrations[&1];
            assert_eq!(record.spell_id, 12345);
            assert_eq!(record.sound_file_id, Some(98765));
            assert_eq!(record.unit_token, "player");
        }
    }
}

#[test]
fn b24_live_combat_transition_recovery_and_environment_isolation() {
    let env = environment();
    let other = environment();
    {
        let mut sim = env.state().borrow_mut();
        assert!(!sim.mythic_plus.is_active);
        assert!(!sim.world.encounter_in_progress);
        assert!(!sim.private_aura_sound_registrations.pvp_match_active);
        sim.mythic_plus.is_active = true;
        sim.player.in_combat = true;
    }
    addon(
        &env,
        "Reject(C_UnitAuras.AddPrivateAuraAppliedSound, 'insecure registration denied', Sound)",
    );
    env.state().borrow_mut().player.in_combat = false;
    if cfg!(feature = "retail-12-0-7") {
        addon(&env, "LiveID = SoundProbe()");
        assert_eq!(env.eval::<f64>("return LiveID").unwrap(), 1.0);
    } else {
        addon(
            &env,
            "Reject(C_UnitAuras.AddPrivateAuraAppliedSound, 'insecure registration denied', Sound)",
        );
    }
    env.state().borrow_mut().mythic_plus.is_active = false;
    addon(&env, "RecoveredID = SoundProbe()");
    let expected_id = if cfg!(feature = "retail-12-0-7") {
        2.0
    } else {
        1.0
    };
    assert_eq!(env.eval::<f64>("return RecoveredID").unwrap(), expected_id);
    assert!(
        other
            .state()
            .borrow()
            .private_aura_sound_registrations
            .registrations
            .is_empty()
    );
    addon(&other, "OtherID = SoundProbe()");
    assert_eq!(other.eval::<f64>("return OtherID").unwrap(), 1.0);
}

#[test]
fn b24_out_of_combat_permission_does_not_declassify_secret_sound_or_extras() {
    let env = environment();
    install_secrets(&env);
    env.state().borrow_mut().mythic_plus.is_active = true;
    env.state().borrow_mut().player.in_combat = false;
    env.exec("SecureID = C_UnitAuras.AddPrivateAuraAppliedSound(SecretSound)")
        .unwrap();
    assert_eq!(env.eval::<f64>("return SecureID").unwrap(), 1.0);
    addon(
        &env,
        r#"
        Reject(C_UnitAuras.AddPrivateAuraAppliedSound, 'untainted caller', SecretSound)
        Reject(C_UnitAuras.AddPrivateAuraAppliedSound, 'untainted caller', false, 0, SecretExtra)
        assert(issecretvalue(SecretSound) and issecretvalue(SecretExtra))
    "#,
    );
    let sim = env.state().borrow();
    assert_eq!(sim.private_aura_sound_registrations.registrations.len(), 1);
    assert_eq!(sim.private_aura_sound_registrations.next_id, Some(2));
}

#[cfg(feature = "retail-12-0-7")]
fn vehicle_fixture(env: &WowLuaEnv) {
    env.state().borrow_mut().player.buffs = vec![AuraInfo {
        name: "Vehicle beacon".into(),
        spell_id: 99023,
        icon: 135841,
        duration: 30.0,
        expiration_time: 30.0,
        applications: 1,
        source_unit: "vehicle-beacon".into(),
        is_helpful: true,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: false,
        is_from_player_or_player_pet: false,
        dispel_type: None,
        aura_instance_id: 923,
    }];
}

#[cfg(feature = "retail-12-0-7")]
fn check_vehicle(env: &WowLuaEnv, expected: bool) {
    env.exec(&format!(
        r#"
        local bySlot = C_UnitAuras.GetAuraDataBySlot('player', 923)
        local byIndex = C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL')
        local byID = C_UnitAuras.GetAuraDataByAuraInstanceID('player', 923)
        assert(bySlot and byIndex and byID, 'all three lookup results required')
        for _, aura in ipairs({{bySlot, byIndex, byID}}) do
            assert(aura.spellId == 99023)
            assert(aura.isFromPlayerOrPlayerPet == {expected})
            assert(not issecretvalue(aura.isFromPlayerOrPlayerPet))
        end
    "#
    ))
    .unwrap();
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn b23_vehicle_classification_changes_live_without_rewriting_stored_aura() {
    let env = environment();
    vehicle_fixture(&env);
    assert!(
        env.state()
            .borrow()
            .player_controlled_vehicle_sources
            .is_empty()
    );
    check_vehicle(&env, false);
    env.state()
        .borrow_mut()
        .player_controlled_vehicle_sources
        .insert("vehicle-beacon".into());
    check_vehicle(&env, true);
    assert!(!env.state().borrow().player.buffs[0].is_from_player_or_player_pet);
    env.state()
        .borrow_mut()
        .player_controlled_vehicle_sources
        .clear();
    check_vehicle(&env, false);
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn b23_source_identity_controls_and_environments_are_independent() {
    let env = environment();
    let other = environment();
    vehicle_fixture(&env);
    vehicle_fixture(&other);
    env.state()
        .borrow_mut()
        .player_controlled_vehicle_sources
        .insert("unrelated-vehicle".into());
    check_vehicle(&env, false);
    env.state()
        .borrow_mut()
        .player_controlled_vehicle_sources
        .insert("vehicle-beacon".into());
    check_vehicle(&env, true);
    check_vehicle(&other, false);
    for source in ["player", "pet", "party1"] {
        let explicit_marker = source != "party1";
        {
            let mut sim = env.state().borrow_mut();
            sim.player.buffs[0].source_unit = source.into();
            sim.player.buffs[0].is_from_player_or_player_pet = explicit_marker;
        }
        check_vehicle(&env, explicit_marker);
    }
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn b23_vehicle_flag_is_public_to_addons_and_slot_unit_remains_never_secret() {
    let env = environment();
    vehicle_fixture(&env);
    install_secrets(&env);
    env.state()
        .borrow_mut()
        .player_controlled_vehicle_sources
        .insert("vehicle-beacon".into());
    env.exec("assert(not pcall(C_UnitAuras.GetAuraDataBySlot, SecretUnit, 923))")
        .unwrap();
    addon(
        &env,
        r#"
        local aura = C_UnitAuras.GetAuraDataBySlot('player', 923)
        assert(aura.isFromPlayerOrPlayerPet == true)
        assert(not issecretvalue(aura.isFromPlayerOrPlayerPet))
        assert(not pcall(C_UnitAuras.GetAuraDataBySlot, SecretUnit, 923))
    "#,
    );
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn b28_empty_catalog_defaults_and_live_shipped_id_membership() {
    let env = environment();
    assert!(env.state().borrow().known_shipped_asset_ids.is_empty());
    assert!(env.state().borrow().known_loose_asset_paths.is_empty());
    env.exec(
        r#"
        CheckAsset(C_UIFileAsset.IsKnownFile, 136243, false)
        CheckAsset(C_UIFileAsset.IsLooseFile, 136243, false)
        CheckAsset(C_UIFileAsset.IsKnownFile, 'Interface/Icons/Trade_Engineering.blp', false)
        assert(C_UIFileAsset.GetFileID(136243) == 136243)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .known_shipped_asset_ids
        .insert(136243);
    env.exec(
        r#"
        CheckAsset(C_UIFileAsset.IsKnownFile, 136243, true)
        CheckAsset(C_UIFileAsset.IsKnownFile, 'iNtErFaCe\\Icons\\Trade_Engineering.blp', true)
        CheckAsset(C_UIFileAsset.IsLooseFile, 136243, false)
        CheckAsset(C_UIFileAsset.IsKnownFile, 98765, false)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().known_shipped_asset_ids.clear();
    env.exec("CheckAsset(C_UIFileAsset.IsKnownFile, 136243, false)")
        .unwrap();
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn b28_registered_absent_file_and_unregistered_present_file_are_not_io_queries() {
    let env = environment();
    // Uses only the audit cache if an integrator executes this fixture on this host.
    let root = tempfile::Builder::new()
        .prefix("b28-")
        .tempdir_in("/home/osso-test/.cache/wow-ui-sim-audit")
        .unwrap();
    let present = root.path().join("present.ogg");
    std::fs::write(&present, b"audio fixture").unwrap();
    let path = present
        .to_str()
        .unwrap()
        .replace('\\', "/")
        .to_ascii_lowercase();
    env.exec(&format!("PresentPath = {path:?}")).unwrap();
    env.exec("CheckAsset(C_UIFileAsset.IsKnownFile, PresentPath, false)")
        .unwrap();
    env.state()
        .borrow_mut()
        .known_loose_asset_paths
        .insert(path);
    env.exec("CheckAsset(C_UIFileAsset.IsLooseFile, PresentPath, true)")
        .unwrap();
    std::fs::remove_file(&present).unwrap();
    env.exec("CheckAsset(C_UIFileAsset.IsKnownFile, PresentPath, true); CheckAsset(C_UIFileAsset.IsLooseFile, PresentPath, true)").unwrap();
    env.state()
        .borrow_mut()
        .known_loose_asset_paths
        .insert("interface/addons/slice/missing.ogg".into());
    env.exec(
        r#"
        CheckAsset(C_UIFileAsset.IsKnownFile, 'Interface\\AddOns\\Slice\\Missing.ogg', true)
        CheckAsset(C_UIFileAsset.IsLooseFile, 'Interface/AddOns/Slice/Missing.ogg', true)
        CheckAsset(C_UIFileAsset.IsKnownFile, 'Interface/AddOns/Slice/Missing', false)
        assert(C_UIFileAsset.GetFileID('Interface/AddOns/Slice/Missing.ogg') == nil)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn b28_catalog_isolation_and_host_revocation_are_live() {
    let env = environment();
    let other = environment();
    env.state()
        .borrow_mut()
        .known_loose_asset_paths
        .insert("interface/addons/slice/missing.ogg".into());
    env.exec("CheckAsset(C_UIFileAsset.IsLooseFile, 'Interface/AddOns/Slice/Missing.ogg', true)")
        .unwrap();
    other
        .exec("CheckAsset(C_UIFileAsset.IsLooseFile, 'Interface/AddOns/Slice/Missing.ogg', false)")
        .unwrap();
    env.state().borrow_mut().known_loose_asset_paths.clear();
    env.exec("CheckAsset(C_UIFileAsset.IsKnownFile, 'Interface/AddOns/Slice/Missing.ogg', false)")
        .unwrap();
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn b28_secure_secrets_preserve_values_arity_and_public_results() {
    let env = environment();
    install_secrets(&env);
    env.state()
        .borrow_mut()
        .known_shipped_asset_ids
        .insert(136243);
    env.state()
        .borrow_mut()
        .known_loose_asset_paths
        .insert("interface/addons/slice/missing.ogg".into());
    env.exec(
        r#"
        CheckAsset(C_UIFileAsset.IsKnownFile, SecretID, true, SecretExtra)
        CheckAsset(C_UIFileAsset.IsLooseFile, SecretID, false, SecretExtra)
        CheckAsset(C_UIFileAsset.IsKnownFile, SecretPath, true)
        CheckAsset(C_UIFileAsset.IsLooseFile, SecretPath, true)
        assert(issecretvalue(SecretID) and issecretvalue(SecretPath))
        assert(secretunwrap(SecretID) == 136243 and debug.getstacktaint() == nil)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn b28_tainted_secret_extras_authenticate_before_any_public_type_validation() {
    let env = environment();
    install_secrets(&env);
    addon(
        &env,
        r#"
        for _, query in ipairs({C_UIFileAsset.IsKnownFile, C_UIFileAsset.IsLooseFile}) do
            Reject(query, 'untainted caller', SecretID)
            Reject(query, 'untainted caller', SecretPath)
            Reject(query, 'untainted caller', false, 0, SecretExtra)
            Reject(query, 'untainted caller', {}, SecretExtra)
            CheckAsset(query, 136243, false, false, {})
        end
        assert(issecretvalue(SecretExtra))
    "#,
    );
    assert!(env.state().borrow().known_shipped_asset_ids.is_empty());
    assert!(env.state().borrow().known_loose_asset_paths.is_empty());
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn b28_inferred_type_and_numeric_boundary_policy_is_not_numeric_coercion() {
    let env = environment();
    env.exec(
        r#"
        for _, query in ipairs({C_UIFileAsset.IsKnownFile, C_UIFileAsset.IsLooseFile}) do
            for _, asset in ipairs({0, -1, 1.25, 4294967296, math.huge, -math.huge, 0/0}) do
                CheckAsset(query, asset, false)
            end
            Reject(query, 'asset must', nil)
            Reject(query, 'asset must', false)
            Reject(query, 'asset must', {})
            CheckAsset(query, '136243', false)
            CheckAsset(query, '', false)
        end
    "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn b24_cached_legacy_forwarding_keeps_out_of_combat_permission_and_removal() {
    let env = environment();
    let addons = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(
        env!("CARGO_MANIFEST_DIR"),
    ));
    let source =
        std::fs::read_to_string(addons.join("Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua"))
            .expect("actual cached deprecated chunk");
    env.exec("SetCVar('loadDeprecationFallbacks', '1')")
        .unwrap();
    env.exec(&source).unwrap();
    env.state().borrow_mut().mythic_plus.is_active = true;
    env.state().borrow_mut().player.in_combat = false;
    addon(&env, "ForwardedID = SoundProbe()");
    assert_eq!(env.eval::<f64>("return ForwardedID").unwrap(), 1.0);
    assert_eq!(
        env.state()
            .borrow()
            .private_aura_sound_registrations
            .registrations[&1]
            .trigger,
        0
    );
    env.state().borrow_mut().player.in_combat = true;
    addon(
        &env,
        "Reject(C_UnitAuras.AddPrivateAuraAppliedSound, 'insecure registration denied', Sound)",
    );
    addon(
        &env,
        "assert(select('#', C_UnitAuras.RemovePrivateAuraAppliedSound(ForwardedID)) == 0)",
    );
    let sim = env.state().borrow();
    assert!(
        sim.private_aura_sound_registrations
            .registrations
            .is_empty()
    );
    assert!(sim.private_aura_sound_registrations.live_ids.is_empty());
    assert_eq!(sim.private_aura_sound_registrations.next_id, Some(2));
}
