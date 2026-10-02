//! Batch50, exact row328: arg2 NeverSecret only.
//! Rejection and strict validation are INFERRED simulator policies, not native parity.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string, wrap_secret};
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create space-limit environment");
    env.exec(
        r#"
        SpaceLimitFrame = CreateFrame('Frame')
        assert(SpaceLimitFrame:GetObjectType() == 'Frame')
        SpaceLimitFrame:SetAlpha(0.625)
        SpaceLimitFrame.marker = 37
        SpaceLimitTable = {marker = 53, text = ' table  sentinel '}
        SpaceLimitCaller = {text = ' a   b  c ', limit = 1, marker = 71}
        function CheckSpaceResult(text, limit, expected)
            local function check(...)
                assert(select('#', ...) == 1, 'exactly one result')
                local actual = ...
                assert(not issecretvalue(actual), 'public result remains unwrapped')
                assert(type(actual) == 'string' and actual == expected, 'byte-exact string')
                return actual
            end
            return check(C_StringUtil.RemoveContiguousSpaces(text, limit))
        end
        function CheckSpaceState()
            assert(SpaceLimitFrame:GetObjectType() == 'Frame')
            assert(SpaceLimitFrame:GetAlpha() == 0.625 and SpaceLimitFrame.marker == 37)
            assert(SpaceLimitTable.marker == 53 and SpaceLimitTable.text == ' table  sentinel ')
            assert(SpaceLimitCaller.text == ' a   b  c ')
            assert(SpaceLimitCaller.limit == 1 and SpaceLimitCaller.marker == 71)
        end
        function CheckSpaceRecovery()
            CheckSpaceResult(SpaceLimitCaller.text, SpaceLimitCaller.limit, ' a b c ')
            CheckSpaceState()
        end
        function RejectSpaceLimit(text, value)
            local before = debug.getstacktaint()
            local wasSecret = issecretvalue(value)
            assert(type(C_StringUtil.RemoveContiguousSpaces) == 'function', 'actual C API required')
            local ok, err = pcall(C_StringUtil.RemoveContiguousSpaces, text, value)
            assert(not ok and type(err) == 'string' and #err > 0, 'limit must reject')
            assert(issecretvalue(value) == wasSecret, 'no declassification')
            assert(debug.getstacktaint() == before, 'rejection preserves caller taint')
            CheckSpaceRecovery()
            assert(debug.getstacktaint() == before, 'public recovery preserves caller taint')
        end
        function InSpaceCallerContexts(probe)
            assert(issecure(), 'secure control context')
            probe()
            assert(issecure(), 'secure context preserved')
            local function addon()
                assert(debug.getstacktaint() == 'SpaceLimitFixture')
                probe()
                assert(debug.getstacktaint() == 'SpaceLimitFixture', 'tainted context preserved')
            end
            debug.setobjecttaint(addon, 'SpaceLimitFixture')
            addon()
            assert(issecure(), 'addon return restores secure caller')
        end
        "#,
    )
    .expect("install assertions and sentinel inputs without replacing C API");
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("actual VM security helpers");
    for (name, number) in [
        ("SpaceSecretZero", 0.0),
        ("SpaceSecretOne", 1.0),
        ("SpaceSecretTwo", 2.0),
        ("SpaceSecretLarge", 1e100),
    ] {
        let wrapper = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(wrapper);
        let inserted = lua.set_global_val(name, wrapper);
        lua.state_mut().pop();
        inserted.expect("root actual host-secret NUMBER during publication");
    }
    let wrapper = wrap_host_secret_string(lua.state_mut(), "1");
    lua.state_mut().push(wrapper);
    let inserted = lua.set_global_val("SpaceSecretNumericString", wrapper);
    lua.state_mut().pop();
    inserted.expect("root actual host-secret STRING during publication");
    for (original_name, secret_name) in [
        ("SpaceLimitFrame", "SpaceSecretFrame"),
        ("SpaceLimitTable", "SpaceSecretTable"),
    ] {
        let original = lua.get_global_val(original_name);
        assert!(
            !original.is_nil(),
            "actual wrong object remains globally rooted"
        );
        lua.state_mut().push(original);
        let wrapper = wrap_secret(lua.state_mut(), original).expect("wrap actual wrong object");
        lua.state_mut().push(wrapper);
        let inserted = lua.set_global_val(secret_name, wrapper);
        lua.state_mut().pop();
        lua.state_mut().pop();
        inserted.expect("root original and secret wrapper during publication");
    }
    drop(lua);
    env.exec(
        r#"
        SpaceSecretNames = {'SpaceSecretZero', 'SpaceSecretOne', 'SpaceSecretTwo',
            'SpaceSecretLarge', 'SpaceSecretNumericString', 'SpaceSecretFrame', 'SpaceSecretTable'}
        SpaceSecretValues = {}
        for i, name in ipairs(SpaceSecretNames) do
            local value = _G[name]
            assert(issecretvalue(value), 'real VM secret, never Lua marker')
            SpaceSecretValues[i] = value
        end
        function CheckSpaceSecretRoots()
            assert(#SpaceSecretValues == 7 and #SpaceSecretNames == 7)
            for i, name in ipairs(SpaceSecretNames) do
                assert(issecretvalue(SpaceSecretValues[i]) and issecretvalue(_G[name]))
                assert(rawequal(SpaceSecretValues[i], _G[name]), 'wrapper identity unchanged')
            end
        end
        "#,
    )
    .expect("retain global and list roots without inspecting private payloads");
}

fn secret_env() -> WowLuaEnv {
    let env = fixture_env();
    install_host_secrets(&env);
    env
}

#[test]
fn secret_number_limits_reject_on_ordinary_valid_text_in_both_contexts() {
    secret_env()
        .exec(
            r#"
            InSpaceCallerContexts(function()
                CheckSpaceResult(' a   b  c ', 1e100, ' a   b  c ')
                for _, value in ipairs({SpaceSecretZero, SpaceSecretOne, SpaceSecretTwo, SpaceSecretLarge}) do
                    assert(issecretvalue(value))
                    RejectSpaceLimit(' a   b  c ', value)
                end
                CheckSpaceSecretRoots()
            end)
            "#,
        )
        .expect("arg2 secret NUMBER rejects, not a missing or invalid text failure");
}

#[test]
fn empty_text_cannot_short_circuit_secret_number_limit_rejection() {
    secret_env()
        .exec(
            r#"
            InSpaceCallerContexts(function()
                for _, value in ipairs({SpaceSecretZero, SpaceSecretOne, SpaceSecretTwo, SpaceSecretLarge}) do
                    RejectSpaceLimit('', value)
                end
                CheckSpaceSecretRoots()
            end)
            "#,
        )
        .expect("empty valid STRING still authenticates arg2 in both contexts");
}

#[test]
fn space_free_text_cannot_short_circuit_secret_number_limit_rejection() {
    secret_env()
        .exec(
            r#"
            InSpaceCallerContexts(function()
                for _, value in ipairs({SpaceSecretZero, SpaceSecretOne, SpaceSecretTwo, SpaceSecretLarge}) do
                    RejectSpaceLimit('plain-text', value)
                end
                CheckSpaceSecretRoots()
            end)
            "#,
        )
        .expect("space-free valid STRING still authenticates arg2 in both contexts");
}

#[test]
fn secret_numeric_string_limit_rejects_without_coercion_in_both_contexts() {
    secret_env()
        .exec(
            r#"
            InSpaceCallerContexts(function()
                assert(issecretvalue(SpaceSecretNumericString))
                RejectSpaceLimit(' a   b  c ', SpaceSecretNumericString)
                CheckSpaceSecretRoots()
            end)
            "#,
        )
        .expect("actual secret STRING numeric-like payload does not become public NUMBER");
}

#[test]
fn wrapped_actual_frame_and_table_limits_reject_in_both_contexts() {
    secret_env()
        .exec(
            r#"
            InSpaceCallerContexts(function()
                assert(SpaceLimitFrame:GetObjectType() == 'Frame')
                assert(type(SpaceLimitTable) == 'table' and SpaceLimitTable.marker == 53)
                RejectSpaceLimit(' a   b  c ', SpaceSecretFrame)
                RejectSpaceLimit(' a   b  c ', SpaceSecretTable)
                CheckSpaceSecretRoots()
            end)
            "#,
        )
        .expect("actual wrapped wrong objects reject without assuming Frame VM representation");
}

#[test]
fn forced_gc_preserves_rooted_secret_identity_state_and_public_recovery() {
    secret_env()
        .exec(
            r#"
            local originalFrame, originalTable = SpaceLimitFrame, SpaceLimitTable
            collectgarbage('collect')
            CheckSpaceSecretRoots()
            InSpaceCallerContexts(function()
                local before = debug.getstacktaint()
                for _, value in ipairs(SpaceSecretValues) do
                    RejectSpaceLimit(' x    y ', value)
                end
                collectgarbage('collect')
                CheckSpaceSecretRoots()
                assert(rawequal(originalFrame, SpaceLimitFrame))
                assert(rawequal(originalTable, SpaceLimitTable))
                CheckSpaceRecovery()
                assert(debug.getstacktaint() == before)
            end)
            collectgarbage('collect')
            CheckSpaceSecretRoots()
            CheckSpaceState()
            "#,
        )
        .expect(
            "forced collection does not declassify, replace roots, mutate inputs or taint callers",
        );
}

#[test]
fn public_three_limit_matrix_preserves_non_space_bytes_and_single_public_string() {
    let env = fixture_env();
    env.exec(
        r#"
        local suffix = '\t\n\0' .. string.char(194, 160, 255)
        local text = ' A   B  ' .. suffix .. '  C '
        local cases = {
            {0, 'AB' .. suffix .. 'C'},
            {1, ' A B ' .. suffix .. ' C '},
            {2, ' A  B  ' .. suffix .. '  C '},
        }
        InSpaceCallerContexts(function()
            local before = debug.getstacktaint()
            for _, case in ipairs(cases) do
                SpacePublicResult = CheckSpaceResult(text, case[1], case[2])
            end
            CheckSpaceRecovery()
            assert(debug.getstacktaint() == before)
        end)
        "#,
    )
    .expect("representative ASCII control matrix, not generalized whitespace normalization");
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    assert!(
        matches!(lua.get_global_val("SpacePublicResult"), rilua::Val::Str(_)),
        "public result is an actual unwrapped VM STRING"
    );
}

#[test]
fn retained_public_strict_limit_policy_rejects_and_recovers_in_both_contexts() {
    fixture_env()
        .exec(
            r#"
            InSpaceCallerContexts(function()
                -- Existing simulator policy only; not native validation evidence.
                for _, value in ipairs({'1', false, {}, SpaceLimitFrame, -1, 0.5,
                    0/0, math.huge, -math.huge}) do
                    RejectSpaceLimit(' a   b  c ', value)
                end
                RejectSpaceLimit(' a   b  c ', nil)
                local before = debug.getstacktaint()
                local ok, err = pcall(C_StringUtil.RemoveContiguousSpaces, ' a   b  c ')
                assert(not ok and type(err) == 'string' and #err > 0, 'required arg2')
                CheckSpaceRecovery()
                assert(debug.getstacktaint() == before)
            end)
            "#,
        )
        .expect(
            "existing required strict finite nonnegative integral NUMBER policy remains unchanged",
        );
}
