//! Publication/absence only: no signature, output, security, or behavior parity claim.
#![cfg(feature = "client-retail")]

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde_json::{Value, json};
use wow_ui_sim::lua_api::WowLuaEnv;

// Reuse the real cached Game preload, not the panel fixture's stubbed environment.
#[path = "common/prefork_full_ui_preload.rs"]
mod full_ui;

const REGISTER: &str = include_str!("../data/patch-api/sources/12.1.0-wikitext-register.json");
const KNOWN_GAPS: &str = include_str!("data/patch_12_1_0_sweep_known_gaps.json");
const ROW_COUNT: usize = 778;

#[derive(Deserialize)]
struct Register {
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    section: String,
    direction: String,
    symbol: String,
    #[serde(default, alias = "default")]
    page_default: Option<String>,
}

type ProbeResult = (String, String, bool, Option<String>, Option<String>);

// One classifier; constructor choices are per object kind, never per source symbol.
const CLASSIFIER: &str = r#"
local section, symbol, removed = ...
local function result(kind, detail, ok, value, default)
    return kind, detail, ok, value, default
end
local function matches_type(raw, lookup)
    if removed then return raw == 'nil' and lookup == 'nil' end
    return raw == 'function' or raw == 'table'
end
-- Removed symbols may be republished by cached Blizzard deprecation fallbacks
-- (loadDeprecationFallbacks defaults on); a native alias has no Lua source and stays a gap.
local function deprecated_fallback_source(value)
    if type(value) ~= 'function' then return nil end
    local source = debug.getinfo(value, 'S').source or ''
    if string.find(source, 'Deprecated', 1, true) then return source end
    return nil
end
local function removed_result(kind, detail, raw_value, raw, lookup)
    local source = deprecated_fallback_source(raw_value)
    if source then
        return result(kind, detail .. '; deprecated-fallback=' .. source, true)
    end
    return result(kind, detail, matches_type(raw, lookup))
end
local function resolve_raw_path(path)
    local value = _G
    for part in string.gmatch(path, '[^.]+') do
        if type(value) ~= 'table' then return nil end
        value = rawget(value, part)
    end
    return value
end
local function resolve_lookup_path(path)
    local value = _G
    for part in string.gmatch(path, '[^.:]+') do
        if value == nil then return nil end
        value = value[part]
    end
    return value
end
local function probe_publication()
    local parent_path, member = string.match(symbol, '^(.*)[.:]([^.:]+)$')
    if not parent_path then
        local raw_value = rawget(_G, symbol)
        local raw = type(raw_value)
        local lookup_ok, lookup_value = pcall(function() return _G[symbol] end)
        local lookup = lookup_ok and type(lookup_value) or 'lookup-error'
        local detail = 'raw=' .. raw .. '; lookup=' .. lookup
        if removed then return removed_result('global', detail, raw_value, raw, lookup) end
        return result('global', detail, matches_type(raw, lookup))
    end
    local parent = resolve_raw_path(parent_path)
    if parent == nil then
        local lookup_ok, lookup_value = pcall(resolve_lookup_path, symbol)
        local lookup = lookup_ok and type(lookup_value) or 'lookup-error'
        return result('member', 'parent raw-absent; raw=nil; lookup=' .. lookup,
            removed and lookup == 'nil')
    end
    if type(parent) ~= 'table' then
        return result('unprobeable', 'parent is not a table: ' .. type(parent), false)
    end
    local raw_ok, raw_value = pcall(rawget, parent, member)
    if not raw_ok then return result('unprobeable', tostring(raw_value), false) end
    local lookup_ok, lookup_value = pcall(function() return parent[member] end)
    local raw = type(raw_value)
    local lookup = lookup_ok and type(lookup_value) or 'lookup-error'
    local detail = 'raw=' .. raw .. '; lookup=' .. lookup
    if removed then return removed_result('member', detail, raw_value, raw, lookup) end
    return result('member', detail, raw == 'function')
end
local function create_object(owner)
    local frame = CreateFrame('Frame')
    local factories = {
        Frame = function() return frame end,
        FrameScriptObject = function() return frame end,
        ScriptRegion = function() return frame end,
        FontString = function() return frame:CreateFontString() end,
        TextureBase = function() return frame:CreateTexture() end,
        VectorGraphics = function() return frame:CreateVectorGraphics() end,
        AnimationGroup = function() return frame:CreateAnimationGroup() end,
        Animation = function() return frame:CreateAnimationGroup():CreateAnimation() end,
        RadialProgress = function()
            local animation = frame:CreateAnimationGroup():CreateAnimation('RadialProgress')
            if animation:GetObjectType() ~= 'RadialProgress' then
                error('RadialProgress factory returned ' .. animation:GetObjectType())
            end
            return animation
        end,
        DurationTextBinding = function() return C_DurationUtil.CreateDurationTextBinding() end,
        SecondsFormatter = function() return C_StringUtil.CreateSecondsFormatter() end,
    }
    local factory = factories[owner]
    if factory then return factory() end
    return CreateFrame(owner)
end
local function probe_object()
    local owner, method = string.match(symbol, '^([^:]+):([^:]+)$')
    if not owner then return result('unprobeable', 'object symbol has no colon', false) end
    local created, object = pcall(create_object, owner)
    if not created or object == nil then
        return result('unprobeable', 'factory: ' .. tostring(object), false)
    end
    local lookup = type(object[method])
    local ok = lookup == 'function'
    if removed then ok = lookup == 'nil' end
    return result('object-method', owner .. '; lookup=' .. lookup, ok)
end
local function probe_event()
    if string.find(symbol, '*', 1, true) then
        return result('unprobeable', 'wildcard occurrence is not a concrete event', false)
    end
    local frame = CreateFrame('Frame')
    local called, returned = pcall(frame.RegisterEvent, frame, symbol)
    local registered = called and frame:IsEventRegistered(symbol) == true
    if registered then frame:UnregisterEvent(symbol) end
    local unknown = not called and string.find(tostring(returned), 'Attempt to register unknown event', 1, true) ~= nil
    local ok = called and registered
    if removed then ok = unknown or (called and not registered) end
    local detail = 'pcall=' .. tostring(called) .. '; return=' .. tostring(returned)
    return result('event', detail .. '; registered=' .. tostring(registered), ok)
end
local function probe_cvar()
    local value = C_CVar.GetCVar(symbol)
    local default = C_CVar.GetCVarDefault(symbol)
    local ok = value ~= nil and default ~= nil
    if removed then ok = value == nil and default == nil end
    return result('cvar', 'value/default queried', ok, value, default)
end
local function classify()
    if section == 'events' then return probe_event() end
    if section == 'cvars' then return probe_cvar() end
    if section == 'widgets' or section == 'scriptobjects' then return probe_object() end
    return probe_publication()
end
local called, kind, detail, ok, value, default = pcall(classify)
if not called then return result('probe-error', tostring(kind), false) end
return kind, detail, ok, value, default
"#;

// Lua decimal byte escapes avoid JSON's incompatible \u escapes and script injection.
fn quote_lua(value: &str) -> String {
    let escaped: String = value.bytes().map(|byte| format!("\\{byte:03}")).collect();
    format!("\"{escaped}\"")
}

fn classify_entry(env: &WowLuaEnv, entry: &Entry) -> Value {
    let code = format!(
        "return (function(...) {CLASSIFIER} end)({}, {}, {})",
        quote_lua(&entry.section),
        quote_lua(&entry.symbol),
        entry.direction == "removed",
    );
    let (kind, detail, ok, value, default) = env
        .eval::<ProbeResult>(&code)
        .unwrap_or_else(|error| ("probe-error".into(), error.to_string(), false, None, None));
    let default_mismatch =
        entry.section == "cvars" && entry.page_default.is_some() && entry.page_default != default;
    if default_mismatch {
        eprintln!(
            "CVar default mismatch {}: page={:?}, observed={default:?}",
            entry.id, entry.page_default
        );
    }
    json!({
        "expected": {
            "section": entry.section,
            "direction": entry.direction,
            "symbol": entry.symbol,
            "publication": if entry.direction == "removed" { "absent" } else { "published" },
            "page_default": entry.page_default,
        },
        "observed": {
            "kind": kind, "detail": detail, "value": value, "default": default,
            "default_mismatch": default_mismatch,
        },
        "ok": ok,
    })
}

fn read_register() -> Register {
    // A full scratch register allows negative controls without changing committed data.
    let source = match std::env::var_os("P1210_SWEEP_REGISTER") {
        Some(path) => std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read P1210_SWEEP_REGISTER {path:?}: {error}")),
        None => REGISTER.to_owned(),
    };
    let register: Register =
        serde_json::from_str(&source).expect("parse 12.1.0 wikitext register");
    assert_eq!(
        register.entries.len(),
        ROW_COUNT,
        "supplemental row count changed"
    );
    let ids: BTreeSet<_> = register.entries.iter().map(|entry| &entry.id).collect();
    assert_eq!(ids.len(), ROW_COUNT, "duplicate source IDs");
    for entry in &register.entries {
        assert!(matches!(
            entry.section.as_str(),
            "global-api" | "framexml" | "scriptobjects" | "widgets" | "events" | "cvars"
        ));
        assert!(matches!(
            entry.direction.as_str(),
            "added" | "removed" | "changed"
        ));
    }
    register
}

fn write_results_if_requested(results: &BTreeMap<String, Value>) {
    if let Some(path) = std::env::var_os("P1210_SWEEP_OUT") {
        let json = serde_json::to_vec_pretty(results).expect("serialize sweep results");
        std::fs::write(&path, json)
            .unwrap_or_else(|error| panic!("write P1210_SWEEP_OUT {path:?}: {error}"));
    }
}

#[test]
fn patch_12_1_0_publication_sweep() {
    let register = read_register();
    let known: BTreeSet<String> = serde_json::from_str(KNOWN_GAPS).expect("parse known-gap IDs");
    let results = crate::common::with_exclusive_workload(|| {
        let env = full_ui::preload_full_game_ui().expect("load the full cached Game UI");
        register
            .entries
            .iter()
            .map(|entry| (entry.id.clone(), classify_entry(&env, entry)))
            .collect::<BTreeMap<_, _>>()
    });
    // Persist every result before the mismatch assertion, including the first RED run.
    write_results_if_requested(&results);
    let non_ok: BTreeSet<String> = results
        .iter()
        .filter(|(_, result)| result["ok"] == false)
        .map(|(id, _)| id.clone())
        .collect();
    let new_gaps: Vec<_> = non_ok.difference(&known).collect();
    let resolved_gaps: Vec<_> = known.difference(&non_ok).collect();
    assert_eq!(
        non_ok, known,
        "new gaps: {new_gaps:?}; resolved/stale gaps: {resolved_gaps:?}"
    );
}
