use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::process::{Command, Stdio};
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

const PATCH_MANIFEST_SCHEMA: &str = "framexml-patch-audit/v2";
const PERMANENT_PROJECT_SCOPE_RULE: &str = "permanent-project-scope";
const INTENTIONAL_GAPS_REFERENCE: &str = "AGENTS.md#intentional-gaps";
const INTENTIONAL_GAPS_HEADING: &str = "## Intentional Gaps";
const NO_3D_RULE_MARKER: &str = "**No 3D rendering**";
const PROVENANCE_ONLY_NOTE: &str = "Provenance-only: no runtime behavior claimed.";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchAuditManifest {
    pub schema: String,
    pub patch: String,
    pub target: AuditTarget,
    pub source: PatchListSource,
    pub output: PatchAuditOutput,
    pub rows: Vec<PatchAuditRow>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditTarget {
    pub flavor: AuditFlavor,
    pub build: String,
    pub cache_manifest: String,
    pub cache_hash: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchListSource {
    pub path: String,
    pub hash: String,
    pub added_count: usize,
    #[serde(default)]
    pub changed_count: usize,
    pub removed_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PatchSourceOccurrence {
    direction: ChangeDirection,
    category: String,
    symbol: String,
    detail: Option<String>,
    #[serde(rename = "before")]
    _before: Option<serde_json::Value>,
    #[serde(rename = "after")]
    _after: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchAuditOutput {
    pub checklist: String,
    pub inventory: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchAuditRow {
    pub id: String,
    pub symbol: String,
    pub change: ChangeDirection,
    pub status: Option<AuditStatus>,
    pub resolution: ResolutionKind,
    #[serde(default)]
    pub provenance_only: bool,
    pub owner: String,
    pub load_addon: Option<String>,
    pub evidence: Vec<AuditEvidence>,
    pub tests: Vec<String>,
    pub assertions: Vec<AuditAssertion>,
    pub commit: Option<String>,
    pub approval_id: Option<String>,
    pub scope_exception: Option<ScopeException>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeException {
    pub rule: String,
    pub reference: String,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeDirection {
    Added,
    Changed,
    Removed,
}

impl ChangeDirection {
    fn as_str(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Changed => "changed",
            Self::Removed => "removed",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuditStatus {
    Implemented,
    BestEffort,
    EvidenceRequired,
    ExceptionRequested,
}

impl AuditStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Implemented => "implemented",
            Self::BestEffort => "best-effort",
            Self::EvidenceRequired => "evidence-required",
            Self::ExceptionRequested => "exception-requested",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ResolutionKind {
    Untriaged,
    VendorPresent,
    Compat,
    Behavioral,
    ProvenanceOnly,
    LoadOnDemand,
    Removed,
    CrossFlavor,
    StaleSnapshot,
    ReversedSnapshot,
    Unsafe,
    Impossible,
}

impl ResolutionKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Untriaged => "untriaged",
            Self::VendorPresent => "vendor-present",
            Self::Compat => "compat",
            Self::Behavioral => "behavioral",
            Self::ProvenanceOnly => "provenance-only",
            Self::LoadOnDemand => "load-on-demand",
            Self::Removed => "removed",
            Self::CrossFlavor => "cross-flavor",
            Self::StaleSnapshot => "stale-snapshot",
            Self::ReversedSnapshot => "reversed-snapshot",
            Self::Unsafe => "unsafe",
            Self::Impossible => "impossible",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuditFlavor {
    Retail,
    Ptr,
    Wrath,
    Mists,
    Era,
    Anniversary,
    #[serde(rename = "wowforever")]
    WowForever,
}

impl AuditFlavor {
    fn as_str(self) -> &'static str {
        match self {
            Self::Retail => "retail",
            Self::Ptr => "ptr",
            Self::Wrath => "wrath",
            Self::Mists => "mists",
            Self::Era => "era",
            Self::Anniversary => "anniversary",
            Self::WowForever => "wowforever",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuditPhase {
    Initialization,
    PostCore,
    PostLoad,
    BeforeAddon,
    AfterAddon,
    PostReset,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditEvidence {
    pub kind: EvidenceKind,
    pub reference: String,
    pub summary: String,
    pub source_hash: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceKind {
    Source,
    Runtime,
    Test,
    Manual,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditAssertion {
    pub flavor: AuditFlavor,
    pub phase: AuditPhase,
    pub expected: ExpectedPresence,
    pub expected_type: Option<LuaType>,
    pub addon: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExpectedPresence {
    Present,
    Absent,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LuaType {
    Function,
    Table,
    String,
    Number,
    Boolean,
    Userdata,
    Thread,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationSet {
    pub schema: String,
    pub manifest_hash: String,
    pub observations: Vec<Observation>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub row_id: String,
    pub flavor: AuditFlavor,
    pub phase: AuditPhase,
    pub present: bool,
    pub observed_type: Option<LuaType>,
    pub addon: Option<String>,
}

impl AuditFlavor {
    fn active() -> Self {
        match ACTIVE {
            ClientProfile::Retail => Self::Retail,
            ClientProfile::Ptr => Self::Ptr,
            ClientProfile::Wrath => Self::Wrath,
            ClientProfile::Mists => Self::Mists,
            ClientProfile::Era => Self::Era,
            ClientProfile::Anniversary => Self::Anniversary,
            ClientProfile::WowForever => Self::WowForever,
        }
    }
}

pub fn observe_assertion(
    env: &WowLuaEnv,
    row: &PatchAuditRow,
    assertion: &AuditAssertion,
) -> Result<Observation, String> {
    let active_flavor = require_active_flavor(row, assertion)?;
    let (present, observed_type) = read_lua_symbol(env, row)?;
    Ok(Observation {
        row_id: row.id.clone(),
        flavor: active_flavor,
        phase: assertion.phase,
        present,
        observed_type,
        addon: assertion.addon.clone(),
    })
}

fn require_active_flavor(
    row: &PatchAuditRow,
    assertion: &AuditAssertion,
) -> Result<AuditFlavor, String> {
    let active_flavor = AuditFlavor::active();
    if assertion.flavor == active_flavor {
        return Ok(active_flavor);
    }
    Err(format!(
        "row {} requires {} but active profile is {}",
        row.id,
        assertion.flavor.as_str(),
        active_flavor.as_str()
    ))
}

fn read_lua_symbol(
    env: &WowLuaEnv,
    row: &PatchAuditRow,
) -> Result<(bool, Option<LuaType>), String> {
    let symbol = serde_json::to_string(&row.symbol)
        .map_err(|error| format!("failed to encode symbol {}: {error}", row.id))?;
    let script = format!(
        r#"
        local value = _G
        for part in string.gmatch({symbol}, "[^.]+") do
            if type(value) ~= "table" then return false, nil end
            value = value[part]
            if value == nil then return false, nil end
        end
        return true, type(value)
        "#
    );
    let (present, observed_type): (bool, Option<String>) = env
        .eval(&script)
        .map_err(|error| format!("failed to observe {}: {error}", row.id))?;
    let observed_type = observed_type.as_deref().map(parse_lua_type).transpose()?;
    Ok((present, observed_type))
}

fn parse_lua_type(lua_type: &str) -> Result<LuaType, String> {
    match lua_type {
        "function" => Ok(LuaType::Function),
        "table" => Ok(LuaType::Table),
        "string" => Ok(LuaType::String),
        "number" => Ok(LuaType::Number),
        "boolean" => Ok(LuaType::Boolean),
        "userdata" => Ok(LuaType::Userdata),
        "thread" => Ok(LuaType::Thread),
        other => Err(format!("unsupported Lua observation type: {other}")),
    }
}

pub fn generate_initialization_observations(
    manifest: &PatchAuditManifest,
    manifest_json: &str,
) -> Result<ObservationSet, String> {
    let active_flavor = AuditFlavor::active();
    if manifest.target.flavor != active_flavor {
        return Err(format!(
            "manifest targets {} but active profile is {}",
            manifest.target.flavor.as_str(),
            active_flavor.as_str()
        ));
    }
    let env = WowLuaEnv::new().map_err(|error| format!("failed to initialize Lua: {error}"))?;
    let mut observations = Vec::new();
    for row in &manifest.rows {
        for assertion in &row.assertions {
            if assertion.phase == AuditPhase::Initialization && assertion.flavor == active_flavor {
                observations.push(observe_assertion(&env, row, assertion)?);
            }
        }
    }
    Ok(build_observation_set(manifest_json, observations))
}

pub fn build_observation_set(
    manifest_json: &str,
    observations: Vec<Observation>,
) -> ObservationSet {
    ObservationSet {
        schema: "framexml-patch-observations/v1".to_string(),
        manifest_hash: format!("{:x}", Sha256::digest(manifest_json.as_bytes())),
        observations,
    }
}

pub fn parse_manifest(json: &str) -> Result<PatchAuditManifest, String> {
    serde_json::from_str(json).map_err(|error| format!("invalid patch audit manifest: {error}"))
}

pub fn parse_observations(json: &str) -> Result<ObservationSet, String> {
    serde_json::from_str(json).map_err(|error| format!("invalid observation set: {error}"))
}

pub fn validate_manifest(manifest: &PatchAuditManifest) -> Result<(), String> {
    validate_manifest_metadata(manifest)?;
    let actual_counts = validate_manifest_rows(&manifest.rows, manifest.target.flavor)?;
    validate_direction_count("added", manifest.source.added_count, actual_counts["added"])?;
    validate_direction_count(
        "changed",
        manifest.source.changed_count,
        actual_counts["changed"],
    )?;
    validate_direction_count(
        "removed",
        manifest.source.removed_count,
        actual_counts["removed"],
    )
}

fn validate_manifest_metadata(manifest: &PatchAuditManifest) -> Result<(), String> {
    if manifest.schema != PATCH_MANIFEST_SCHEMA {
        return Err(format!("unsupported manifest schema: {}", manifest.schema));
    }
    require_text("patch", &manifest.patch)?;
    require_text("target.build", &manifest.target.build)?;
    require_text("target.cache_manifest", &manifest.target.cache_manifest)?;
    require_sha256("target.cache_hash", &manifest.target.cache_hash)?;
    require_text("source.path", &manifest.source.path)?;
    require_sha256("source.hash", &manifest.source.hash)?;
    require_text("output.checklist", &manifest.output.checklist)?;
    require_text("output.inventory", &manifest.output.inventory)
}

fn validate_manifest_rows(
    rows: &[PatchAuditRow],
    target_flavor: AuditFlavor,
) -> Result<BTreeMap<&'static str, usize>, String> {
    let mut row_ids = HashSet::new();
    let mut counts = BTreeMap::from([("added", 0usize), ("changed", 0usize), ("removed", 0usize)]);
    for row in rows {
        require_text("row.symbol", &row.symbol)?;
        validate_symbol_path(&row.symbol)?;
        let expected_id = format!("{}:{}", row.change.as_str(), row.symbol);
        if row.id != expected_id {
            return Err(format!("row {} must use id {expected_id}", row.id));
        }
        if !row_ids.insert(row.id.as_str()) {
            return Err(format!("duplicate row id: {}", row.id));
        }
        *counts
            .get_mut(row.change.as_str())
            .expect("known direction") += 1;
        require_text(&format!("{}.owner", row.id), &row.owner)?;
        validate_row(row, target_flavor)?;
    }
    Ok(counts)
}

fn validate_symbol_path(symbol: &str) -> Result<(), String> {
    let valid = symbol.split('.').all(|segment| {
        let mut characters = segment.chars();
        characters
            .next()
            .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
            && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
    });
    if !valid {
        return Err(format!("invalid symbol path: {symbol}"));
    }
    Ok(())
}

fn validate_direction_count(direction: &str, expected: usize, actual: usize) -> Result<(), String> {
    if actual != expected {
        return Err(format!(
            "{direction} count mismatch: source={expected} rows={actual}"
        ));
    }
    Ok(())
}

fn validate_row(row: &PatchAuditRow, target_flavor: AuditFlavor) -> Result<(), String> {
    validate_provenance_only_flag(row)?;
    if row.resolution == ResolutionKind::Untriaged {
        return validate_untriaged_row(row);
    }
    validate_resolved_row(row, target_flavor)
}

fn validate_provenance_only_flag(row: &PatchAuditRow) -> Result<(), String> {
    let uses_provenance_resolution = row.resolution == ResolutionKind::ProvenanceOnly;
    if row.provenance_only == uses_provenance_resolution {
        return Ok(());
    }
    Err(format!(
        "row {} provenance_only flag and provenance-only resolution must match",
        row.id
    ))
}

fn validate_untriaged_row(row: &PatchAuditRow) -> Result<(), String> {
    if row.status.is_some() {
        return Err(format!("row {} untriaged status must be null", row.id));
    }
    if let Some(notes) = &row.notes {
        require_text(&format!("{}.notes", row.id), notes)?;
    }
    let has_resolution_data = row.provenance_only
        || !row.evidence.is_empty()
        || !row.tests.is_empty()
        || !row.assertions.is_empty()
        || row.commit.is_some()
        || row.approval_id.is_some()
        || row.scope_exception.is_some()
        || row.load_addon.is_some();
    if has_resolution_data {
        return Err(format!("row {} untriaged fields must remain empty", row.id));
    }
    Ok(())
}

fn validate_resolved_row(row: &PatchAuditRow, target_flavor: AuditFlavor) -> Result<(), String> {
    let status = row
        .status
        .ok_or_else(|| format!("row {} resolved status must be set", row.id))?;
    validate_status_resolution(row, status)?;
    validate_load_addon_field(row)?;
    validate_evidence(row)?;
    validate_assertions(row)?;
    validate_resolution_contract(row, target_flavor)?;
    validate_focused_tests(row, status)?;
    validate_optional_metadata(row)
}

fn validate_load_addon_field(row: &PatchAuditRow) -> Result<(), String> {
    match (row.resolution, row.load_addon.as_deref()) {
        (ResolutionKind::LoadOnDemand, Some(addon)) => {
            require_text(&format!("{}.load_addon", row.id), addon)
        }
        (ResolutionKind::LoadOnDemand, None) => {
            Err(format!("row {} load-on-demand requires load_addon", row.id))
        }
        (_, Some(_)) => Err(format!(
            "row {} load_addon is only valid for load-on-demand resolution",
            row.id
        )),
        (_, None) => Ok(()),
    }
}

fn validate_evidence(row: &PatchAuditRow) -> Result<(), String> {
    if row.evidence.is_empty() {
        return Err(format!("row {} requires evidence", row.id));
    }
    for evidence in &row.evidence {
        require_text(
            &format!("{}.evidence.reference", row.id),
            &evidence.reference,
        )?;
        require_text(&format!("{}.evidence.summary", row.id), &evidence.summary)?;
        let hash = evidence.source_hash.as_deref().unwrap_or_default();
        require_sha256(&format!("{}.evidence.source_hash", row.id), hash)?;
    }
    Ok(())
}

fn validate_assertions(row: &PatchAuditRow) -> Result<(), String> {
    if row.resolution == ResolutionKind::Behavioral {
        return if row.assertions.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "row {} behavioral resolution must not carry Lua presence assertions",
                row.id
            ))
        };
    }
    if matches!(
        row.resolution,
        ResolutionKind::Unsafe | ResolutionKind::Impossible | ResolutionKind::ProvenanceOnly
    ) && row.assertions.is_empty()
    {
        return Ok(());
    }
    if row.assertions.is_empty() {
        return Err(format!("row {} requires an assertion", row.id));
    }
    for assertion in &row.assertions {
        validate_assertion(row, assertion)?;
    }
    Ok(())
}

fn validate_focused_tests(row: &PatchAuditRow, status: AuditStatus) -> Result<(), String> {
    if row.resolution == ResolutionKind::ProvenanceOnly {
        return Ok(());
    }
    if matches!(
        status,
        AuditStatus::EvidenceRequired | AuditStatus::ExceptionRequested
    ) {
        return Ok(());
    }
    if row.tests.is_empty() {
        return Err(format!("row {} requires a focused test", row.id));
    }
    for test in &row.tests {
        require_text(&format!("{}.tests", row.id), test)?;
    }
    Ok(())
}

fn validate_optional_metadata(row: &PatchAuditRow) -> Result<(), String> {
    for (field, value) in [
        ("commit", row.commit.as_deref()),
        ("approval_id", row.approval_id.as_deref()),
        ("notes", row.notes.as_deref()),
    ] {
        if let Some(value) = value {
            require_text(&format!("{}.{}", row.id, field), value)?;
        }
    }
    if row.approval_id.is_some() && row.scope_exception.is_some() {
        return Err(format!(
            "row {} cannot set both approval_id and scope_exception",
            row.id
        ));
    }
    if let Some(scope_exception) = &row.scope_exception {
        validate_scope_exception(row, scope_exception)?;
    }
    Ok(())
}

fn validate_scope_exception(
    row: &PatchAuditRow,
    scope_exception: &ScopeException,
) -> Result<(), String> {
    require_text(
        &format!("{}.scope_exception.rule", row.id),
        &scope_exception.rule,
    )?;
    require_text(
        &format!("{}.scope_exception.reference", row.id),
        &scope_exception.reference,
    )?;
    require_text(
        &format!("{}.scope_exception.summary", row.id),
        &scope_exception.summary,
    )?;
    if row.status != Some(AuditStatus::ExceptionRequested)
        || row.resolution != ResolutionKind::Impossible
    {
        return Err(format!(
            "row {} scope_exception requires exception-requested status with impossible resolution",
            row.id
        ));
    }
    if scope_exception.rule != PERMANENT_PROJECT_SCOPE_RULE {
        return Err(format!(
            "row {} scope_exception rule must be {PERMANENT_PROJECT_SCOPE_RULE}",
            row.id
        ));
    }
    if scope_exception.reference != INTENTIONAL_GAPS_REFERENCE {
        return Err(format!(
            "row {} scope_exception reference must be {INTENTIONAL_GAPS_REFERENCE}",
            row.id
        ));
    }
    Ok(())
}

fn validate_status_resolution(row: &PatchAuditRow, status: AuditStatus) -> Result<(), String> {
    let allowed = match row.resolution {
        ResolutionKind::Untriaged => false,
        ResolutionKind::CrossFlavor
        | ResolutionKind::StaleSnapshot
        | ResolutionKind::ReversedSnapshot
        | ResolutionKind::ProvenanceOnly => status == AuditStatus::BestEffort,
        ResolutionKind::Unsafe | ResolutionKind::Impossible => matches!(
            status,
            AuditStatus::EvidenceRequired | AuditStatus::ExceptionRequested
        ),
        ResolutionKind::VendorPresent
        | ResolutionKind::Compat
        | ResolutionKind::Behavioral
        | ResolutionKind::LoadOnDemand
        | ResolutionKind::Removed => {
            matches!(status, AuditStatus::Implemented | AuditStatus::BestEffort)
        }
    };
    if !allowed {
        return Err(format!(
            "row {} status {} is incompatible with resolution {}",
            row.id,
            status.as_str(),
            row.resolution.as_str()
        ));
    }
    Ok(())
}

fn validate_assertion(row: &PatchAuditRow, assertion: &AuditAssertion) -> Result<(), String> {
    match (assertion.expected, assertion.expected_type) {
        (ExpectedPresence::Present, None) => {
            return Err(format!(
                "row {} present assertion requires expected_type",
                row.id
            ));
        }
        (ExpectedPresence::Absent, Some(_)) => {
            return Err(format!(
                "row {} absent assertion must not set expected_type",
                row.id
            ));
        }
        _ => {}
    }
    if let Some(addon) = &assertion.addon {
        require_text(&format!("{}.assertion.addon", row.id), addon)?;
    }
    Ok(())
}

fn validate_resolution_contract(
    row: &PatchAuditRow,
    target_flavor: AuditFlavor,
) -> Result<(), String> {
    match row.resolution {
        ResolutionKind::LoadOnDemand => validate_load_on_demand_contract(row, target_flavor),
        ResolutionKind::Removed => validate_removed_contract(row),
        ResolutionKind::CrossFlavor => validate_cross_flavor_contract(row, target_flavor),
        ResolutionKind::StaleSnapshot | ResolutionKind::ReversedSnapshot => {
            validate_snapshot_contract(row)
        }
        ResolutionKind::VendorPresent | ResolutionKind::Compat => validate_presence_contract(row),
        ResolutionKind::Behavioral => validate_behavioral_contract(row),
        ResolutionKind::ProvenanceOnly => validate_provenance_only_contract(row),
        ResolutionKind::Unsafe | ResolutionKind::Impossible | ResolutionKind::Untriaged => Ok(()),
    }
}

fn validate_removed_contract(row: &PatchAuditRow) -> Result<(), String> {
    require_assertion(
        row,
        |assertion| {
            assertion.phase == AuditPhase::PostReset
                && assertion.expected == ExpectedPresence::Absent
        },
        "removed requires post-reset absence",
    )
}

fn validate_cross_flavor_contract(
    row: &PatchAuditRow,
    target_flavor: AuditFlavor,
) -> Result<(), String> {
    require_assertion(
        row,
        |assertion| {
            assertion.flavor == target_flavor && assertion.expected == ExpectedPresence::Absent
        },
        "cross-flavor requires target absence",
    )
}

fn validate_snapshot_contract(row: &PatchAuditRow) -> Result<(), String> {
    require_assertion(
        row,
        |assertion| assertion.expected == ExpectedPresence::Absent,
        "snapshot mismatch requires absence",
    )
}

fn validate_behavioral_contract(row: &PatchAuditRow) -> Result<(), String> {
    if row
        .evidence
        .iter()
        .any(|evidence| evidence.kind == EvidenceKind::Test)
    {
        return Ok(());
    }
    Err(format!(
        "row {} behavioral resolution requires test evidence",
        row.id
    ))
}

fn validate_provenance_only_contract(row: &PatchAuditRow) -> Result<(), String> {
    if !row.provenance_only {
        return Err(format!(
            "row {} provenance-only resolution requires provenance_only=true",
            row.id
        ));
    }
    if row
        .evidence
        .iter()
        .any(|evidence| evidence.kind != EvidenceKind::Source)
    {
        return Err(format!(
            "row {} provenance-only resolution permits source evidence only",
            row.id
        ));
    }
    if !row.tests.is_empty() || !row.assertions.is_empty() {
        return Err(format!(
            "row {} provenance-only resolution must not carry runtime tests or assertions",
            row.id
        ));
    }
    if row.commit.is_some() || row.approval_id.is_some() || row.scope_exception.is_some() {
        return Err(format!(
            "row {} provenance-only resolution must not carry commit, approval, or scope metadata",
            row.id
        ));
    }
    if row.notes.as_deref() != Some(PROVENANCE_ONLY_NOTE) {
        return Err(format!(
            "row {} provenance-only notes must be exactly {PROVENANCE_ONLY_NOTE:?}",
            row.id
        ));
    }
    Ok(())
}

fn validate_presence_contract(row: &PatchAuditRow) -> Result<(), String> {
    require_assertion(
        row,
        |assertion| assertion.expected == ExpectedPresence::Present,
        "requires a presence assertion",
    )
}

fn validate_load_on_demand_contract(
    row: &PatchAuditRow,
    target_flavor: AuditFlavor,
) -> Result<(), String> {
    let load_addon = row
        .load_addon
        .as_deref()
        .expect("load_addon field validated before lifecycle contract");
    let assertions_match_addon = row.assertions.iter().all(|assertion| {
        assertion.flavor == target_flavor && assertion.addon.as_deref() == Some(load_addon)
    });
    let has_before_absence = row.assertions.iter().any(|assertion| {
        assertion.phase == AuditPhase::BeforeAddon && assertion.expected == ExpectedPresence::Absent
    });
    let has_after_presence = row.assertions.iter().any(|assertion| {
        assertion.phase == AuditPhase::AfterAddon && assertion.expected == ExpectedPresence::Present
    });
    if assertions_match_addon && has_before_absence && has_after_presence {
        return Ok(());
    }
    Err(format!(
        "row {} load-on-demand requires load_addon-matched target-flavor before-absent and after-present assertions",
        row.id
    ))
}

fn require_assertion(
    row: &PatchAuditRow,
    predicate: impl Fn(&AuditAssertion) -> bool,
    failure: &str,
) -> Result<(), String> {
    if row.assertions.iter().any(predicate) {
        return Ok(());
    }
    Err(format!("row {} {failure}", row.id))
}

fn require_text(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(())
}

fn require_sha256(field: &str, value: &str) -> Result<(), String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{field} must be a 64-character SHA-256"));
    }
    Ok(())
}

pub fn validate_repository(manifest: &PatchAuditManifest, root: &Path) -> Result<(), String> {
    validate_manifest(manifest)?;
    validate_file_hash(
        root,
        &manifest.target.cache_manifest,
        &manifest.target.cache_hash,
    )?;
    validate_file_hash(root, &manifest.source.path, &manifest.source.hash)?;
    validate_source_rows(manifest, root)?;
    validate_resolved_row_artifacts(manifest, root)?;
    validate_checklist(manifest, root)?;
    validate_inventory(manifest, root)
}

fn validate_resolved_row_artifacts(
    manifest: &PatchAuditManifest,
    root: &Path,
) -> Result<(), String> {
    for row in manifest
        .rows
        .iter()
        .filter(|row| row.resolution != ResolutionKind::Untriaged)
    {
        for evidence in &row.evidence {
            let path = reference_path(&evidence.reference);
            validate_file_hash(
                root,
                path,
                evidence.source_hash.as_deref().unwrap_or_default(),
            )?;
            if evidence.kind == EvidenceKind::Test {
                validate_test_reference(root, &evidence.reference)?;
            }
        }
        for test in &row.tests {
            validate_test_reference(root, test)?;
        }
        if let Some(commit) = &row.commit {
            validate_commit(root, commit)?;
        }
        if let Some(scope_exception) = &row.scope_exception {
            validate_scope_exception_reference(root, scope_exception)?;
        }
    }
    Ok(())
}

fn validate_scope_exception_reference(
    root: &Path,
    scope_exception: &ScopeException,
) -> Result<(), String> {
    let path = scope_exception
        .reference
        .split_once('#')
        .map_or(scope_exception.reference.as_str(), |(path, _)| path);
    let full_path = root.join(path);
    if !full_path.is_file() {
        return Err(format!(
            "scope exception reference does not exist: {}",
            full_path.display()
        ));
    }
    let contents = std::fs::read_to_string(&full_path).map_err(|error| {
        format!(
            "failed to read scope exception reference {}: {error}",
            full_path.display()
        )
    })?;
    if !contents.contains(INTENTIONAL_GAPS_HEADING) || !contents.contains(NO_3D_RULE_MARKER) {
        return Err(format!(
            "scope exception reference must contain {INTENTIONAL_GAPS_HEADING} and No 3D rendering"
        ));
    }
    Ok(())
}

fn validate_checklist(manifest: &PatchAuditManifest, root: &Path) -> Result<(), String> {
    let checklist_path = root.join(&manifest.output.checklist);
    let actual = std::fs::read_to_string(&checklist_path)
        .map_err(|error| format!("failed to read {}: {error}", checklist_path.display()))?;
    let expected = format!("{}\n", render_checklist(manifest));
    if actual != expected {
        return Err(format!(
            "generated checklist drift: {}",
            checklist_path.display()
        ));
    }
    Ok(())
}

fn validate_source_rows(manifest: &PatchAuditManifest, root: &Path) -> Result<(), String> {
    let contents = std::fs::read_to_string(root.join(&manifest.source.path))
        .map_err(|error| format!("failed to read patch source: {error}"))?;
    let source: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|error| format!("invalid patch source JSON: {error}"))?;
    let expected = source_row_ids(&source)?;
    let actual: Vec<&str> = manifest.rows.iter().map(|row| row.id.as_str()).collect();
    if actual != expected {
        return Err("manifest row order/content differs from patch source".to_string());
    }
    Ok(())
}

fn source_symbols(
    source: &serde_json::Value,
    direction: &str,
    required: bool,
) -> Result<Vec<String>, String> {
    let Some(values) = source.get(direction) else {
        return if required {
            Err(format!("patch source missing {direction} array"))
        } else {
            Ok(Vec::new())
        };
    };
    values
        .as_array()
        .ok_or_else(|| format!("patch source {direction} must be an array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("patch source {direction} contains a non-string"))
        })
        .collect()
}

fn source_occurrence_row_ids(source: &serde_json::Value) -> Result<Vec<String>, String> {
    let values = source["occurrences"]
        .as_array()
        .ok_or_else(|| "patch source occurrences must be an array".to_string())?;
    let mut grouped = BTreeMap::from([
        ("added", Vec::new()),
        ("changed", Vec::new()),
        ("removed", Vec::new()),
    ]);
    for value in values {
        let occurrence: PatchSourceOccurrence =
            serde_json::from_value(value.clone()).map_err(|error| {
                format!("invalid patch source occurrence direction/category/symbol: {error}")
            })?;
        require_text("patch source occurrence.category", &occurrence.category)?;
        if occurrence
            .detail
            .as_deref()
            .is_some_and(|detail| detail.trim().is_empty())
        {
            return Err("patch source occurrence detail must not be blank".to_string());
        }
        validate_symbol_path(&occurrence.symbol)?;
        grouped
            .get_mut(occurrence.direction.as_str())
            .expect("known direction")
            .push(format!(
                "{}:{}",
                occurrence.direction.as_str(),
                occurrence.symbol
            ));
    }
    Ok(grouped.into_values().flatten().collect())
}

fn source_array_row_ids(source: &serde_json::Value) -> Result<Vec<String>, String> {
    Ok(source_symbols(source, "added", true)?
        .into_iter()
        .map(|symbol| format!("added:{symbol}"))
        .chain(
            source_symbols(source, "changed", false)?
                .into_iter()
                .map(|symbol| format!("changed:{symbol}")),
        )
        .chain(
            source_symbols(source, "removed", true)?
                .into_iter()
                .map(|symbol| format!("removed:{symbol}")),
        )
        .collect())
}

fn source_row_ids(source: &serde_json::Value) -> Result<Vec<String>, String> {
    if source.get("occurrences").is_some() {
        source_occurrence_row_ids(source)
    } else {
        source_array_row_ids(source)
    }
}

fn validate_inventory(manifest: &PatchAuditManifest, root: &Path) -> Result<(), String> {
    let path = root.join(&manifest.output.inventory);
    let contents = std::fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let actual: Vec<(String, String)> = contents
        .lines()
        .filter_map(|line| {
            let line = line.strip_prefix("| `")?;
            let (symbol, rest) = line.split_once("` | ")?;
            let (status, _) = rest.split_once(" | ")?;
            Some((symbol.to_string(), status.trim().to_string()))
        })
        .collect();
    let expected: Vec<(String, String)> = manifest
        .rows
        .iter()
        .map(|row| {
            (
                row.symbol.clone(),
                row.status
                    .map_or("untriaged", AuditStatus::as_str)
                    .to_string(),
            )
        })
        .collect();
    if actual != expected {
        return Err(format!("inventory drift: {}", path.display()));
    }
    Ok(())
}

fn reference_path(reference: &str) -> &str {
    reference
        .split_once("::")
        .map_or(reference, |(path, _)| path)
}

fn validate_file_hash(root: &Path, relative: &str, expected: &str) -> Result<(), String> {
    let path = root.join(relative);
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual != expected {
        return Err(format!(
            "hash mismatch for {}: expected {expected}, observed {actual}",
            path.display()
        ));
    }
    Ok(())
}

fn validate_test_reference(root: &Path, reference: &str) -> Result<(), String> {
    require_text("test reference", reference)?;
    let (path, symbol) = reference
        .split_once("::")
        .map_or((reference, None), |(path, symbol)| (path, Some(symbol)));
    let contents = std::fs::read_to_string(root.join(path))
        .map_err(|error| format!("failed to read test {path}: {error}"))?;
    if let Some(symbol) = symbol {
        require_text("test symbol", symbol)?;
        let source = strip_rust_comments(&contents);
        if !defines_rust_test(&source, symbol)? {
            return Err(format!("test {path} does not define test case {symbol}"));
        }
    }
    Ok(())
}

fn defines_rust_test(source: &str, symbol: &str) -> Result<bool, String> {
    let escaped = regex::escape(symbol);
    let function_pattern = format!(r"(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+{escaped}\s*\(");
    let definition = Regex::new(&format!(r"^\s*{function_pattern}"))
        .map_err(|error| format!("invalid test definition pattern: {error}"))?;
    let lines: Vec<&str> = source.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        if definition.is_match(line) && preceding_attributes_include_test(&lines[..index]) {
            return Ok(true);
        }
    }

    let prefork_pattern =
        format!(r"(?m)^[ \t]*prefork_full_ui_case!\s*\{{\s*(?:#\[[^\n]*\]\s*)*{function_pattern}");
    Regex::new(&prefork_pattern)
        .map(|prefork_definition| prefork_definition.is_match(source))
        .map_err(|error| format!("invalid prefork test definition pattern: {error}"))
}

fn preceding_attributes_include_test(lines: &[&str]) -> bool {
    lines
        .iter()
        .rev()
        .map(|line| line.trim())
        .take_while(|line| line.is_empty() || line.starts_with("#["))
        .any(|line| line == "#[test]")
}

fn strip_rust_comments(contents: &str) -> String {
    let mut result = String::with_capacity(contents.len());
    let mut in_block_comment = false;
    for line in contents.lines() {
        let mut remainder = line;
        loop {
            if in_block_comment {
                let Some((_, after)) = remainder.split_once("*/") else {
                    break;
                };
                remainder = after;
                in_block_comment = false;
                continue;
            }
            let before_line_comment = remainder
                .split_once("//")
                .map_or(remainder, |(code, _)| code);
            let Some((before, after)) = before_line_comment.split_once("/*") else {
                result.push_str(before_line_comment);
                break;
            };
            result.push_str(before);
            remainder = after;
            in_block_comment = true;
        }
        result.push('\n');
    }
    result
}

fn validate_commit(root: &Path, commit: &str) -> Result<(), String> {
    let object = Command::new("git")
        .args(["cat-file", "-e", &format!("{commit}^{{commit}}")])
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to inspect commit {commit}: {error}"))?;
    if !object.success() {
        return Err(format!("commit {commit} does not resolve"));
    }
    let ancestor = Command::new("git")
        .args(["merge-base", "--is-ancestor", commit, "HEAD"])
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to inspect ancestry for {commit}: {error}"))?;
    if !ancestor.success() {
        return Err(format!("commit {commit} is not an ancestor of HEAD"));
    }
    Ok(())
}

pub fn validate_complete(
    manifest: &PatchAuditManifest,
    root: &Path,
    manifest_json: &str,
    observations: &ObservationSet,
) -> Result<(), String> {
    validate_repository(manifest, root)?;
    if observations.schema != "framexml-patch-observations/v1" {
        return Err(format!(
            "unsupported observation schema: {}",
            observations.schema
        ));
    }
    validate_observation_binding(manifest_json, observations)?;
    let mut approvals = HashSet::new();
    for row in &manifest.rows {
        validate_completion_row(row, &mut approvals)?;
    }
    validate_observations(manifest, &observations.observations)
}

fn validate_observation_binding(
    manifest_json: &str,
    observations: &ObservationSet,
) -> Result<(), String> {
    let manifest_hash = format!("{:x}", Sha256::digest(manifest_json.as_bytes()));
    if observations.manifest_hash != manifest_hash {
        return Err("observation manifest hash does not match audited manifest".to_string());
    }
    Ok(())
}

fn validate_completion_row<'a>(
    row: &'a PatchAuditRow,
    approvals: &mut HashSet<&'a str>,
) -> Result<(), String> {
    if row.resolution == ResolutionKind::Untriaged {
        return Err(format!("row {} remains untriaged", row.id));
    }
    if row.resolution == ResolutionKind::ProvenanceOnly {
        return Ok(());
    }
    match row.status.expect("resolved row status validated") {
        AuditStatus::EvidenceRequired => Err(format!("row {} remains evidence-required", row.id)),
        AuditStatus::ExceptionRequested => validate_exception_approval(row, approvals),
        AuditStatus::Implemented | AuditStatus::BestEffort if row.commit.is_none() => {
            Err(format!("row {} requires a commit", row.id))
        }
        AuditStatus::Implemented | AuditStatus::BestEffort => Ok(()),
    }
}

fn validate_exception_approval<'a>(
    row: &'a PatchAuditRow,
    approvals: &mut HashSet<&'a str>,
) -> Result<(), String> {
    let Some(approval) = row.approval_id.as_deref() else {
        return if row.scope_exception.is_some() {
            Ok(())
        } else {
            Err(format!(
                "row {} requires an approval_id or scope_exception",
                row.id
            ))
        };
    };
    let prefix = format!("user-chat:{}:", row.id);
    if !approval.starts_with(&prefix) || approval.len() == prefix.len() {
        return Err(format!(
            "row {} approval_id must start with {prefix}",
            row.id
        ));
    }
    if !approvals.insert(approval) {
        return Err(format!("duplicate approval_id: {approval}"));
    }
    Ok(())
}

pub fn validate_observations(
    manifest: &PatchAuditManifest,
    observations: &[Observation],
) -> Result<(), String> {
    let expected_count: usize = manifest.rows.iter().map(|row| row.assertions.len()).sum();
    if observations.len() != expected_count {
        return Err(format!(
            "observation count mismatch: expected {expected_count}, observed {}",
            observations.len()
        ));
    }
    let mut used = HashSet::new();
    for row in &manifest.rows {
        for assertion in &row.assertions {
            let (index, observation) = find_matching_observation(row, assertion, observations)?;
            if !used.insert(index) {
                return Err(format!(
                    "observation {index} matched more than one assertion"
                ));
            }
            validate_observation(row, assertion, observation)?;
        }
    }
    Ok(())
}

fn find_matching_observation<'a>(
    row: &PatchAuditRow,
    assertion: &AuditAssertion,
    observations: &'a [Observation],
) -> Result<(usize, &'a Observation), String> {
    let matches: Vec<(usize, &Observation)> = observations
        .iter()
        .enumerate()
        .filter(|(_, observation)| observation_matches(row, assertion, observation))
        .collect();
    if matches.len() == 1 {
        return Ok(matches[0]);
    }
    Err(format!(
        "{} expected exactly one observation for {:?}/{:?}/{:?}, found {}",
        row.id,
        assertion.flavor,
        assertion.phase,
        assertion.addon,
        matches.len()
    ))
}

fn observation_matches(
    row: &PatchAuditRow,
    assertion: &AuditAssertion,
    observation: &Observation,
) -> bool {
    observation.row_id == row.id
        && observation.flavor == assertion.flavor
        && observation.phase == assertion.phase
        && observation.addon == assertion.addon
}

fn validate_observation(
    row: &PatchAuditRow,
    assertion: &AuditAssertion,
    observation: &Observation,
) -> Result<(), String> {
    let expected_present = assertion.expected == ExpectedPresence::Present;
    if observation.present != expected_present {
        return Err(format!(
            "{} expected present={expected_present}, observed present={}",
            row.id, observation.present
        ));
    }
    if assertion.expected_type != observation.observed_type {
        return Err(format!(
            "{} expected type {:?}, observed {:?}",
            row.id, assertion.expected_type, observation.observed_type
        ));
    }
    Ok(())
}

pub fn render_summary(manifest: &PatchAuditManifest) -> String {
    let mut counts = BTreeMap::from([
        ("implemented", 0usize),
        ("best-effort", 0usize),
        ("evidence-required", 0usize),
        ("exception-requested", 0usize),
        ("untriaged", 0usize),
    ]);
    for row in &manifest.rows {
        let status = row.status.map_or("untriaged", AuditStatus::as_str);
        *counts.get_mut(status).expect("known status") += 1;
    }
    format!(
        "Patch {} for {} {}: {} rows ({} implemented, {} best-effort, {} evidence-required, {} exception-requested, {} untriaged)\nSource: {}\nCache: {} ({})",
        manifest.patch,
        manifest.target.flavor.as_str(),
        manifest.target.build,
        manifest.rows.len(),
        counts["implemented"],
        counts["best-effort"],
        counts["evidence-required"],
        counts["exception-requested"],
        counts["untriaged"],
        manifest.source.path,
        manifest.target.cache_manifest,
        manifest.target.cache_hash,
    )
}

pub fn render_checklist(manifest: &PatchAuditManifest) -> String {
    manifest
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let status = row.status.map_or("untriaged", AuditStatus::as_str);
            format!(
                "{}. [{}] `{}` — {}",
                index + 1,
                status,
                row.id,
                row.resolution.as_str()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use wow_ui_sim::loader::load_addon;

    fn fixture_manifest(row: &str) -> PatchAuditManifest {
        parse_fixture_manifest(row).expect("fixture should parse")
    }

    fn parse_fixture_manifest(row: &str) -> Result<PatchAuditManifest, String> {
        parse_manifest(&format!(
            r#"{{
              "schema":"framexml-patch-audit/v2",
              "patch":"12.1.0",
              "target":{{"flavor":"ptr","build":"12.1.0","cache_manifest":"cache","cache_hash":"{}"}},
              "source":{{"path":"source","hash":"{}","added_count":1,"removed_count":0}},
              "output":{{"checklist":"checklist","inventory":"inventory"}},
              "rows":[{row}]
            }}"#,
            "a".repeat(64),
            "b".repeat(64)
        ))
    }

    fn scope_exception_row(
        resolution: &str,
        approval_id: Option<&str>,
        rule: &str,
        reference: &str,
        summary: &str,
    ) -> String {
        let approval_id = serde_json::to_string(&approval_id).expect("approval ID should encode");
        format!(
            r#"{{
              "id":"added:Fixture","symbol":"Fixture","change":"added",
              "status":"exception-requested","resolution":"{resolution}","owner":"project-scope",
              "evidence":[{{"kind":"source","reference":"fixture.lua","summary":"evidence","source_hash":"{}"}}],
              "tests":[],"assertions":[],"commit":null,"approval_id":{approval_id},
              "scope_exception":{{"rule":"{rule}","reference":"{reference}","summary":"{summary}"}},
              "notes":"scope exception"
            }}"#,
            "c".repeat(64)
        )
    }

    fn resolved_row(status: &str, resolution: &str, assertion: &str) -> String {
        format!(
            r#"{{
              "id":"added:Fixture","symbol":"Fixture","change":"added",
              "status":"{status}","resolution":"{resolution}","owner":"Blizzard_Test",
              "evidence":[{{"kind":"source","reference":"fixture.lua","summary":"evidence","source_hash":"{}"}}],
              "tests":["tests/fixture.rs::fixture_test"],"assertions":[{assertion}],
              "commit":"1234567890","approval_id":null,"notes":null
            }}"#,
            "c".repeat(64)
        )
    }

    fn load_on_demand_row(assertions: &str, load_addon: &str) -> String {
        resolved_row("best-effort", "load-on-demand", assertions).replace(
            r#""owner":"Blizzard_Test","#,
            &format!(r#""owner":"internal-module","load_addon":"{load_addon}","#),
        )
    }

    fn behavioral_row(status: &str, evidence_kind: &str, assertions: &str) -> String {
        format!(
            r#"{{
              "id":"added:Fixture","symbol":"Fixture","change":"added",
              "status":"{status}","resolution":"behavioral","owner":"simulator-model",
              "evidence":[{{"kind":"{evidence_kind}","reference":"tests/fixture.rs::fixture_test","summary":"behavior evidence","source_hash":"{}"}}],
              "tests":["tests/fixture.rs::fixture_test"],"assertions":[{assertions}],
              "commit":"1234567890","approval_id":null,"notes":"behavioral contract"
            }}"#,
            "c".repeat(64)
        )
    }

    fn evidence_required_row(resolution: &str, evidence_kind: &str) -> String {
        format!(
            r#"{{
              "id":"added:Fixture","symbol":"Fixture","change":"added",
              "status":"evidence-required","resolution":"{resolution}","owner":"evidence-collector",
              "evidence":[{{"kind":"{evidence_kind}","reference":"tests/fixture.rs::fixture_test","summary":"evidence pending implementation","source_hash":"{}"}}],
              "tests":[],"assertions":[],"commit":null,"approval_id":null,"notes":"awaiting evidence"
            }}"#,
            "c".repeat(64)
        )
    }

    fn provenance_only_row(
        status: &str,
        resolution: &str,
        provenance_only: Option<bool>,
        evidence_kind: &str,
        tests: &str,
    ) -> String {
        let provenance_only = provenance_only.map_or(String::new(), |value| {
            format!(r#","provenance_only":{value}"#)
        });
        format!(
            r#"{{
              "id":"added:Fixture","symbol":"Fixture","change":"added",
              "status":"{status}","resolution":"{resolution}","owner":"source-register",
              "evidence":[{{"kind":"{evidence_kind}","reference":"data/patch-api/sources/fixture.json","summary":"typedef source metadata","source_hash":"{}"}}],
              "tests":{tests},"assertions":[],"commit":null,"approval_id":null,"scope_exception":null,
              "notes":"Provenance-only: no runtime behavior claimed."{provenance_only}
            }}"#,
            "c".repeat(64)
        )
    }

    #[test]
    fn provenance_only_source_row_validates_and_is_completion_eligible() {
        let manifest = fixture_manifest(&provenance_only_row(
            "best-effort",
            "provenance-only",
            Some(true),
            "source",
            "[]",
        ));

        validate_manifest(&manifest).expect("provenance-only source row should validate");
        let mut approvals = HashSet::new();
        validate_completion_row(&manifest.rows[0], &mut approvals)
            .expect("provenance-only source row should be completion-eligible");
    }

    fn assert_provenance_only_rejected(row: &str) {
        match parse_fixture_manifest(row) {
            Err(error) => assert!(error.contains("provenance"), "unexpected error: {error}"),
            Ok(manifest) => {
                let error = validate_manifest(&manifest)
                    .expect_err("invalid provenance-only row should be rejected");
                assert!(error.contains("provenance"), "unexpected error: {error}");
            }
        }
    }

    #[test]
    fn provenance_only_resolution_requires_explicit_flag() {
        assert_provenance_only_rejected(&provenance_only_row(
            "best-effort",
            "provenance-only",
            None,
            "source",
            "[]",
        ));
    }

    #[test]
    fn provenance_only_rows_reject_runtime_test_evidence() {
        assert_provenance_only_rejected(&provenance_only_row(
            "best-effort",
            "provenance-only",
            Some(true),
            "test",
            r#"["tests/fixture.rs::fixture_test"]"#,
        ));
    }

    #[test]
    fn provenance_only_requires_best_effort_status_and_resolution() {
        assert_provenance_only_rejected(&provenance_only_row(
            "evidence-required",
            "provenance-only",
            Some(true),
            "source",
            "[]",
        ));
        assert_provenance_only_rejected(&provenance_only_row(
            "best-effort",
            "behavioral",
            Some(true),
            "source",
            "[]",
        ));
    }

    #[test]
    fn evidence_required_status_uses_kebab_case_for_deserialization_and_rendering() {
        let status: AuditStatus =
            serde_json::from_str(r#""evidence-required""#).expect("status should parse");

        assert_eq!(status, AuditStatus::EvidenceRequired);
        assert_eq!(status.as_str(), "evidence-required");
    }

    #[test]
    fn evidence_required_unsafe_rows_validate_without_approval_or_commit() {
        let manifest = fixture_manifest(&evidence_required_row("unsafe", "source"));

        validate_manifest(&manifest).expect("evidence-required unsafe row should validate");
        assert!(manifest.rows[0].approval_id.is_none());
        assert!(manifest.rows[0].commit.is_none());
    }

    #[test]
    fn evidence_required_impossible_rows_validate_without_approval_or_commit() {
        let manifest = fixture_manifest(&evidence_required_row("impossible", "source"));

        validate_manifest(&manifest).expect("evidence-required impossible row should validate");
        assert!(manifest.rows[0].approval_id.is_none());
        assert!(manifest.rows[0].commit.is_none());
    }

    #[test]
    fn evidence_required_status_is_rejected_for_behavioral_resolution() {
        let manifest = fixture_manifest(&evidence_required_row("behavioral", "test"));

        let error = validate_manifest(&manifest)
            .expect_err("evidence-required status must not resolve behavioral rows");

        assert!(error.contains("behavioral"), "unexpected error: {error}");
    }

    #[test]
    fn evidence_required_status_cannot_complete() {
        let manifest = fixture_manifest(&evidence_required_row("unsafe", "source"));
        let mut approvals = HashSet::new();

        let error = validate_completion_row(&manifest.rows[0], &mut approvals)
            .expect_err("evidence-required rows must not complete");

        assert!(
            error.contains("evidence-required"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn evidence_required_status_appears_in_summary() {
        let manifest = fixture_manifest(&evidence_required_row("unsafe", "source"));

        assert!(render_summary(&manifest).contains("evidence-required"));
    }

    #[test]
    fn evidence_required_status_appears_in_checklist() {
        let manifest = fixture_manifest(&evidence_required_row("unsafe", "source"));

        assert!(render_checklist(&manifest).contains("[evidence-required]"));
    }

    fn assertion(flavor: &str, phase: &str, expected: &str) -> String {
        let ty = if expected == "present" {
            r#", "expected_type":"function""#
        } else {
            ""
        };
        format!(r#"{{"flavor":"{flavor}","phase":"{phase}","expected":"{expected}"{ty}}}"#)
    }

    #[test]
    fn behavioral_resolution_accepts_implemented_and_best_effort_test_rows() {
        for status in ["implemented", "best-effort"] {
            let manifest = fixture_manifest(&behavioral_row(status, "test", ""));

            validate_manifest(&manifest).expect("behavioral test row should validate");
        }
    }

    #[test]
    fn behavioral_resolution_rejects_lua_presence_assertions() {
        let row = behavioral_row(
            "best-effort",
            "test",
            &assertion("ptr", "post-core", "present"),
        );
        let manifest = fixture_manifest(&row);

        let error = validate_manifest(&manifest)
            .expect_err("behavioral rows must not carry Lua presence assertions");

        assert!(error.contains("behavioral"), "unexpected error: {error}");
    }

    #[test]
    fn behavioral_resolution_requires_test_evidence() {
        let row = behavioral_row("best-effort", "source", "");
        let manifest = fixture_manifest(&row);

        let error = validate_manifest(&manifest)
            .expect_err("behavioral rows need at least one test evidence item");

        assert!(error.contains("test"), "unexpected error: {error}");
    }

    #[test]
    fn behavioral_resolution_rejects_exception_requested_status() {
        let manifest = fixture_manifest(&behavioral_row("exception-requested", "test", ""));

        let error = validate_manifest(&manifest)
            .expect_err("behavioral resolution cannot request an unsafe exception");

        assert!(error.contains("behavioral"), "unexpected error: {error}");
    }

    #[test]
    fn behavioral_rows_accept_zero_runtime_observations() {
        let manifest = fixture_manifest(&behavioral_row("best-effort", "test", ""));
        validate_manifest(&manifest).expect("behavioral row should validate");

        validate_observations(&manifest, &[])
            .expect("behavioral-only manifests should require no runtime observations");
    }

    #[test]
    fn initialization_generator_rejects_manifest_for_other_profile() {
        let row = resolved_row(
            "best-effort",
            "cross-flavor",
            &assertion(AuditFlavor::active().as_str(), "initialization", "absent"),
        );
        let mut manifest = fixture_manifest(&row);
        manifest.target.flavor = if AuditFlavor::active() == AuditFlavor::Ptr {
            AuditFlavor::Retail
        } else {
            AuditFlavor::Ptr
        };

        let error = generate_initialization_observations(&manifest, "manifest bytes")
            .expect_err("other-profile manifest must be rejected");

        assert!(error.contains("targets"));
    }

    #[test]
    fn initialization_generator_emits_only_active_initialization_assertions() {
        let flavor = AuditFlavor::active().as_str();
        let assertions = format!(
            r#"{{"flavor":"{flavor}","phase":"initialization","expected":"absent"}},{{"flavor":"{flavor}","phase":"post-load","expected":"absent"}}"#
        );
        let row = resolved_row("best-effort", "cross-flavor", &assertions);
        let mut manifest = fixture_manifest(&row);
        manifest.target.flavor = AuditFlavor::active();

        let observations = generate_initialization_observations(&manifest, "manifest bytes")
            .expect("initialization observations should generate");

        assert_eq!(observations.observations.len(), 1);
        assert_eq!(
            observations.observations[0].phase,
            AuditPhase::Initialization
        );
    }

    #[test]
    fn runtime_observations_cover_present_absent_and_lod_phases() {
        let flavor = AuditFlavor::active().as_str();
        let env = WowLuaEnv::new().expect("environment should initialize");

        env.exec("Fixture = function() end")
            .expect("vendor fixture should publish");
        let vendor_row = resolved_row(
            "best-effort",
            "vendor-present",
            &assertion(flavor, "post-core", "present"),
        );
        let vendor_manifest = fixture_manifest(&vendor_row);
        let vendor = observe_assertion(
            &env,
            &vendor_manifest.rows[0],
            &vendor_manifest.rows[0].assertions[0],
        )
        .expect("vendor observation should succeed");
        validate_observations(&vendor_manifest, &[vendor])
            .expect("vendor observation should validate");

        env.exec("Fixture = nil")
            .expect("absent fixture should clear");
        let absent_row = resolved_row(
            "best-effort",
            "cross-flavor",
            &assertion(flavor, "post-load", "absent"),
        );
        let mut absent_manifest = fixture_manifest(&absent_row);
        absent_manifest.target.flavor = AuditFlavor::active();
        let absent = observe_assertion(
            &env,
            &absent_manifest.rows[0],
            &absent_manifest.rows[0].assertions[0],
        )
        .expect("absent observation should succeed");
        validate_observations(&absent_manifest, &[absent])
            .expect("absent observation should validate");

        let directory = tempfile::tempdir().expect("temporary addon directory should create");
        let addon_directory = directory.path().join("ObservationFixture");
        std::fs::create_dir(&addon_directory).expect("fixture addon directory should create");
        let toc_path = addon_directory.join("ObservationFixture.toc");
        let mut toc = std::fs::File::create(&toc_path).expect("fixture TOC should create");
        writeln!(toc, "## Title: ObservationFixture").unwrap();
        writeln!(toc, "## LoadOnDemand: 1").unwrap();
        writeln!(toc, "ObservationFixture.lua").unwrap();
        std::fs::write(
            addon_directory.join("ObservationFixture.lua"),
            "Fixture = function() end\n",
        )
        .expect("fixture Lua should write");
        let lod_assertions = format!(
            r#"{{"flavor":"{flavor}","phase":"before-addon","expected":"absent","addon":"ObservationFixture"}},{{"flavor":"{flavor}","phase":"after-addon","expected":"present","expected_type":"function","addon":"ObservationFixture"}}"#
        );
        let lod_row = load_on_demand_row(&lod_assertions, "ObservationFixture");
        let mut lod_manifest = fixture_manifest(&lod_row);
        lod_manifest.target.flavor = AuditFlavor::active();
        let before = observe_assertion(
            &env,
            &lod_manifest.rows[0],
            &lod_manifest.rows[0].assertions[0],
        )
        .expect("pre-load observation should succeed");
        load_addon(&env.loader_env(), &toc_path).expect("fixture addon should load");
        let after = observe_assertion(
            &env,
            &lod_manifest.rows[0],
            &lod_manifest.rows[0].assertions[1],
        )
        .expect("post-load observation should succeed");
        validate_observations(&lod_manifest, &[before, after])
            .expect("LoD observations should validate");

        let observation_set = build_observation_set("exact manifest bytes", Vec::new());
        assert_eq!(
            observation_set.manifest_hash,
            format!("{:x}", Sha256::digest(b"exact manifest bytes"))
        );
    }

    fn checked_in_patch_manifest_paths(root: &Path) -> Vec<PathBuf> {
        let directory = root.join("data/patch-api");
        let mut manifests = Vec::new();
        for entry in std::fs::read_dir(&directory).expect("patch manifest directory should read") {
            let path = entry.expect("directory entry should read").path();
            if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
                manifests.push(path);
            }
        }
        manifests.sort();
        assert!(
            !manifests.is_empty(),
            "at least one patch manifest is required"
        );
        manifests
    }

    #[test]
    fn checked_in_evidence_references_are_repository_relative_and_stable() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for path in checked_in_patch_manifest_paths(root) {
            let manifest_path = path
                .strip_prefix(root)
                .expect("checked-in manifest should be inside repository");
            let json = std::fs::read_to_string(&path).expect("manifest should read");
            let manifest = parse_manifest(&json).expect("manifest should parse");
            for row in manifest.rows {
                for evidence in row.evidence {
                    let reference = reference_path(&evidence.reference);
                    let reference_path = Path::new(reference);
                    assert_ne!(
                        reference_path,
                        manifest_path,
                        "{} row {} references its own manifest",
                        path.display(),
                        row.id
                    );
                    assert!(
                        !reference_path.is_absolute(),
                        "{} row {} has absolute evidence reference {}",
                        path.display(),
                        row.id,
                        evidence.reference
                    );
                    assert!(
                        !reference.starts_with("~/"),
                        "{} row {} has home-relative evidence reference {}",
                        path.display(),
                        row.id,
                        evidence.reference
                    );
                }
            }
        }
    }

    #[test]
    fn checked_in_evidence_references_are_git_tracked() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for path in checked_in_patch_manifest_paths(root) {
            let json = std::fs::read_to_string(&path).expect("manifest should read");
            let manifest = parse_manifest(&json).expect("manifest should parse");
            for row in manifest.rows {
                for evidence in row.evidence {
                    let reference = reference_path(&evidence.reference);
                    let output = std::process::Command::new("git")
                        .args([
                            "-C",
                            root.to_str().expect("repository path should be UTF-8"),
                        ])
                        .args(["ls-files", "--error-unmatch", "--", reference])
                        .output()
                        .expect("git ls-files should run");
                    assert!(
                        output.status.success(),
                        "{} row {} evidence reference {} is not Git-tracked: {}",
                        path.display(),
                        row.id,
                        evidence.reference,
                        String::from_utf8_lossy(&output.stderr).trim()
                    );
                }
            }
        }
    }

    #[test]
    fn checked_in_patch_manifests_parse() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for path in checked_in_patch_manifest_paths(root) {
            let json = std::fs::read_to_string(&path).expect("manifest should read");
            parse_manifest(&json).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        }
    }

    #[test]
    fn checked_in_patch_manifest_checklists_match_rendered_output() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for path in checked_in_patch_manifest_paths(root) {
            let json = std::fs::read_to_string(&path).expect("manifest should read");
            let manifest = parse_manifest(&json).expect("manifest should parse");
            validate_checklist(&manifest, root)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        }
    }

    #[test]
    fn checked_in_provenance_only_flags_match_resolutions() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for path in checked_in_patch_manifest_paths(root) {
            let json = std::fs::read_to_string(&path).expect("manifest should read");
            let manifest = parse_manifest(&json).expect("manifest should parse");
            for row in manifest.rows {
                let uses_provenance_resolution = row.resolution == ResolutionKind::ProvenanceOnly;
                assert_eq!(
                    row.provenance_only,
                    uses_provenance_resolution,
                    "{} row {} provenance_only flag and provenance-only resolution must match",
                    path.display(),
                    row.id
                );
            }
        }
    }

    #[test]
    fn checked_in_provenance_only_notes_are_canonical() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for path in checked_in_patch_manifest_paths(root) {
            let json = std::fs::read_to_string(&path).expect("manifest should read");
            let manifest = parse_manifest(&json).expect("manifest should parse");
            for row in manifest.rows {
                if row.provenance_only {
                    assert_eq!(
                        row.notes.as_deref(),
                        Some(PROVENANCE_ONLY_NOTE),
                        "{} row {} provenance-only note must be canonical",
                        path.display(),
                        row.id
                    );
                }
            }
        }
    }

    #[test]
    fn every_checked_in_patch_manifest_matches_repository_artifacts() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for path in checked_in_patch_manifest_paths(root) {
            let json = std::fs::read_to_string(&path).expect("manifest should read");
            let manifest = parse_manifest(&json).expect("manifest should parse");
            validate_repository(&manifest, root)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        }
    }

    #[test]
    fn malformed_symbol_paths_are_rejected() {
        let row = r#"{"id":"added:A..B","symbol":"A..B","change":"added","status":null,"resolution":"untriaged","owner":"unknown","load_addon":null,"evidence":[],"tests":[],"assertions":[],"commit":null,"approval_id":null,"notes":"pending"}"#;
        let manifest = fixture_manifest(row);

        let error = validate_manifest(&manifest).expect_err("malformed path must fail");

        assert!(error.contains("symbol path"));
    }

    fn changed_manifest(changed_count: usize, row_id: &str) -> PatchAuditManifest {
        parse_manifest(&format!(
            r#"{{
              "schema":"framexml-patch-audit/v2",
              "patch":"12.0.7",
              "target":{{"flavor":"retail","build":"12.0.7","cache_manifest":"cache","cache_hash":"{}"}},
              "source":{{"path":"source","hash":"{}","added_count":0,"removed_count":0,"changed_count":{changed_count}}},
              "output":{{"checklist":"checklist","inventory":"inventory"}},
              "rows":[{{
                "id":"{row_id}","symbol":"Fixture","change":"changed",
                "status":null,"resolution":"untriaged","owner":"unknown",
                "evidence":[],"tests":[],"assertions":[],"commit":null,"approval_id":null,"notes":"pending"
              }}]
            }}"#,
            "a".repeat(64),
            "b".repeat(64)
        ))
        .expect("changed manifest should parse")
    }

    #[test]
    fn changed_direction_accepts_changed_row_id_and_count() {
        let manifest = changed_manifest(1, "changed:Fixture");

        validate_manifest(&manifest).expect("changed row should validate with matching count");
        assert_eq!(manifest.rows[0].id, "changed:Fixture");
    }

    #[test]
    fn changed_direction_count_mismatch_is_rejected() {
        let manifest = changed_manifest(0, "changed:Fixture");

        let error = validate_manifest(&manifest).expect_err("changed count mismatch must fail");

        assert!(error.contains("changed count mismatch: source=0 rows=1"));
    }

    #[test]
    fn changed_direction_requires_changed_row_id_prefix() {
        let manifest = changed_manifest(1, "added:Fixture");

        let error = validate_manifest(&manifest).expect_err("changed rows need changed IDs");

        assert!(error.contains("row added:Fixture must use id changed:Fixture"));
    }

    #[test]
    fn missing_changed_count_defaults_to_zero() {
        let manifest = fixture_manifest(
            r#"{"id":"added:Fixture","symbol":"Fixture","change":"added","status":null,"resolution":"untriaged","owner":"unknown","evidence":[],"tests":[],"assertions":[],"commit":null,"approval_id":null,"notes":"pending"}"#,
        );

        assert_eq!(manifest.source.changed_count, 0);
    }

    #[test]
    fn untriaged_rows_have_no_final_status() {
        let manifest = fixture_manifest(
            r#"{"id":"added:Fixture","symbol":"Fixture","change":"added","status":null,"resolution":"untriaged","owner":"unknown","evidence":[],"tests":[],"assertions":[],"commit":null,"approval_id":null,"notes":"pending"}"#,
        );
        validate_manifest(&manifest).expect("neutral draft row should validate");
        assert!(render_checklist(&manifest).contains("[untriaged]"));
    }

    #[test]
    fn untriaged_rows_reject_blank_notes() {
        let manifest = fixture_manifest(
            r#"{"id":"added:Fixture","symbol":"Fixture","change":"added","status":null,"resolution":"untriaged","owner":"unknown","evidence":[],"tests":[],"assertions":[],"commit":null,"approval_id":null,"notes":""}"#,
        );
        assert!(
            validate_manifest(&manifest)
                .unwrap_err()
                .contains("notes must not be empty")
        );
    }

    #[test]
    fn untriaged_rows_reject_exception_status() {
        let manifest = fixture_manifest(
            r#"{"id":"added:Fixture","symbol":"Fixture","change":"added","status":"exception-requested","resolution":"untriaged","owner":"unknown","evidence":[],"tests":[],"assertions":[],"commit":null,"approval_id":null,"notes":null}"#,
        );
        assert!(
            validate_manifest(&manifest)
                .unwrap_err()
                .contains("status must be null")
        );
    }

    #[test]
    fn exception_status_requires_unsafe_or_impossible_resolution() {
        let row = resolved_row(
            "exception-requested",
            "compat",
            &assertion("ptr", "post-load", "present"),
        );
        let manifest = fixture_manifest(&row);
        assert!(
            validate_manifest(&manifest)
                .unwrap_err()
                .contains("incompatible")
        );
    }

    #[test]
    fn exception_rows_without_lua_surface_accept_no_assertions() {
        for resolution in ["unsafe", "impossible"] {
            let row = resolved_row("exception-requested", resolution, "");
            let manifest = fixture_manifest(&row);

            validate_manifest(&manifest)
                .expect("non-Lua exception evidence should not require a presence assertion");
        }
    }

    #[test]
    fn impossible_scope_exception_with_repository_rule_provenance_is_valid() {
        let row = scope_exception_row(
            "impossible",
            None,
            "permanent-project-scope",
            "AGENTS.md#intentional-gaps",
            "The required subsystem is permanently outside project scope.",
        );
        let manifest = parse_fixture_manifest(&row).expect("scope exception should parse");

        validate_manifest(&manifest).expect("valid scope exception should validate");
    }

    #[test]
    fn scope_exception_and_user_chat_approval_are_mutually_exclusive() {
        let row = scope_exception_row(
            "impossible",
            Some("user-chat:added:Fixture:approval-1"),
            "permanent-project-scope",
            "AGENTS.md#intentional-gaps",
            "The required subsystem is permanently outside project scope.",
        );
        let manifest = parse_fixture_manifest(&row).expect("scope exception should parse");

        let error = validate_manifest(&manifest)
            .expect_err("scope exception and approval must be rejected together");
        assert!(error.contains("approval_id"), "unexpected error: {error}");
        assert!(
            error.contains("scope_exception"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn scope_exception_is_rejected_for_unsafe_resolution() {
        let row = scope_exception_row(
            "unsafe",
            None,
            "permanent-project-scope",
            "AGENTS.md#intentional-gaps",
            "The required subsystem is permanently outside project scope.",
        );
        let manifest = parse_fixture_manifest(&row).expect("scope exception should parse");

        let error = validate_manifest(&manifest)
            .expect_err("scope exception should require impossible resolution");
        assert!(error.contains("impossible"), "unexpected error: {error}");
    }

    #[test]
    fn scope_exception_rejects_wrong_repository_rule() {
        let row = scope_exception_row(
            "impossible",
            None,
            "temporary-gap",
            "AGENTS.md#intentional-gaps",
            "The required subsystem is permanently outside project scope.",
        );
        let manifest = parse_fixture_manifest(&row).expect("scope exception should parse");

        let error = validate_manifest(&manifest)
            .expect_err("scope exception should require the repository rule");
        assert!(
            error.contains("permanent-project-scope"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn scope_exception_rejects_wrong_repository_rule_reference() {
        let row = scope_exception_row(
            "impossible",
            None,
            "permanent-project-scope",
            "docs/wiki/unsupported.md",
            "The required subsystem is permanently outside project scope.",
        );
        let manifest = parse_fixture_manifest(&row).expect("scope exception should parse");

        let error = validate_manifest(&manifest)
            .expect_err("scope exception should require the AGENTS rule reference");
        assert!(
            error.contains("AGENTS.md#intentional-gaps"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn scope_exception_reference_requires_no_3d_rule_text() {
        let directory = tempfile::tempdir().expect("temporary repository should create");
        std::fs::write(
            directory.path().join("AGENTS.md"),
            "# Project rules\n\n## Intentional Gaps\n",
        )
        .expect("AGENTS fixture should write");
        let scope_exception = ScopeException {
            rule: PERMANENT_PROJECT_SCOPE_RULE.to_string(),
            reference: INTENTIONAL_GAPS_REFERENCE.to_string(),
            summary: "The required subsystem is permanently outside project scope.".to_string(),
        };

        let error = validate_scope_exception_reference(directory.path(), &scope_exception)
            .expect_err("scope reference without the no-3D rule must fail");
        assert!(
            error.contains("No 3D rendering"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn completion_accepts_repository_scope_exception_without_user_chat_approval() {
        let row = scope_exception_row(
            "impossible",
            None,
            PERMANENT_PROJECT_SCOPE_RULE,
            INTENTIONAL_GAPS_REFERENCE,
            "The required subsystem is permanently outside project scope.",
        );
        let manifest = fixture_manifest(&row);
        let mut approvals = HashSet::new();

        validate_completion_row(&manifest.rows[0], &mut approvals)
            .expect("repository scope exception should complete without user approval");
        assert!(approvals.is_empty());
    }

    fn assert_observation_mismatch(assertion_json: String, observation_json: &str) {
        let row = resolved_row("best-effort", "vendor-present", &assertion_json);
        let manifest = fixture_manifest(&row);
        let observations = parse_observations(&format!(
            r#"{{"schema":"framexml-patch-observations/v1","manifest_hash":"{}","observations":[{observation_json}]}}"#,
            "d".repeat(64)
        ))
        .expect("observations should parse");
        assert!(validate_observations(&manifest, &observations.observations).is_err());
    }

    #[test]
    fn vendor_present_falsifier_rejects_wrong_flavor() {
        assert_observation_mismatch(
            assertion("ptr", "post-core", "present"),
            r#"{"row_id":"added:Fixture","flavor":"retail","phase":"post-core","present":true,"observed_type":"function","addon":null}"#,
        );
    }

    #[test]
    fn load_on_demand_falsifier_rejects_wrong_phase() {
        assert_observation_mismatch(
            assertion("ptr", "after-addon", "present"),
            r#"{"row_id":"added:Fixture","flavor":"ptr","phase":"before-addon","present":true,"observed_type":"function","addon":null}"#,
        );
    }

    #[test]
    fn cross_flavor_falsifier_rejects_target_leak() {
        assert_observation_mismatch(
            assertion("ptr", "post-load", "absent"),
            r#"{"row_id":"added:Fixture","flavor":"ptr","phase":"post-load","present":true,"observed_type":"function","addon":null}"#,
        );
    }

    #[test]
    fn absent_observation_rejects_non_null_type() {
        assert_observation_mismatch(
            assertion("ptr", "post-load", "absent"),
            r#"{"row_id":"added:Fixture","flavor":"ptr","phase":"post-load","present":false,"observed_type":"function","addon":null}"#,
        );
    }

    #[test]
    fn removed_after_reset_falsifier_rejects_resurrection() {
        assert_observation_mismatch(
            assertion("ptr", "post-reset", "absent"),
            r#"{"row_id":"added:Fixture","flavor":"ptr","phase":"post-reset","present":true,"observed_type":"function","addon":null}"#,
        );
    }

    #[test]
    fn source_rows_include_changed_occurrences_between_added_and_removed() {
        let source = serde_json::json!({
            "added": ["AddedFixture"],
            "changed": ["ChangedFixture"],
            "removed": ["RemovedFixture"]
        });

        let row_ids = source_row_ids(&source).expect("source rows should parse");

        assert_eq!(
            row_ids,
            vec![
                "added:AddedFixture",
                "changed:ChangedFixture",
                "removed:RemovedFixture"
            ]
        );
    }

    #[test]
    fn generic_occurrence_source_rows_are_grouped_by_direction() {
        let source = serde_json::json!({
            "occurrences": [
                {"direction":"removed","category":"widget","symbol":"RemovedFixture"},
                {"direction":"added","category":"global","symbol":"AddedFixture"},
                {"direction":"changed","category":"event","symbol":"ChangedFixture"}
            ]
        });

        let row_ids = source_row_ids(&source).expect("occurrence source rows should parse");

        assert_eq!(
            row_ids,
            vec![
                "added:AddedFixture",
                "changed:ChangedFixture",
                "removed:RemovedFixture"
            ]
        );
    }

    #[test]
    fn categorized_occurrence_payloads_preserve_row_ids_and_reject_unknown_fields() {
        let source = serde_json::json!({
            "occurrences": [
                {
                    "direction":"removed",
                    "category":"widget",
                    "symbol":"RemovedFixture",
                    "before":{"kind":"Frame","methods":["Hide"]}
                },
                {
                    "direction":"added",
                    "category":"global",
                    "symbol":"AddedFixture",
                    "after":{"kind":"function","signature":["string"]}
                },
                {
                    "direction":"changed",
                    "category":"event",
                    "symbol":"ChangedFixture",
                    "before":{"payload":{"old":true}},
                    "after":["new",{"version":2}]
                }
            ]
        });

        let row_ids = source_row_ids(&source)
            .expect("optional occurrence payloads should not affect row identity");

        assert_eq!(
            row_ids,
            vec![
                "added:AddedFixture",
                "changed:ChangedFixture",
                "removed:RemovedFixture"
            ]
        );

        let invalid_source = serde_json::json!({
            "occurrences": [{
                "direction":"added",
                "category":"global",
                "symbol":"UnknownFieldFixture",
                "unknown":true
            }]
        });
        let error = source_row_ids(&invalid_source)
            .expect_err("unknown occurrence fields must remain rejected");
        assert!(
            error.contains("unknown field") && error.contains("unknown"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn categorized_occurrence_detail_preserves_direction_symbol_row_id() {
        let source = serde_json::json!({
            "occurrences": [
                {
                    "direction":"added",
                    "category":"global",
                    "symbol":"Fixture",
                    "detail":"new global API"
                }
            ]
        });

        let row_ids = source_row_ids(&source)
            .expect("nonblank occurrence detail should not affect row identity");

        assert_eq!(row_ids, vec!["added:Fixture"]);
    }

    #[test]
    fn categorized_occurrence_detail_rejects_blank_detail() {
        let source = serde_json::json!({
            "occurrences": [
                {
                    "direction":"added",
                    "category":"global",
                    "symbol":"Fixture",
                    "detail":""
                }
            ]
        });

        let error =
            source_row_ids(&source).expect_err("blank occurrence detail should be rejected");
        assert!(
            error.contains("detail must not be blank"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn generic_occurrence_source_rejects_invalid_direction() {
        let source = serde_json::json!({
            "occurrences": [
                {"direction":"renamed","category":"global","symbol":"Fixture"}
            ]
        });

        let error = source_row_ids(&source).expect_err("invalid direction should be rejected");
        assert!(error.contains("direction"), "unexpected error: {error}");
    }

    #[test]
    fn generic_occurrence_source_rejects_blank_category() {
        let source = serde_json::json!({
            "occurrences": [
                {"direction":"added","category":"","symbol":"Fixture"}
            ]
        });

        let error = source_row_ids(&source).expect_err("blank category should be rejected");
        assert!(error.contains("category"), "unexpected error: {error}");
    }

    #[test]
    fn source_drift_is_rejected() {
        let root = std::env::temp_dir().join(format!(
            "wow-ui-sim-patch-source-drift-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("temporary directory should create");
        std::fs::write(root.join("source"), r#"{"added":["Wrong"],"removed":[]}"#)
            .expect("temporary source should write");
        let manifest = fixture_manifest(
            r#"{"id":"added:Fixture","symbol":"Fixture","change":"added","status":null,"resolution":"untriaged","owner":"unknown","evidence":[],"tests":[],"assertions":[],"commit":null,"approval_id":null,"notes":null}"#,
        );
        assert!(
            validate_source_rows(&manifest, &root)
                .unwrap_err()
                .contains("differs")
        );
        std::fs::remove_dir_all(root).expect("temporary directory should remove");
    }

    #[test]
    fn mismatched_evidence_hash_is_rejected() {
        let root =
            std::env::temp_dir().join(format!("wow-ui-sim-patch-hash-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("temporary directory should create");
        std::fs::write(root.join("evidence"), "real contents")
            .expect("temporary evidence should write");
        assert!(
            validate_file_hash(&root, "evidence", &"0".repeat(64))
                .unwrap_err()
                .contains("hash mismatch")
        );
        std::fs::remove_dir_all(root).expect("temporary directory should remove");
    }

    #[test]
    fn fake_test_and_commit_references_are_rejected() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(validate_test_reference(root, "tests/not-real.rs::not_real").is_err());
        assert!(validate_commit(root, "0000000000000000000000000000000000000000").is_err());
    }

    #[test]
    fn defines_rust_test_accepts_prefork_marker_and_rejects_unmarked_function() {
        let source = r#"
#[cfg(feature = "client-retail")]

prefork_full_ui_case! {
fn migrated(env: &WowLuaEnv) {
    let _env = env;
}
}

fn unmarked(env: &WowLuaEnv) {}
"#;

        assert!(defines_rust_test(source, "migrated").expect("pattern should compile"));
        assert!(!defines_rust_test(source, "unmarked").expect("pattern should compile"));
    }

    #[test]
    fn comments_and_prefixes_do_not_satisfy_named_test_references() {
        let root = std::env::temp_dir().join(format!(
            "wow-ui-sim-patch-test-reference-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("temporary directory should create");
        std::fs::write(
            root.join("fixture.rs"),
            "// fn exact_test() {}\nfn exact_test_suffix() {}\n/* fn exact_test() {} */\n",
        )
        .expect("temporary test should write");
        assert!(validate_test_reference(&root, "fixture.rs::exact_test").is_err());
        std::fs::remove_dir_all(root).expect("temporary directory should remove");
    }

    #[test]
    fn load_on_demand_requires_declared_addon_and_target_flavor() {
        let assertions = r#"{"flavor":"ptr","phase":"before-addon","expected":"absent","addon":"Addon_A"},{"flavor":"retail","phase":"after-addon","expected":"present","expected_type":"function","addon":"Addon_A"}"#;
        let manifest = fixture_manifest(&load_on_demand_row(assertions, "Addon_A"));
        assert!(validate_manifest(&manifest).is_err());

        let assertions = r#"{"flavor":"ptr","phase":"before-addon","expected":"absent","addon":"Addon_A"},{"flavor":"ptr","phase":"after-addon","expected":"present","expected_type":"function","addon":"Addon_B"}"#;
        let manifest = fixture_manifest(&load_on_demand_row(assertions, "Addon_A"));
        assert!(validate_manifest(&manifest).is_err());

        let assertions = r#"{"flavor":"ptr","phase":"before-addon","expected":"absent","addon":"Addon_A"},{"flavor":"ptr","phase":"after-addon","expected":"present","expected_type":"function","addon":"Addon_A"}"#;
        let manifest = fixture_manifest(&load_on_demand_row(assertions, "Addon_B"));
        assert!(validate_manifest(&manifest).is_err());

        let manifest = fixture_manifest(&load_on_demand_row(assertions, "Addon_A"));
        validate_manifest(&manifest).expect("declared addon lifecycle should validate");
    }

    #[test]
    fn cross_flavor_contract_uses_manifest_target() {
        let row = resolved_row(
            "best-effort",
            "cross-flavor",
            &assertion("retail", "post-load", "absent"),
        );
        let mut manifest = fixture_manifest(&row);
        manifest.target.flavor = AuditFlavor::Retail;
        validate_manifest(&manifest).expect("target-flavor absence should validate");
    }

    #[test]
    fn completion_requires_commit_for_non_exception_rows() {
        let json = resolved_row(
            "best-effort",
            "vendor-present",
            &assertion("ptr", "post-load", "present"),
        )
        .replace(r#""commit":"1234567890""#, r#""commit":null"#);
        let manifest = fixture_manifest(&json);
        let mut approvals = HashSet::new();
        assert!(
            validate_completion_row(&manifest.rows[0], &mut approvals)
                .unwrap_err()
                .contains("requires a commit")
        );
    }

    #[test]
    fn completion_requires_item_bound_exception_approval() {
        let json = resolved_row(
            "exception-requested",
            "unsafe",
            &assertion("ptr", "post-load", "absent"),
        )
        .replace(
            r#""approval_id":null"#,
            r#""approval_id":"user-chat:added:Other:approval-1""#,
        );
        let manifest = fixture_manifest(&json);
        let mut approvals = HashSet::new();
        assert!(
            validate_completion_row(&manifest.rows[0], &mut approvals)
                .unwrap_err()
                .contains("must start")
        );
    }

    #[test]
    fn observations_must_bind_to_exact_manifest_bytes() {
        let observations = parse_observations(&format!(
            r#"{{"schema":"framexml-patch-observations/v1","manifest_hash":"{}","observations":[]}}"#,
            "0".repeat(64)
        ))
        .expect("observations should parse");
        assert!(
            validate_observation_binding("different manifest", &observations)
                .unwrap_err()
                .contains("does not match")
        );
    }

    #[test]
    fn unknown_schema_fields_are_rejected() {
        let json = format!(
            r#"{{"schema":"framexml-patch-audit/v2","patch":"x","target":{{"flavor":"ptr","build":"x","cache_manifest":"x","cache_hash":"{}","typo":true}},"source":{{"path":"x","hash":"{}","added_count":0,"removed_count":0}},"output":{{"checklist":"x","inventory":"x"}},"rows":[]}}"#,
            "a".repeat(64),
            "b".repeat(64)
        );
        assert!(parse_manifest(&json).is_err());
    }
}
