//! Next53 exact411: INFERRED decimal/truncation contract, not native parity.
//! Locale providers are explicit fixture inputs; the API under test is never replaced.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string, wrap_secret,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("startup must permit the desired formatter");
    env.exec(
        r#"
        BULocale = 'enUS'
        BULocaleReads = 0
        -- Explicit current-global provider input, not native locale acquisition.
        function GetLocale()
            BULocaleReads = BULocaleReads + 1
            return BULocale
        end
        BUFrame = CreateFrame('Frame')
        assert(BUFrame:GetObjectType() == 'Frame')
        BUFrame:SetAlpha(0.625)
        BUFrame.marker = 37
        BUInputs = {number = 1234.5, natural = false, marker = 53}
        function BUResult(expected, ...)
            assert(select('#', ...) == 1, 'one result')
            local actual = ...
            assert(not issecretvalue(actual), 'inferred public result')
            assert(type(actual) == 'string' and actual == expected, 'localized decimal bytes')
        end
        function BUState()
            assert(BUFrame:GetObjectType() == 'Frame')
            assert(BUFrame:GetAlpha() == 0.625 and BUFrame.marker == 37)
            assert(BUInputs.number == 1234.5 and BUInputs.natural == false)
            assert(BUInputs.marker == 53)
        end
        function BURecovery()
            BUResult('1,234.5', BreakUpLargeNumbers(1234.5, false))
            BUState()
        end
        function BUReject(...)
            local before = debug.getstacktaint()
            assert(type(BreakUpLargeNumbers) == 'function', 'actual global required')
            local ok, err = pcall(BreakUpLargeNumbers, ...)
            assert(not ok and type(err) == 'string' and #err > 0, 'strict rejection')
            assert(debug.getstacktaint() == before, 'rejection preserves taint')
        end
        function BURejectSecretNatural(value)
            assert(issecretvalue(value), 'authentic VM secret')
            local reads = BULocaleReads
            BUReject(1234.5, value)
            assert(BULocaleReads == reads, 'reject before locale acquisition/formatting')
            assert(issecretvalue(value), 'no declassification')
            BURecovery()
        end
        function BUContexts(probe)
            assert(issecure(), 'secure control')
            probe()
            assert(issecure(), 'secure caller preserved')
            local function addon()
                assert(debug.getstacktaint() == 'BreakUpFixture')
                probe()
                assert(debug.getstacktaint() == 'BreakUpFixture', 'addon taint preserved')
            end
            debug.setobjecttaint(addon, 'BreakUpFixture')
            addon()
            assert(issecure(), 'return restores secure caller')
        end
        "#,
    )
    .expect("install provider and assertions without replacing formatter");
    env
}

fn secret_env() -> WowLuaEnv {
    let env = fixture_env();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("actual VM security helpers");
    for (name, payload) in [("BUSecretFalse", false), ("BUSecretTrue", true)] {
        let wrapper = wrap_host_secret_bool(lua.state_mut(), payload);
        lua.state_mut().push(wrapper);
        let inserted = lua.set_global_val(name, wrapper);
        lua.state_mut().pop();
        inserted.expect("root authentic secret BOOL during publication");
    }
    let number = wrap_host_secret_number(lua.state_mut(), 1234.5);
    lua.state_mut().push(number);
    let inserted = lua.set_global_val("BUSecretNumber", number);
    lua.state_mut().pop();
    inserted.expect("root authentic secret NUMBER");
    let text = wrap_host_secret_string(lua.state_mut(), "true");
    lua.state_mut().push(text);
    let inserted = lua.set_global_val("BUSecretString", text);
    lua.state_mut().pop();
    inserted.expect("root authentic secret STRING");
    for (original_name, secret_name) in
        [("BUFrame", "BUSecretFrame"), ("BUInputs", "BUSecretTable")]
    {
        let original = lua.get_global_val(original_name);
        assert!(!original.is_nil(), "original object is globally rooted");
        lua.state_mut().push(original);
        let wrapper = wrap_secret(lua.state_mut(), original).expect("wrap actual object");
        lua.state_mut().push(wrapper);
        let inserted = lua.set_global_val(secret_name, wrapper);
        lua.state_mut().pop();
        lua.state_mut().pop();
        inserted.expect("root original and wrapper during publication");
    }
    drop(lua);
    env.exec(
        r#"
        BUSecretNames = {'BUSecretFalse', 'BUSecretTrue', 'BUSecretNumber',
            'BUSecretString', 'BUSecretFrame', 'BUSecretTable'}
        BUSecretValues = {}
        for i, name in ipairs(BUSecretNames) do
            assert(issecretvalue(_G[name]))
            BUSecretValues[i] = _G[name]
        end
        function BUSecretRoots()
            assert(#BUSecretValues == 6)
            for i, name in ipairs(BUSecretNames) do
                assert(issecretvalue(_G[name]) and issecretvalue(BUSecretValues[i]))
                assert(rawequal(_G[name], BUSecretValues[i]), 'wrapper identity preserved')
            end
        end
        "#,
    )
    .expect("retain authentic global/list roots without private payload inspection");
    env
}

#[test]
fn startup_formatter_returns_one_public_grouped_string_with_default_agreement() {
    fixture_env()
        .exec(
            r#"
        BUResult('1,234', BreakUpLargeNumbers(1234))
        BUResult('1,234.5', BreakUpLargeNumbers(1234.5))
        BUResult('1,234.5', BreakUpLargeNumbers(1234.5, nil))
        BUResult('1,234.5', BreakUpLargeNumbers(1234.5, false))
    "#,
        )
        .expect("desired grouped formatter survives WowLuaEnv startup");
}

#[test]
fn inferred_natural_truncates_both_signs_toward_zero_not_floor() {
    fixture_env()
        .exec(
            r#"
        BUResult('1,234', BreakUpLargeNumbers(1234.75, true))
        BUResult('-1,234', BreakUpLargeNumbers(-1234.75, true))
        BUResult('1,234.75', BreakUpLargeNumbers(1234.75, false))
        BUResult('-1,234.75', BreakUpLargeNumbers(-1234.75, false))
        BUResult('0', BreakUpLargeNumbers(0.75, true))
        BUResult('-1', BreakUpLargeNumbers(-1.75, true))
    "#,
        )
        .expect("explicit guess: observable natural branch, not native semantics");
}

#[test]
fn inferred_zero_and_group_boundaries_have_decimal_not_abbreviated_output() {
    fixture_env()
        .exec(
            r#"
        for _, row in ipairs({{0, '0'}, {999, '999'}, {1000, '1,000'},
            {-1000, '-1,000'}, {1000000, '1,000,000'}}) do
            BUResult(row[2], BreakUpLargeNumbers(row[1], false))
        end
    "#,
        )
        .expect("finite concrete boundaries use grouping");
}

#[test]
fn current_wow_locale_provider_mutations_apply_immediately() {
    fixture_env()
        .exec(
            r#"
        BUResult('1,234.5', BreakUpLargeNumbers(1234.5))
        BULocale = 'deDE'
        BUResult('1.234,5', BreakUpLargeNumbers(1234.5))
        BUResult('1.234', BreakUpLargeNumbers(1234.75, true))
        BULocale = 'frFR'
        BUResult('1\226\128\175234,5', BreakUpLargeNumbers(1234.5))
        BULocale = 'enUS'
        BURecovery()
    "#,
        )
        .expect("explicit WoW locale inputs, narrow no-break space in French");
}

#[test]
fn formatter_keeps_provider_identity_and_caller_inputs_read_only() {
    fixture_env()
        .exec(
            r#"
        local provider, inputs, frame = GetLocale, BUInputs, BUFrame
        BUResult('1,234.5', BreakUpLargeNumbers(inputs.number, inputs.natural))
        assert(rawequal(provider, GetLocale) and BULocale == 'enUS')
        assert(rawequal(inputs, BUInputs) and rawequal(frame, BUFrame))
        BUState()
    "#,
        )
        .expect("formatter reads explicit inputs without mutating provider or objects");
}

#[test]
fn locale_and_input_fixtures_are_isolated_between_environments() {
    let german = fixture_env();
    let english = fixture_env();
    german
        .exec("BULocale = 'deDE'; BUInputs.marker = 99")
        .expect("mutate one fixture");
    german
        .exec("BUResult('1.234,5', BreakUpLargeNumbers(1234.5)); assert(BUInputs.marker == 99)")
        .expect("first environment keeps its own provider");
    english
        .exec("assert(BULocale == 'enUS'); BURecovery()")
        .expect("second environment does not inherit locale or input mutations");
}

#[test]
fn required_public_number_rejects_missing_nil_and_wrong_types_then_recovers() {
    fixture_env()
        .exec(
            r#"
        BUReject()
        BUReject(nil, false)
        for _, value in ipairs({'1234.5', true, false, {}, function() end, BUFrame}) do
            BUReject(value, false)
            BURecovery()
        end
    "#,
        )
        .expect("strict number policy, no string coercion or object stringification");
}

#[test]
fn nonfinite_numbers_reject_in_both_natural_modes_then_recover() {
    fixture_env()
        .exec(
            r#"
        for _, value in ipairs({0/0, 1/0, -1/0}) do
            BUReject(value, false)
            BUReject(value, true)
            BURecovery()
        end
    "#,
        )
        .expect("inferred finite-only formatter policy");
}

#[test]
fn optional_public_natural_rejects_nonbool_values_then_recovers() {
    fixture_env()
        .exec(
            r#"
        for _, value in ipairs({0, 1, 'false', 'true', {}, function() end, BUFrame}) do
            BUReject(1234.5, value)
            BURecovery()
        end
    "#,
        )
        .expect("optional bool accepts neither numeric nor truthy coercion");
}

#[test]
fn authentic_secret_false_natural_rejects_even_in_secure_context() {
    secret_env()
        .exec("assert(issecure()); BURejectSecretNatural(BUSecretFalse); BUSecretRoots()")
        .expect("NeverSecret does not permit decoding false payload");
}

#[test]
fn authentic_secret_true_natural_rejects_even_in_secure_context() {
    secret_env()
        .exec("assert(issecure()); BURejectSecretNatural(BUSecretTrue); BUSecretRoots()")
        .expect("NeverSecret does not permit decoding true payload");
}

#[test]
fn secret_number_and_string_natural_reject_before_formatting() {
    secret_env()
        .exec(
            r#"
        BURejectSecretNatural(BUSecretNumber)
        BURejectSecretNatural(BUSecretString)
        BUSecretRoots()
    "#,
        )
        .expect("wrong secret payload types remain opaque");
}

#[test]
fn wrapped_actual_frame_and_table_natural_reject_without_representation_assumptions() {
    secret_env()
        .exec(
            r#"
        assert(BUFrame:GetObjectType() == 'Frame')
        BURejectSecretNatural(BUSecretFrame)
        BURejectSecretNatural(BUSecretTable)
        BUSecretRoots()
    "#,
        )
        .expect("actual objects, not presumed userdata or Lua secret markers");
}

#[test]
fn gc_keeps_global_list_and_stack_secret_roots_identical_and_secret() {
    secret_env()
        .exec(
            r#"
        local held = BUSecretTrue
        local frame, inputs = BUFrame, BUInputs
        collectgarbage('collect')
        BUSecretRoots()
        for _, value in ipairs(BUSecretValues) do
            collectgarbage('collect')
            BURejectSecretNatural(value)
            assert(issecretvalue(value))
        end
        collectgarbage('collect')
        assert(issecretvalue(held) and rawequal(held, BUSecretTrue))
        assert(rawequal(frame, BUFrame) and rawequal(inputs, BUInputs))
        BUSecretRoots()
        BURecovery()
    "#,
        )
        .expect("forced GC preserves roots, identity, state and fresh public recovery");
}

#[test]
fn secure_and_stamped_tainted_calls_keep_taint_across_rejection_and_recovery() {
    secret_env()
        .exec(
            r#"
        BUContexts(function()
            local before = debug.getstacktaint()
            for _, value in ipairs(BUSecretValues) do BURejectSecretNatural(value) end
            BUReject(1234.5, 'true')
            BURecovery()
            BUResult('1,234', BreakUpLargeNumbers(1234.75, true))
            assert(debug.getstacktaint() == before)
            BUSecretRoots()
        end)
    "#,
        )
        .expect("public formatting and all secret natural rejections preserve caller context");
}

#[test]
fn secret_arg1_rejection_is_conservative_local_policy_not_row411_permission() {
    secret_env()
        .exec(
            r#"
        BUContexts(function()
            BUReject(BUSecretNumber, false)
            assert(issecretvalue(BUSecretNumber))
            BUSecretRoots()
            BURecovery()
        end)
    "#,
        )
        .expect("local conservative arg1 policy; native permission/result secrecy unknown");
}
