//! Batch45: exact Retail 12.0.5 source/register row378, fixture/spec only.
//! Mapping, strict representations, miss/errors and wrapper-output policy are INFERRED.
//! Native aura-access restrictions and secrecy from secret curve POINTS are not modeled.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string, wrap_secret};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn aura(id: i32, dispel: Option<&str>, helpful: bool) -> AuraInfo {
    AuraInfo {
        name: format!("Dispel fixture {id}"),
        spell_id: 99001,
        icon: 134973,
        duration: 30.0,
        expiration_time: 45.0,
        applications: 2,
        source_unit: "pet".into(),
        is_helpful: helpful,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: dispel.map(str::to_owned),
        aura_instance_id: id,
    }
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create dispel color environment");
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![
            aura(100, None, true),
            aura(101, Some("Magic"), false),
            aura(102, Some("Curse"), true),
            aura(103, Some("Disease"), false),
            aura(104, Some("Poison"), true),
            aura(109, Some("Enrage"), false),
        ];
        let party = state
            .party_members
            .first_mut()
            .expect("seeded party roster");
        party.buffs = vec![aura(201, Some("Poison"), true)];
        party.debuffs = vec![aura(202, Some("Curse"), false)];
    }
    env.exec(
        r#"
        function PackDispel(...) return {n = select('#', ...), ...} end
        function AssertDispelRGBA(color, expected)
            assert(color ~= nil and type(color.GetRGBA) == 'function')
            local values = PackDispel(color:GetRGBA())
            assert(values.n == 4)
            for index = 1, 4 do
                assert(type(values[index]) == 'number')
                assert(math.abs(values[index] - expected[index]) < 1e-9,
                    'dispel curve RGBA channel mismatch: ' .. index)
            end
            local r, g, b = color:GetRGB()
            assert(r == values[1] and g == values[2] and b == values[3])
        end
        function MakeDispelStep()
            local curve = C_CurveUtil.CreateColorCurve()
            curve:SetType(Enum.LuaCurveType.Step)
            for _, x in ipairs({0, 1, 2, 3, 4, 9}) do
                curve:AddPoint(x, CreateColor(x / 10, (10 - x) / 20, (x + 1) / 20, (x + 2) / 20))
            end
            return curve
        end
        function DispelExpected(x) return {x / 10, (10 - x) / 20, (x + 1) / 20, (x + 2) / 20} end
        DispelCurve = MakeDispelStep()
        -- Locals/varargs retain the actual values across GC, including native userdata.
        function RootedDispelCall(...)
            assert(type(C_UnitAuras.GetAuraDispelTypeColor) == 'function')
            local args, originals, secrets, taints = PackDispel(...), PackDispel(...), {}, {}
            local caller = debug.getstacktaint()
            for index = 1, args.n do
                secrets[index] = issecretvalue(args[index])
                taints[index] = PackDispel(issecurevariable(args, index))
            end
            collectgarbage('collect')
            local result = PackDispel(pcall(C_UnitAuras.GetAuraDispelTypeColor, unpack(args, 1, args.n)))
            collectgarbage('collect')
            for index = 1, args.n do
                local value, original = args[index], originals[index]
                if type(value) == 'number' and value ~= value then
                    assert(original ~= original)
                else
                    assert(rawequal(value, original), 'rooted argument identity changed')
                end
                assert(issecretvalue(value) == secrets[index], 'argument secrecy changed')
                local secure, taint = issecurevariable(args, index)
                assert(secure == taints[index][1] and taint == taints[index][2])
            end
            assert(debug.getstacktaint() == caller, 'caller taint changed')
            -- Recovery stays inside the same executing closure/caller, without clearing taint.
            local recovery = PackDispel(C_UnitAuras.GetAuraDispelTypeColor('player', 101, DispelCurve))
            assert(recovery.n == 1 and not issecretvalue(recovery[1]))
            AssertDispelRGBA(recovery[1], DispelExpected(1))
            assert(debug.getstacktaint() == caller)
            return unpack(result, 1, result.n)
        end
        function CheckDispel(expected, ...)
            local result = PackDispel(RootedDispelCall(...))
            assert(result[1] and result.n == 2, 'one successful ColorMixin-compatible result required')
            assert(not issecretvalue(result[2]))
            AssertDispelRGBA(result[2], expected)
            return result[2]
        end
        function RejectDispel(...)
            local ok, err = RootedDispelCall(...)
            assert(not ok and type(err) == 'string' and #err > 0, 'explicit dispel argument/query error required')
            return err
        end
        function RejectSecretDispel(...)
            local err = RejectDispel(...)
            -- Actual VM guard category, not native-client error-message parity.
            assert(string.find(err, 'untainted', 1, true), 'authentication must deny before a miss/type error')
        end
        "#,
    )
    .expect("install fixture assertions and real native curves only");
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("actual VM security helpers");
    assert!(rilua::api::state_is_secure(lua.state_mut()));
    for (name, text) in [
        ("DispelSecretUnit", "player"),
        ("DispelSecretParty", "party1"),
        ("DispelSecretMissing", "missing-unit"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root host-secret STRING before allocation/GC");
    }
    for (name, number) in [
        ("DispelSecretID", 101.0),
        ("DispelSecretPartyID", 202.0),
        ("DispelSecretFraction", 101.5),
        ("DispelSecretMissingID", 99999.0),
    ] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root host-secret NUMBER before allocation/GC");
    }
    // Generic payload wrapping is supported by the actual VM. Start with a
    // genuine native color-curve userdata; never simulate secrecy via Lua fields.
    let native = lua.get_global_val("DispelCurve");
    assert!(matches!(native, rilua::Val::Userdata(_)));
    lua.state_mut().push(native);
    let wrapper = wrap_secret(lua.state_mut(), native).expect("secure host wraps real color curve");
    lua.state_mut().push(wrapper);
    let inserted = lua.set_global_val("DispelSecretCurve", wrapper);
    lua.state_mut().pop();
    lua.state_mut().pop();
    inserted.expect("root wrapper and original native curve before subsequent GC");
}

#[test]
fn step_curve_remaps_each_inferred_typed_dispel_id_not_builtin_palette() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, case in ipairs({{100, 0}, {101, 1}, {102, 2}, {103, 3}, {104, 4}, {109, 9}}) do
            CheckDispel(DispelExpected(case[2]), 'player', case[1], DispelCurve)
        end
        "#,
    )
    .expect("INFERRED None0/Magic1/Curse2/Disease3/Poison4/Enrage9 mapping");
}

#[test]
fn linear_curve_uses_numeric_dispel_x_for_all_channels() {
    let env = fixture_env();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:SetType(Enum.LuaCurveType.Linear)
        curve:AddPoint(0, CreateColor(0.1, 0.9, 0.2, 0.3))
        curve:AddPoint(10, CreateColor(0.9, 0.1, 0.7, 0.8))
        for _, case in ipairs({{100, 0}, {101, 1}, {102, 2}, {103, 3}, {104, 4}, {109, 9}}) do
            local x = case[2]
            CheckDispel({0.1 + 0.08*x, 0.9 - 0.08*x, 0.2 + 0.05*x, 0.3 + 0.05*x},
                'player', case[1], curve)
        end
        "#,
    )
    .expect("real linear evaluation, not a fixed dispel-color catalog");
}

#[test]
fn supplied_curve_controls_magic_and_nondispellable_rgba() {
    let env = fixture_env();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:SetType(Enum.LuaCurveType.Step)
        curve:AddPoint(0, CreateColor(0.8, 0.7, 0.6, 0.5))
        curve:AddPoint(1, CreateColor(0.4, 0.3, 0.2, 0.1))
        CheckDispel({0.8, 0.7, 0.6, 0.5}, 'player', 100, curve)
        CheckDispel({0.4, 0.3, 0.2, 0.1}, 'player', 101, curve)
        CheckDispel(DispelExpected(1), 'player', 101, DispelCurve)
        "#,
    )
    .expect("INFERRED nondispel x0 is not hardcoded transparent fallback");
}

#[test]
fn seeded_party_helpful_and_harmful_records_are_distinct_unit_sources() {
    let env = fixture_env();
    env.exec(
        r#"
        CheckDispel(DispelExpected(4), 'party1', 201, DispelCurve)
        CheckDispel(DispelExpected(2), 'party1', 202, DispelCurve)
        RejectDispel('player', 201, DispelCurve)
        RejectDispel('party1', 101, DispelCurve)
        "#,
    )
    .expect("both party stores, no cross-unit instance borrowing");
}

#[test]
fn signed_i32_ids_include_zero_negative_and_endpoints_when_records_exist() {
    let env = fixture_env();
    for id in [0, -1, i32::MIN, i32::MAX] {
        env.state()
            .borrow_mut()
            .player
            .buffs
            .push(aura(id, Some("Disease"), true));
    }
    env.exec(
        r#"
        for _, id in ipairs({0, -1, -2147483648, 2147483647}) do
            CheckDispel(DispelExpected(3), 'player', id, DispelCurve)
        end
        "#,
    )
    .expect("INFERRED signed-i32 representation, not positive-only IDs");
}

#[test]
fn required_unit_and_instance_id_reject_omission_nil_and_coercion() {
    let env = fixture_env();
    env.exec(
        r#"
        RejectDispel()
        RejectDispel('player')
        RejectDispel(nil, 101, DispelCurve)
        RejectDispel('player', nil, DispelCurve)
        for _, value in ipairs({false, true, 101, {}, function() end, newproxy(true)}) do
            RejectDispel(value, 101, DispelCurve)
        end
        for _, value in ipairs({'101', false, true, {}, function() end, newproxy(true)}) do
            RejectDispel('player', value, DispelCurve)
            RejectDispel('missing-unit', value, DispelCurve)
        end
        "#,
    )
    .expect("INFERRED actual STRING/NUMBER only, before lookup");
}

#[test]
fn malformed_utf8_and_nonfinite_fractional_out_of_range_ids_reject_before_miss() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, unit in ipairs({string.char(255), 'player' .. string.char(192, 128)}) do
            RejectDispel(unit, 101, DispelCurve)
        end
        for _, unit in ipairs({'player', 'missing-unit'}) do
            for _, id in ipairs({101.5, -1.5, 0/0, math.huge, -math.huge, 2147483648, -2147483649}) do
                RejectDispel(unit, id, DispelCurve)
            end
        end
        "#,
    )
    .expect("INFERRED strict UTF-8 and finite integral signed-i32 validation");
}

#[test]
fn required_curve_rejects_plain_color_numeric_curve_spoofs_and_arbitrary_userdata() {
    let env = fixture_env();
    env.exec(
        r#"
        RejectDispel('player', 101)
        RejectDispel('player', 101, nil)
        local spoof = {Evaluate = function() return CreateColor(0.1, 0.2, 0.3, 0.4) end,
            EvaluateUnpacked = function() return 0.1, 0.2, 0.3, 0.4 end,
            GetPointCount = function() return 1 end}
        for _, curve in ipairs({CreateColor(1, 1, 1, 1), {}, spoof,
            C_CurveUtil.CreateCurve(), newproxy(true), CreateFrame('Frame'), false, 1, 'curve'}) do
            RejectDispel('player', 101, curve)
            RejectDispel('missing-unit', 101, curve)
        end
        "#,
    )
    .expect("required actual LuaColorCurveObject private registry identity");
}

#[test]
fn missing_unit_or_instance_errors_instead_of_fabricated_transparent_color() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, unit in ipairs({'missing-unit', '', 'party99', 'unité'}) do
            RejectDispel(unit, 101, DispelCurve)
        end
        for _, id in ipairs({99999, -2, -2147483648, 2147483647}) do
            RejectDispel('player', id, DispelCurve)
        end
        RejectDispel('party1', 99999, DispelCurve)
        "#,
    )
    .expect("INFERRED explicit miss error from valid-instance/nonnil declaration");
}

#[test]
fn unknown_stored_dispel_string_errors_without_invented_catalog_or_none_alias() {
    let env = fixture_env();
    env.state().borrow_mut().player.buffs.extend([
        aura(301, Some("UncataloguedFixture"), false),
        aura(302, Some(""), true),
        aura(303, Some("magic"), false),
    ]);
    env.exec(
        r#"
        RejectDispel('player', 301, DispelCurve)
        RejectDispel('player', 302, DispelCurve)
        RejectDispel('player', 303, DispelCurve)
        "#,
    )
    .expect("INFERRED unknown stored string error; None is separate typed state");
}

#[test]
fn secure_caller_accepts_each_and_mixed_authentic_secret_arguments() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        CheckDispel(DispelExpected(1), DispelSecretUnit, 101, DispelCurve)
        CheckDispel(DispelExpected(1), 'player', DispelSecretID, DispelCurve)
        CheckDispel(DispelExpected(1), DispelSecretUnit, DispelSecretID, DispelCurve)
        for _, args in ipairs({
            {'player', 101, DispelSecretCurve},
            {DispelSecretUnit, 101, DispelSecretCurve},
            {'player', DispelSecretID, DispelSecretCurve},
            {DispelSecretUnit, DispelSecretID, DispelSecretCurve}}) do
            local result = PackDispel(RootedDispelCall(unpack(args)))
            assert(result[1] and result.n == 2 and issecretvalue(result[2]))
            AssertDispelRGBA(secretunwrap(result[2]), DispelExpected(1))
        end
        CheckDispel(DispelExpected(2), DispelSecretParty, DispelSecretPartyID, DispelCurve)
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("row378 all three AllowedWhenUntainted; wrapped-curve output policy INFERRED");
}

#[test]
fn authenticated_payloads_still_require_actual_types_native_identity_and_valid_records() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        RejectDispel(DispelSecretID, 101, DispelCurve)
        RejectDispel('player', DispelSecretUnit, DispelCurve)
        RejectDispel('player', DispelSecretFraction, DispelCurve)
        RejectDispel(DispelSecretMissing, DispelSecretID, DispelSecretCurve)
        RejectDispel(DispelSecretUnit, DispelSecretMissingID, DispelSecretCurve)
        "#,
    )
    .expect("authentication does not coerce wrong payloads or manufacture missing colors");
    // Root each native/plain value before wrapping through the secure host entry.
    env.exec("DispelInvalidNumericCurve = C_CurveUtil.CreateCurve(); DispelInvalidColor = CreateColor(1, 1, 1, 1)")
        .expect("create concrete wrong-identity inputs");
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        for name in ["DispelInvalidNumericCurve", "DispelInvalidColor"] {
            let original = lua.get_global_val(name);
            lua.state_mut().push(original);
            let wrapper =
                wrap_secret(lua.state_mut(), original).expect("secure generic payload wrap");
            lua.state_mut().push(wrapper);
            let inserted = lua.set_global_val(name, wrapper);
            lua.state_mut().pop();
            lua.state_mut().pop();
            inserted.expect("root wrong-identity wrapper");
        }
    }
    env.exec(
        r#"
        RejectDispel('player', 101, DispelInvalidNumericCurve)
        RejectDispel('player', 101, DispelInvalidColor)
        "#,
    )
    .expect("secret wrapping cannot authenticate a numeric curve or plain color as color curve");
}

#[test]
fn tainted_caller_denies_each_and_mixed_secret_arguments_before_missing_lookup() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local unit, id, curve = DispelSecretUnit, DispelSecretID, DispelSecretCurve
        collectgarbage('collect')
        local function probe()
            assert(debug.getstacktaint() == 'DispelArgumentProbe')
            for _, args in ipairs({
                {unit, 101, DispelCurve}, {'player', id, DispelCurve}, {'player', 101, curve},
                {unit, id, DispelCurve}, {unit, 101, curve}, {'player', id, curve}, {unit, id, curve},
                {DispelSecretMissing, 101, DispelCurve}, {'missing-unit', id, DispelCurve},
                {'missing-unit', 99999, curve}, {unit, 101.5, DispelCurve},
                {false, id, DispelCurve}, {'missing-unit', nil, curve}}) do
                RejectSecretDispel(unpack(args, 1, 3))
            end
            for _, value in ipairs({unit, id, curve}) do
                assert(issecretvalue(value) and not pcall(secretunwrap, value))
            end
            CheckDispel(DispelExpected(1), 'player', 101, DispelCurve)
            assert(debug.getstacktaint() == 'DispelArgumentProbe')
        end
        debug.setobjecttaint(probe, 'DispelArgumentProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        assert(rawequal(unit, DispelSecretUnit) and rawequal(id, DispelSecretID))
        assert(rawequal(curve, DispelSecretCurve))
        CheckDispel(DispelExpected(1), unit, id, DispelCurve)
        "#,
    )
    .expect("secret denial, taint preserved, same-closure public recovery and secure reentry");
}

#[test]
fn secret_curve_result_retains_wrapper_secrecy_and_rooted_gc_identity() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local native, wrapper = DispelCurve, DispelSecretCurve
        collectgarbage('collect')
        assert(rawequal(secretunwrap(wrapper), native))
        local values = PackDispel(RootedDispelCall(DispelSecretUnit, DispelSecretID, wrapper))
        assert(values[1] and values.n == 2)
        local output = values[2]
        assert(issecretvalue(output))
        local original = output
        collectgarbage('collect')
        collectgarbage('collect')
        assert(rawequal(original, output) and rawequal(wrapper, DispelSecretCurve))
        AssertDispelRGBA(secretunwrap(output), DispelExpected(1))
        local function probe()
            assert(debug.getstacktaint() == 'DispelOutputProbe')
            assert(not pcall(secretunwrap, wrapper))
            assert(not pcall(secretunwrap, output))
            assert(not pcall(function() return output:GetRGBA() end))
            RejectSecretDispel('player', 101, wrapper)
            CheckDispel(DispelExpected(1), 'player', 101, native)
            assert(issecretvalue(wrapper) and issecretvalue(output))
            assert(debug.getstacktaint() == 'DispelOutputProbe')
        end
        debug.setobjecttaint(probe, 'DispelOutputProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        assert(rawequal(secretunwrap(wrapper), native) and rawequal(original, output))
        AssertDispelRGBA(secretunwrap(output), DispelExpected(1))
        -- A returned color is an independent curve evaluation snapshot.
        local revealed = secretunwrap(output)
        revealed.r, revealed.g, revealed.b, revealed.a = 0, 0, 0, 0
        AssertDispelRGBA(revealed, {0, 0, 0, 0})
        local again = PackDispel(RootedDispelCall('player', 101, wrapper))
        assert(again[1] and again.n == 2 and issecretvalue(again[2]))
        AssertDispelRGBA(secretunwrap(again[2]), DispelExpected(1))
        "#,
    )
    .expect("INFERRED wrapped-color result, secure actual RGBA, no generic declassification");
}

fn assert_records_unchanged(actual: &[AuraInfo], expected: &[AuraInfo]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(
            (
                (
                    &actual.name,
                    actual.spell_id,
                    actual.icon,
                    actual.applications
                ),
                (actual.duration, actual.expiration_time, &actual.source_unit),
                (actual.is_helpful, actual.is_raid, actual.is_nameplate_only),
                (actual.is_stealable, actual.can_apply_aura),
                (
                    actual.is_from_player_or_player_pet,
                    &actual.dispel_type,
                    actual.aura_instance_id
                ),
            ),
            (
                (
                    &expected.name,
                    expected.spell_id,
                    expected.icon,
                    expected.applications
                ),
                (
                    expected.duration,
                    expected.expiration_time,
                    &expected.source_unit
                ),
                (
                    expected.is_helpful,
                    expected.is_raid,
                    expected.is_nameplate_only
                ),
                (expected.is_stealable, expected.can_apply_aura),
                (
                    expected.is_from_player_or_player_pet,
                    &expected.dispel_type,
                    expected.aura_instance_id
                ),
            ),
        );
    }
}

#[test]
fn evaluation_preserves_curves_points_results_records_blocks_and_provider_state() {
    let env = fixture_env();
    let (player, helpful, harmful) = {
        let state = env.state().borrow();
        (
            state.player.buffs.clone(),
            state.party_members[0].buffs.clone(),
            state.party_members[0].debuffs.clone(),
        )
    };
    env.exec(
        r#"
        local input = CreateColor(0.7, 0.6, 0.5, 0.4)
        local curve = C_CurveUtil.CreateColorCurve()
        curve:SetType(Enum.LuaCurveType.Step)
        curve:AddPoint(0, CreateColor(0, 0, 0, 0))
        curve:AddPoint(1, input)
        local points = curve:GetPoints()
        local result = CheckDispel({0.7, 0.6, 0.5, 0.4}, 'player', 101, curve)
        C_UnitAuras.AddBlockedAura('player', 101)
        C_UnitAuras.AddBlockedAura('party1', 201)
        C_UnitAuras.AddBlockedAura('party1', 202)
        CheckDispel({0.7, 0.6, 0.5, 0.4}, 'player', 101, curve)
        CheckDispel(DispelExpected(4), 'party1', 201, DispelCurve)
        CheckDispel(DispelExpected(2), 'party1', 202, DispelCurve)
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HARMFUL').auraInstanceID == 103)
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil)
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HARMFUL') == nil)
        local dto = AuraUtil.GetAuraDataByAuraInstanceID('player', 101)
        dto.dispelName = 'Poison'
        CheckDispel(DispelExpected(1), 'player', 101, DispelCurve)
        assert(dto.dispelName == 'Poison')
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 101).dispelName == 'Magic')
        C_UnitAuras.SwitchAuraDataProvider()
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 101) == nil)
        CheckDispel({0.7, 0.6, 0.5, 0.4}, 'player', 101, curve)
        RejectDispel('player', 101.5, curve)
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 101) == nil)
        C_UnitAuras.ResetAuraDataProvider()
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 101).dispelName == 'Magic')
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HARMFUL').auraInstanceID == 103)
        assert(curve:GetType() == Enum.LuaCurveType.Step and curve:GetPointCount() == 2)
        local fresh = curve:GetPoints()
        assert(points[1].x == 0 and points[2].x == 1 and fresh[1].x == 0 and fresh[2].x == 1)
        AssertDispelRGBA(points[1].y, {0, 0, 0, 0})
        AssertDispelRGBA(fresh[1].y, {0, 0, 0, 0})
        AssertDispelRGBA(points[2].y, {0.7, 0.6, 0.5, 0.4})
        AssertDispelRGBA(fresh[2].y, {0.7, 0.6, 0.5, 0.4})
        AssertDispelRGBA(input, {0.7, 0.6, 0.5, 0.4})
        AssertDispelRGBA(result, {0.7, 0.6, 0.5, 0.4})
        result.r, result.g, result.b, result.a = 0, 0, 0, 0
        AssertDispelRGBA(result, {0, 0, 0, 0})
        local pointColor = points[2].y
        pointColor.r, pointColor.g, pointColor.b, pointColor.a = 1, 1, 1, 1
        AssertDispelRGBA(pointColor, {1, 1, 1, 1})
        CheckDispel({0.7, 0.6, 0.5, 0.4}, 'player', 101, curve)
        AssertDispelRGBA(input, {0.7, 0.6, 0.5, 0.4})
        "#,
    )
    .expect("blocked-inclusive read-only query and independent ColorMixin/point/DTO snapshots");
    let state = env.state().borrow();
    assert_records_unchanged(&state.player.buffs, &player);
    assert_records_unchanged(&state.party_members[0].buffs, &helpful);
    assert_records_unchanged(&state.party_members[0].debuffs, &harmful);
}

#[test]
fn two_environments_isolate_aura_records_native_curves_blocks_and_provider_selection() {
    let first = fixture_env();
    let second = fixture_env();
    {
        let mut state = second.state().borrow_mut();
        state.player.buffs[2].dispel_type = Some("Poison".into());
        state.party_members[0].debuffs.clear();
    }
    first.exec(
        "C_UnitAuras.AddBlockedAura('player', 101); C_UnitAuras.SwitchAuraDataProvider(); CheckDispel(DispelExpected(1), 'player', 101, DispelCurve)",
    ).expect("first environment selects its own typed record despite provider/block state");
    second.exec(
        r#"
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 101).dispelName == 'Magic')
        assert(C_UnitAuras.GetAuraDataByAuraInstanceID('player', 101).dispelName == 'Magic')
        CheckDispel(DispelExpected(4), 'player', 102, DispelCurve)
        RejectDispel('party1', 202, DispelCurve)
        local curve = C_CurveUtil.CreateColorCurve()
        curve:AddPoint(0, CreateColor(0.9, 0.8, 0.7, 0.6))
        CheckDispel({0.9, 0.8, 0.7, 0.6}, 'player', 102, curve)
        "#,
    ).expect("second environment has independent records, curve registry and default provider/block state");
    first
        .exec(
            r#"
        CheckDispel(DispelExpected(2), 'player', 102, DispelCurve)
        CheckDispel(DispelExpected(2), 'party1', 202, DispelCurve)
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 101) == nil)
        C_UnitAuras.ResetAuraDataProvider()
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HARMFUL').auraInstanceID == 103)
        "#,
        )
        .expect("second environment cannot change first records/curve/block/provider state");
}
