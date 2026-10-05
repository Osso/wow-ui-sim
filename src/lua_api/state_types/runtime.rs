//! Runtime / profiler types: cursor state, per-addon metrics, error records.

use std::collections::{HashMap, HashSet, VecDeque};

/// What is currently held on the cursor (drag-and-drop state).
#[derive(Debug, Clone)]
pub enum CursorInfo {
    /// An action bar spell: PickupAction(slot) removes it from the bar.
    Action { slot: u32, spell_id: u32 },
    /// A spell from the spellbook (doesn't remove from spellbook).
    Spell { spell_id: u32 },
    /// A talent picked from the talent frame. `pvp=true` when sourced
    /// from the PvP talent pane.
    Talent { talent_id: u32, pvp: bool },
    /// A pet-action spell picked from the pet action bar.
    PetAction { slot: u32, spell_id: u32 },
    /// A macro picked up by slot index.
    Macro { macro_index: u32 },
    /// An item picked up from a bag slot, equipment slot, or merchant.
    Item {
        item_id: u32,
        stack_count: i32,
        origin: CursorItemOrigin,
    },
    /// Outfit dragged from the transmog catalog; zero means equipped gear.
    TransmogOutfit { outfit_id: i64 },
    /// Money in copper held on the cursor (PickupPlayerMoney → DropCursorMoney).
    Money { copper: u64 },
}

/// Where a cursor-carried item came from — used to route drops back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorItemOrigin {
    Bag { bag: i32, slot: i32 },
    Equipped { slot: i32 },
    Merchant { index: u32 },
    Unknown,
}

/// Per-addon runtime profiler metrics, updated each frame.
#[derive(Debug, Clone)]
pub struct AddonRuntimeMetrics {
    /// Time spent in this addon's handlers during the current frame (accumulator).
    pub current_frame_ms: f64,
    /// Rolling window of per-frame times (last 60 frames) for RecentAverageTime.
    pub recent_frames: VecDeque<f64>,
    /// Peak time ever recorded in a single frame.
    pub peak_ms: f64,
    /// Session total time (ms) across all frames.
    pub session_total_ms: f64,
    /// Number of frames where this addon had handlers fire.
    pub session_frame_count: u64,
    /// Threshold counters: frames where addon time exceeded N ms.
    pub count_over_1ms: u32,
    pub count_over_5ms: u32,
    pub count_over_10ms: u32,
    pub count_over_50ms: u32,
    pub count_over_100ms: u32,
    pub count_over_500ms: u32,
    pub count_over_1000ms: u32,
}

impl Default for AddonRuntimeMetrics {
    fn default() -> Self {
        Self {
            current_frame_ms: 0.0,
            recent_frames: VecDeque::with_capacity(60),
            peak_ms: 0.0,
            session_total_ms: 0.0,
            session_frame_count: 0,
            count_over_1ms: 0,
            count_over_5ms: 0,
            count_over_10ms: 0,
            count_over_50ms: 0,
            count_over_100ms: 0,
            count_over_500ms: 0,
            count_over_1000ms: 0,
        }
    }
}

/// Application-level frame timing for profiler (total frame time, not just addon time).
#[derive(Debug, Clone, Default)]
pub struct AppFrameMetrics {
    /// Rolling window of total frame times in ms (last 60 frames).
    pub recent_frame_ms: VecDeque<f64>,
    /// Peak frame time ever recorded.
    pub peak_ms: f64,
    /// Session total frame time in ms.
    pub session_total_ms: f64,
    /// Number of frames recorded.
    pub session_frame_count: u64,
}

/// Message identity recorded by `C_AddOnProfiler.AddPerformanceMessageShown`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddonPerformanceMessageKey {
    pub message_type: i32,
    pub metric: i32,
    pub add_on_name: Option<String>,
}

/// Information about a loaded addon.
#[derive(Debug, Clone)]
pub struct AddonInfo {
    /// Folder name (used as addon identifier).
    pub folder_name: String,
    /// Directory selected by the loader for this addon's TOC and loose assets.
    pub addon_dir: Option<std::path::PathBuf>,
    /// Display title from TOC metadata.
    pub title: String,
    /// Notes/description from TOC metadata.
    pub notes: String,
    /// Whether the addon is currently enabled.
    pub enabled: bool,
    /// Whether the addon was successfully loaded.
    pub loaded: bool,
    /// Load on demand flag.
    pub load_on_demand: bool,
    /// Whether `[Bootstrap]` files have already executed for this addon.
    pub bootstrap_loaded: bool,
    /// Whether the addon loads Lua/XML chunks in the secure environment.
    pub use_secure_env: bool,
    /// Optional security status reported by `C_AddOns.GetAddOnSecurity`.
    pub security: Option<String>,
    /// Total load time in seconds (for profiler metrics).
    pub load_time_secs: f64,
    /// Runtime profiler metrics (updated per frame).
    pub runtime: AddonRuntimeMetrics,
    /// Required dependencies declared in TOC (`Dependencies` / `RequiredDep` / `RequiredDeps`).
    /// Surfaced to Lua via `C_AddOns.GetAddOnDependencies` as a variadic of strings.
    pub dependencies: Vec<String>,
    /// Raw TOC metadata exposed through `C_AddOns.GetAddOnMetadata`.
    pub metadata: HashMap<String, String>,
    /// Factory default enabled state, derived from `## DefaultState: disabled`
    /// (`false` only when the TOC opts out; otherwise `true`). Surfaced via
    /// `C_AddOns.IsAddOnDefaultEnabled` and used by the addon list's
    /// reset-to-default action.
    pub default_enabled: bool,
}

impl Default for AddonInfo {
    fn default() -> Self {
        Self {
            folder_name: String::new(),
            addon_dir: None,
            title: String::new(),
            notes: String::new(),
            enabled: false,
            loaded: false,
            load_on_demand: false,
            bootstrap_loaded: false,
            use_secure_env: false,
            security: None,
            load_time_secs: 0.0,
            runtime: AddonRuntimeMetrics::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
            default_enabled: true,
        }
    }
}

/// Saved addon enable state, keyed by addon folder name rather than list index.
///
/// WoW's persisted addon state is a name-based list, so reset behavior must not
/// depend on whatever order addons happen to be registered in this session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AddonEnableSnapshot {
    pub known_addons: HashSet<String>,
    pub disabled_addons: HashSet<String>,
}

impl AddonEnableSnapshot {
    pub fn from_addons(addons: &[AddonInfo]) -> Self {
        let known_addons = addons
            .iter()
            .map(|addon| addon.folder_name.clone())
            .collect();
        let disabled_addons = addons
            .iter()
            .filter(|addon| !addon.enabled)
            .map(|addon| addon.folder_name.clone())
            .collect();
        Self {
            known_addons,
            disabled_addons,
        }
    }

    pub fn saved_enabled(&self, addon_name: &str) -> Option<bool> {
        self.known_addons
            .contains(addon_name)
            .then(|| !self.disabled_addons.contains(addon_name))
    }
}

/// A collected Lua error with optional addon attribution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LuaErrorRecord {
    /// Raw collected error message.
    pub message: String,
    /// Addon name inferred from the loading/executing context or Lua stack.
    pub addon_name: Option<String>,
}

/// Global environment where a missing symbol access originated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NilSymbolEnvironment {
    #[default]
    Public,
    Secure,
}

/// A missing symbol access captured through `_G` or `C_*` namespace `__index` hooks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NilSymbolAccess {
    /// Stable addon index inferred from the loading/executing context.
    pub addon_index: Option<u16>,
    /// Addon name inferred from the loading/executing context.
    pub addon_name: Option<String>,
    /// Logical global environment selected by the loading file.
    pub environment: NilSymbolEnvironment,
    /// Container table where the miss happened (`_G`, `__secureenv`, or a `C_*` namespace).
    pub container: String,
    /// Missing key that resolved to nil.
    pub key: String,
    /// Raw Lua chunk source reported by `debug.getinfo`, if available.
    pub source: Option<String>,
    /// 1-based source line where the nil access happened, if available.
    pub line: Option<i32>,
}

/// Source and owner of one finalized addon-load diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LoadDiagnosticAttribution {
    pub addon_name: String,
    pub environment: NilSymbolEnvironment,
    pub source: Option<String>,
    pub line: Option<i32>,
}

/// Non-fatal regular-global nil access retained for inspection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NilSymbolObservation {
    pub kind: NilSymbolObservationKind,
    pub attribution: LoadDiagnosticAttribution,
}

/// Regular-global symbol shape observed as nil.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NilSymbolObservationKind {
    Global { name: String },
}

/// Missing `C_*` API requirement retained independently from startup health.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MissingRequirement {
    pub kind: MissingRequirementKind,
    pub attribution: LoadDiagnosticAttribution,
}

/// Missing `C_*` namespace or namespace member.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MissingRequirementKind {
    CNamespace { namespace: String },
    CMethod { namespace: String, method: String },
}

/// Diagnostics finalized by one or more addon loads.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoadDiagnostics {
    pub warnings: Vec<String>,
    pub nil_symbol_observations: Vec<NilSymbolObservation>,
    pub missing_requirements: Vec<MissingRequirement>,
}

impl LoadDiagnostics {
    pub fn is_empty(&self) -> bool {
        self.warnings.is_empty()
            && self.nil_symbol_observations.is_empty()
            && self.missing_requirements.is_empty()
    }

    pub fn extend(&mut self, other: Self) {
        self.warnings.extend(other.warnings);
        self.nil_symbol_observations
            .extend(other.nil_symbol_observations);
        self.missing_requirements.extend(other.missing_requirements);
    }
}

impl std::fmt::Display for NilSymbolObservation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let NilSymbolObservationKind::Global { name } = &self.kind;
        write!(
            formatter,
            "{} observed nil global {} in {} environment{}",
            self.attribution.addon_name,
            name,
            environment_name(self.attribution.environment),
            format_diagnostic_location(&self.attribution)
        )
    }
}

impl std::fmt::Display for MissingRequirement {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let required = match &self.kind {
            MissingRequirementKind::CNamespace { namespace } => namespace.clone(),
            MissingRequirementKind::CMethod { namespace, method } => {
                format!("{namespace}.{method}")
            }
        };
        write!(
            formatter,
            "{} needs {} in {} environment{}",
            self.attribution.addon_name,
            required,
            environment_name(self.attribution.environment),
            format_diagnostic_location(&self.attribution)
        )
    }
}

fn environment_name(environment: NilSymbolEnvironment) -> &'static str {
    match environment {
        NilSymbolEnvironment::Public => "public",
        NilSymbolEnvironment::Secure => "secure",
    }
}

fn format_diagnostic_location(attribution: &LoadDiagnosticAttribution) -> String {
    match (&attribution.source, attribution.line) {
        (Some(source), Some(line)) => format!(" (accessed at {source}:{line})"),
        _ => String::new(),
    }
}
