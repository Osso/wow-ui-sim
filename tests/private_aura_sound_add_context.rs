#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_secret};
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("sound Add environment");
    env.exec(
        r#"
        SoundInfo = {unitToken = 'player', spellID = 12345, soundFileID = 98765,
            outputChannel = 'Master'}
        function AddSound()
            assert(type(C_UnitAuras.AddPrivateAuraAppliedSound) == 'function',
                'real legacy Add producer required')
            local id = C_UnitAuras.AddPrivateAuraAppliedSound(SoundInfo)
            assert(type(id) == 'number', 'allocated public sound ID required')
            assert(not issecretvalue(id), 'registration handle remains public')
            return id
        end
        function DenySound()
            assert(type(C_UnitAuras.AddPrivateAuraAppliedSound) == 'function',
                'real legacy Add producer required')
            local before = debug.getstacktaint()
            local ok, err = pcall(C_UnitAuras.AddPrivateAuraAppliedSound, SoundInfo)
            assert(not ok and type(err) == 'string', 'context denies insecure Add')
            assert(err:find('AddPrivateAuraAppliedSound', 1, true), 'contextual error')
            assert(not issecretvalue(err), 'public error')
            assert(debug.getstacktaint() == before, 'denial preserves taint')
        end
        "#,
    )
    .unwrap();
    env
}

fn addon(env: &WowLuaEnv, body: &str) {
    env.exec(&format!(
        "local function probe() assert(debug.getstacktaint() == 'SoundAddon'); {body}; assert(debug.getstacktaint() == 'SoundAddon') end; debug.setobjecttaint(probe, 'SoundAddon'); probe(); assert(issecure())"
    )).expect("actual stamped addon closure and outer secure recovery");
}

fn set_context(env: &WowLuaEnv, encounter: bool, mythic: bool, pvp: bool) {
    let mut state = env.state().borrow_mut();
    state.world.encounter_in_progress = encounter;
    state.mythic_plus.is_active = mythic;
    state.pvp_match_active = pvp;
}

#[test]
fn ordinary_tainted_combat_add_copies_real_payload_and_removes_it() {
    let env = fixture();
    env.state().borrow_mut().player.in_combat = true;
    env.state()
        .borrow_mut()
        .private_aura_sound_registrations
        .next_id = Some(41);
    addon(&env, "SoundID = AddSound()");
    let id: f64 = env.eval("return SoundID").unwrap();
    assert_eq!(id, 41.0);
    {
        let state = env.state().borrow();
        let sounds = &state.private_aura_sound_registrations;
        assert!(sounds.live_ids.contains(&41));
        let stored = &sounds.registrations[&41];
        assert_eq!(stored.trigger, 0);
        assert_eq!(stored.unit_token, "player");
        assert_eq!(stored.spell_id, 12345);
        assert_eq!(stored.sound_file_id, Some(98765));
        assert_eq!(stored.sound_file_name, None);
        assert_eq!(stored.output_channel.as_deref(), Some("Master"));
    }
    env.exec("SoundInfo.spellID = 54321; SoundInfo.soundFileID = nil; SoundInfo.soundFileName = 'Interface\\\\AddOns\\\\Probe\\\\second.ogg'; SoundInfo.outputChannel = nil").unwrap();
    addon(&env, "SecondSoundID = AddSound()");
    let state = env.state().borrow();
    assert_eq!(
        state.private_aura_sound_registrations.registrations[&41].spell_id,
        12345
    );
    let second = &state.private_aura_sound_registrations.registrations[&42];
    assert_eq!(second.spell_id, 54321);
    assert_eq!(second.sound_file_id, None);
    assert_eq!(
        second.sound_file_name.as_deref(),
        Some("Interface\\AddOns\\Probe\\second.ogg")
    );
    assert_eq!(second.output_channel, None);
    drop(state);
    set_context(&env, true, true, true);
    addon(
        &env,
        "assert(select('#', C_UnitAuras.RemovePrivateAuraAppliedSound(SoundID)) == 0)",
    );
    let state = env.state().borrow();
    assert!(
        !state
            .private_aura_sound_registrations
            .live_ids
            .contains(&41)
    );
    assert!(
        !state
            .private_aura_sound_registrations
            .registrations
            .contains_key(&41)
    );
    assert!(
        state
            .private_aura_sound_registrations
            .registrations
            .contains_key(&42)
    );
}

#[test]
fn each_context_denies_addon_without_consuming_id_then_recovers() {
    for context in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let env = fixture();
        set_context(&env, context.0, context.1, context.2);
        // This existing denial test targets restricted M+, not the 12.0.7 exception.
        env.state().borrow_mut().player.in_combat = true;
        addon(&env, "DenySound()");
        {
            let state = env.state().borrow();
            let sounds = &state.private_aura_sound_registrations;
            assert!(sounds.live_ids.is_empty());
            assert!(sounds.registrations.is_empty());
            assert_eq!(sounds.next_id, Some(1));
        }
        set_context(&env, false, false, false);
        addon(&env, "RecoveredID = AddSound()");
        assert_eq!(env.eval::<f64>("return RecoveredID").unwrap(), 1.0);
    }
}

#[test]
fn secure_caller_registers_in_all_contexts_and_environments_are_isolated() {
    let env = fixture();
    let other = fixture();
    set_context(&env, true, true, true);
    assert_eq!(
        env.eval::<f64>("assert(issecure()); return AddSound()")
            .unwrap(),
        1.0
    );
    assert!(
        other
            .state()
            .borrow()
            .private_aura_sound_registrations
            .registrations
            .is_empty()
    );
    addon(&other, "OtherID = AddSound()");
    assert_eq!(other.eval::<f64>("return OtherID").unwrap(), 1.0);
    addon(&env, "DenySound()");
    assert_eq!(
        env.state()
            .borrow()
            .private_aura_sound_registrations
            .registrations
            .len(),
        1
    );
}

#[test]
fn allocator_avoids_host_ids_and_exhaustion_returns_one_nil() {
    let env = fixture();
    env.state()
        .borrow_mut()
        .private_aura_sound_registrations
        .live_ids
        .insert(1);
    assert_eq!(env.eval::<f64>("return AddSound()").unwrap(), 2.0);
    env.state()
        .borrow_mut()
        .private_aura_sound_registrations
        .next_id = Some(u32::MAX);
    assert_eq!(
        env.eval::<f64>("return AddSound()").unwrap(),
        f64::from(u32::MAX)
    );
    env.exec("assert(select('#', C_UnitAuras.AddPrivateAuraAppliedSound(SoundInfo)) == 1); assert(C_UnitAuras.AddPrivateAuraAppliedSound(SoundInfo) == nil)").unwrap();
    let state = env.state().borrow();
    assert_eq!(
        state.private_aura_sound_registrations.registrations.len(),
        2
    );
    assert_eq!(state.private_aura_sound_registrations.next_id, None);
}

#[test]
fn malformed_inputs_leave_host_registration_and_allocator_unchanged() {
    let env = fixture();
    env.exec(
        r#"
        assert(type(C_UnitAuras.AddPrivateAuraAppliedSound) == 'function')
        local inputs = {false, 123, '123', {},
            {unitToken='player', spellID=1.5, soundFileID=9},
            {unitToken='player', spellID=1, soundFileID=math.huge},
            {unitToken='player', spellID=1, soundFileID=9, soundFileName='both'},
            {unitToken='player', spellID=1},
            {unitToken='player', spellID=1, soundFileName=''},
            {unitToken='player', spellID=1, soundFileID=9, outputChannel=false}}
        for _, input in ipairs(inputs) do
            local ok, err = pcall(C_UnitAuras.AddPrivateAuraAppliedSound, input)
            assert(not ok and type(err) == 'string' and not issecretvalue(err))
            assert(err:find('AddPrivateAuraAppliedSound', 1, true))
        end
        assert(not pcall(C_UnitAuras.AddPrivateAuraAppliedSound))
    "#,
    )
    .unwrap();
    assert!(
        env.state()
            .borrow()
            .private_aura_sound_registrations
            .registrations
            .is_empty()
    );
    assert_eq!(
        env.state()
            .borrow()
            .private_aura_sound_registrations
            .next_id,
        Some(1)
    );
    assert_eq!(env.eval::<f64>("return AddSound()").unwrap(), 1.0);
}

fn install_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    let original = lua.get_global_val("SoundInfo");
    lua.state_mut().push(original);
    let wrapped = wrap_secret(lua.state_mut(), original).unwrap();
    lua.state_mut().push(wrapped);
    lua.set_global_val("SecretSoundInfo", wrapped).unwrap();
    lua.state_mut().pop();
    lua.state_mut().pop();
    let number = wrap_host_secret_number(lua.state_mut(), 98765.0);
    lua.state_mut().push(number);
    lua.set_global_val("SecretSoundFileID", number).unwrap();
    lua.state_mut().pop();
}

#[test]
fn real_secret_table_and_field_authenticate_securely_but_deny_addon() {
    let env = fixture();
    install_secrets(&env);
    env.exec(
        r#"
        collectgarbage('collect')
        assert(issecretvalue(SecretSoundInfo))
        assert(type(C_UnitAuras.AddPrivateAuraAppliedSound) == 'function')
        SecretTableID = C_UnitAuras.AddPrivateAuraAppliedSound(SecretSoundInfo)
        SoundInfo.soundFileID = SecretSoundFileID
        SecretFieldID = C_UnitAuras.AddPrivateAuraAppliedSound(SoundInfo)
        assert(issecretvalue(SecretSoundInfo) and issecretvalue(SecretSoundFileID))
    "#,
    )
    .unwrap();
    assert_eq!(env.eval::<f64>("return SecretTableID").unwrap(), 1.0);
    assert_eq!(env.eval::<f64>("return SecretFieldID").unwrap(), 2.0);
    addon(
        &env,
        r#"
        for _, info in ipairs({SecretSoundInfo, SoundInfo}) do
            local ok, err = pcall(C_UnitAuras.AddPrivateAuraAppliedSound, info)
            assert(not ok and type(err) == 'string' and not issecretvalue(err))
        end
        assert(issecretvalue(SecretSoundInfo) and issecretvalue(SecretSoundFileID))
    "#,
    );
    assert_eq!(
        env.state()
            .borrow()
            .private_aura_sound_registrations
            .registrations
            .len(),
        2
    );
    assert_eq!(
        env.state()
            .borrow()
            .private_aura_sound_registrations
            .next_id,
        Some(3)
    );
    env.exec("SoundInfo.unitToken = false").unwrap();
    addon(
        &env,
        r#"
        local ok, err = pcall(C_UnitAuras.AddPrivateAuraAppliedSound, SoundInfo)
        assert(not ok and err:find('secret', 1, true),
            'authenticate later secret field before malformed early public field')
    "#,
    );
    env.exec("SoundInfo.unitToken = 'player'; SoundInfo.soundFileID = 98765")
        .unwrap();
    addon(&env, "RecoveryID = AddSound()");
    assert_eq!(env.eval::<f64>("return RecoveryID").unwrap(), 3.0);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn modern_authenticates_both_arguments_before_trigger_validation() {
    let env = fixture();
    install_secrets(&env);
    addon(
        &env,
        r#"
        assert(type(C_UnitAuras.AddAuraSound) == 'function', 'real modern Add required')
        local ok, err = pcall(C_UnitAuras.AddAuraSound, false, SecretSoundInfo)
        assert(not ok and err:find('secret', 1, true),
            'later secret argument authenticates before malformed public trigger')
    "#,
    );
    assert!(
        env.state()
            .borrow()
            .private_aura_sound_registrations
            .registrations
            .is_empty()
    );
    env.exec("ModernID = C_UnitAuras.AddAuraSound(2, SoundInfo)")
        .unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .private_aura_sound_registrations
            .registrations[&1]
            .trigger,
        2
    );
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn actual_full_deprecated_chunk_preserves_add_context_and_removal() {
    let env = fixture();
    let path = std::path::PathBuf::from(std::env::var_os("HOME").expect("HOME"))
        .join(".cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua");
    let source = std::fs::read_to_string(&path).expect("actual complete cached deprecated chunk");
    env.exec("assert(GetCVarBool('loadDeprecationFallbacks'))")
        .unwrap();
    env.exec_named(&source, &path.to_string_lossy()).unwrap();
    addon(&env, "AliasID = AddSound()");
    assert_eq!(env.eval::<f64>("return AliasID").unwrap(), 1.0);
    assert_eq!(
        env.state()
            .borrow()
            .private_aura_sound_registrations
            .registrations[&1]
            .trigger,
        0
    );
    set_context(&env, true, false, false);
    addon(
        &env,
        "DenySound(); assert(select('#', C_UnitAuras.RemovePrivateAuraAppliedSound(AliasID)) == 0)",
    );
    assert!(
        env.state()
            .borrow()
            .private_aura_sound_registrations
            .registrations
            .is_empty()
    );
}
