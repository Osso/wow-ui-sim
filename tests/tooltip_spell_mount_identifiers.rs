//! Next56 rows333/334/336/337: chosen simulator identifier/security contract.
//! Inputs only; parent owns compiled RED/GREEN. No native parity claim.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string, wrap_secret,
};
use rilua::vm::value::Val;
use wow_ui_sim::lua_api::WowLuaEnv;

const SPELL_LINK: &str = "|cff71d5ff|Hspell:19750|h[Flash of Light]|h|r";
const MOUNT_LINK: &str = "|cff71d5ff|Hspell:23338|h[Swift Palomino]|h|r";
const SECRET_NAMES: [&str; 6] = [
    "TMSecretFalse",
    "TMSecretTrue",
    "TMSecretNumber",
    "TMSecretString",
    "TMSecretFrame",
    "TMSecretTable",
];

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("tooltip fixture startup");
    {
        let state = env.state().borrow();
        let mount = state
            .world
            .mounts
            .iter()
            .find(|mount| mount.spell_id == 23338)
            .expect("actual default mount fixture, not manufactured catalog");
        assert_eq!(mount.name, "Swift Palomino", "first matching host mount");
        assert_eq!(mount.mount_id, 18);
        assert!(mount.is_collected && mount.is_usable);
    }
    env.exec(r#"
        TMFrame = CreateFrame('Frame')
        assert(TMFrame:GetObjectType() == 'Frame')
        TMFrame:SetAlpha(0.625)
        TMFrame.marker = 56
        TMInputs = {marker = 37, identifier = 19750, flag = false}
        function TMPublic(value)
            assert(not issecretvalue(value), 'inferred public DTO')
            if type(value) == 'table' then
                for key, child in pairs(value) do
                    assert(not issecretvalue(key), 'public DTO key')
                    TMPublic(child)
                end
            end
        end
        function TMEqual(left, right)
            assert(type(left) == type(right), 'DTO value types')
            if type(left) ~= 'table' then assert(left == right, 'DTO scalar'); return end
            for key, value in pairs(left) do TMEqual(value, right[key]) end
            for key, value in pairs(right) do TMEqual(left[key], value) end
        end
        function TMData(...)
            assert(select('#', ...) == 1, 'exactly one tooltip result')
            local data = ...
            assert(type(data) == 'table' and type(data.lines) == 'table', 'actual TooltipData')
            assert(data.type == Enum.TooltipDataType.Spell, 'spell-typed tooltip')
            TMPublic(data)
            return data
        end
        function TMCheck(query, identifier, expectedID, expectedName)
            local data = TMData(query(identifier))
            assert(data.id == expectedID, 'resolved ID, never invented miss ID')
            if expectedName then
                assert(data.lines[1].leftText == expectedName, 'meaningful producer title')
                assert(data.wordWrapMinWidth > 0, 'existing payload width')
            else assert(next(data.lines) == nil, 'line-empty miss') end
            return data
        end
        function TMProperties()
            assert(TMFrame:GetObjectType() == 'Frame')
            assert(TMFrame:GetAlpha() == 0.625 and TMFrame.marker == 56)
            assert(TMInputs.marker == 37 and TMInputs.identifier == 19750 and TMInputs.flag == false)
        end
        function TMRecovery()
            TMCheck(C_TooltipInfo.GetSpellByID, 19750, 19750, 'Flash of Light')
            TMCheck(C_TooltipInfo.GetMountBySpellID, 23338, 23338, 'Swift Palomino')
            TMProperties()
        end
        function TMContexts(probe)
            assert(issecure(), 'secure fixture caller')
            probe()
            assert(issecure(), 'secure caller preserved')
            local function addon()
                assert(debug.getstacktaint() == 'Tooltip56Fixture')
                probe()
                assert(debug.getstacktaint() == 'Tooltip56Fixture', 'ordinary taint preserved')
            end
            debug.setobjecttaint(addon, 'Tooltip56Fixture')
            addon()
            assert(issecure(), 'return restores secure caller')
        end
        function TMReject(query, ...)
            assert(type(query) == 'function', 'actual registered query required')
            local before = debug.getstacktaint()
            local ok, err = pcall(query, ...)
            assert(not ok and type(err) == 'string' and #err > 0, 'argument rejection')
            assert(string.find(err, 'C_TooltipInfo', 1, true), 'public API error context')
            assert(not string.find(err, 'PRIVATE-Tooltip56', 1, true), 'no private string payload leak')
            assert(debug.getstacktaint() == before, 'rejection preserves caller taint')
        end
        function TMRejectOptional(query, identifier, position, value)
            assert(issecretvalue(value), 'authentic host VM secret')
            local args = {identifier, nil, nil, nil, nil, nil}
            args[position] = value
            TMReject(query, unpack(args, 1, query == C_TooltipInfo.GetMountBySpellID and 2 or 6))
            assert(issecretvalue(value), 'no declassification')
            TMProperties()
        end
        function TMFrameCheck(method, query, identifier)
            local data = TMData(query(identifier))
            GameTooltip[method](GameTooltip, identifier)
            TMEqual(GameTooltip.processingInfo.tooltipData, data)
            assert(GameTooltip:NumLines() > 0, 'actual rendered lines populated')
        end
    "#).expect("assertions and real frame only; queries never replaced");
    env
}

fn seeded_env() -> WowLuaEnv {
    let env = fixture_env();
    let mut state = env.state().borrow_mut();
    state.spell_id_aliases.clear();
    for (alias, id) in [
        ("flash of light", 19750),
        ("swift palomino", 23338),
        ("19750", 19750),
        ("23338", 23338),
        (SPELL_LINK, 19750),
        (MOUNT_LINK, 23338),
        ("private-tooltip56-23338", 23338),
    ] {
        state.spell_id_aliases.insert(alias.to_lowercase(), id);
    }
    drop(state);
    env
}

fn secret_env() -> WowLuaEnv {
    let env = seeded_env();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("actual VM security helpers");
    for (name, payload) in [("TMSecretFalse", false), ("TMSecretTrue", true)] {
        let value = wrap_host_secret_bool(lua.state_mut(), payload);
        publish_secret(&mut *lua, name, value);
    }
    let number = wrap_host_secret_number(lua.state_mut(), 19750.0);
    publish_secret(&mut *lua, "TMSecretNumber", number);
    let text = wrap_host_secret_string(lua.state_mut(), "PRIVATE-Tooltip56-23338");
    publish_secret(&mut *lua, "TMSecretString", text);
    for (original, name) in [("TMFrame", "TMSecretFrame"), ("TMInputs", "TMSecretTable")] {
        let value = lua.get_global_val(original);
        let Val::Table(reference) = value else {
            panic!("original {original} is an actual table");
        };
        let table = lua
            .state_mut()
            .gc
            .tables
            .get(reference)
            .expect("live original table");
        if original == "TMFrame" {
            assert!(
                table.backing().is_some(),
                "actual frame table host metadata"
            );
        }
        lua.state_mut().push(value);
        let wrapper = wrap_secret(lua.state_mut(), value).expect("wrap real rooted table/frame");
        publish_secret(&mut *lua, name, wrapper);
        lua.state_mut().pop();
    }
    drop(lua);
    env.exec(
        r#"
        TMSecrets = {TMSecretFalse, TMSecretTrue, TMSecretNumber,
            TMSecretString, TMSecretFrame, TMSecretTable}
        function TMSecrecy()
            assert(#TMSecrets == 6)
            for _, value in ipairs(TMSecrets) do assert(issecretvalue(value)) end
            assert(issecretvalue(TMSecretFalse) and issecretvalue(TMSecretTrue))
            assert(issecretvalue(TMSecretNumber) and issecretvalue(TMSecretString))
            assert(issecretvalue(TMSecretFrame) and issecretvalue(TMSecretTable))
        end
    "#,
    )
    .expect("global and list GC roots without tainted BOOL equality");
    env
}

fn publish_secret(lua: &mut impl LuaApiMut, name: &str, value: Val) {
    lua.state_mut().push(value);
    let inserted = lua.set_global_val(name, value);
    lua.state_mut().pop();
    inserted.expect("root authentic wrapper during publication");
}

fn snapshot_roots(env: &WowLuaEnv) -> Vec<(Val, u64)> {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    SECRET_NAMES
        .iter()
        .map(|name| {
            let value = lua.get_global_val(name);
            let Val::Userdata(reference) = value else {
                panic!("host wrapper missing: {name}");
            };
            let sequence = lua
                .state_mut()
                .gc
                .userdata
                .get(reference)
                .expect("published wrapper remains live")
                .alloc_seq();
            (value, sequence)
        })
        .collect()
}

fn exec_secret_probe(env: &WowLuaEnv, script: &str) {
    let roots = snapshot_roots(env); // Metadata only; snapshots do not create GC roots.
    let aliases = env.state().borrow().spell_id_aliases.clone();
    let mounts = mount_snapshot(env);
    env.exec(script)
        .expect("real query rejects secrets and recovers in both contexts");
    assert_eq!(
        snapshot_roots(env),
        roots,
        "wrapper identity and live allocation sequence"
    );
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let Val::Table(list) = lua.get_global_val("TMSecrets") else {
        panic!("secret list root lost");
    };
    for (index, (value, _)) in roots.iter().enumerate() {
        assert_eq!(
            lua.state_mut()
                .gc
                .tables
                .get(list)
                .expect("list live")
                .get_int(index as i64 + 1),
            *value,
            "host list identity; no tainted secret BOOL rawequal"
        );
    }
    let exported = lua.get_global_val("TMExported");
    if !exported.is_nil() {
        let Val::Table(list) = exported else {
            panic!("stack-root export table");
        };
        for (index, (value, _)) in roots.iter().enumerate() {
            assert_eq!(
                lua.state_mut()
                    .gc
                    .tables
                    .get(list)
                    .expect("export live")
                    .get_int(index as i64 + 1),
                *value
            );
        }
    }
    drop(lua);
    assert_eq!(
        env.state().borrow().spell_id_aliases,
        aliases,
        "aliases read-only"
    );
    assert_eq!(
        mount_snapshot(env),
        mounts,
        "declared mount state read-only"
    );
}

fn mount_snapshot(env: &WowLuaEnv) -> Vec<(u32, String, u32, u32, bool, bool, u32)> {
    env.state()
        .borrow()
        .world
        .mounts
        .iter()
        .map(|mount| {
            (
                mount.mount_id,
                mount.name.clone(),
                mount.spell_id,
                mount.icon,
                mount.is_collected,
                mount.is_usable,
                mount.mount_type,
            )
        })
        .collect()
}

fn rendered_lines(env: &WowLuaEnv) -> Vec<(String, Option<String>, (f32, f32, f32), bool)> {
    let state = env.state().borrow();
    let id = state
        .widgets
        .get_id_by_name("GameTooltip")
        .expect("actual tooltip frame");
    state
        .tooltips
        .get(&id)
        .expect("applied rendered TooltipData")
        .lines
        .iter()
        .map(|line| {
            (
                line.left_text.clone(),
                line.right_text.clone(),
                line.left_color,
                line.wrap,
            )
        })
        .collect()
}

#[test]
fn known_spell_numeric_keeps_generated_title_and_nonempty_lines() {
    fixture_env()
        .exec(
            r#"
        local d = TMCheck(C_TooltipInfo.GetSpellByID, 19750, 19750, 'Flash of Light')
        assert(#d.lines >= 3, 'real cast/description payload, not nil shim')
        assert(type(d.lines[2].leftText) == 'string' and #d.lines[2].leftText > 0)
        assert(type(d.lines[#d.lines].leftText) == 'string' and #d.lines[#d.lines].leftText > 0)
    "#,
        )
        .unwrap();
}

#[test]
fn known_mount_numeric_uses_existing_declared_host_mount() {
    fixture_env()
        .exec(
            r#"
        local d = TMCheck(C_TooltipInfo.GetMountBySpellID, 23338, 23338, 'Swift Palomino')
        assert(d.lines[3].leftText == 'Summons this mount.' and d.lines[3].wrapText)
    "#,
        )
        .unwrap();
}

#[test]
fn spell_number_name_and_colored_link_aliases_have_equivalent_dtos() {
    seeded_env()
        .exec(&format!(
            r#"
        local baseline = TMCheck(C_TooltipInfo.GetSpellByID, 19750, 19750, 'Flash of Light')
        for _, alias in ipairs({{'19750', 'FLASH OF LIGHT', '{SPELL_LINK}'}}) do
            TMEqual(TMData(C_TooltipInfo.GetSpellByID(alias)), baseline)
        end
    "#
        ))
        .unwrap();
}

#[test]
fn mount_number_name_and_colored_link_aliases_have_equivalent_dtos() {
    seeded_env()
        .exec(&format!(
            r#"
        local baseline = TMCheck(C_TooltipInfo.GetMountBySpellID, 23338, 23338, 'Swift Palomino')
        for _, alias in ipairs({{'23338', 'SWIFT PALOMINO', '{MOUNT_LINK}'}}) do
            TMEqual(TMData(C_TooltipInfo.GetMountBySpellID(alias)), baseline)
        end
    "#
        ))
        .unwrap();
}

#[test]
fn numeric_alias_precedes_identity_and_returns_resolved_payload_id() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("23338".into(), 19750);
    env.exec(
        r#"
        for _, query in ipairs({C_TooltipInfo.GetSpellByID, C_TooltipInfo.GetMountBySpellID}) do
            TMEqual(TMCheck(query, 23338, 19750, 'Flash of Light'), TMData(query(19750)))
            TMEqual(TMData(query('23338')), TMData(query(19750)))
        end
    "#,
    )
    .unwrap();
    env.state().borrow_mut().spell_id_aliases.remove("23338");
    env.exec("TMCheck(C_TooltipInfo.GetMountBySpellID, 23338, 23338, 'Swift Palomino'); TMCheck(C_TooltipInfo.GetMountBySpellID, '23338', nil, nil)").unwrap();
}

#[test]
fn full_link_alias_wins_over_its_embedded_spell_number() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert(MOUNT_LINK.to_lowercase(), 19750);
    env.exec(&format!(
        r#"
        TMCheck(C_TooltipInfo.GetSpellByID, '{MOUNT_LINK}', 19750, 'Flash of Light')
        TMCheck(C_TooltipInfo.GetMountBySpellID, '{MOUNT_LINK}', 19750, 'Flash of Light')
    "#
    ))
    .unwrap();
}

#[test]
fn live_alias_changes_are_visible_without_cached_query_results() {
    let env = seeded_env();
    env.exec("TMCheck(C_TooltipInfo.GetSpellByID, 'flash of light', 19750, 'Flash of Light')")
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("flash of light".into(), 23338);
    env.exec("TMCheck(C_TooltipInfo.GetSpellByID, 'flash of light', 23338, nil); TMCheck(C_TooltipInfo.GetMountBySpellID, 'flash of light', 23338, 'Swift Palomino')").unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .remove("flash of light");
    env.exec("TMCheck(C_TooltipInfo.GetSpellByID, 'flash of light', nil, nil); TMCheck(C_TooltipInfo.GetMountBySpellID, 'flash of light', nil, nil)").unwrap();
}

#[test]
fn numeric_unknowns_preserve_identified_line_empty_spell_tooltips() {
    fixture_env()
        .exec(
            r#"
        for _, query in ipairs({C_TooltipInfo.GetSpellByID, C_TooltipInfo.GetMountBySpellID}) do
            for _, id in ipairs({0, 4294967294, 4294967295}) do TMCheck(query, id, id, nil) end
        end
    "#,
        )
        .unwrap();
}

#[test]
fn unseeded_public_strings_return_one_unidentified_empty_spell_tooltip() {
    let env = fixture_env();
    env.state().borrow_mut().spell_id_aliases.clear();
    env.exec(&format!(
        r#"
        for _, query in ipairs({{C_TooltipInfo.GetSpellByID, C_TooltipInfo.GetMountBySpellID}}) do
            for _, text in ipairs({{'', 'unknown', '19750', 'Flash of Light',
                'Swift Palomino', 'spell:19750', '{SPELL_LINK}', '{MOUNT_LINK}'}}) do
                TMCheck(query, text, nil, nil)
            end
        end
    "#
    ))
    .unwrap();
}

#[test]
fn result_mutation_and_repeated_queries_leave_payload_sources_read_only() {
    let env = seeded_env();
    let aliases = env.state().borrow().spell_id_aliases.clone();
    let mounts = mount_snapshot(&env);
    env.exec(r#"
        for _, case in ipairs({{C_TooltipInfo.GetSpellByID, 'flash of light', 19750, 'Flash of Light'},
                {C_TooltipInfo.GetMountBySpellID, 'swift palomino', 23338, 'Swift Palomino'}}) do
            local a = TMCheck(unpack(case))
            local b = TMCheck(unpack(case))
            assert(not rawequal(a, b) and not rawequal(a.lines, b.lines), 'fresh results')
            a.id = 17; a.lines[1].leftText = 'caller changed title'
            TMCheck(unpack(case))
            assert(b.id == case[3] and b.lines[1].leftText == case[4])
        end
        for _, query in ipairs({C_TooltipInfo.GetSpellByID, C_TooltipInfo.GetMountBySpellID}) do
            local a, b = TMCheck(query, 'missing', nil, nil), TMCheck(query, 'missing', nil, nil)
            assert(not rawequal(a, b) and not rawequal(a.lines, b.lines), 'fresh miss')
            a.id = 17; a.lines[1] = {leftText = 'mutated miss'}
            TMCheck(query, 'missing', nil, nil)
            assert(b.id == nil and next(b.lines) == nil)
        end
        TMProperties()
    "#).unwrap();
    assert_eq!(env.state().borrow().spell_id_aliases, aliases);
    assert_eq!(mount_snapshot(&env), mounts);
}

#[test]
fn separate_environments_do_not_share_alias_updates() {
    let first = seeded_env();
    let second = seeded_env();
    first
        .state()
        .borrow_mut()
        .spell_id_aliases
        .insert("swift palomino".into(), 19750);
    first
        .exec("TMCheck(C_TooltipInfo.GetMountBySpellID, 'swift palomino', 19750, 'Flash of Light')")
        .unwrap();
    second
        .exec("TMCheck(C_TooltipInfo.GetMountBySpellID, 'swift palomino', 23338, 'Swift Palomino')")
        .unwrap();
}

#[test]
fn strict_public_identifiers_reject_invalid_representations_in_both_contexts() {
    seeded_env()
        .exec(
            r#"
        TMContexts(function()
            for _, query in ipairs({C_TooltipInfo.GetSpellByID, C_TooltipInfo.GetMountBySpellID}) do
                TMReject(query); TMReject(query, nil)
                for _, value in ipairs({false, true, {}, TMFrame, -1, 19750.5,
                    4294967296, math.huge, -math.huge, 0/0}) do TMReject(query, value) end
            end
            TMRecovery()
        end)
    "#,
        )
        .unwrap();
}

#[test]
fn documented_public_optional_combinations_keep_current_ignored_payloads() {
    seeded_env().exec(r#"
        local spell = TMData(C_TooltipInfo.GetSpellByID(19750))
        local mount = TMData(C_TooltipInfo.GetMountBySpellID(23338))
        TMEqual(TMData(C_TooltipInfo.GetSpellByID(19750, nil, nil, nil, nil, nil)), spell)
        TMEqual(TMData(C_TooltipInfo.GetMountBySpellID(23338, nil)), mount)
        for _, flag in ipairs({false, true}) do
            TMEqual(TMData(C_TooltipInfo.GetMountBySpellID('swift palomino', flag)), mount)
            TMEqual(TMData(C_TooltipInfo.GetSpellByID('flash of light', flag, flag, flag, 17, flag)), spell)
        end
        TMEqual(TMData(C_TooltipInfo.GetSpellByID(19750, true, false, true, 17, false)), spell)
        TMEqual(TMData(C_TooltipInfo.GetSpellByID(19750, false, true, false, nil, true)), spell)
    "#).unwrap();
}

#[test]
fn ordinary_tainted_public_calls_preserve_stack_taint_and_payload() {
    seeded_env().exec(r#"
        TMContexts(function()
            local before = debug.getstacktaint()
            TMEqual(TMData(C_TooltipInfo.GetSpellByID('flash of light', true, false, true, 17, false)),
                TMData(C_TooltipInfo.GetSpellByID(19750)))
            TMEqual(TMData(C_TooltipInfo.GetMountBySpellID('swift palomino', true)),
                TMData(C_TooltipInfo.GetMountBySpellID(23338)))
            TMRecovery()
            assert(debug.getstacktaint() == before)
        end)
    "#).unwrap();
}

#[test]
fn secret_identifiers_conservatively_reject_all_actual_vm_representations() {
    let env = secret_env();
    exec_secret_probe(
        &env,
        r#"
        TMContexts(function()
            for _, query in ipairs({C_TooltipInfo.GetSpellByID, C_TooltipInfo.GetMountBySpellID}) do
                for _, value in ipairs(TMSecrets) do
                    TMReject(query, value)
                    assert(issecretvalue(value))
                end
            end
            TMSecrecy(); TMRecovery()
        end)
    "#,
    );
}

#[test]
fn mount_never_secret_optional_rejects_all_vm_kinds_before_known_or_missing_payload() {
    let env = secret_env();
    exec_secret_probe(
        &env,
        r#"
        TMContexts(function()
            for _, identifier in ipairs({23338, 'swift palomino', 'missing', 4294967295}) do
                for _, value in ipairs(TMSecrets) do
                    TMRejectOptional(C_TooltipInfo.GetMountBySpellID, identifier, 2, value)
                end
            end
            TMSecrecy(); TMRecovery()
        end)
    "#,
    );
}

#[test]
fn spell_never_secret_all_five_positions_reject_host_true_and_false() {
    let env = secret_env();
    exec_secret_probe(
        &env,
        r#"
        TMContexts(function()
            for position = 2, 6 do
                for _, value in ipairs({TMSecretTrue, TMSecretFalse}) do
                    TMRejectOptional(C_TooltipInfo.GetSpellByID, 'flash of light', position, value)
                    TMRejectOptional(C_TooltipInfo.GetSpellByID, 'missing', position, value)
                end
            end
            TMSecrecy(); TMRecovery()
        end)
    "#,
    );
}

#[test]
fn spell_never_secret_all_five_positions_reject_host_number_and_string() {
    let env = secret_env();
    exec_secret_probe(
        &env,
        r#"
        TMContexts(function()
            for position = 2, 6 do
                for _, value in ipairs({TMSecretNumber, TMSecretString}) do
                    TMRejectOptional(C_TooltipInfo.GetSpellByID, 19750, position, value)
                    TMRejectOptional(C_TooltipInfo.GetSpellByID, 4294967295, position, value)
                end
            end
            TMSecrecy(); TMRecovery()
        end)
    "#,
    );
}

#[test]
fn spell_never_secret_all_five_positions_reject_wrapped_real_frame_and_table() {
    let env = secret_env();
    exec_secret_probe(
        &env,
        r#"
        TMContexts(function()
            for position = 2, 6 do
                for _, value in ipairs({TMSecretFrame, TMSecretTable}) do
                    TMRejectOptional(C_TooltipInfo.GetSpellByID, 'flash of light', position, value)
                    TMRejectOptional(C_TooltipInfo.GetSpellByID, 'missing', position, value)
                end
            end
            TMSecrecy(); TMRecovery()
        end)
    "#,
    );
}

#[test]
fn original_frame_identifier_routes_match_direct_dto_and_actual_rendered_lines() {
    let env = seeded_env();
    for (method, query, id, name, link) in [
        (
            "SetSpellByID",
            "GetSpellByID",
            19750,
            "flash of light",
            SPELL_LINK,
        ),
        (
            "SetMountBySpellID",
            "GetMountBySpellID",
            23338,
            "swift palomino",
            MOUNT_LINK,
        ),
    ] {
        env.exec(&format!(
            "TMFrameCheck('{method}', C_TooltipInfo.{query}, {id})"
        ))
        .unwrap();
        let numeric_lines = rendered_lines(&env);
        assert!(
            !numeric_lines.is_empty(),
            "meaningful actual rendered payload"
        );
        for alias in [id.to_string(), name.to_string(), link.to_string()] {
            env.exec(&format!(
                "TMFrameCheck('{method}', C_TooltipInfo.{query}, '{alias}')"
            ))
            .unwrap();
            assert_eq!(
                rendered_lines(&env),
                numeric_lines,
                "original identifier frame payload"
            );
        }
    }
}

#[test]
fn frame_secret_inputs_fail_without_mutating_prior_actual_tooltip_payload() {
    let env = secret_env();
    env.exec("GameTooltip:SetSpellByID(19750)").unwrap();
    let prior = rendered_lines(&env);
    exec_secret_probe(
        &env,
        r#"
        local original = GameTooltip.processingInfo.tooltipData
        TMContexts(function()
            for _, value in ipairs(TMSecrets) do
                for _, method in ipairs({'SetSpellByID', 'SetMountBySpellID'}) do
                    local ok, err = pcall(GameTooltip[method], GameTooltip, value)
                    assert(not ok and type(err) == 'string' and #err > 0)
                    assert(rawequal(GameTooltip.processingInfo.tooltipData, original))
                end
                local ok, err = pcall(GameTooltip.SetMountBySpellID, GameTooltip, 'swift palomino', value)
                assert(not ok and type(err) == 'string' and #err > 0)
                assert(rawequal(GameTooltip.processingInfo.tooltipData, original))
            end
            TMSecrecy(); TMRecovery()
        end)
    "#,
    );
    assert_eq!(
        rendered_lines(&env),
        prior,
        "failed setter leaves actual rendered payload intact"
    );
    env.exec(
        "TMFrameCheck('SetMountBySpellID', C_TooltipInfo.GetMountBySpellID, 'swift palomino')",
    )
    .unwrap();
    assert_eq!(
        rendered_lines(&env)[0].0,
        "Swift Palomino",
        "valid public frame recovery"
    );
}

#[test]
fn forced_gc_preserves_stack_global_list_roots_and_caller_properties() {
    let env = secret_env();
    exec_secret_probe(
        &env,
        r#"
        TMContexts(function()
            local a,b,c,d,e,f = unpack(TMSecrets)
            local frame, inputs = TMFrame, TMInputs
            collectgarbage('collect')
            local roots = {a,b,c,d,e,f}
            for _, value in ipairs(roots) do
                assert(issecretvalue(value))
                TMReject(C_TooltipInfo.GetSpellByID, value)
                TMReject(C_TooltipInfo.GetMountBySpellID, value)
                TMRejectOptional(C_TooltipInfo.GetMountBySpellID, 'swift palomino', 2, value)
                for position = 2, 6 do
                    TMRejectOptional(C_TooltipInfo.GetSpellByID, 'flash of light', position, value)
                end
            end
            collectgarbage('collect')
            assert(rawequal(frame, TMFrame) and rawequal(inputs, TMInputs))
            TMSecrecy(); TMRecovery()
            -- Host inspects wrapper handles after secure return, not BOOL rawequal under taint.
            TMExported = roots
        end)
        assert(issecure())
    "#,
    );
}
