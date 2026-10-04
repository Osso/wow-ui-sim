#![cfg(feature = "retail-12-0-7")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create named CVar environment");
    env.exec(
        r#"
        -- Expected defaults are INFERRED simulator choices, not native evidence.
        AuditCVarCases = {
            { 'assistedCombatReduceHighlights', '1' },
            { 'developerLogFilterDebug', '0' },
            { 'developerLogFilterError', '1' },
            { 'developerLogFilterFatal', '1' },
            { 'developerLogFilterNormal', '1' },
            { 'developerLogFilterSpam', '0' },
        }
        function CheckAuditCVar(name, value, default)
            local function checkOne(expected, ...)
                assert(select('#', ...) == 1, name .. ': one result')
                assert((...) == expected, name .. ': exact result')
            end
            checkOne(value, GetCVar(name))
            checkOne(value, C_CVar.GetCVar(name))
            checkOne(default, GetCVarDefault(name))
            checkOne(default, C_CVar.GetCVarDefault(name))
            checkOne(value == '1', GetCVarBool(name))
            checkOne(value == '1', C_CVar.GetCVarBool(name))
        end
        function RoundTripAuditCVar(name, default)
            -- Read before writing: SetCVar can create unknown override keys.
            CheckAuditCVar(name, default, default)
            local changed = default == '1' and '0' or '1'
            assert(SetCVar(name, changed) == true, name .. ': global write')
            CheckAuditCVar(name, changed, default)
            assert(C_CVar.SetCVar(name, default) == true, name .. ': namespace reset')
            CheckAuditCVar(name, default, default)
            assert(C_CVar.SetCVar(string.upper(name), changed) == true)
            CheckAuditCVar(name, changed, default)
            CheckAuditCVar(string.upper(name), changed, default)
            -- Explicit reset, not the no-op ResetTestCvars hook.
            assert(SetCVar(name, GetCVarDefault(name)) == true)
            CheckAuditCVar(name, default, default)
            assert(SetCVar(name, changed) == true)
            assert(C_CVar.SetCVar(name, C_CVar.GetCVarDefault(name)) == true)
            CheckAuditCVar(name, default, default)
        end
    "#,
    )
    .expect("install assertions without replacing any CVar API or seeding defaults");
    env
}

fn assert_round_trip(name: &str, default: &str) {
    fixture_env()
        .exec(&format!("RoundTripAuditCVar('{name}', '{default}')"))
        .expect("named default, mutable value and explicit reset through both surfaces");
}

#[test]
fn assisted_combat_reduce_highlights_round_trip() {
    assert_round_trip("assistedCombatReduceHighlights", "1");
}

#[test]
fn developer_log_filter_debug_round_trip() {
    assert_round_trip("developerLogFilterDebug", "0");
}

#[test]
fn developer_log_filter_error_round_trip() {
    assert_round_trip("developerLogFilterError", "1");
}

#[test]
fn developer_log_filter_fatal_round_trip() {
    assert_round_trip("developerLogFilterFatal", "1");
}

#[test]
fn developer_log_filter_normal_round_trip() {
    assert_round_trip("developerLogFilterNormal", "1");
}

#[test]
fn developer_log_filter_spam_round_trip() {
    assert_round_trip("developerLogFilterSpam", "0");
}

#[test]
fn named_cvars_coerce_boolean_numeric_and_string_values() {
    fixture_env()
        .exec(
            r#"
            for _, entry in ipairs(AuditCVarCases) do
                local name, default = entry[1], entry[2]
                CheckAuditCVar(name, default, default)
                for _, sample in ipairs({
                    { true, '1' }, { false, '0' },
                    { 1, '1' }, { 0, '0' }, { 2, '2' },
                    { 'true', 'true' }, { '01', '01' }, { '', '' },
                }) do
                    assert(SetCVar(name, sample[1]) == true)
                    CheckAuditCVar(name, sample[2], default)
                    assert(C_CVar.SetCVar(name, sample[1]) == true)
                    CheckAuditCVar(name, sample[2], default)
                end
                assert(SetCVar(name, GetCVarDefault(name)) == true)
                CheckAuditCVar(name, default, default)
            end
        "#,
        )
        .expect("INFERRED stringify inputs and treat only stored '1' as true");
}

#[test]
fn named_cvars_do_not_share_overrides_or_defaults_between_environments() {
    let first = fixture_env();
    first
        .exec(
            r#"
            for _, entry in ipairs(AuditCVarCases) do
                local name, default = entry[1], entry[2]
                CheckAuditCVar(name, default, default)
                local changed = default == '1' and '0' or '1'
                assert(SetCVar(name, changed) == true)
                CheckAuditCVar(name, changed, default)
            end
        "#,
        )
        .expect("mutate first environment before constructing second");
    let second = fixture_env();
    second
        .exec(
            r#"
            for _, entry in ipairs(AuditCVarCases) do
                local name, default = entry[1], entry[2]
                CheckAuditCVar(name, default, default)
                assert(C_CVar.SetCVar(name, '2') == true)
                CheckAuditCVar(name, '2', default)
            end
        "#,
        )
        .expect("new environment starts at defaults, not earlier overrides");
    first
        .exec(
            r#"
            for _, entry in ipairs(AuditCVarCases) do
                local name, default = entry[1], entry[2]
                local changed = default == '1' and '0' or '1'
                CheckAuditCVar(name, changed, default)
                assert(C_CVar.SetCVar(name, C_CVar.GetCVarDefault(name)) == true)
                CheckAuditCVar(name, default, default)
            end
        "#,
        )
        .expect("second environment cannot change first or its defaults");
    second
        .exec(
            r#"
            for _, entry in ipairs(AuditCVarCases) do
                local name, default = entry[1], entry[2]
                CheckAuditCVar(name, '2', default)
                assert(SetCVar(name, GetCVarDefault(name)) == true)
                CheckAuditCVar(name, default, default)
            end
        "#,
        )
        .expect("first environment reset cannot reset second");
}

#[test]
fn named_cvar_writes_do_not_change_sibling_cvars() {
    fixture_env()
        .exec(
            r#"
            for index, entry in ipairs(AuditCVarCases) do
                local name, default = entry[1], entry[2]
                CheckAuditCVar(name, default, default)
                local changed = default == '1' and '0' or '1'
                assert(C_CVar.SetCVar(name, changed) == true)
                for siblingIndex, sibling in ipairs(AuditCVarCases) do
                    local expected = siblingIndex == index and changed or sibling[2]
                    CheckAuditCVar(sibling[1], expected, sibling[2])
                end
                assert(SetCVar(name, GetCVarDefault(name)) == true)
                CheckAuditCVar(name, default, default)
            end
        "#,
        )
        .expect("six separate CVar values, not one shared flag");
}

#[test]
fn unknown_cvar_is_not_a_builtin_default() {
    fixture_env()
        .exec(
            r#"
            local name = 'p1207B05UnknownCVarControl'
            CheckAuditCVar(name, nil, nil)
            -- Existing general setter permits unknown overrides; that does not
            -- establish a built-in CVar addition or its factory default.
            assert(SetCVar(name, '1') == true)
            CheckAuditCVar(name, '1', nil)
            assert(C_CVar.SetCVar(name, '0') == true)
            CheckAuditCVar(name, '0', nil)
            for _, entry in ipairs(AuditCVarCases) do
                CheckAuditCVar(entry[1], entry[2], entry[2])
            end
        "#,
        )
        .expect("unknown override remains distinct from six built-in defaults");
}
