//! Source311: chosen arg2 NeverSecret boundary, not native permission parity.
//! Tests/spec inputs only; parent owns compiled RED before production changes.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string, wrap_secret,
};
use rilua::vm::value::Val;
use wow_ui_sim::lua_api::WowLuaEnv;

const SECRET_NAMES: [&str; 6] = [
    "GLSecretFalse",
    "GLSecretTrue",
    "GLSecretNumber",
    "GLSecretString",
    "GLSecretFrame",
    "GLSecretTable",
];

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("spell-link fixture startup");
    env.exec(
        r#"
        GLLink = '|cff71d5ff|Hspell:19750|h[Flash of Light]|h|r'
        GLFrame = CreateFrame('Frame')
        GLFrame:SetAlpha(0.625)
        GLFrame.marker = 311
        GLInputs = {identifier = 19750, marker = 37, flag = false}
        function GLState()
            assert(GLFrame:GetObjectType() == 'Frame')
            assert(GLFrame:GetAlpha() == 0.625 and GLFrame.marker == 311)
            assert(GLInputs.identifier == 19750 and GLInputs.marker == 37)
            assert(GLInputs.flag == false)
        end
        function GLRecovery()
            assert(C_Spell.GetSpellLink(19750) == GLLink, 'real generated spell link')
            assert(C_Spell.GetSpellLink(4294967295) == nil, 'real catalog miss')
            GLState()
        end
        function GLContexts(probe)
            assert(issecure(), 'secure caller')
            probe()
            assert(issecure(), 'secure caller preserved')
            local function addon()
                assert(debug.getstacktaint() == 'Glyph311Fixture')
                probe()
                assert(debug.getstacktaint() == 'Glyph311Fixture', 'addon taint preserved')
            end
            debug.setobjecttaint(addon, 'Glyph311Fixture')
            addon()
            assert(issecure(), 'return restores secure caller')
        end
        function GLReject(identifier, glyph)
            assert(issecretvalue(glyph), 'authentic host VM secret')
            local before = debug.getstacktaint()
            local frame, inputs = GLFrame, GLInputs
            local ok, err = pcall(C_Spell.GetSpellLink, identifier, glyph)
            assert(not ok and type(err) == 'string' and #err > 0,
                'secret arg2 rejected rather than provider link or nil')
            assert(not string.find(err, 'PRIVATE-Glyph311', 1, true), 'no private text leak')
            assert(not string.find(err, '987654321', 1, true), 'no private number leak')
            assert(debug.getstacktaint() == before, 'rejection preserves caller taint')
            assert(issecretvalue(glyph), 'no declassification')
            assert(rawequal(frame, GLFrame) and rawequal(inputs, GLInputs))
            GLRecovery()
            assert(debug.getstacktaint() == before, 'public recovery preserves taint')
        end
        function GLRejectMatrix(glyph)
            GLReject(19750, glyph)
            GLReject(4294967295, glyph)
            -- Invalid arg1 is not newly validated: secret arg2 must still reject.
            GLReject(nil, glyph)
            GLReject(false, glyph)
            GLReject('PRIVATE-Glyph311-Identifier', glyph)
            GLReject(GLFrame, glyph)
            GLReject(GLInputs, glyph)
        end
        "#,
    )
    .expect("real API, frame and public controls; no replaced callbacks");
    env
}

fn publish_secret(lua: &mut impl LuaApiMut, name: &str, value: Val) {
    lua.state_mut().push(value);
    let inserted = lua.set_global_val(name, value);
    lua.state_mut().pop();
    inserted.expect("root host secret during publication");
}

fn secret_env() -> WowLuaEnv {
    let env = fixture_env();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("actual VM security helpers");
    for (name, payload) in [("GLSecretFalse", false), ("GLSecretTrue", true)] {
        let value = wrap_host_secret_bool(lua.state_mut(), payload);
        publish_secret(&mut *lua, name, value);
    }
    let number = wrap_host_secret_number(lua.state_mut(), 987654321.0);
    publish_secret(&mut *lua, "GLSecretNumber", number);
    let text = wrap_host_secret_string(lua.state_mut(), "PRIVATE-Glyph311-Payload");
    publish_secret(&mut *lua, "GLSecretString", text);
    for (original_name, secret_name) in
        [("GLFrame", "GLSecretFrame"), ("GLInputs", "GLSecretTable")]
    {
        let original = lua.get_global_val(original_name);
        let Val::Table(reference) = original else {
            panic!("original {original_name} must be an actual table");
        };
        if original_name == "GLFrame" {
            assert!(
                lua.state_mut()
                    .gc
                    .tables
                    .get(reference)
                    .expect("live frame table")
                    .backing()
                    .is_some(),
                "wrap real frame backing, not a fabricated frame-shaped table"
            );
        }
        lua.state_mut().push(original);
        let value = wrap_secret(lua.state_mut(), original).expect("wrap real frame/table");
        publish_secret(&mut *lua, secret_name, value);
        lua.state_mut().pop();
    }
    drop(lua);
    env.exec(
        r#"
        GLSecrets = {GLSecretFalse, GLSecretTrue, GLSecretNumber,
            GLSecretString, GLSecretFrame, GLSecretTable}
        function GLSecrecy()
            assert(#GLSecrets == 6)
            for _, value in ipairs(GLSecrets) do assert(issecretvalue(value)) end
            assert(issecretvalue(GLSecretFalse) and issecretvalue(GLSecretTrue))
            assert(issecretvalue(GLSecretNumber) and issecretvalue(GLSecretString))
            assert(issecretvalue(GLSecretFrame) and issecretvalue(GLSecretTable))
        end
        "#,
    )
    .expect("retain real wrapper roots without Lua secret equality");
    env
}

fn snapshot_roots(env: &WowLuaEnv) -> Vec<(Val, u64)> {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    SECRET_NAMES
        .iter()
        .map(|name| {
            let value = lua.get_global_val(name);
            let Val::Userdata(reference) = value else {
                panic!("host secret wrapper missing: {name}");
            };
            let sequence = lua
                .state_mut()
                .gc
                .userdata
                .get(reference)
                .expect("wrapper remains live")
                .alloc_seq();
            (value, sequence)
        })
        .collect()
}

fn exec_secret_probe(script: &str) {
    let env = secret_env();
    let roots = snapshot_roots(&env); // Host metadata only; no payload reads or VM roots.
    let aliases = env.state().borrow().spell_id_aliases.clone();
    env.exec(script)
        .expect("arg2 rejection, caller preservation and public recovery");
    assert_eq!(snapshot_roots(&env), roots, "same live host wrappers");
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let Val::Table(list) = lua.get_global_val("GLSecrets") else {
        panic!("secret list root missing");
    };
    for (index, (value, _)) in roots.iter().enumerate() {
        assert_eq!(
            lua.state_mut()
                .gc
                .tables
                .get(list)
                .expect("root list remains live")
                .get_int(index as i64 + 1),
            *value,
            "host list wrapper identity; never tainted BOOL rawequal"
        );
    }
    drop(lua);
    assert_eq!(
        env.state().borrow().spell_id_aliases,
        aliases,
        "aliases unchanged"
    );
}

#[test]
fn spell_link_glyph_known_spell_retains_exact_real_link() {
    fixture_env()
        .exec("GLContexts(function() assert(C_Spell.GetSpellLink(19750) == GLLink) end)")
        .expect("existing positive provider output in secure and addon contexts");
}

#[test]
fn spell_link_glyph_unknown_spell_retains_nil() {
    fixture_env()
        .exec("GLContexts(function() assert(C_Spell.GetSpellLink(4294967295) == nil) end)")
        .expect("existing provider miss in secure and addon contexts");
}

#[test]
fn spell_link_glyph_public_optionals_remain_ignored() {
    fixture_env()
        .exec(
            r#"
            GLContexts(function()
                assert(C_Spell.GetSpellLink(19750, nil) == GLLink)
                assert(C_Spell.GetSpellLink(4294967295, nil) == nil)
                for _, glyph in ipairs({0, 123, -1, true, false, 'ordinary', GLFrame, GLInputs}) do
                    assert(not issecretvalue(glyph))
                    assert(C_Spell.GetSpellLink(19750, glyph) == GLLink)
                    assert(C_Spell.GetSpellLink(4294967295, glyph) == nil)
                end
                GLRecovery()
            end)
            "#,
        )
        .expect("preserve current ignored public arg2 behavior without new glyph validation");
}

#[test]
fn spell_link_glyph_secret_bools_reject_before_known_miss_invalid_identifier() {
    exec_secret_probe(
        "GLContexts(function() GLRejectMatrix(GLSecretFalse); GLRejectMatrix(GLSecretTrue); GLSecrecy() end)",
    );
}

#[test]
fn spell_link_glyph_secret_number_rejects_before_known_miss_invalid_identifier() {
    exec_secret_probe("GLContexts(function() GLRejectMatrix(GLSecretNumber); GLSecrecy() end)");
}

#[test]
fn spell_link_glyph_secret_string_rejects_without_payload_leak() {
    exec_secret_probe("GLContexts(function() GLRejectMatrix(GLSecretString); GLSecrecy() end)");
}

#[test]
fn spell_link_glyph_secret_real_frame_and_table_reject_without_mutation() {
    exec_secret_probe(
        "GLContexts(function() GLRejectMatrix(GLSecretFrame); GLRejectMatrix(GLSecretTable); GLSecrecy() end)",
    );
}

#[test]
fn spell_link_glyph_forced_gc_preserves_wrappers_and_public_recovery() {
    exec_secret_probe(
        r#"
        GLContexts(function()
            local before = debug.getstacktaint()
            for _, glyph in ipairs(GLSecrets) do
                GLRejectMatrix(glyph)
                collectgarbage('collect')
                GLSecrecy()
                GLReject(19750, glyph)
                GLRecovery()
                assert(debug.getstacktaint() == before, 'GC/recovery preserves caller taint')
            end
            collectgarbage('collect')
            GLRecovery()
        end)
        "#,
    );
}
