//! Integration tests for the vehicle / possess / taxi globals registered in
//! `src/lua_api/globals/real/vehicle_possession.rs` plus the channel branch of
//! `UnitChannelInfo` in `globals/utility_system_spell/spell_api.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::CastingState;

#[test]
fn unit_on_taxi_returns_false_for_non_player() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player.on_taxi = true;

    let on_taxi: bool = env.eval("return UnitOnTaxi('target')").unwrap();

    assert!(!on_taxi);
}

#[test]
fn unit_has_vehicle_ui_reads_player_flag() {
    let env = WowLuaEnv::new().expect("env");
    let before: bool = env.eval("return UnitHasVehicleUI('player')").unwrap();
    assert!(!before);

    env.state().borrow_mut().player.has_vehicle_ui = true;
    let after: bool = env.eval("return UnitHasVehicleUI('player')").unwrap();
    assert!(after);
}

#[test]
fn unit_has_vehicle_ui_returns_false_for_non_player() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player.has_vehicle_ui = true;
    let target: bool = env.eval("return UnitHasVehicleUI('target')").unwrap();
    assert!(!target);
}

#[test]
fn unit_in_vehicle_reads_player_flag() {
    let env = WowLuaEnv::new().expect("env");
    let before: bool = env.eval("return UnitInVehicle('player')").unwrap();
    assert!(!before);

    env.state().borrow_mut().player.in_vehicle = true;
    let after: bool = env.eval("return UnitInVehicle('player')").unwrap();
    assert!(after);
}

#[test]
fn unit_in_vehicle_returns_false_for_non_player() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player.in_vehicle = true;
    let target: bool = env.eval("return UnitInVehicle('target')").unwrap();
    assert!(!target);
}

#[test]
fn unit_has_vehicle_player_frame_ui_reads_player_flag() {
    let env = WowLuaEnv::new().expect("env");
    let before: bool = env
        .eval("return UnitHasVehiclePlayerFrameUI('player')")
        .unwrap();
    assert!(!before);

    env.state().borrow_mut().player.has_vehicle_ui = true;
    let after: bool = env
        .eval("return UnitHasVehiclePlayerFrameUI('player')")
        .unwrap();
    assert!(after);
}

#[test]
fn unit_has_vehicle_player_frame_ui_returns_false_for_non_player() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player.has_vehicle_ui = true;
    let target: bool = env
        .eval("return UnitHasVehiclePlayerFrameUI('target')")
        .unwrap();
    assert!(!target);
}

#[test]
fn unit_controlling_vehicle_reads_player_flag() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player.controlling_vehicle = true;
    let val: bool = env.eval("return UnitControllingVehicle('player')").unwrap();
    assert!(val);
    let other: bool = env.eval("return UnitControllingVehicle('party1')").unwrap();
    assert!(!other);
}

#[test]
fn unit_on_taxi_reads_player_flag() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player.on_taxi = true;
    let val: bool = env.eval("return UnitOnTaxi('player')").unwrap();
    assert!(val);
}

#[test]
fn can_exit_vehicle_when_in_vehicle() {
    let env = WowLuaEnv::new().expect("env");
    let before: bool = env.eval("return CanExitVehicle()").unwrap();
    assert!(!before);

    env.state().borrow_mut().player.has_vehicle_ui = true;
    let with_ui: bool = env.eval("return CanExitVehicle()").unwrap();
    assert!(with_ui);
}

#[test]
fn can_exit_vehicle_when_on_taxi() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player.on_taxi = true;
    let val: bool = env.eval("return CanExitVehicle()").unwrap();
    assert!(val);
}

#[test]
fn vehicle_exit_clears_flags_and_fires_event() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.player.in_vehicle = true;
        state.player.has_vehicle_ui = true;
        state.player.controlling_vehicle = true;
    }

    env.exec(
        "events_seen = {}\n\
         local f = CreateFrame('Frame')\n\
         f:RegisterEvent('UNIT_EXITED_VEHICLE')\n\
         f:SetScript('OnEvent', function(_, _, unit) table.insert(events_seen, unit) end)\n\
         VehicleExit()",
    )
    .expect("VehicleExit");

    let state = env.state().borrow();
    assert!(!state.player.in_vehicle);
    assert!(!state.player.has_vehicle_ui);
    assert!(!state.player.controlling_vehicle);
    drop(state);

    let event_count: i32 = env.eval("return #events_seen").unwrap();
    assert_eq!(event_count, 1, "UNIT_EXITED_VEHICLE should fire once");
    let event_unit: String = env.eval("return events_seen[1]").unwrap();
    assert_eq!(event_unit, "player");
}

#[test]
fn vehicle_exit_is_noop_outside_vehicle() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        "events_seen = {}\n\
         local f = CreateFrame('Frame')\n\
         f:RegisterEvent('UNIT_EXITED_VEHICLE')\n\
         f:SetScript('OnEvent', function(_, _, unit) table.insert(events_seen, unit) end)\n\
         VehicleExit()",
    )
    .expect("VehicleExit");

    let event_count: i32 = env.eval("return #events_seen").unwrap();
    assert_eq!(event_count, 0, "no event when not in vehicle");
}

#[test]
fn taxi_request_early_landing_sets_flag() {
    let env = WowLuaEnv::new().expect("env");
    assert!(!env.state().borrow().player.taxi_early_landing_requested);
    env.exec("TaxiRequestEarlyLanding()").expect("call");
    assert!(env.state().borrow().player.taxi_early_landing_requested);
}

#[test]
fn unit_vehicle_skin_defaults_to_empty_string() {
    let env = WowLuaEnv::new().expect("env");
    let skin: String = env.eval("return UnitVehicleSkin('player')").unwrap();
    assert_eq!(skin, "");
}

#[test]
fn unit_vehicle_skin_reads_player_field() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player.vehicle_skin = Some("MechagonShredder".into());
    let skin: String = env.eval("return UnitVehicleSkin('player')").unwrap();
    assert_eq!(skin, "MechagonShredder");
}

#[test]
fn unit_vehicle_skin_only_for_player() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player.vehicle_skin = Some("MechagonShredder".into());
    let target: String = env.eval("return UnitVehicleSkin('target')").unwrap();
    assert_eq!(target, "");
    let party: String = env.eval("return UnitVehicleSkin('party1')").unwrap();
    assert_eq!(party, "");
}

#[test]
fn unit_vehicle_skin_treats_action_bar_controller_branch() {
    // Mirrors `ActionBarController_UpdateAll`: skinned override bar shows when
    // the skin is non-empty; falls back to the default path on empty.
    let env = WowLuaEnv::new().expect("env");
    let unskinned: bool = env
        .eval(
            r#"
            local skin = UnitVehicleSkin('player')
            return skin and skin ~= ''
            "#,
        )
        .unwrap();
    assert!(
        !unskinned,
        "empty skin should not request the override skin"
    );

    env.state().borrow_mut().player.vehicle_skin = Some("Demolisher".into());
    let skinned: bool = env
        .eval(
            r#"
            local skin = UnitVehicleSkin('player')
            return skin and skin ~= ''
            "#,
        )
        .unwrap();
    assert!(skinned, "non-empty skin should request the override skin");
}

#[test]
fn unit_channel_info_returns_nil_when_not_channeling() {
    let env = WowLuaEnv::new().expect("env");
    let nil_first: bool = env.eval("return UnitChannelInfo('player') == nil").unwrap();
    assert!(nil_first);
}

#[test]
fn unit_channel_info_returns_channel_state() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.channeling = Some(CastingState {
            spell_id: 740,
            spell_name: "Tranquility".to_string(),
            icon_path: "Interface/Icons/Spell_Nature_Tranquility".to_string(),
            start_time: 100.0,
            end_time: 108.0,
            cast_id: 42,
            num_empower_stages: 0,
        });
    }

    let name: String = env
        .eval("return select(1, UnitChannelInfo('player'))")
        .unwrap();
    assert_eq!(name, "Tranquility");

    let spell_id: i64 = env
        .eval("return select(8, UnitChannelInfo('player'))")
        .unwrap();
    assert_eq!(spell_id, 740);

    let not_interruptible: bool = env
        .eval("return select(7, UnitChannelInfo('player'))")
        .unwrap();
    assert!(!not_interruptible);

    let start_ms: f64 = env
        .eval("return select(4, UnitChannelInfo('player'))")
        .unwrap();
    assert!((start_ms - 100_000.0).abs() < 1e-6);

    let num_empower_stages: i64 = env
        .eval("return select(10, UnitChannelInfo('player'))")
        .unwrap();
    assert_eq!(num_empower_stages, 0);
}

#[test]
fn unit_channel_info_returns_empower_stage_count() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.channeling = Some(CastingState {
            spell_id: 361469,
            spell_name: "Living Flame".to_string(),
            icon_path: "Interface/Icons/Ability_Evoker_LivingFlame".to_string(),
            start_time: 100.0,
            end_time: 103.0,
            cast_id: 43,
            num_empower_stages: 4,
        });
    }

    let num_empower_stages: i64 = env
        .eval("return select(10, UnitChannelInfo('player'))")
        .unwrap();
    assert_eq!(num_empower_stages, 4);
}

#[test]
fn unit_channel_info_only_for_player() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.channeling = Some(CastingState {
            spell_id: 740,
            spell_name: "Tranquility".to_string(),
            icon_path: "".to_string(),
            start_time: 0.0,
            end_time: 8.0,
            cast_id: 1,
            num_empower_stages: 0,
        });
    }
    let target_nil: bool = env.eval("return UnitChannelInfo('target') == nil").unwrap();
    assert!(target_nil);
}
