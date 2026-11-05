//! P5 caster sub-batch: secret selector -> secret identity; simulator policy.
#![cfg(all(
    feature = "retail-12-1-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
use rilua::{LuaApi, LuaApiMut};
use wow_ui_sim::lua_api::{WowLuaEnv, state::AuraInfo};
const GUID: &str = "Creature-0-1-2-3-19750-000000ABCD";

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetTarget('Caster fixture', 11, 1, true)")
        .unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.current_target.as_mut().unwrap().guid = GUID.into();
        sim.player.buffs = vec![AuraInfo {
            name: "Caster fixture".into(),
            spell_id: 19750,
            icon: 134973,
            duration: 99.0,
            expiration_time: 600.0,
            applications: 1,
            source_unit: "target".into(),
            is_helpful: true,
            is_raid: false,
            is_nameplate_only: false,
            is_stealable: false,
            can_apply_aura: true,
            is_from_player_or_player_pet: false,
            dispel_type: None,
            aura_instance_id: 101,
        }];
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

// AuraInfo lacks PartialEq; snapshot its complete data plus the selected source GUID.
fn snapshot_caster_rows(env: &WowLuaEnv) -> (String, Option<String>) {
    let sim = env.state().borrow();
    (
        format!("{:?}", sim.player.buffs),
        sim.current_target
            .as_ref()
            .map(|target| target.guid.clone()),
    )
}

fn load_original_caster(env: &WowLuaEnv) {
    let profile = if cfg!(feature = "client-ptr") {
        "ptr"
    } else {
        "retail"
    };
    let root = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
    let source=std::fs::read_to_string(root.join(format!(".cache/wow-ui-sim/blizzard-ui/{profile}/AddOns/Blizzard_AuraContainer/Blizzard_AuraContainerSources.lua"))).unwrap();
    let begin = source
        .find("function AuraContainerPublicAuraSourceMixin:GetAuraCasterGUID(")
        .unwrap();
    let end = begin + source[begin..].find("-- Private auras").unwrap();
    env.exec("AuraContainerPublicAuraSourceMixin = {}").unwrap();
    env.exec(&source[begin..end])
        .expect("unchanged original public aura caster helper");
}

#[test]
fn original_public_source_resolves_configured_caster_and_preserves_identity_secrecy() {
    let env = fixture();
    let before_rows = snapshot_caster_rows(&env);
    load_original_caster(&env);
    env.exec(r#"
        local expectedTaint = nil
        local function probe()
            local before=debug.getstacktaint()
            assert(before == expectedTaint, 'explicit caller context')
            for _,id in ipairs({101,Selector}) do
                assert(select('#',AuraContainerPublicAuraSourceMixin:GetAuraCasterGUID('player',id))==1)
                local guid=AuraContainerPublicAuraSourceMixin:GetAuraCasterGUID('player',id)
                assert(issecretvalue(guid)==issecretvalue(id))
                if issecretvalue(id) and before then
                    assert(not canaccessvalue(guid))
                    assert(not pcall(secretunwrap,guid))
                    assert(not pcall(function() return guid..'' end))
                    CasterSecretResult=guid
                else
                    assert(secretunwrap(guid)=='Creature-0-1-2-3-19750-000000ABCD')
                end
            end
            assert(issecretvalue(Selector) and debug.getstacktaint()==before)
        end
        probe()
        debug.setobjecttaint(probe,'P5CasterProbe'); expectedTaint = 'P5CasterProbe'
        probe()
        assert(debug.getstacktaint() == nil, 'outer context before reading tainted publication')
        assert(issecretvalue(CasterSecretResult))
        assert(debug.getstacktaint() == 'P5CasterProbe', 'global read taints the Lua observer')
        assert(not pcall(secretunwrap, CasterSecretResult), 'tainted observer cannot unwrap')
    "#).expect("original consumer reaches selector and returns configured GUID with selected identity policy");
    assert_published_caster_guid(&env);
    assert_eq!(snapshot_caster_rows(&env), before_rows);
    assert_eq!(
        env.state().borrow().current_target.as_ref().unwrap().guid,
        GUID
    );
    assert_eq!(env.state().borrow().player.buffs[0].source_unit, "target");
}

fn assert_published_caster_guid(env: &WowLuaEnv) {
    // A Lua read of the tainted global taints its observer. Inspect the emitted
    // result from the trusted host after that Lua context has ended instead.
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let result = lua.get_global_val("CasterSecretResult");
    assert!(rilua::table_security::is_secret_value(lua.state(), result));
    let payload = rilua::table_security::unwrap_secret(lua.state(), result).unwrap();
    let rilua::Val::Str(reference) = payload else {
        panic!("caster identity must contain a GUID string");
    };
    let guid = lua.state().gc.string_arena.get(reference).unwrap();
    assert_eq!(guid.data(), GUID.as_bytes());
}

#[test]
fn caster_instance_misses_invalid_controls_and_existing_restriction_independence() {
    let env = fixture();
    let before_rows = snapshot_caster_rows(&env);
    for restricted in [false, true, false] {
        env.state().borrow_mut().restricted_actions[0] = if restricted { 2 } else { 0 };
        env.exec(r#"
            local expectedTaint = nil
        local function probe()
                local before=debug.getstacktaint()
            assert(before == expectedTaint, 'explicit caller context')
                for _,id in ipairs({999,Missing,-1,Invalid}) do
                    assert(select('#',C_UnitAuras.GetAuraCasterGUID('player',id))==1)
                    local result=C_UnitAuras.GetAuraCasterGUID('player',id)
                    assert(result==nil and not issecretvalue(result))
                end
                assert(select('#',C_UnitAuras.GetAuraCasterGUID('missing',Selector))==1)
                assert(C_UnitAuras.GetAuraCasterGUID('missing',Selector)==nil)
                assert(not pcall(C_UnitAuras.GetAuraCasterGUID,'player',SecretText))
                assert(not pcall(C_UnitAuras.GetAuraCasterGUID,'player',{}))
                assert(not pcall(C_UnitAuras.GetAuraCasterGUID,SecretUnit,Selector))
                assert(C_UnitAuras.GetAuraCasterGUID('player',101.5)=='Creature-0-1-2-3-19750-000000ABCD')
                local guid=C_UnitAuras.GetAuraCasterGUID('player',Fraction)
                assert(issecretvalue(guid))
                if not before then assert(secretunwrap(guid)=='Creature-0-1-2-3-19750-000000ABCD') end
                assert(issecretvalue(Selector) and debug.getstacktaint()==before)
                if before then assert(not canaccessvalue(Selector)) end
            end
            probe()
            debug.setobjecttaint(probe,'P5CasterProbe'); expectedTaint = 'P5CasterProbe'
            probe()
        "#).expect("one-nil misses, existing i32 coercion, nonnumeric denial and input opacity");
    }
    assert_eq!(snapshot_caster_rows(&env), before_rows);
}

#[test]
fn original_source_keeps_absent_caster_as_one_ordinary_nil() {
    let env = fixture();
    env.state().borrow_mut().current_target = None;
    let before_rows = snapshot_caster_rows(&env);
    load_original_caster(&env);
    env.exec(r#"
        local expectedTaint = nil
        local function probe()
            local before=debug.getstacktaint()
            assert(before == expectedTaint, 'explicit caller context')
            for _,id in ipairs({101,Selector}) do
                assert(select('#',AuraContainerPublicAuraSourceMixin:GetAuraCasterGUID('player',id))==1)
                local value=AuraContainerPublicAuraSourceMixin:GetAuraCasterGUID('player',id)
                assert(value==nil and not issecretvalue(value))
            end
            assert(issecretvalue(Selector) and debug.getstacktaint()==before)
        end
        probe()
        debug.setobjecttaint(probe,'P5CasterProbe'); expectedTaint = 'P5CasterProbe'
        probe()
    "#).expect("missing caster is not an empty GUID or a secret nil wrapper");
    assert_eq!(snapshot_caster_rows(&env), before_rows);
}
