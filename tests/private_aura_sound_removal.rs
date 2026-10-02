//! Batch49 row403 inputs/fixtures only: INFERRED removal model, not native parity.
//! No producer is published here; compiled RED belongs to the parent.
#![cfg(feature = "retail-12-0-5")]

use std::collections::HashSet;

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string, wrap_secret};
use wow_ui_sim::c_api::c_unit_aura_classification::AuraSpellClassification;
use wow_ui_sim::c_api::private_aura_sounds::PrivateAuraSoundRegistrations;
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create sound removal environment");
    env.exec(
        r#"
        function RemoveSound(identifier)
            assert(type(C_UnitAuras.RemovePrivateAuraAppliedSound) == 'function',
                'real registered legacy remover required')
            assert(select('#', C_UnitAuras.RemovePrivateAuraAppliedSound(identifier)) == 0,
                'exactly zero Lua results')
        end
        function RejectSound(...)
            local remove = C_UnitAuras.RemovePrivateAuraAppliedSound
            assert(type(remove) == 'function', 'real registered legacy remover required')
            local before = debug.getstacktaint()
            local ok, err = pcall(remove, ...)
            assert(not ok and type(err) == 'string', 'reject invalid argument')
            assert(err:find('RemovePrivateAuraAppliedSound', 1, true), 'contextual API error')
            assert(not issecretvalue(err), 'public error')
            assert(debug.getstacktaint() == before, 'rejection preserves taint')
        end
    "#,
    )
    .expect("install assertions, never replace API");
    env
}

fn seeded_env() -> WowLuaEnv {
    let env = fixture_env();
    env.state()
        .borrow_mut()
        .private_aura_sound_registrations
        .live_ids
        .extend([101, 202]);
    env
}

fn assert_ids(env: &WowLuaEnv, expected: &[u32]) {
    assert_eq!(
        env.state()
            .borrow()
            .private_aura_sound_registrations
            .live_ids,
        expected.iter().copied().collect::<HashSet<_>>()
    );
}

fn install_host_secrets(env: &WowLuaEnv) {
    env.exec(
        "SoundWrongObject = CreateFrame('Frame'); assert(type(SoundWrongObject) == 'table'); assert(SoundWrongObject:GetObjectType() == 'Frame')",
    )
    .expect("actual FrameTable behavior, not invented userdata");
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("actual VM security helpers");
    for (name, number) in [("SoundSecretKnown", 101.0), ("SoundSecretUnknown", 303.0)] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root actual secret NUMBER");
    }
    for (name, text) in [
        ("SoundSecretStringHit", "101"),
        ("SoundSecretStringUnknown", "sound-secret-unknown"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root actual secret STRING");
    }
    let frame = lua.get_global_val("SoundWrongObject");
    assert!(!frame.is_nil(), "actual frame globally rooted");
    lua.state_mut().push(frame);
    let wrapper = wrap_secret(lua.state_mut(), frame).expect("wrap actual wrong object");
    lua.state_mut().push(wrapper);
    let inserted = lua.set_global_val("SoundSecretObject", wrapper);
    lua.state_mut().pop();
    lua.state_mut().pop();
    inserted.expect("root actual frame wrapper");
}

fn reject_host_secrets(env: &WowLuaEnv, tainted: bool) {
    install_host_secrets(env);
    // A seeded string alias must not make a secret STRING a public numeric ID.
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("101".into(), 101);
    env.exec(
        r#"
        SoundSecretRoots = {SoundSecretKnown, SoundSecretUnknown, SoundSecretStringHit,
            SoundSecretStringUnknown, SoundSecretObject}
        collectgarbage('collect')
        function ProbeSoundSecret(value)
            local before = debug.getstacktaint()
            assert(issecretvalue(value), 'real VM secret before rejection')
            local ok, err = pcall(function() RejectSound(value) end)
            assert(ok, err)
            assert(issecretvalue(value), 'no declassification')
            assert(debug.getstacktaint() == before, 'secret rejection preserves caller taint')
        end
    "#,
    )
    .unwrap();
    for index in 1..=5 {
        let call = format!("ProbeSoundSecret(SoundSecretRoots[{index}])");
        let code = if tainted {
            format!(
                "local function addon() assert(debug.getstacktaint() == 'SoundSecretFixture'); {call}; assert(debug.getstacktaint() == 'SoundSecretFixture') end; debug.setobjecttaint(addon, 'SoundSecretFixture'); addon(); assert(issecure())"
            )
        } else {
            format!("assert(issecure()); {call}; assert(issecure())")
        };
        env.exec(&code)
            .expect("conservative rejection in actual caller context");
        assert_ids(env, &[101, 202]);
        env.exec(
            r#"
            assert(rawequal(SoundSecretRoots[1], SoundSecretKnown))
            assert(rawequal(SoundSecretRoots[2], SoundSecretUnknown))
            assert(rawequal(SoundSecretRoots[3], SoundSecretStringHit))
            assert(rawequal(SoundSecretRoots[4], SoundSecretStringUnknown))
            assert(rawequal(SoundSecretRoots[5], SoundSecretObject))
            assert(SoundWrongObject:GetObjectType() == 'Frame')
        "#,
        )
        .expect("rooted identity and actual frame survive rejection and GC");
    }
    let recovery = if tainted {
        "local function addon() assert(debug.getstacktaint() == 'SoundSecretFixture'); RemoveSound(101); assert(debug.getstacktaint() == 'SoundSecretFixture') end; debug.setobjecttaint(addon, 'SoundSecretFixture'); addon(); assert(issecure())"
    } else {
        "assert(issecure()); RemoveSound(101); assert(issecure())"
    };
    env.exec(recovery)
        .expect("public recovery without replacing API");
    assert_ids(env, &[202]);
    assert_eq!(env.state().borrow().spell_id_aliases.get("101"), Some(&101));
}

#[test]
fn empty_default_has_no_acquired_sound_ids() {
    assert!(PrivateAuraSoundRegistrations::default().live_ids.is_empty());
    let env = fixture_env();
    assert_ids(&env, &[]);
    env.exec("RemoveSound(101)")
        .expect("empty model valid removal no-op");
    assert_ids(&env, &[]);
}

#[test]
fn seeded_removal_is_immediate_and_returns_zero_values() {
    let env = seeded_env();
    env.exec("RemoveSound(101)").unwrap();
    assert_ids(&env, &[202]);
}

#[test]
fn unknown_valid_id_preserves_all_live_ids() {
    let env = seeded_env();
    env.exec("RemoveSound(303)").unwrap();
    assert_ids(&env, &[101, 202]);
}

#[test]
fn repeated_removal_is_a_valid_zero_result_noop() {
    let env = seeded_env();
    env.exec("RemoveSound(101)").unwrap();
    assert_ids(&env, &[202]);
    env.exec("RemoveSound(101)").unwrap();
    assert_ids(&env, &[202]);
}

#[test]
fn public_numeric_u32_endpoints_remove_only_their_ids() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .private_aura_sound_registrations
        .live_ids
        .extend([0, u32::MAX]);
    env.exec("RemoveSound(0)").unwrap();
    assert_ids(&env, &[101, 202, u32::MAX]);
    env.exec("RemoveSound(4294967295)").unwrap();
    assert_ids(&env, &[101, 202]);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn direct_modern_and_legacy_removers_share_real_state() {
    let env = seeded_env();
    env.exec("assert(select('#', C_UnitAuras.RemoveAuraSound(101)) == 0)")
        .unwrap();
    assert_ids(&env, &[202]);
    env.exec("RemoveSound(101); RemoveSound(202)").unwrap();
    assert_ids(&env, &[]);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn actual_cached_deprecated_file_keeps_legacy_removal_durable_after_bootstrap() {
    let env = seeded_env();
    // These are real bootstrap prerequisites. No synthetic namespace or vendor alias.
    env.exec(
        r#"
        assert(GetCVarBool('loadDeprecationFallbacks'), 'default CVar must be 1')
        assert(type(C_UnitAuras) == 'table')
        assert(type(Enum.UnitAuraSoundTrigger) == 'table')
        assert(type(Enum.UnitAuraSoundTrigger.Added) == 'number')
        assert(type(Enum.CustomAuraButtonDispelTypeTextureStyle.BorderWithIcon) == 'number')
        assert(type(Enum.CustomAuraButtonDispelTypeTextureStyle.PreserveAsset) == 'number')
        assert(type(C_SpecializationInfo) == 'table')
    "#,
    )
    .expect("actual vendor file prerequisites supplied by existing bootstrap");
    let path = std::path::PathBuf::from(std::env::var_os("HOME").expect("HOME for runtime cache"))
        .join(".cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua");
    let source =
        std::fs::read_to_string(&path).expect("required actual cached Blizzard file, no fallback");
    env.exec_named(&source, &format!("@{}", path.display()))
        .expect("execute entire unmodified cached vendor chunk after bootstrap");
    env.exec(
        r#"
        local function addon()
            assert(debug.getstacktaint() == 'SoundVendorFixture')
            RemoveSound(101)
            assert(debug.getstacktaint() == 'SoundVendorFixture')
        end
        debug.setobjecttaint(addon, 'SoundVendorFixture')
        addon(); assert(issecure())
    "#,
    )
    .expect("post-vendor legacy removal still reaches actual live ID set");
    assert_ids(&env, &[202]);
    env.exec("RemoveSound(101); RemoveSound(202)").unwrap();
    assert_ids(&env, &[]);
}

#[test]
fn host_replacement_changes_the_current_removal_set() {
    let env = seeded_env();
    env.exec("RemoveSound(101)").unwrap();
    assert_ids(&env, &[202]);
    env.state().borrow_mut().private_aura_sound_registrations = PrivateAuraSoundRegistrations {
        live_ids: HashSet::from([101, 303]),
    };
    env.exec("RemoveSound(202)").unwrap();
    assert_ids(&env, &[101, 303]);
    env.exec("RemoveSound(101)").unwrap();
    assert_ids(&env, &[303]);
}

#[test]
fn removal_is_environment_local_and_leaves_other_maps_unchanged() {
    let first = seeded_env();
    let second = seeded_env();
    {
        let mut state = first.state().borrow_mut();
        state.spell_id_aliases.insert("101".into(), 202);
        state
            .cooldown_aura_associations
            .cooldown_spell_ids
            .extend([(101, 901), (202, 702)]);
        state.aura_spell_classifications.spells.insert(
            101,
            AuraSpellClassification {
                is_big_defensive: true,
                is_private: false,
            },
        );
    }
    let aliases = first.state().borrow().spell_id_aliases.clone();
    let cooldowns = first
        .state()
        .borrow()
        .cooldown_aura_associations
        .cooldown_spell_ids
        .clone();
    first.exec("SoundCaller = {id = 101, marker = 17}; RemoveSound(SoundCaller.id); assert(SoundCaller.id == 101 and SoundCaller.marker == 17)").unwrap();
    assert_ids(&first, &[202]);
    assert_ids(&second, &[101, 202]);
    let state = first.state().borrow();
    assert_eq!(state.spell_id_aliases, aliases);
    assert_eq!(
        state.cooldown_aura_associations.cooldown_spell_ids,
        cooldowns
    );
    assert_eq!(state.aura_spell_classifications.spells.len(), 1);
    let flags = state.aura_spell_classifications.spells.get(&101).unwrap();
    assert!(flags.is_big_defensive);
    assert!(!flags.is_private);
    drop(state);
    second.exec("RemoveSound(202)").unwrap();
    assert_ids(&second, &[101]);
    assert_ids(&first, &[202]);
}

#[test]
fn strict_invalid_arguments_reject_without_mutation_then_allow_public_recovery() {
    let env = seeded_env();
    env.exec("SoundFrame = CreateFrame('Frame'); assert(type(SoundFrame) == 'table'); assert(SoundFrame:GetObjectType() == 'Frame')").unwrap();
    for arguments in [
        "",
        "nil",
        "'101'",
        "true",
        "false",
        "{}",
        "SoundFrame",
        "function() end",
        "coroutine.create(function() end)",
        "0/0",
        "math.huge",
        "-math.huge",
        "-1",
        "101.5",
        "4294967296",
    ] {
        env.exec(&format!("RejectSound({arguments})"))
            .expect("required actual public finite integral u32 NUMBER");
        assert_ids(&env, &[101, 202]);
    }
    env.exec("RemoveSound(101); assert(SoundFrame:GetObjectType() == 'Frame')")
        .unwrap();
    assert_ids(&env, &[202]);
}

#[test]
fn public_tainted_closure_removes_live_id_without_caller_or_combat_gate() {
    let env = seeded_env();
    env.state().borrow_mut().player.in_combat = true;
    env.exec(
        r#"
        local function addon()
            assert(debug.getstacktaint() == 'SoundPublicFixture')
            assert(InCombatLockdown(), 'actual modeled combat state')
            RemoveSound(101)
            assert(debug.getstacktaint() == 'SoundPublicFixture')
        end
        debug.setobjecttaint(addon, 'SoundPublicFixture')
        assert(issecure()); addon(); assert(issecure())
    "#,
    )
    .expect("literal removal of restriction, not blanket secret permission");
    assert_ids(&env, &[202]);
    assert!(env.state().borrow().player.in_combat);
}

#[test]
fn real_rooted_vm_secrets_reject_secure_without_mutation_or_declassification() {
    reject_host_secrets(&seeded_env(), false);
}

#[test]
fn real_rooted_vm_secrets_reject_tainted_without_mutation_or_declassification() {
    reject_host_secrets(&seeded_env(), true);
}
