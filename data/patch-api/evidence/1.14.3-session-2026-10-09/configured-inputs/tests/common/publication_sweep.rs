//! Register-driven publication/absence sweep shared by the per-patch sweep tests.
//! Publication/absence only: no signature, output, security, or behavior parity claim.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde_json::{Value, json};
use wow_ui_sim::lua_api::WowLuaEnv;

/// One patch's sweep inputs. Env var names select a scratch register / results file.
pub(crate) struct SweepSpec {
    pub register: &'static str,
    pub known_gaps: &'static str,
    pub row_count: usize,
    pub register_env: &'static str,
    pub out_env: &'static str,
    /// Later patches, oldest first, within the same client line. A later add/remove
    /// overrides expected publication (recorded as `superseded_by`). Registers from
    /// another client history never supersede this patch.
    pub later_registers: &'static [&'static str],
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ClientLine {
    #[default]
    Retail,
    MistsClassic,
    ClassicEra,
    WrathClassic,
}

impl ClientLine {
    fn matches_profile(self, profile: wow_ui_sim::client_profile::ClientProfile) -> bool {
        use wow_ui_sim::client_profile::ClientProfile;
        matches!(
            (self, profile),
            (Self::Retail, ClientProfile::Retail | ClientProfile::Ptr)
                | (Self::MistsClassic, ClientProfile::Mists)
                | (Self::WrathClassic, ClientProfile::Wrath)
                | (
                    Self::ClassicEra,
                    ClientProfile::Era | ClientProfile::Anniversary
                )
        )
    }
}

#[derive(Deserialize)]
struct Register {
    #[serde(default)]
    client_line: ClientLine,
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
pub(crate) struct Entry {
    pub(crate) id: String,
    pub(crate) section: String,
    pub(crate) direction: String,
    pub(crate) symbol: String,
    #[serde(default, alias = "default")]
    pub(crate) page_default: Option<String>,
    #[serde(default)]
    pub(crate) kind: Option<String>,
}

/// Expected publication after applying later-patch supersession.
struct Expectation<'a> {
    removed: bool,
    superseded_by: Option<&'a str>,
}

type ProbeResult = (String, String, bool, Option<String>, Option<String>);

// One classifier; constructor choices are per object kind, never per source symbol.
const CLASSIFIER: &str = r#"
local section, symbol, removed, entryKind, aliasTarget, aliasSource = ...
local function result(kind, detail, ok, value, default)
    return kind, detail, ok, value, default
end
local function matches_type(raw, lookup)
    if removed then return raw == 'nil' and lookup == 'nil' end
    return raw == 'function' or raw == 'table'
end
-- Removed symbols may be republished by cached Blizzard deprecation fallbacks
-- (loadDeprecationFallbacks defaults on). Direct aliases retain the destination's
-- source, so loaded cached assignments also need exact target identity proof.
local function deprecated_fallback_source(value)
    if type(value) ~= 'function' then return nil end
    local source = debug.getinfo(value, 'S').source or ''
    if string.find(source, 'Deprecated', 1, true) then return source end
    if aliasTarget ~= '' and GetCVarBool('loadDeprecationFallbacks') then
        local target = _G
        for part in string.gmatch(aliasTarget, '[^.]+') do
            if type(target) ~= 'table' then return nil end
            target = rawget(target, part)
        end
        if rawequal(value, target) then return aliasSource .. '; alias=' .. aliasTarget end
    end
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
        Object = function() return frame end,
        Frame = function() return frame end,
        FrameScriptObject = function() return frame end,
        ScriptRegion = function() return frame end,
        ScriptRegionResizing = function() return frame end,
        WorldFrame = function() return WorldFrame end,
        Scale = function() return frame:CreateAnimationGroup():CreateAnimation('Scale') end,
        Path = function() return frame:CreateAnimationGroup():CreateAnimation('Path') end,
        FlipBook = function()
            return frame:CreateTexture():CreateAnimationGroup():CreateAnimation('FlipBook')
        end,
        Region = function() return frame:CreateTexture() end,
        Line = function() return frame:CreateLine() end,
        Alpha = function() return frame:CreateAnimationGroup():CreateAnimation('Alpha') end,
        ColorCurveObject = function() return C_CurveUtil.CreateColorCurve() end,
        CurveObject = function() return C_CurveUtil.CreateCurve() end,
        -- CurveObjectBase is the interface implemented by the scalar curve.
        CurveObjectBase = function() return C_CurveUtil.CreateCurve() end,
        UnitHealPredictionCalculator = function() return CreateUnitHealPredictionCalculator() end,
        FontString = function() return frame:CreateFontString() end,
        -- FontInstance is an interface shared by fonts and text regions, not a frame type.
        FontInstance = function() return frame:CreateFontString() end,
        TextureBase = function() return frame:CreateTexture() end,
        Texture = function() return frame:CreateTexture() end,
        MaskTexture = function() return frame:CreateMaskTexture() end,
        VectorGraphics = function() return frame:CreateVectorGraphics() end,
        AnimationGroup = function() return frame:CreateAnimationGroup() end,
        Animation = function() return frame:CreateAnimationGroup():CreateAnimation() end,
        VertexColor = function()
            return frame:CreateTexture():CreateAnimationGroup():CreateAnimation('VertexColor')
        end,
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
local function probe_command()
    local present = false
    for _, record in ipairs(C_Console.GetAllCommands()) do
        if record.command == symbol and record.commandType == Enum.ConsoleCommandType.Command then
            present = true
            break
        end
    end
    return result('command', 'Command record present=' .. tostring(present), present ~= removed)
end
local function classify()
    if entryKind == 'widget-script' then
        -- Older handler links include an owner; unqualified movie handlers retain their owner.
        local owner, script = string.match(symbol, '^([^:]+):([^:]+)$')
        if not owner then owner, script = string.match(symbol, '^(%S+)%s+(%S+)$') end
        -- Literal OnTooltip handlers belong to GameTooltip, not MovieFrame.
        owner = owner or (string.match(symbol, '^OnTooltip') and 'GameTooltip' or 'MovieFrame')
        script = script or symbol
        local object = create_object(owner)
        local supported = object:HasScript(script)
        return result('widget-script', owner .. ' HasScript=' .. tostring(supported), supported ~= removed)
    end
    if entryKind == 'click-modifier' then
        local value = GetModifiedClick(symbol)
        local unknown = GetModifiedClick('PUBLICATION_SWEEP_UNKNOWN_CLICK_MODIFIER')
        -- The temporary input defaults return NONE for unknown actions too.
        -- No catalog discriminates existence; this is not a console command.
        return result('unprobeable', 'click-modifier value=' .. tostring(value)
            .. '; unknown=' .. tostring(unknown) .. '; no discriminating catalog', false)
    end
    if entryKind == 'command' then return probe_command() end
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

pub(crate) fn probe_entry(
    env: &WowLuaEnv,
    entry: &Entry,
    removed: bool,
    aliases: &BTreeMap<String, (String, String)>,
) -> ProbeResult {
    let (target, source) = aliases.get(&entry.symbol).cloned().unwrap_or_default();
    let code = format!(
        "return (function(...) {CLASSIFIER} end)({}, {}, {}, {}, {}, {})",
        quote_lua(&entry.section),
        quote_lua(&entry.symbol),
        removed,
        quote_lua(entry.kind.as_deref().unwrap_or("")),
        quote_lua(&target),
        quote_lua(&source),
    );
    env.eval::<ProbeResult>(&code)
        .unwrap_or_else(|error| ("probe-error".into(), error.to_string(), false, None, None))
}

fn classify_entry(
    env: &WowLuaEnv,
    entry: &Entry,
    expectation: &Expectation,
    aliases: &BTreeMap<String, (String, String)>,
) -> Value {
    let (kind, detail, ok, value, default) = probe_entry(env, entry, expectation.removed, aliases);
    let default_mismatch = entry.section == "cvars"
        && entry.kind.as_deref() != Some("command")
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
        assert_eq!(
            register.entries.len(),
            row_count,
            "register row count changed"
        );
    }
    for entry in &register.entries {
        assert!(matches!(
            entry.section.as_str(),
            "global-api"
                | "framexml"
                | "scriptobjects"
                | "widgets"
                | "events"
                | "cvars"
                | "commands"
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
fn later_publication(spec: &SweepSpec, client_line: ClientLine) -> BTreeMap<String, Entry> {
    let mut latest = BTreeMap::new();
    for source in spec.later_registers {
        let register = parse_register(source, None);
        if register.client_line != client_line {
            continue;
        }
        for entry in register.entries {
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
        _ => Expectation {
            removed: own_removed,
            superseded_by: None,
        },
    }
}

fn write_results_if_requested(out_env: &str, results: &BTreeMap<String, Value>) {
    if let Some(path) = std::env::var_os(out_env) {
        let json = serde_json::to_vec_pretty(results).expect("serialize sweep results");
        std::fs::write(&path, json)
            .unwrap_or_else(|error| panic!("write {out_env} {path:?}: {error}"));
    }
}

/// Attribute only direct assignments in loaded, unmodified cached deprecation files
/// (any `*Deprecated*.lua` under a loaded addon). No hand-maintained symbol whitelist,
/// and no source rewrite; the probe still requires exact target identity at runtime.
pub(crate) fn read_deprecated_aliases(env: &WowLuaEnv) -> BTreeMap<String, (String, String)> {
    let root =
        wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("resolve cached Blizzard UI");
    let mut aliases = BTreeMap::new();
    for addon in read_sorted_dir(&root) {
        let Some(name) = addon
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_owned)
        else {
            continue;
        };
        if !addon.is_dir() || !addon_is_loaded(env, &name) {
            continue;
        }
        for file in deprecation_files(&addon) {
            let source = std::fs::read_to_string(&file)
                .unwrap_or_else(|error| panic!("read {}: {error}", file.display()));
            let label = file.to_string_lossy();
            aliases.extend(
                source
                    .lines()
                    .filter_map(|line| parse_deprecated_alias(line, &label)),
            );
        }
    }
    aliases
}

fn addon_is_loaded(env: &WowLuaEnv, name: &str) -> bool {
    env.eval(&format!("return C_AddOns.IsAddOnLoaded({name:?}) == true"))
        .expect("query addon load state")
}

fn read_sorted_dir(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
        .map(|entry| entry.expect("read dir entry").path())
        .collect();
    entries.sort();
    entries
}

fn deprecation_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for path in read_sorted_dir(dir) {
        if path.is_dir() {
            files.extend(deprecation_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "lua")
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("Deprecated"))
        {
            files.push(path);
        }
    }
    files
}

fn parse_deprecated_alias(line: &str, source: &str) -> Option<(String, (String, String))> {
    let (name, target) = line.trim().trim_end_matches(';').split_once('=')?;
    let name = name.trim();
    let target = target.trim();
    let is_identifier = |value: &str| {
        !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    };
    let is_path = |value: &str| value.split('.').all(is_identifier);
    let (namespace, member) = target.split_once('.')?;
    if !is_path(name)
        || name.matches('.').count() > 1
        || !namespace.starts_with("C_")
        || !is_identifier(namespace)
        || !is_identifier(member)
    {
        return None;
    }
    Some((name.into(), (target.into(), source.into())))
}

/// Probe every register row in one cached Game environment and require the non-ok
/// ID set to equal the reviewed known-gap set exactly.
pub(crate) fn run_publication_sweep(env: &WowLuaEnv, spec: &SweepSpec) {
    run_sweep(env, spec, || read_deprecated_aliases(env));
}

/// Bare factory measurement: no cached publisher/deprecation files are consulted.
#[cfg(any(feature = "client-wrath", feature = "client-retail"))]
pub(crate) fn run_factory_publication_sweep(env: &WowLuaEnv, spec: &SweepSpec) {
    run_sweep(env, spec, BTreeMap::new);
}

fn run_sweep(
    env: &WowLuaEnv,
    spec: &SweepSpec,
    read_aliases: impl FnOnce() -> BTreeMap<String, (String, String)>,
) {
    let register = read_register(spec);
    assert!(
        register
            .client_line
            .matches_profile(wow_ui_sim::client_profile::ACTIVE),
        "register client line does not match active profile"
    );
    let later = later_publication(spec, register.client_line);
    let known: BTreeSet<String> =
        serde_json::from_str(spec.known_gaps).expect("parse known-gap IDs");
    let aliases = read_aliases();
    let results = register
        .entries
        .iter()
        .map(|entry| {
            let expectation = expectation_for(entry, &later);
            (
                entry.id.clone(),
                classify_entry(env, entry, &expectation, &aliases),
            )
        })
        .collect::<BTreeMap<_, _>>();
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
