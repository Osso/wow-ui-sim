//! Retirement and successor observables for all 21 deprecated extract occurrences.
//! Native absence and post-cache absence/alias identity are separate boundaries.
#![cfg(feature = "retail-12-0-0")]

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "common/publication_sweep.rs"]
mod sweep;

#[derive(Deserialize)]
struct Data {
    summary_id: String,
    rows: BTreeMap<String, Row>,
    known_successor_gaps: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Row {
    old: String,
    successor: Option<String>,
    probe: Option<String>,
    loaded_alias: Option<String>,
}

fn read_data() -> Data {
    let data: Data =
        serde_json::from_str(include_str!("data/patch_12_0_0_deprecated.json")).unwrap();
    assert_eq!(data.rows.len(), 21);
    assert_eq!(data.summary_id, "deprecated api-removal-summary-128");
    for (id, row) in &data.rows {
        assert_eq!(row.successor.is_some(), row.probe.is_some(), "{id}");
    }
    data
}

const OBSERVE_SYMBOL: &str = r#"
    local function read(path, raw)
        local value = _G
        for part in string.gmatch(path, '[^.]+') do
            if type(value) ~= 'table' then return nil end
            if raw then value = rawget(value, part) else value = value[part] end
        end
        return value
    end
    local symbol, target = ...
    local raw, ordinary = read(symbol, true), read(symbol, false)
    if target == '' then
        return raw == nil and ordinary == nil
    end
    local alias = read(target, true)
    return type(raw) == 'function' and type(alias) == 'function'
        and rawequal(raw, alias) and rawequal(ordinary, alias)
"#;

fn symbol_matches(env: &WowLuaEnv, symbol: &str, target: Option<&str>) -> bool {
    let target = target.unwrap_or("");
    let script = format!("return (function(...) {OBSERVE_SYMBOL} end)({symbol:?}, {target:?})");
    env.eval(&script)
        .expect("observe raw and ordinary API identity")
}

fn observe_native_retirement(data: &Data) -> Vec<String> {
    let native = WowLuaEnv::new().expect("create native-only environment");
    data.rows
        .iter()
        .filter_map(|(id, row)| {
            (!symbol_matches(&native, &row.old, None))
                .then(|| format!("{id}: native still published"))
        })
        .collect()
}

fn seed_successor_inputs(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::charge_state::SpellChargeState;
    let mut state = env.state().borrow_mut();
    state.cooldowns_restricted = false;
    state.spell_charges.clear();
    state.spell_charges.insert(
        19750,
        SpellChargeState {
            current_charges: 2,
            max_charges: 3,
            recharge_start: 312.0,
            recharge_duration: 237.0,
            charge_mod_rate: 1.25,
        },
    );
    state.spell_cast_counts.extend([(19750, 7), (642, 2)]);
    state.known_spells.insert(19750);
    state.known_spells.remove(&999999999);
    state.merchant_items = vec![6948];
    state.artifact_relic_items.clear();
    state.artifact_relic_items.insert(12345);
}

fn observe_cached_retirement(env: &WowLuaEnv, data: &Data) -> Vec<String> {
    let aliases = sweep::read_deprecated_aliases(env);
    data.rows
        .iter()
        .filter_map(|(id, row)| {
            let matches = match &row.loaded_alias {
                None => symbol_matches(env, &row.old, None),
                Some(target) => aliases.get(&row.old).is_some_and(|(cached_target, _)| {
                    cached_target == target && symbol_matches(env, &row.old, Some(target))
                }),
            };
            (!matches)
                .then(|| format!("{id}: cached absence or exact loaded alias identity failed"))
        })
        .collect()
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn patch_12_0_0_deprecated_native_retirement() {
    let failures = observe_native_retirement(&read_data());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

prefork_full_ui_case! {
fn patch_12_0_0_deprecated_retirement_and_successors(env: &WowLuaEnv) {
    let data = read_data();
    let mut retirement_failures = observe_native_retirement(&data);
    retirement_failures.extend(observe_cached_retirement(env, &data));
    seed_successor_inputs(env);
    let mut failures = BTreeMap::new();
    for (id, row) in &data.rows {
        if let Some(probe) = &row.probe {
            if let Err(error) = env.exec(probe) {
                eprintln!("{id}: successor gap: {error}");
                failures.insert(id.clone(), error.to_string());
            } else {
                eprintln!("{id}: retirement and successor proven");
            }
        } else {
            eprintln!("{id}: retirement proven (no successor listed)");
        }
    }
    eprintln!("{}: retirement failures={retirement_failures:?}", data.summary_id);
    let observed: BTreeSet<_> = failures.keys().collect();
    let known: BTreeSet<_> = data.known_successor_gaps.keys().collect();
    assert!(retirement_failures.is_empty() && observed == known,
        "retirement failures={retirement_failures:?}; new or resolved successor gaps: {failures:#?}");
}
}
