//! SS-P1 PTR-only preservation through the unchanged cached spell consumer.
//! Draft: not compiled or executed. No restricted-tainted setter acceptance.
#![cfg(all(feature = "retail-12-0-5", feature = "client-ptr"))]

use rilua::{LuaApi, LuaApiMut};
use std::time::{Duration, Instant};
use wow_ui_sim::c_api::charge_state::SpellChargeState;
use wow_ui_sim::lua_api::state::SpellCooldownState;
use wow_ui_sim::lua_api::{LossOfControlInfo, WowLuaEnv};

fn seed_ptr_spell_state(restricted: bool) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("SS-P1 PTR environment");
    {
        let mut state = env.state().borrow_mut();
        state.start_time = Instant::now() - Duration::from_secs(20);
        state.cooldowns_restricted = restricted;
        state.gcd = None;
        state.spell_cooldowns.clear();
        state.spell_cooldowns.insert(
            19750,
            SpellCooldownState {
                start: 12.0,
                duration: 80.0,
            },
        );
        state.spell_charges.clear();
        state.spell_charges.insert(
            19750,
            SpellChargeState {
                current_charges: 1,
                max_charges: 3,
                recharge_start: 12.0,
                recharge_duration: 40.0,
                charge_mod_rate: 2.0,
            },
        );
        state.spell_loss_of_control.clear();
        state.spell_loss_of_control.insert(
            19750,
            LossOfControlInfo {
                start_time: 14.0,
                duration: 24.0,
                mod_rate: 2.0,
                is_active: true,
                should_replace_normal_cooldown: false,
            },
        );
    }
    install_secret_spell_selector(&env);
    env
}

fn install_secret_spell_selector(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let selector = rilua::table_security::wrap_host_secret_number(lua.state_mut(), 19750.0);
    lua.state_mut().push(selector);
    let inserted = lua.set_global_val("Ssp1PtrSelector", selector);
    lua.state_mut().pop();
    inserted.expect("root authentic numeric-secret spell selector");
}

fn load_original_ptr_consumer(env: &WowLuaEnv) {
    let path = crate::common::blizzard_ui_dir()
        .join("Blizzard_ActionBar/Shared/ActionButton.lua");
    let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("required original PTR consumer {}: {error}", path.display())
    });
    let start = source
        .find("local defaultCooldownInfo =")
        .expect("PTR defaults boundary");
    let end = start
        + source[start..]
            .find("function ClearActionButtonCooldowns(")
            .expect("PTR consumer end boundary");
    // Delimit fixture isolation only: execute original bytes, not reconstructed Lua.
    // These searches are not assertions about implementation shape or API ordering.
    env.exec(&source[start..end])
        .expect("unchanged original PTR consumer block");
    env.exec(
        r#"
        Ssp1PtrButton = CreateFrame('Button')
        Ssp1PtrButton.cooldown = CreateFrame('Cooldown', 'Ssp1PtrNormal', Ssp1PtrButton)
        Ssp1PtrButton.chargeCooldown = CreateFrame('Cooldown', 'Ssp1PtrCharge', Ssp1PtrButton)
        Ssp1PtrButton.lossOfControlCooldown = CreateFrame('Cooldown', 'Ssp1PtrLoc', Ssp1PtrButton)
        Ssp1PtrButton.enableLOCCooldown = true
        "#,
    )
    .expect("real PTR consumer Cooldown widgets");
}

fn invoke_original_ptr_consumer(env: &WowLuaEnv, selector: &str, tainted: bool) {
    let expected_taint = if tainted { "'Ssp1PtrProbe'" } else { "nil" };
    env.exec(&format!(
        r#"
        local function invoke()
            local before = debug.getstacktaint()
            assert(before == {expected_taint}, 'caller security')
            assert(issecretvalue(Ssp1PtrSelector), 'authentic selector remains secret')
            if {tainted} then
                assert(not canaccessvalue(Ssp1PtrSelector))
                assert(not pcall(secretunwrap, Ssp1PtrSelector))
                assert(not pcall(function() return Ssp1PtrSelector + 1 end))
            end
            Ssp1PtrButton.spellID = {selector}
            ActionButton_UpdateCooldown(Ssp1PtrButton)
            assert(issecretvalue(Ssp1PtrSelector), 'no selector exposure')
            if {tainted} then
                assert(not canaccessvalue(Ssp1PtrSelector))
                assert(not pcall(secretunwrap, Ssp1PtrSelector))
                assert(not pcall(function() return Ssp1PtrSelector + 1 end))
            end
            assert(debug.getstacktaint() == before, 'caller taint preserved')
        end
        if {tainted} then debug.setobjecttaint(invoke, 'Ssp1PtrProbe') end
        invoke()
        "#,
    ))
    .expect("original PTR cooldown-before-charges-before-LoC execution");
}

fn assert_widget_timing(env: &WowLuaEnv, name: &str, expected: (f64, f64, f64)) {
    let state = env.state().borrow();
    let id = state
        .widgets
        .get_id_by_name(name)
        .expect("named real Cooldown widget");
    let frame = state.widgets.get(id).expect("Cooldown widget state");
    assert_eq!(
        (
            frame.cooldown_start,
            frame.cooldown_duration,
            frame.cooldown_mod_rate
        ),
        expected,
        "{name}",
    );
}

#[test]
fn original_ptr_action_button_preserves_ssp1_secret_spell_selection() {
    // The fourth combination (restricted + tainted widget setters) is excluded.
    for (restricted, tainted) in [(false, false), (false, true), (true, false)] {
        let env = seed_ptr_spell_state(restricted);
        load_original_ptr_consumer(&env);
        for selector in ["19750", "Ssp1PtrSelector"] {
            env.state()
                .borrow_mut()
                .spell_loss_of_control
                .get_mut(&19750)
                .expect("supplied LoC row")
                .should_replace_normal_cooldown = false;
            env.exec(
                r#"
                Ssp1PtrButton.cooldown:Clear()
                Ssp1PtrButton.chargeCooldown:Clear()
                Ssp1PtrButton.lossOfControlCooldown:Clear()
                "#,
            )
            .expect("clear widgets before each selector");
            invoke_original_ptr_consumer(&env, selector, tainted);
            assert_widget_timing(&env, "Ssp1PtrNormal", (12.0, 80.0, 1.0));
            assert_widget_timing(&env, "Ssp1PtrCharge", (12.0, 40.0, 2.0));
            assert_widget_timing(&env, "Ssp1PtrLoc", (14.0, 24.0, 2.0));

            env.state()
                .borrow_mut()
                .spell_loss_of_control
                .get_mut(&19750)
                .expect("supplied LoC row")
                .should_replace_normal_cooldown = true;
            invoke_original_ptr_consumer(&env, selector, tainted);
            assert_widget_timing(&env, "Ssp1PtrNormal", (0.0, 0.0, 1.0));
            assert_widget_timing(&env, "Ssp1PtrCharge", (0.0, 0.0, 1.0));
            assert_widget_timing(&env, "Ssp1PtrLoc", (14.0, 24.0, 2.0));
        }
    }
}
