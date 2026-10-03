#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string, wrap_secret};
use wow_ui_sim::lua_api::WowLuaEnv;

const IDENTIFIER_LINK: &str = "|cff71d5ff|Hspell:101|h[Fixture Maw]|h|r";
const FIRST_LINK: &str = "|Hmawpower:101|h[First Fixture]|h";
const SECOND_LINK: &str = "|Hmawpower:202|h[Second Fixture]|h";

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Maw power environment");
    env.exec(
        r#"
        function CheckMaw(identifier, atlas, link)
            local function checkAtlas(...)
                assert(select('#', ...) == 1, 'INFERRED one nullable atlas')
                local value = ...
                assert(value == atlas and not issecretvalue(value), 'exact public atlas')
                if atlas ~= nil then assert(type(value) == 'string') end
            end
            local function checkLink(...)
                if link == nil then
                    assert(select('#', ...) == 0, 'INFERRED zero results on link miss')
                else
                    assert(select('#', ...) == 1, 'one link result')
                    local value = ...
                    assert(type(value) == 'string' and value == link, 'exact stored link')
                    assert(not issecretvalue(value), 'INFERRED public link')
                end
            end
            checkAtlas(C_Spell.GetMawPowerBorderAtlasBySpellID(identifier))
            checkLink(C_Spell.GetMawPowerLinkBySpellID(identifier))
        end
        function RejectMaw(...)
            for _, name in ipairs({'GetMawPowerBorderAtlasBySpellID', 'GetMawPowerLinkBySpellID'}) do
                local query = C_Spell[name]
                assert(type(query) == 'function', 'registered provider required')
                local ok, err = pcall(query, ...)
                assert(not ok and type(err) == 'string' and #err > 0, 'argument rejected')
            end
        end
    "#,
    )
    .expect("install behavioral assertions, not replacement APIs");
    env
}

fn seeded_env() -> WowLuaEnv {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        state.maw_powers.border_atlases.extend([
            (101, "fixture-border-A".into()),
            (202, "fixture-border-B".into()),
        ]);
        state.maw_powers.links.extend([
            (101, FIRST_LINK.into()),
            (202, SECOND_LINK.into()),
        ]);
        state.spell_id_aliases.clear();
        state.spell_id_aliases.insert("fixture maw".into(), 101);
        state.spell_id_aliases.insert(IDENTIFIER_LINK.to_lowercase(), 202);
    }
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    env.exec("MawWrongObject = CreateFrame('Frame'); assert(MawWrongObject:GetObjectType() == 'Frame')")
        .expect("real frame fixture");
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("VM security helpers");
    for (name, text) in [
        ("MawSecretName", "fixture maw"),
        ("MawSecretLink", IDENTIFIER_LINK),
        ("MawSecretUnknown", "unseeded"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic secret STRING");
    }
    for (name, number) in [("MawSecretNumber", 101.0), ("MawSecretMissing", 303.0)] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic secret NUMBER");
    }
    let native = lua.get_global_val("MawWrongObject");
    assert!(!native.is_nil(), "frame remains globally rooted");
    lua.state_mut().push(native);
    let wrapper = wrap_secret(lua.state_mut(), native).expect("wrap actual wrong object");
    lua.state_mut().push(wrapper);
    let inserted = lua.set_global_val("MawSecretObject", wrapper);
    lua.state_mut().pop();
    lua.state_mut().pop();
    inserted.expect("root wrapper and original object");
}

#[test]
fn empty_defaults_and_unknown_identifiers_have_distinct_miss_arities() {
    let env = fixture_env();
    assert!(env.state().borrow().maw_powers.border_atlases.is_empty());
    assert!(env.state().borrow().maw_powers.links.is_empty());
    env.exec("CheckMaw(101, nil, nil); CheckMaw('unseeded', nil, nil)")
        .expect("empty host inputs do not manufacture values");
    seeded_env().exec(
        "CheckMaw(303, nil, nil); CheckMaw('unseeded', nil, nil); CheckMaw('', nil, nil); CheckMaw('101', nil, nil); CheckMaw('|Hspell:101|h[Fixture Maw]|h', nil, nil)",
    ).expect("strings require explicit aliases, no numeric coercion or hyperlink parser");
}

#[test]
fn numbers_names_and_full_link_aliases_return_exact_host_strings() {
    seeded_env().exec(&format!(
        "CheckMaw(101, 'fixture-border-A', '{FIRST_LINK}'); CheckMaw(202, 'fixture-border-B', '{SECOND_LINK}'); CheckMaw('FIXTURE Maw', 'fixture-border-A', '{FIRST_LINK}'); CheckMaw('{IDENTIFIER_LINK}', 'fixture-border-B', '{SECOND_LINK}')",
    )).expect("full link alias overrides embedded 101 with resolved 202");
}

#[test]
fn numeric_alias_precedence_and_removal_apply_to_both_queries() {
    let env = seeded_env();
    env.state().borrow_mut().spell_id_aliases.insert("101".into(), 202);
    env.exec(&format!(
        "CheckMaw(101, 'fixture-border-B', '{SECOND_LINK}'); CheckMaw('101', 'fixture-border-B', '{SECOND_LINK}')",
    )).unwrap();
    env.state().borrow_mut().spell_id_aliases.remove("101");
    env.exec(&format!(
        "CheckMaw(101, 'fixture-border-A', '{FIRST_LINK}'); CheckMaw('101', nil, nil)",
    )).unwrap();
}

#[test]
fn alias_changes_read_resolved_key_without_falling_back_to_previous_record() {
    let env = seeded_env();
    env.state().borrow_mut().spell_id_aliases.insert("fixture maw".into(), 202);
    env.exec(&format!("CheckMaw('fixture maw', 'fixture-border-B', '{SECOND_LINK}')"))
        .unwrap();
    env.state().borrow_mut().spell_id_aliases.insert("fixture maw".into(), 303);
    env.exec("CheckMaw('fixture maw', nil, nil)").unwrap();
}

#[test]
fn maps_are_independent_and_replacement_removal_are_live() {
    let env = seeded_env();
    {
        let mut state = env.state().borrow_mut();
        state.maw_powers.border_atlases.insert(101, "replacement-atlas".into());
        state.maw_powers.links.remove(&101);
    }
    env.exec("CheckMaw(101, 'replacement-atlas', nil)").unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.maw_powers.border_atlases.remove(&101);
        state.maw_powers.links.insert(101, "host-link-without-synthesis".into());
    }
    env.exec(&format!(
        "CheckMaw(101, nil, 'host-link-without-synthesis'); CheckMaw(202, 'fixture-border-B', '{SECOND_LINK}')",
    )).unwrap();
}

#[test]
fn u32_endpoints_and_empty_strings_are_explicit_hits() {
    let env = seeded_env();
    {
        let mut state = env.state().borrow_mut();
        state.maw_powers.border_atlases.extend([(0, String::new()), (u32::MAX, "last-atlas".into())]);
        state.maw_powers.links.extend([(0, String::new()), (u32::MAX, "last-link".into())]);
    }
    env.exec("CheckMaw(0, '', ''); CheckMaw(4294967295, 'last-atlas', 'last-link')")
        .expect("INFERRED shared u32 boundary; stored empty strings are not misses");
}

#[test]
fn invalid_public_representations_error_before_lookup() {
    let env = seeded_env();
    env.state().borrow_mut().spell_id_aliases.insert("101".into(), 202);
    env.exec(
        r#"
        RejectMaw(); RejectMaw(nil)
        for _, value in ipairs({false, true, {}, function() end, coroutine.create(function() end),
            CreateFrame('Frame'), 0/0, math.huge, -math.huge, -1, 101.5, 4294967296,
            string.char(255), string.char(192, 175)}) do
            RejectMaw(value)
        end
        CheckMaw(303, nil, nil)
    "#,
    ).expect("INFERRED strict public identifier validation, no fractional alias bypass");
}

#[test]
fn queries_are_read_only_and_environments_are_isolated() {
    let first = seeded_env();
    let second = seeded_env();
    let aliases = first.state().borrow().spell_id_aliases.clone();
    let atlases = first.state().borrow().maw_powers.border_atlases.clone();
    let links = first.state().borrow().maw_powers.links.clone();
    first.exec(&format!(
        r#"
        MawCaller = {{identifier = '{IDENTIFIER_LINK}', marker = 17}}
        CheckMaw(MawCaller.identifier, 'fixture-border-B', '{SECOND_LINK}')
        collectgarbage('collect')
        assert(MawCaller.identifier == '{IDENTIFIER_LINK}' and MawCaller.marker == 17)
        CheckMaw(303, nil, nil)
    "#,
    )).unwrap();
    assert_eq!(first.state().borrow().spell_id_aliases, aliases);
    assert_eq!(first.state().borrow().maw_powers.border_atlases, atlases);
    assert_eq!(first.state().borrow().maw_powers.links, links);
    first.state().borrow_mut().maw_powers.border_atlases.insert(101, "first-only".into());
    first.exec(&format!("CheckMaw(101, 'first-only', '{FIRST_LINK}')")).unwrap();
    second.exec(&format!("CheckMaw(101, 'fixture-border-A', '{FIRST_LINK}')")).unwrap();
}

#[test]
fn public_queries_work_secure_and_tainted_without_changing_caller_taint() {
    let env = seeded_env();
    env.exec(&format!(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            CheckMaw(101, 'fixture-border-A', '{FIRST_LINK}')
            CheckMaw('fixture maw', 'fixture-border-A', '{FIRST_LINK}')
            CheckMaw('{IDENTIFIER_LINK}', 'fixture-border-B', '{SECOND_LINK}')
            CheckMaw('unknown', nil, nil)
            assert(debug.getstacktaint() == before)
        end
        assert(issecure()); probe(); assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'MawPublicFixture')
            probe()
            assert(debug.getstacktaint() == 'MawPublicFixture')
        end
        debug.setobjecttaint(addon, 'MawPublicFixture')
        addon(); assert(issecure())
    "#,
    )).expect("ordinary public identifiers preserve secure and tainted context");
}

#[test]
fn authentic_secrets_reject_in_both_contexts_after_gc_and_public_queries_recover() {
    let env = seeded_env();
    install_host_secrets(&env);
    env.exec(&format!(
        r#"
        local rooted = {{MawSecretNumber, MawSecretName, MawSecretLink,
            MawSecretUnknown, MawSecretMissing, MawSecretObject}}
        collectgarbage('collect')
        local function probe()
            local before = debug.getstacktaint()
            for _, value in ipairs(rooted) do
                assert(issecretvalue(value), 'authentic VM secret')
                RejectMaw(value)
                assert(issecretvalue(value), 'no declassification')
                assert(debug.getstacktaint() == before, 'caller taint preserved')
            end
            assert(rawequal(rooted[1], MawSecretNumber))
            assert(rawequal(rooted[2], MawSecretName))
            assert(rawequal(rooted[6], MawSecretObject))
            CheckMaw(101, 'fixture-border-A', '{FIRST_LINK}')
            assert(debug.getstacktaint() == before, 'public recovery')
        end
        assert(issecure()); probe(); assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'MawSecretFixture')
            probe()
            assert(debug.getstacktaint() == 'MawSecretFixture')
        end
        debug.setobjecttaint(addon, 'MawSecretFixture')
        addon(); assert(issecure())
    "#,
    )).expect("INFERRED conservative rejection, not native AllowedWhenTainted");
}
