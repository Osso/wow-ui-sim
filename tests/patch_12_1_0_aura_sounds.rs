//! 12.1.0 aura sounds: AddAuraSound triggers (Added, ApplicationsIncreased,
//! Removed) play when the player's auras change, on any aura, and stop after
//! RemoveAuraSound. Playback is observed through the native playback log.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::aura;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn set_auras_and_tick(env: &WowLuaEnv, auras: Vec<AuraInfo>) {
    env.state().borrow_mut().player.buffs = auras;
    env.fire_unit_aura_full_update("player").unwrap();
    env.fire_on_update(0.016).unwrap();
}

/// Played sounds since the last call as `soundFile:trigger:spell:instance`.
fn take_playbacks(env: &WowLuaEnv) -> Vec<String> {
    let mut state = env.state().borrow_mut();
    std::mem::take(&mut state.private_aura_sound_registrations.playbacks)
        .into_iter()
        .map(|played| {
            let file = played
                .sound_file_name
                .clone()
                .or_else(|| played.sound_file_id.map(|id| id.to_string()))
                .unwrap_or_default();
            format!(
                "{file}:{}:{}:{}",
                played.trigger, played.spell_id, played.aura_instance_id
            )
        })
        .collect()
}

fn stacked(id: i32, applications: i32) -> AuraInfo {
    AuraInfo {
        applications,
        ..aura(id)
    }
}

#[test]
fn aura_sound_triggers_play_on_add_application_and_removal() {
    let env = WowLuaEnv::new().unwrap();
    set_auras_and_tick(&env, vec![stacked(500, 1)]);
    env.exec(
        r#"
        local t = Enum.UnitAuraSoundTrigger
        local function sound(spellID, file)
            return {unitToken = 'player', spellID = spellID, soundFileName = file, outputChannel = 'Master'}
        end
        SoundAdded = C_UnitAuras.AddAuraSound(t.Added, sound(501, 'added.ogg'))
        SoundStack = C_UnitAuras.AddAuraSound(t.ApplicationsIncreased, sound(500, 'stack.ogg'))
        SoundRemoved = C_UnitAuras.AddAuraSound(t.Removed, sound(500, 'removed.ogg'))
        SoundById = C_UnitAuras.AddAuraSound(t.Added, {unitToken = 'player', spellID = 501, soundFileID = 567})
        SoundOtherUnit = C_UnitAuras.AddAuraSound(t.Added, {unitToken = 'target', spellID = 501, soundFileName = 'target.ogg'})
        SoundLegacy = C_UnitAuras.AddPrivateAuraAppliedSound(sound(502, 'legacy.ogg'))
        "#,
    )
    .unwrap();
    env.fire_on_update(0.016).unwrap();
    assert!(
        take_playbacks(&env).is_empty(),
        "auras present at registration are a silent baseline"
    );

    set_auras_and_tick(&env, vec![stacked(500, 1), aura(501)]);
    assert_eq!(take_playbacks(&env), ["added.ogg:0:501:501", "567:0:501:501"]);

    set_auras_and_tick(&env, vec![stacked(500, 3), aura(501)]);
    assert_eq!(take_playbacks(&env), ["stack.ogg:1:500:500"]);
    set_auras_and_tick(&env, vec![stacked(500, 2), aura(501)]);
    assert!(
        take_playbacks(&env).is_empty(),
        "losing applications is not ApplicationsIncreased"
    );

    set_auras_and_tick(&env, vec![aura(501), aura(502)]);
    assert_eq!(
        take_playbacks(&env),
        ["removed.ogg:2:500:500", "legacy.ogg:0:502:502"],
        "removal fires Removed; the legacy API is an Added sound for a non-private aura"
    );

    env.exec("C_UnitAuras.RemoveAuraSound(SoundAdded); C_UnitAuras.RemoveAuraSound(SoundById)")
        .unwrap();
    set_auras_and_tick(&env, vec![aura(502)]);
    set_auras_and_tick(&env, vec![aura(501), aura(502)]);
    assert!(
        take_playbacks(&env).is_empty(),
        "removed registrations no longer play"
    );
}
