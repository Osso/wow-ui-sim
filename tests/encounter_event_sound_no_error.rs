#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("encounter sound environment");
    env.exec(
        r#"
        assert(C_EncounterEvents.HasEventInfo(1))
        assert(C_EncounterEvents.HasEventInfo(2))
        -- Cached EncounterEventSoundTrigger values: text warning=0, finished=1.
        C_EncounterEvents.SetEventSound(1, 0, {
            file = 123456, channel = 'Master', volume = 0.25,
        })
        C_EncounterEvents.SetEventSound(1, 1, {
            file = 654321, channel = 'SFX', volume = 0.75,
        })
        "#,
    )
    .expect("seed existing event/trigger sound state through the public setter");
    env
}

#[test]
fn configured_event_sound_returns_stored_fields_and_one_result() {
    let env = fixture_env();
    env.exec(
        r#"
        assert(select('#', C_EncounterEvents.GetEventSound(1, 0)) == 1)
        local warning = C_EncounterEvents.GetEventSound(1, 0)
        assert(type(warning) == 'table')
        assert(warning.file == 123456)
        assert(warning.channel == 'Master')
        assert(warning.volume == 0.25)
        assert(select('#', C_EncounterEvents.GetEventSound(1, 1)) == 1)
        local finished = C_EncounterEvents.GetEventSound(1, 1)
        assert(finished.file == 654321)
        assert(finished.channel == 'SFX')
        assert(finished.volume == 0.75)
        "#,
    )
    .expect("configured lookups return event/trigger-specific sound records without error");
}

#[test]
fn missing_sound_overrides_return_no_values_without_error() {
    let env = fixture_env();
    env.exec(
        r#"
        assert(not C_EncounterEvents.HasEventInfo(900001))
        -- INFERRED: missing overrides return zero values, not a single nil.
        assert(select('#', C_EncounterEvents.GetEventSound(2, 0)) == 0)
        assert(select('#', C_EncounterEvents.GetEventSound(900001, 0)) == 0)
        assert(select('#', C_EncounterEvents.GetEventSound(1, 2)) == 0)
        -- INFERRED: nil clears only the selected override.
        C_EncounterEvents.SetEventSound(1, 0, nil)
        assert(select('#', C_EncounterEvents.GetEventSound(1, 0)) == 0)
        local retained = C_EncounterEvents.GetEventSound(1, 1)
        assert(retained.file == 654321 and retained.volume == 0.75)
        "#,
    )
    .expect("known, unknown, unconfigured and cleared sounds do not error");
}

#[test]
fn returned_sound_table_mutation_does_not_change_stored_override() {
    let env = fixture_env();
    env.exec(
        r#"
        -- INFERRED native copy policy; assert existing model isolation.
        local returned = C_EncounterEvents.GetEventSound(1, 0)
        returned.file = 999999
        returned.channel = 'Music'
        returned.volume = 0
        local nextSound = C_EncounterEvents.GetEventSound(1, 0)
        assert(not rawequal(returned, nextSound))
        assert(nextSound.file == 123456)
        assert(nextSound.channel == 'Master')
        assert(nextSound.volume == 0.25)
        "#,
    )
    .expect("returned DTO is detached from the configured sound state");
}

#[test]
fn addon_sound_lookup_does_not_error_or_change_stack_taint() {
    let env = fixture_env();
    env.exec(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == 'EncounterSoundProbe')
            assert(select('#', C_EncounterEvents.GetEventSound(1, 0)) == 1)
            local sound = C_EncounterEvents.GetEventSound(1, 0)
            assert(sound.file == 123456)
            assert(sound.channel == 'Master' and sound.volume == 0.25)
            assert(debug.getstacktaint() == before)
            -- INFERRED missing-override arity, including addon callers.
            assert(select('#', C_EncounterEvents.GetEventSound(2, 0)) == 0)
            assert(select('#', C_EncounterEvents.GetEventSound(900001, 0)) == 0)
            assert(debug.getstacktaint() == before)
        end
        debug.setobjecttaint(probe, 'EncounterSoundProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        assert(C_EncounterEvents.GetEventSound(1, 0).file == 123456)
        "#,
    )
    .expect("public sound arguments stay callable from addon-tainted code");
}
