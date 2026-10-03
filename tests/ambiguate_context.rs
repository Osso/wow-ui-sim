//! B74 exact415: Ambiguate context NeverSecret; row416 remains unmodeled.
//! Public mappings and rejection ordering/message are simulator requirements, not native parity.
#![cfg(feature = "retail-12-0-5")]

use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string,
};
use rilua::vm::value::Val;
use rilua::{LuaApi, LuaApiMut};
use wow_ui_sim::lua_api::WowLuaEnv;

const ASSERTIONS: &str = r#"
    function ACInCaller(addon, probe)
        assert(debug.getstacktaint() == nil, 'secure entry')
        if addon then
            local function caller()
                assert(debug.getstacktaint() == 'AmbiguateFixture', 'addon entry')
                probe()
                assert(debug.getstacktaint() == 'AmbiguateFixture', 'addon exit')
            end
            debug.setobjecttaint(caller, 'AmbiguateFixture')
            caller()
        else
            probe()
        end
        assert(debug.getstacktaint() == nil, 'secure caller restored')
    end
    function ACPublic(fullName, context, expected)
        local before = debug.getstacktaint()
        local result = Ambiguate(fullName, context)
        assert(type(result) == 'string' and result == expected, 'public mapping')
        assert(debug.getstacktaint() == before, 'public call preserves caller taint')
    end
    function ACReject(fullName, context)
        local before = debug.getstacktaint()
        assert(issecretvalue(context), 'authentic VM secret context')
        collectgarbage('collect')
        local ok, err = pcall(Ambiguate, fullName, context)
        assert(not ok, 'secret context must reject')
        assert(type(err) == 'string' and
            string.find(err, 'Ambiguate argument #2 must not be secret', 1, true),
            'arg2 marker must win before fullname parsing')
        assert(debug.getstacktaint() == before, 'rejection preserves caller taint')
        assert(issecretvalue(context), 'context remains secret')
        ACPublic('Alessio-Silvermoon', 'short', 'Alessio')
        ACPublic('Alessio-Silvermoon', 'none', 'Alessio-Silvermoon')
        collectgarbage('collect')
        assert(issecretvalue(context), 'context survives recovery and collection')
        assert(debug.getstacktaint() == before, 'recovery preserves caller taint')
    end
"#;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("initialize actual Ambiguate provider");
    env.exec(ASSERTIONS)
        .expect("install assertions without replacing Ambiguate");
    env
}

fn secret_env() -> WowLuaEnv {
    let env = fixture_env();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("actual VM secret helpers");
    // Publish one at a time: each wrapper is stack-rooted before global insertion.
    let number = wrap_host_secret_number(lua.state_mut(), 17.5);
    lua.state_mut().push(number);
    let inserted = lua.set_global_val("ACSecretNumber", number);
    lua.state_mut().pop();
    inserted.expect("root host NUM context");
    let boolean = wrap_host_secret_bool(lua.state_mut(), false);
    lua.state_mut().push(boolean);
    let inserted = lua.set_global_val("ACSecretBool", boolean);
    lua.state_mut().pop();
    inserted.expect("root host BOOL context");
    let string = wrap_host_secret_string(lua.state_mut(), "none");
    lua.state_mut().push(string);
    let inserted = lua.set_global_val("ACSecretString", string);
    lua.state_mut().pop();
    inserted.expect("root host STR context");
    drop(lua);
    env
}

fn snapshot_context_roots(env: &WowLuaEnv) -> Vec<(Val, u64)> {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    ["ACSecretNumber", "ACSecretBool", "ACSecretString"]
        .into_iter()
        .map(|name| {
            let value = lua.get_global_val(name);
            let Val::Userdata(reference) = value else {
                panic!("host secret context missing: {name}");
            };
            let sequence = lua
                .state_mut()
                .gc
                .userdata
                .get(reference)
                .expect("globally rooted context remains live")
                .alloc_seq();
            (value, sequence)
        })
        .collect()
}

fn reject_context(secret_global: &str, full_name_expression: &str, addon: bool) {
    let env = secret_env();
    // Host metadata only, not payload inspection or extra VM roots.
    let before = snapshot_context_roots(&env);
    let script = format!(
        "ACInCaller({addon}, function() ACReject({full_name_expression}, {secret_global}) end)"
    );
    env.exec(&script)
        .expect("reject secret arg2 before parsing arg1, retain taint and recover");
    assert_eq!(
        before,
        snapshot_context_roots(&env),
        "global wrapper identities and allocation liveness survive GC"
    );
}

macro_rules! context_rejections {
    ($( $name:ident: $secret:literal, $full_name:literal, $addon:literal; )*) => {
        $(
            #[test]
            fn $name() {
                reject_context($secret, $full_name, $addon);
            }
        )*
    };
}

context_rejections! {
    secure_num_context_rejects_with_valid_fullname: "ACSecretNumber", "'Alessio-Silvermoon'", false;
    secure_num_context_rejects_before_nil_fullname: "ACSecretNumber", "nil", false;
    secure_num_context_rejects_before_table_fullname: "ACSecretNumber", "{}", false;
    secure_bool_context_rejects_with_valid_fullname: "ACSecretBool", "'Alessio-Silvermoon'", false;
    secure_bool_context_rejects_before_nil_fullname: "ACSecretBool", "nil", false;
    secure_bool_context_rejects_before_table_fullname: "ACSecretBool", "{}", false;
    secure_str_context_rejects_with_valid_fullname: "ACSecretString", "'Alessio-Silvermoon'", false;
    secure_str_context_rejects_before_nil_fullname: "ACSecretString", "nil", false;
    secure_str_context_rejects_before_table_fullname: "ACSecretString", "{}", false;
    addon_num_context_rejects_with_valid_fullname: "ACSecretNumber", "'Alessio-Silvermoon'", true;
    addon_num_context_rejects_before_nil_fullname: "ACSecretNumber", "nil", true;
    addon_num_context_rejects_before_table_fullname: "ACSecretNumber", "{}", true;
    addon_bool_context_rejects_with_valid_fullname: "ACSecretBool", "'Alessio-Silvermoon'", true;
    addon_bool_context_rejects_before_nil_fullname: "ACSecretBool", "nil", true;
    addon_bool_context_rejects_before_table_fullname: "ACSecretBool", "{}", true;
    addon_str_context_rejects_with_valid_fullname: "ACSecretString", "'Alessio-Silvermoon'", true;
    addon_str_context_rejects_before_nil_fullname: "ACSecretString", "nil", true;
    addon_str_context_rejects_before_table_fullname: "ACSecretString", "{}", true;
}

#[test]
fn public_short_and_other_contexts_preserve_existing_hyphen_pattern() {
    fixture_env()
        .exec(
            r#"
            ACInCaller(false, function()
                ACPublic('Alessio-Silvermoon', 'short', 'Alessio')
                ACPublic('Alessio', 'short', 'Alessio')
                ACPublic('Alessio-Silvermoon-EU', 'short', 'Alessio')
                ACPublic('Alessio-', 'short', 'Alessio-')
                ACPublic('Alessio-Silvermoon', 'fixture-context', 'Alessio')
                ACPublic('Alessio-Silvermoon', '', 'Alessio')
            end)
            "#,
        )
        .expect("non-none public contexts retain existing simulator mapping");
}

#[test]
fn public_none_preserves_fullname_with_or_without_hyphens() {
    fixture_env()
        .exec(
            r#"
            ACInCaller(false, function()
                ACPublic('Alessio-Silvermoon', 'none', 'Alessio-Silvermoon')
                ACPublic('Alessio', 'none', 'Alessio')
                ACPublic('Alessio-Silvermoon-EU', 'none', 'Alessio-Silvermoon-EU')
                ACPublic('Alessio-', 'none', 'Alessio-')
            end)
            "#,
        )
        .expect("none retains the full public name");
}

#[test]
fn addon_public_short_transforms_without_changing_caller_taint() {
    fixture_env()
        .exec(
            r#"
            ACInCaller(true, function()
                ACPublic('Alessio-Silvermoon', 'short', 'Alessio')
                ACPublic('Alessio', 'short', 'Alessio')
                ACPublic('Alessio-Silvermoon-EU', 'short', 'Alessio')
                ACPublic('Alessio-', 'short', 'Alessio-')
            end)
            "#,
        )
        .expect("addon public short mapping retains addon stack taint");
}

#[test]
fn addon_public_none_retains_fullname_without_changing_caller_taint() {
    fixture_env()
        .exec(
            r#"
            ACInCaller(true, function()
                ACPublic('Alessio-Silvermoon', 'none', 'Alessio-Silvermoon')
                ACPublic('Alessio', 'none', 'Alessio')
                ACPublic('Alessio-Silvermoon-EU', 'none', 'Alessio-Silvermoon-EU')
                ACPublic('Alessio-', 'none', 'Alessio-')
            end)
            "#,
        )
        .expect("addon public none mapping retains addon stack taint");
}
