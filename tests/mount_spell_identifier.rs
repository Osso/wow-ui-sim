//! Exact291: C_MountJournal inputs, not the existing C_Spell companion.
//! Explicit aliases and strict rejection are simulator policy, not native grammar.
//! AllowedWhenTainted secret permissions remain UNMODELED; no native credit.
#![cfg(feature = "retail-12-0-5")]

use std::collections::HashMap;

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;

const LINK: &str = "|cff71d5ff|Hspell:40192|h[Fixture Mount]|h|r";

#[derive(Debug, PartialEq)]
struct Inputs {
    relations: Vec<(u32, u32)>,
    aliases: HashMap<String, u32>,
}

fn read_inputs(env: &WowLuaEnv) -> Inputs {
    let state = env.state().borrow();
    Inputs {
        relations: state
            .world
            .mounts
            .iter()
            .map(|mount| (mount.spell_id, mount.mount_id))
            .collect(),
        aliases: state.spell_id_aliases.clone(),
    }
}

fn create_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("mount identifier environment");
    env.state().borrow_mut().spell_id_aliases.clear();
    env.exec(
        r#"
        function CheckMount(identifier, expected)
            assert(type(C_MountJournal) == 'table', 'actual journal namespace required')
            assert(type(C_MountJournal.GetMountFromSpell) == 'function', 'actual journal provider required')
            local result = C_MountJournal.GetMountFromSpell(identifier)
            assert(result == expected, 'spell-to-mount relation')
        end
        function RejectMount(...)
            assert(type(C_MountJournal) == 'table', 'actual journal namespace required')
            assert(type(C_MountJournal.GetMountFromSpell) == 'function', 'actual journal provider required')
            local ok, err = pcall(C_MountJournal.GetMountFromSpell, ...)
            assert(not ok and type(err) == 'string' and #err > 0, 'reject invalid input explicitly')
        end
        "#,
    )
    .expect("assertion helpers only; never replace the API");
    env
}

fn insert_alias(env: &WowLuaEnv, key: &str, spell_id: u32) {
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert(key.to_lowercase(), spell_id);
}

fn install_host_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("VM security helpers");
    for (name, number) in [
        ("MountSecretNumber", 458.0),
        ("MountSecretMissing", 999999.0),
    ] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root actual secret NUMBER");
    }
    for (name, text) in [
        ("MountSecretName", "fixture mount"),
        ("MountSecretLink", LINK),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root actual secret STRING");
    }
}

#[test]
fn numeric_brown_horse_spell_returns_existing_mount() {
    let env = create_env();
    env.exec("CheckMount(458, 6)").unwrap();
    assert!(read_inputs(&env).relations.contains(&(458, 6)));
}

#[test]
fn unknown_public_spell_returns_nil() {
    let env = create_env();
    env.exec("CheckMount(999999, nil)").unwrap();
    assert!(
        !read_inputs(&env)
            .relations
            .iter()
            .any(|(spell, _)| *spell == 999999)
    );
}

#[test]
fn explicit_same_id_alias_preserves_numeric_relation() {
    let env = create_env();
    insert_alias(&env, "458", 458);
    env.exec("CheckMount(458, 6); CheckMount('458', 6)")
        .unwrap();
    assert_eq!(read_inputs(&env).aliases.get("458"), Some(&458));
}

#[test]
fn numeric_alias_precedes_identity_using_another_existing_mount() {
    let env = create_env();
    insert_alias(&env, "458", 40192);
    env.exec("CheckMount(458, 107); CheckMount('458', 107); CheckMount(40192, 107)")
        .unwrap();
    assert!(read_inputs(&env).relations.contains(&(40192, 107)));
}

#[test]
fn explicit_name_alias_uses_lowercase_registry_key() {
    let env = create_env();
    insert_alias(&env, "fixture mount", 458);
    env.exec("CheckMount('FIXTURE MOUNT', 6)").unwrap();
    assert_eq!(read_inputs(&env).aliases.get("fixture mount"), Some(&458));
}

#[test]
fn numeric_string_requires_explicit_alias() {
    let env = create_env();
    env.exec("CheckMount('458', nil); CheckMount(458, 6)")
        .unwrap();
    insert_alias(&env, "458", 458);
    env.exec("CheckMount('458', 6)").unwrap();
}

#[test]
fn link_shaped_alias_uses_registered_value_not_embedded_spell() {
    let env = create_env();
    insert_alias(&env, LINK, 458);
    env.exec(&format!("CheckMount('{LINK}', 6); CheckMount(40192, 107)"))
        .unwrap();
    assert_eq!(
        read_inputs(&env).aliases.get(&LINK.to_lowercase()),
        Some(&458)
    );
}

#[test]
fn unregistered_strings_do_not_infer_names_or_link_grammar() {
    let env = create_env();
    env.exec(&format!(
        "CheckMount('Brown Horse', nil); CheckMount('fixture mount', nil); CheckMount('', nil); CheckMount('{LINK}', nil); CheckMount(458, 6)"
    )).unwrap();
    assert!(read_inputs(&env).aliases.is_empty());
}

#[test]
fn registered_alias_to_missing_relation_returns_nil() {
    let env = create_env();
    insert_alias(&env, "missing mount", 999999);
    env.exec("CheckMount('missing mount', nil); CheckMount(458, 6)")
        .unwrap();
    assert_eq!(
        read_inputs(&env).aliases.get("missing mount"),
        Some(&999999)
    );
}

#[test]
fn existing_relation_mount_id_and_spell_updates_are_live() {
    let env = create_env();
    insert_alias(&env, "fixture mount", 458);
    env.exec("CheckMount(458, 6); CheckMount('fixture mount', 6)")
        .unwrap();
    {
        let mut state = env.state().borrow_mut();
        let mount = state
            .world
            .mounts
            .iter_mut()
            .find(|mount| mount.mount_id == 6)
            .unwrap();
        mount.mount_id = 6006;
    }
    env.exec("CheckMount(458, 6006); CheckMount('fixture mount', 6006)")
        .unwrap();
    {
        let mut state = env.state().borrow_mut();
        let mount = state
            .world
            .mounts
            .iter_mut()
            .find(|mount| mount.mount_id == 6006)
            .unwrap();
        mount.spell_id = 999998;
    }
    env.exec("CheckMount(458, nil); CheckMount('fixture mount', nil); CheckMount(999998, 6006)")
        .unwrap();
    assert!(read_inputs(&env).relations.contains(&(999998, 6006)));
}

#[test]
fn removal_of_actual_mount_record_invalidates_numeric_and_alias_queries() {
    let env = create_env();
    insert_alias(&env, "fixture mount", 458);
    env.exec("CheckMount(458, 6); CheckMount('fixture mount', 6)")
        .unwrap();
    env.state()
        .borrow_mut()
        .world
        .mounts
        .retain(|mount| mount.mount_id != 6);
    env.exec("CheckMount(458, nil); CheckMount('fixture mount', nil); CheckMount(40192, 107)")
        .unwrap();
    assert!(!read_inputs(&env).relations.contains(&(458, 6)));
}

#[test]
fn alias_replace_remove_and_numeric_identity_recovery_are_live() {
    let env = create_env();
    insert_alias(&env, "fixture mount", 458);
    insert_alias(&env, "458", 40192);
    env.exec("CheckMount('fixture mount', 6); CheckMount(458, 107)")
        .unwrap();
    insert_alias(&env, "fixture mount", 40192);
    env.exec("CheckMount('fixture mount', 107)").unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.spell_id_aliases.remove("fixture mount");
        state.spell_id_aliases.remove("458");
    }
    env.exec("CheckMount('fixture mount', nil); CheckMount('458', nil); CheckMount(458, 6)")
        .unwrap();
    assert!(read_inputs(&env).aliases.is_empty());
}

#[test]
fn queries_are_read_only_and_alias_registries_are_environment_local() {
    let first = create_env();
    let second = create_env();
    insert_alias(&first, "fixture mount", 458);
    let first_before = read_inputs(&first);
    let second_before = read_inputs(&second);
    first
        .exec("CheckMount('fixture mount', 6); CheckMount(458, 6); CheckMount(999999, nil)")
        .unwrap();
    second
        .exec("CheckMount('fixture mount', nil); CheckMount(458, 6)")
        .unwrap();
    assert_eq!(read_inputs(&first), first_before);
    assert_eq!(read_inputs(&second), second_before);
    insert_alias(&first, "fixture mount", 40192);
    first.exec("CheckMount('fixture mount', 107)").unwrap();
    second.exec("CheckMount('fixture mount', nil)").unwrap();
    assert_eq!(read_inputs(&second), second_before);
}

#[test]
fn inferred_public_type_policy_rejects_missing_and_wrong_vm_types() {
    let env = create_env();
    let before = read_inputs(&env);
    env.exec(
        r#"
        RejectMount(); RejectMount(nil)
        local frame = CreateFrame('Frame')
        assert(frame:GetObjectType() == 'Frame')
        for _, value in ipairs({false, true, {}, function() end,
            coroutine.create(function() end), frame}) do
            RejectMount(value)
        end
        CheckMount(458, 6)
        "#,
    )
    .expect("INFERRED conservative public type policy, not native parity");
    assert_eq!(read_inputs(&env), before);
}

#[test]
fn inferred_public_domain_policy_validates_before_alias_coercion() {
    let env = create_env();
    for key in ["0", "458", "4294967295", "\u{fffd}"] {
        insert_alias(&env, key, 458);
    }
    let before = read_inputs(&env);
    env.exec(
        r#"
        for _, value in ipairs({0/0, math.huge, -math.huge, -1, 458.5,
            4294967296, string.char(255), string.char(192, 175)}) do
            RejectMount(value)
        end
        CheckMount(0, 6); CheckMount(4294967295, 6); CheckMount(458, 6)
        "#,
    )
    .expect("INFERRED finite integral u32 / UTF-8 policy, including valid endpoints");
    assert_eq!(read_inputs(&env), before);
}

#[test]
fn actual_vm_secrets_reject_as_unmodeled_policy_in_secure_and_tainted_calls() {
    let env = create_env();
    insert_alias(&env, "fixture mount", 458);
    insert_alias(&env, LINK, 458);
    install_host_secrets(&env);
    let before = read_inputs(&env);
    env.exec(
        r#"
        MountSecretRoots = {MountSecretNumber, MountSecretMissing, MountSecretName, MountSecretLink}
        collectgarbage('collect')
        local function probe()
            local before = debug.getstacktaint()
            for index, value in ipairs(MountSecretRoots) do
                assert(issecretvalue(value), 'actual VM secret before call')
                RejectMount(value)
                assert(issecretvalue(value), 'no declassification')
                assert(rawequal(value, MountSecretRoots[index]), 'no replacement')
                assert(debug.getstacktaint() == before, 'caller taint unchanged')
            end
            CheckMount(458, 6); CheckMount('fixture mount', 6); CheckMount(999999, nil)
            assert(debug.getstacktaint() == before, 'public recovery preserves context')
        end
        assert(issecure()); probe(); assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'MountIdentifierFixture')
            probe()
            assert(debug.getstacktaint() == 'MountIdentifierFixture')
        end
        debug.setobjecttaint(addon, 'MountIdentifierFixture')
        addon(); assert(issecure())
        "#,
    )
    .expect("AllowedWhenTainted UNMODELED: conservative simulator rejection, no native credit");
    assert_eq!(read_inputs(&env), before);
}
