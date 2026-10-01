//! Source-observed Forever subset; no hardware or native DTO parity claim.
#![cfg(feature = "client-wowforever")]

use rilua::LuaApiMut;
use wow_ui_sim::c_api::c_game_pad::{MappedStick, MappedStickSnapshot};
use wow_ui_sim::loader::{
    MissingRequirement, MissingRequirementKind,
    discover_blizzard_addon_closure_for_screen_with_overrides, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::NilSymbolEnvironment;
use wow_ui_sim::screen::ScreenKind;

fn snapshot(sticks: &[(&str, f64, f64)]) -> MappedStickSnapshot {
    MappedStickSnapshot {
        sticks: sticks
            .iter()
            .map(|&(name, x, y)| MappedStick {
                config_name: name.into(),
                x,
                y,
            })
            .collect(),
    }
}

fn cached_initializer(mapped: Option<MappedStickSnapshot>) -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .expect("Forever cache must exist; no downloads");
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    env.state().borrow_mut().gamepad_mapped_sticks = mapped;
    let closure = discover_blizzard_addon_closure_for_screen_with_overrides(
        &ui,
        ScreenKind::Game,
        &["Blizzard_GamepadSharedUtility"],
        &[],
    );
    assert!(
        closure
            .iter()
            .any(|(name, _)| name == "Blizzard_GamepadSharedUtility")
    );
    for (_, toc) in closure {
        let result = load_addon(&env.loader_env(), &toc).expect("unchanged cached TOC must load");
        assert!(result.warnings.is_empty(), "{result:?}");
        assert!(result.lua_files + result.xml_files > 0, "{result:?}");
        for requirement in &result.missing_requirements {
            // Existing truthy Reveal stub changes debug-event setup. Retain only
            // this known limitation; it is not an innocent nil probe or parity.
            assert!(
                result.name == "Blizzard_SharedXML" && known_reveal_setup(requirement),
                "{}: unexpected requirement: {requirement:?}",
                result.name
            );
            eprintln!("retained existing Reveal debug-setup limitation: {requirement:?}");
        }
    }
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "{:?}",
        env.state().borrow().lua_errors
    );
    env.exec("assert(type(GamepadSharedUtility.BindingStack.InputAxisBinding.Init) == 'function')")
        .unwrap();
    env
}

fn known_reveal_setup(requirement: &MissingRequirement) -> bool {
    matches!(
        &requirement.kind,
        MissingRequirementKind::CNamespace { namespace } if namespace == "C_Reveal"
    ) && requirement.attribution.addon_name == "Blizzard_SharedXML"
        && requirement.attribution.environment == NilSymbolEnvironment::Public
        && requirement.attribution.source.as_deref() == Some("DebugBarManager.lua")
        && requirement.attribution.line == Some(106)
}

#[test]
fn absent_snapshot_centers_both_sides_in_cached_initializer() {
    let env = cached_initializer(None);
    env.exec(
        r#"
        assert(C_GamePad.GetDeviceMappedState() == nil)
        assert(C_GamePad.StickIndexToConfigName(0) == nil)
        assert(inputBindingAxisListener.left.isCentered == true)
        assert(inputBindingAxisListener.right.isCentered == true)
        "#,
    )
    .unwrap();
}

#[test]
fn configured_movement_and_camera_drive_cached_initializer() {
    let env = cached_initializer(Some(snapshot(&[
        ("Movement", 0.3, 0.4),
        ("Camera", 0.0, 0.0),
    ])));
    env.exec(
        r#"
        local mapped = C_GamePad.GetDeviceMappedState()
        assert(mapped.stickCount == 2 and #mapped.sticks == 2)
        assert(math.abs(mapped.sticks[1].len - 0.5) < 1e-12)
        assert(mapped.sticks[2].len == 0)
        assert(C_GamePad.StickIndexToConfigName(0) == 'Movement')
        assert(C_GamePad.StickIndexToConfigName(1) == 'Camera')
        assert(inputBindingAxisListener.left.isCentered == false)
        assert(inputBindingAxisListener.right.isCentered == true)
        "#,
    )
    .unwrap();
}

#[test]
fn reordered_names_select_the_configured_side_in_cached_initializer() {
    let env = cached_initializer(Some(snapshot(&[
        ("Camera", 0.0, -0.75),
        ("Movement", 0.0, 0.0),
    ])));
    env.exec(
        r#"
        assert(C_GamePad.StickIndexToConfigName(0) == 'Camera')
        assert(C_GamePad.StickIndexToConfigName(1) == 'Movement')
        assert(C_GamePad.GetDeviceMappedState().sticks[1].len == 0.75)
        assert(inputBindingAxisListener.left.isCentered == true)
        assert(inputBindingAxisListener.right.isCentered == false)
        "#,
    )
    .unwrap();
}

#[test]
fn queries_follow_replaced_snapshots_without_exposing_mutable_model_tables() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().gamepad_mapped_sticks = Some(snapshot(&[("Movement", 3.0, 4.0)]));
    env.exec(
        r#"
        oldMapped = C_GamePad.GetDeviceMappedState()
        assert(oldMapped.stickCount == 1 and #oldMapped.sticks == 1)
        assert(oldMapped.sticks[1].len == 5)
        oldMapped.sticks[1].len = 99
        oldMapped.stickCount = 80
        assert(C_GamePad.GetDeviceMappedState().sticks[1].len == 5)
        assert(C_GamePad.GetDeviceMappedState().stickCount == 1)
        assert(C_GamePad.StickIndexToConfigName(1) == nil)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().gamepad_mapped_sticks =
        Some(snapshot(&[("Camera", 0.0, 0.0), ("Auxiliary", 0.0, 2.0)]));
    env.exec(
        r#"
        local mapped = C_GamePad.GetDeviceMappedState()
        assert(mapped.stickCount == 2 and #mapped.sticks == 2)
        assert(mapped.sticks[1].len == 0 and mapped.sticks[2].len == 2)
        assert(C_GamePad.StickIndexToConfigName(0) == 'Camera')
        assert(C_GamePad.StickIndexToConfigName(1) == 'Auxiliary')
        assert(C_GamePad.StickIndexToConfigName(2) == nil)
        assert(C_GamePad.StickIndexToConfigName(-1) == nil)
        assert(C_GamePad.StickIndexToConfigName(0.5) == nil)
        assert(not pcall(C_GamePad.StickIndexToConfigName, '0'))
        "#,
    )
    .unwrap();
    env.state().borrow_mut().gamepad_mapped_sticks = Some(snapshot(&[]));
    env.exec("local s = C_GamePad.GetDeviceMappedState(); assert(s.stickCount == 0 and #s.sticks == 0); assert(C_GamePad.StickIndexToConfigName(0) == nil)")
        .unwrap();
    env.state().borrow_mut().gamepad_mapped_sticks = None;
    env.exec("assert(C_GamePad.GetDeviceMappedState() == nil); assert(C_GamePad.StickIndexToConfigName(0) == nil)")
        .unwrap();
}

#[test]
fn mapped_snapshots_are_environment_local_and_independent_of_input_style() {
    use wow_ui_sim::c_api::c_input_interface_style::InputInterfaceStyle;
    let first = WowLuaEnv::new().unwrap();
    let second = WowLuaEnv::new().unwrap();
    first.state().borrow_mut().gamepad_mapped_sticks = Some(snapshot(&[("Movement", 1.0, 0.0)]));
    second
        .state()
        .borrow_mut()
        .set_input_interface_style(InputInterfaceStyle::Gamepad);
    first.exec("local f = CreateFrame('Frame'); f:EnableGamePadStick(true); f:EnableGamePadButton(true); assert(f:IsGamePadStickEnabled() and f:IsGamePadButtonEnabled()); assert(not IsUsingGamepad()); assert(C_GamePad.GetDeviceMappedState().sticks[1].len == 1)").unwrap();
    second.exec("assert(IsUsingGamepad()); assert(C_GamePad.GetDeviceMappedState() == nil); assert(C_GamePad.StickIndexToConfigName(0) == nil)").unwrap();
    second.state().borrow_mut().gamepad_mapped_sticks = Some(snapshot(&[("Camera", 0.0, 0.0)]));
    first
        .exec("assert(C_GamePad.StickIndexToConfigName(0) == 'Movement')")
        .unwrap();
    second
        .exec("assert(C_GamePad.StickIndexToConfigName(0) == 'Camera')")
        .unwrap();
}

#[test]
fn queries_preserve_caller_taint_and_do_not_unwrap_opaque_selectors() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().gamepad_mapped_sticks = Some(snapshot(&[("Movement", 1.0, 0.0)]));
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let value = rilua::table_security::wrap_secret(lua.state_mut(), rilua::Val::Num(0.0))
            .expect("host opaque numeric selector");
        lua.state_mut().push(value);
        lua.set_global_val("SecretStickIndex", value).unwrap();
        lua.state_mut().pop();
    }
    env.exec(
        r#"
        assert(issecretvalue(SecretStickIndex))
        -- Explicit simulator policy: opaque selectors are not decoded even
        -- for untainted callers. Native secret acceptance is unknown.
        assert(not pcall(C_GamePad.StickIndexToConfigName, SecretStickIndex))
        local function addon()
            assert(debug.getstacktaint() == 'MappedStickProbe')
            assert(C_GamePad.GetDeviceMappedState().sticks[1].len == 1)
            assert(debug.getstacktaint() == 'MappedStickProbe')
            assert(C_GamePad.StickIndexToConfigName(0) == 'Movement')
            assert(debug.getstacktaint() == 'MappedStickProbe')
            assert(not pcall(C_GamePad.StickIndexToConfigName, SecretStickIndex))
            assert(issecretvalue(SecretStickIndex))
            assert(debug.getstacktaint() == 'MappedStickProbe')
        end
        debug.setobjecttaint(addon, 'MappedStickProbe')
        addon()
        "#,
    )
    .unwrap();
}
