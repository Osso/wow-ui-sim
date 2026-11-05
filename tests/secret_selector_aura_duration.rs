//! P5 duration sub-batch; public units only, no Retail CarryOver model.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
use rilua::LuaApiMut;
use wow_ui_sim::c_api::aura_duration::SpellAuraDuration;
use wow_ui_sim::lua_api::{WowLuaEnv, state::AuraInfo};

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        // Freeze elapsed() at zero without replacing either API or consumer.
        sim.start_time = std::time::Instant::now() + std::time::Duration::from_secs(60);
        sim.player.buffs = vec![AuraInfo {
            name: "Pandemic fixture".into(),
            spell_id: 19750,
            icon: 134973,
            duration: 99.0,
            expiration_time: 4.0,
            applications: 1,
            source_unit: "player".into(),
            is_helpful: true,
            is_raid: false,
            is_nameplate_only: false,
            is_stealable: false,
            can_apply_aura: true,
            is_from_player_or_player_pet: true,
            dispel_type: None,
            aura_instance_id: 101,
        }];
        sim.spell_aura_durations.insert(
            19750,
            SpellAuraDuration {
                base_duration_seconds: 20.0,
                max_carryover_seconds: 6.0,
            },
        );
    }
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    for (name, number) in [
        ("Selector", 101.0),
        ("Missing", 999.0),
        ("Invalid", -1.0),
        ("Fraction", 101.5),
    ] {
        let value = rilua::table_security::wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        lua.set_global_val(name, value).unwrap();
        lua.state_mut().pop();
    }
    for (name, text) in [("SecretText", "101"), ("SecretUnit", "player")] {
        let value = rilua::table_security::wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        lua.set_global_val(name, value).unwrap();
        lua.state_mut().pop();
    }
    drop(lua);
    env
}

// AuraInfo lacks PartialEq; same-build data snapshots cover every stored aura field.
fn snapshot_duration_rows(
    env: &WowLuaEnv,
) -> (String, std::collections::HashMap<u32, SpellAuraDuration>) {
    let sim = env.state().borrow();
    (
        format!("{:?}", sim.player.buffs),
        sim.spell_aura_durations.clone(),
    )
}

#[cfg(feature = "client-ptr")]
#[test]
fn original_ptr_pandemic_window_uses_secret_instance() {
    let env = fixture();
    let before_rows = snapshot_duration_rows(&env);
    let root = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
    let source = std::fs::read_to_string(root.join(".cache/wow-ui-sim/blizzard-ui/ptr/AddOns/Blizzard_AuraContainer/Blizzard_AuraContainerUtil.lua")).unwrap();
    let begin = source
        .find("function AuraContainerUtil.GetPandemicWindow(")
        .unwrap();
    let end = begin
        + source[begin..]
            .find("function AuraContainerUtil.AddUnitEventRegistration(")
            .unwrap();
    let shared = std::fs::read_to_string(root.join(".cache/wow-ui-sim/blizzard-ui/ptr/AddOns/Blizzard_AuraContainer/Blizzard_AuraContainerShared.lua")).unwrap();
    let enum_begin = shared.find("AuraContainerAuraDataType =").unwrap();
    let enum_end = enum_begin
        + shared[enum_begin..]
            .find("AuraContainerItemEnchantmentSlot =")
            .unwrap();
    env.exec(&shared[enum_begin..enum_end])
        .expect("original aura data type enum");
    env.exec("AuraContainerUtil = {}").unwrap();
    env.exec(&source[begin..end])
        .expect("unchanged original PTR pandemic consumer");
    env.exec(
        r#"
        local expectedTaint = nil
        local function probe()
            local before = debug.getstacktaint()
            assert(before == expectedTaint, 'explicit caller context')
            for _,id in ipairs({101,Selector}) do
                local aura = {auraType=AuraContainerAuraDataType.Aura, isPrivate=false, auraInstanceID=id, expirationTime=4}
                assert(select('#',AuraContainerUtil.GetPandemicWindow('player',aura)) == 2)
                local first,last = AuraContainerUtil.GetPandemicWindow('player',aura)
                assert(first == aura.expirationTime-4 and last == aura.expirationTime)
                assert(not issecretvalue(first) and not issecretvalue(last))
            end
            for _,id in ipairs({999,Missing}) do
                local aura = {auraType=AuraContainerAuraDataType.Aura, isPrivate=false, auraInstanceID=id, expirationTime=4}
                assert(select('#',AuraContainerUtil.GetPandemicWindow('player',aura)) == 1)
                assert(AuraContainerUtil.GetPandemicWindow('player',aura) == nil)
            end
            assert(issecretvalue(Selector) and debug.getstacktaint() == before)
        end
        probe()
        debug.setobjecttaint(probe,'P5Probe'); expectedTaint = 'P5Probe'
        probe()
    "#,
    )
    .expect("exact PTR [expiration-4,expiration] for both selectors/callers");
    assert_eq!(snapshot_duration_rows(&env), before_rows);
    assert_eq!(env.state().borrow().player.buffs[0].expiration_time, 4.0);
    assert_eq!(
        env.state().borrow().spell_aura_durations[&19750].base_duration_seconds,
        20.0
    );
}

#[test]
fn aura_duration_instance_controls_preserve_plain_results_and_nil_arity() {
    let env = fixture();
    let before_rows = snapshot_duration_rows(&env);
    for restricted in [false, true] {
        env.state().borrow_mut().restricted_actions[0] = if restricted { 2 } else { 0 };
        env.exec(&format!(
            r#"
            Restricted = {}
        "#,
            restricted
        ))
        .unwrap();
        env.exec(r#"
            local expectedTaint = nil
        local function probe()
                local before = debug.getstacktaint()
            assert(before == expectedTaint, 'explicit caller context')
                if Restricted and before then
                    for _,query in ipairs({C_UnitAuras.GetAuraBaseDuration,C_UnitAuras.GetRefreshExtendedDuration}) do
                        assert(not pcall(query,'player',101))
                        assert(not pcall(query,'player',Selector))
                    end
                    assert(issecretvalue(Selector) and not canaccessvalue(Selector))
                    assert(debug.getstacktaint() == before)
                    return
                end
                for index,query in ipairs({C_UnitAuras.GetAuraBaseDuration,C_UnitAuras.GetRefreshExtendedDuration}) do
                    for _,id in ipairs({101,Selector}) do
                        assert(select('#',query('player',id)) == 1)
                        local value = query('player',id)
                        assert(value == (index == 1 and 20 or 24) and not issecretvalue(value))
                    end
                    for _,id in ipairs({999,Missing,-1,Invalid,101.5,Fraction}) do
                        assert(select('#',query('player',id)) == 1 and query('player',id) == nil)
                    end
                    for _,id in ipairs({{},SecretText}) do assert(not pcall(query,'player',id)) end
                    assert(not pcall(query,SecretUnit,Selector))
                    assert(not pcall(query,'player',Selector,Selector), 'optional spell secrets not widened')
                    assert(select('#',query('missing',Selector)) == 1 and query('missing',Selector) == nil)
                end
                assert(issecretvalue(Selector) and debug.getstacktaint() == before)
                if before then
                    assert(not canaccessvalue(Selector))
                    assert(not pcall(function() return Selector + 0 end))
                end
            end
            probe()
            debug.setobjecttaint(probe,'P5Probe'); expectedTaint = 'P5Probe'
            probe()
        "#).expect("duration-only policy and exact controls");
    }
    assert_eq!(snapshot_duration_rows(&env), before_rows);
}
