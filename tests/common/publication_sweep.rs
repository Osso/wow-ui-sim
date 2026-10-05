//! Register-driven publication/absence sweep shared by the per-patch sweep tests.
//! Publication/absence only: no signature, output, security, or behavior parity claim.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde_json::{Value, json};
use wow_ui_sim::lua_api::WowLuaEnv;

// Reuse the real cached Game preload, not the panel fixture's stubbed environment.
#[path = "prefork_full_ui_preload.rs"]
mod full_ui;

/// One patch's sweep inputs. Env var names select a scratch register / results file.
pub(crate) struct SweepSpec {
    pub register: &'static str,
    pub known_gaps: &'static str,
    pub row_count: usize,
    pub register_env: &'static str,
    pub out_env: &'static str,
    /// Wikitext registers of later patches, oldest first. The Retail build carries the
    /// latest surface, so a later add/remove of the same symbol overrides this patch's
    /// expected publication (recorded as `superseded_by`).
    pub later_registers: &'static [&'static str],
}

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
    #[serde(default)]
    kind: Option<String>,
}

/// Expected publication after applying later-patch supersession.
struct Expectation<'a> {
    removed: bool,
    superseded_by: Option<&'a str>,
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
        Region = function() return frame:CreateTexture() end,
        ColorCurveObject = function() return C_CurveUtil.CreateColorCurve() end,
        CurveObject = function() return C_CurveUtil.CreateCurve() end,
        -- CurveObjectBase is the interface implemented by the scalar curve.
        CurveObjectBase = function() return C_CurveUtil.CreateCurve() end,
        UnitHealPredictionCalculator = function() return CreateUnitHealPredictionCalculator() end,
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
        DurationObject = function() return C_DurationUtil.CreateDuration() end,
        DurationManualClock = function() return C_DurationUtil.CreateManualClock() end,
        -- The manual clock is the only Lua-constructible DurationClock.
        DurationClock = function() return C_DurationUtil.CreateManualClock() end,
        Font = function()
            return rawget(_G, 'PublicationSweepFont') or CreateFont('PublicationSweepFont')
        end,
        ModelSceneActorBase = function() return CreateFrame('ModelScene'):CreateActor() end,
        ModelSceneActor = function() return CreateFrame('ModelScene'):CreateActor() end,
        AbbreviateConfig = function() return CreateAbbreviateConfig() end,
        AbbreviatedNumberFormatter = function()
            return C_StringUtil.CreateAbbreviatedNumberFormatter()
        end,
        -- NumericFormatter is the interface the abbreviated formatter implements.
        NumericFormatter = function() return C_StringUtil.CreateAbbreviatedNumberFormatter() end,
        NumericRuleFormatter = function() return C_StringUtil.CreateNumericRuleFormatter() end,
        HousingCatalogSearcher = function() return C_HousingCatalog.CreateCatalogSearcher() end,
    }
    local factory = factories[owner]
    if factory then return factory() end
    return CreateFrame(owner)
end
local function probe_object()
    local owner, method = string.match(symbol, '^([^:]+):([^:]+)$')
    if not owner then
        local created, object = pcall(create_object, symbol)
        local object_type = type(object)
        local published = created and (object_type == 'table' or object_type == 'userdata')
        local ok = published
        if removed then ok = not published end
        return result('object-kind', 'factory=' .. tostring(created) .. '; type=' .. object_type, ok)
    end
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

fn probe_entry(env: &WowLuaEnv, entry: &Entry, removed: bool) -> ProbeResult {
    // Console commands have no Lua publication surface to probe.
    if entry.kind.as_deref() == Some("command") {
        let detail = "console command; no Lua publication probe".to_owned();
        return ("unprobeable".into(), detail, false, None, None);
    }
    let code = format!(
        "return (function(...) {CLASSIFIER} end)({}, {}, {})",
        quote_lua(&entry.section),
        quote_lua(&entry.symbol),
        removed,
    );
    env.eval::<ProbeResult>(&code)
        .unwrap_or_else(|error| ("probe-error".into(), error.to_string(), false, None, None))
}

fn classify_entry(env: &WowLuaEnv, entry: &Entry, expectation: &Expectation) -> Value {
    let (kind, detail, ok, value, default) = probe_entry(env, entry, expectation.removed);
    let default_mismatch = entry.section == "cvars"
        && !expectation.removed
        && entry.page_default.is_some()
        && entry.page_default != default;
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
            "publication": if expectation.removed { "absent" } else { "published" },
            "page_default": entry.page_default,
            "superseded_by": expectation.superseded_by,
        },
        "observed": {
            "kind": kind, "detail": detail, "value": value, "default": default,
            "default_mismatch": default_mismatch,
        },
        "ok": ok,
    })
}

fn parse_register(source: &str, row_count: Option<usize>) -> Register {
    let register: Register = serde_json::from_str(source).expect("parse wikitext register");
    let ids: BTreeSet<_> = register.entries.iter().map(|entry| &entry.id).collect();
    assert_eq!(ids.len(), register.entries.len(), "duplicate source IDs");
    if let Some(row_count) = row_count {
        assert_eq!(register.entries.len(), row_count, "register row count changed");
    }
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

fn read_register(spec: &SweepSpec) -> Register {
    // A full scratch register allows negative controls without changing committed data.
    let source = match std::env::var_os(spec.register_env) {
        Some(path) => std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {} {path:?}: {error}", spec.register_env)),
        None => spec.register.to_owned(),
    };
    parse_register(&source, Some(spec.row_count))
}

/// Latest later-patch add/remove per symbol; `changed` rows keep publication as-is.
fn later_publication(spec: &SweepSpec) -> BTreeMap<String, Entry> {
    let mut latest = BTreeMap::new();
    for source in spec.later_registers {
        for entry in parse_register(source, None).entries {
            if entry.direction != "changed" {
                latest.insert(entry.symbol.clone(), entry);
            }
        }
    }
    latest
}

fn expectation_for<'a>(entry: &Entry, later: &'a BTreeMap<String, Entry>) -> Expectation<'a> {
    let own_removed = entry.direction == "removed";
    match later.get(&entry.symbol) {
        Some(newer) if (newer.direction == "removed") != own_removed => Expectation {
            removed: !own_removed,
            superseded_by: Some(&newer.id),
        },
        _ => Expectation { removed: own_removed, superseded_by: None },
    }
}

fn write_results_if_requested(out_env: &str, results: &BTreeMap<String, Value>) {
    if let Some(path) = std::env::var_os(out_env) {
        let json = serde_json::to_vec_pretty(results).expect("serialize sweep results");
        std::fs::write(&path, json)
            .unwrap_or_else(|error| panic!("write {out_env} {path:?}: {error}"));
    }
}

/// Probe every register row in one cached Game environment and require the non-ok
/// ID set to equal the reviewed known-gap set exactly.
pub(crate) fn run_publication_sweep(spec: &SweepSpec) {
    let register = read_register(spec);
    let later = later_publication(spec);
    let known: BTreeSet<String> =
        serde_json::from_str(spec.known_gaps).expect("parse known-gap IDs");
    let results = crate::common::with_exclusive_workload(|| {
        let env = full_ui::preload_full_game_ui().expect("load the full cached Game UI");
        register
            .entries
            .iter()
            .map(|entry| {
                let expectation = expectation_for(entry, &later);
                (entry.id.clone(), classify_entry(&env, entry, &expectation))
            })
            .collect::<BTreeMap<_, _>>()
    });
    // Persist every result before the mismatch assertion, including the first RED run.
    write_results_if_requested(spec.out_env, &results);
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
