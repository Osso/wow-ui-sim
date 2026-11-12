//! Console Variable (CVar) storage.
//!
//! CVars are configuration values that addons can read/write.
//! Defaults come from WoW's built-in cvars, overrides are persisted to disk.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Default path for persisted CVar overrides.
fn default_storage_path() -> PathBuf {
    crate::paths::default_constructor_session_wtf_path().join("cvars.json")
}

/// CVar storage with defaults and overrides.
pub struct CVarStorage {
    /// Default values (lowercase key -> value)
    defaults: HashMap<String, String>,
    /// Original-case names (lowercase key -> original name)
    original_names: HashMap<String, String>,
    /// Runtime-registered defaults (lowercase key -> value).
    registered_defaults: RwLock<HashMap<String, String>>,
    /// Original-case names for runtime-registered CVars.
    registered_original_names: RwLock<HashMap<String, String>>,
    /// Runtime overrides (lowercase key -> value), persisted to disk.
    overrides: RwLock<HashMap<String, String>>,
    /// Effective session-only values, never included in persisted overrides.
    session_overrides: RwLock<HashMap<String, SessionCVar>>,
    /// Hidden host/server mirrors, separate from Lua-visible CVar reads.
    server_mirrored: RwLock<HashMap<String, String>>,
    /// Path to persist overrides.
    storage_path: PathBuf,
}

#[derive(Default)]
struct SessionCVar {
    /// Startup values replaced by regular writes.
    imported: Option<String>,
    /// Explicit temporary values that survive regular writes until removed.
    temporary: Option<String>,
}

impl CVarStorage {
    /// Create storage with defaults parsed from YAML, loading persisted overrides from disk.
    pub fn new() -> Self {
        Self::with_path(default_storage_path())
    }

    /// Load and persist CVars in an explicitly selected session directory.
    pub fn with_session_wtf_path(root: &Path) -> Self {
        Self::with_path(root.join("cvars.json"))
    }

    fn with_path(path: PathBuf) -> Self {
        let (defaults, original_names) = parse_default_cvars();
        let overrides = load_overrides(&path);
        Self {
            defaults,
            original_names,
            registered_defaults: RwLock::new(HashMap::new()),
            registered_original_names: RwLock::new(HashMap::new()),
            overrides: RwLock::new(overrides),
            session_overrides: RwLock::new(HashMap::new()),
            server_mirrored: RwLock::new(server_mirrored_store_defaults()),
            storage_path: path,
        }
    }

    /// Isolate persisted subsystem models by this store's identity and client profile.
    pub(crate) fn model_storage_dir(&self) -> PathBuf {
        self.storage_path
            .with_extension("")
            .join(crate::client_profile::ACTIVE.cache_subdir())
            .join("SavedVariables")
    }

    /// Get the effective value (temporary, imported, persisted, then default).
    pub fn get(&self, name: &str) -> Option<String> {
        let key = name.to_lowercase();
        if is_profile_removed_cvar_key(&key) {
            return None;
        }
        if let Some(value) = self
            .session_overrides
            .read()
            .unwrap()
            .get(&key)
            .and_then(|values| values.temporary.as_ref().or(values.imported.as_ref()))
        {
            return Some(value.clone());
        }
        if let Some(value) = self.overrides.read().unwrap().get(&key) {
            return Some(value.clone());
        }
        // Fall back to defaults
        self.defaults
            .get(&key)
            .cloned()
            .or_else(|| self.registered_defaults.read().unwrap().get(&key).cloned())
    }

    /// Get the default value for a CVar.
    pub fn get_default(&self, name: &str) -> Option<String> {
        let key = name.to_lowercase();
        if is_profile_removed_cvar_key(&key) {
            return None;
        }
        self.defaults
            .get(&key)
            .cloned()
            .or_else(|| self.registered_defaults.read().unwrap().get(&key).cloned())
    }

    /// Whether this name was registered at runtime rather than supplied by the catalog.
    pub(crate) fn is_runtime_registered(&self, name: &str) -> bool {
        self.registered_defaults
            .read()
            .unwrap()
            .contains_key(&name.to_lowercase())
    }

    /// Get a CVar as a boolean ("1" = true, anything else = false).
    pub fn get_bool(&self, name: &str) -> bool {
        self.get(name).as_deref() == Some("1")
    }

    /// Set a CVar value and persist to disk.
    pub fn set(&self, name: &str, value: &str) -> bool {
        let key = name.to_lowercase();
        self.overrides
            .write()
            .unwrap()
            .insert(key.clone(), value.to_string());
        if let std::collections::hash_map::Entry::Occupied(mut entry) =
            self.session_overrides.write().unwrap().entry(key)
        {
            entry.get_mut().imported = None;
            if entry.get().temporary.is_none() {
                entry.remove();
            }
        }
        self.save();
        true
    }

    /// Seed an imported value for this launch without writing it to disk.
    pub fn set_imported(&self, name: &str, value: &str) {
        self.session_overrides
            .write()
            .unwrap()
            .entry(name.to_lowercase())
            .or_default()
            .imported = Some(value.to_string());
    }

    /// Replace one known host/server mirror value without exposing it to CVar getters.
    pub(crate) fn set_server_mirrored(&self, name: &str, value: &str) -> Option<String> {
        let key = name.to_lowercase();
        let mut server_mirrored = self.server_mirrored.write().unwrap();
        let current = server_mirrored.get_mut(&key)?;
        Some(std::mem::replace(current, value.to_owned()))
    }

    /// Read a hidden server mirror as a CVar boolean.
    pub(crate) fn get_server_mirrored_bool(&self, name: &str) -> Option<bool> {
        self.server_mirrored
            .read()
            .unwrap()
            .get(&name.to_lowercase())
            .map(|value| value == "1")
    }

    /// Whether the selected profile has a hidden server-mirrored CVar with this name.
    pub(crate) fn is_server_mirrored(&self, name: &str) -> bool {
        self.server_mirrored
            .read()
            .unwrap()
            .contains_key(&name.to_lowercase())
    }

    /// Set an effective temporary value without changing or saving persisted state.
    pub(crate) fn set_temporary(&self, name: &str, value: &str) {
        self.session_overrides
            .write()
            .unwrap()
            .entry(name.to_lowercase())
            .or_default()
            .temporary = Some(value.to_owned());
    }

    /// Remove a temporary value so reads reveal the next available value.
    pub(crate) fn remove_temporary(&self, name: &str) {
        if let std::collections::hash_map::Entry::Occupied(mut entry) = self
            .session_overrides
            .write()
            .unwrap()
            .entry(name.to_lowercase())
        {
            entry.get_mut().temporary = None;
            if entry.get().imported.is_none() {
                entry.remove();
            }
        }
    }

    /// Get all known CVar names in original case (defaults + overrides).
    pub fn all_keys(&self) -> Vec<String> {
        let mut keys: std::collections::HashSet<String> = self.defaults.keys().cloned().collect();
        for key in self.registered_defaults.read().unwrap().keys() {
            keys.insert(key.clone());
        }
        for key in self.overrides.read().unwrap().keys() {
            keys.insert(key.clone());
        }
        keys.extend(
            self.session_overrides
                .read()
                .unwrap()
                .iter()
                .filter(|(_, values)| values.imported.is_some())
                .map(|(key, _)| key.clone()),
        );
        keys.retain(|key| !is_profile_removed_cvar_key(key));
        let mut sorted: Vec<String> = keys
            .into_iter()
            .map(|k| {
                self.original_names
                    .get(&k)
                    .cloned()
                    .or_else(|| {
                        self.registered_original_names
                            .read()
                            .unwrap()
                            .get(&k)
                            .cloned()
                    })
                    .unwrap_or(k)
            })
            .collect();
        sorted.sort_by_key(|name| name.to_lowercase());
        sorted
    }

    /// Register a new CVar with a default value.
    pub fn register(&self, name: &str, default: Option<&str>) {
        let key = name.to_lowercase();
        if self.defaults.contains_key(&key) || is_profile_removed_cvar_key(&key) {
            return;
        }

        self.registered_original_names
            .write()
            .unwrap()
            .entry(key.clone())
            .or_insert_with(|| name.to_string());

        let value = default.unwrap_or("0");
        self.registered_defaults
            .write()
            .unwrap()
            .entry(key)
            .or_insert_with(|| value.to_string());
    }

    /// Persist current overrides to disk, except session-only CVars.
    fn save(&self) {
        if let Some(parent) = self.storage_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let overrides = self.overrides.read().unwrap();
        let persisted: HashMap<&String, &String> = overrides
            .iter()
            .filter(|(key, _)| !is_session_only_cvar_key(key))
            .collect();
        if let Ok(json) = serde_json::to_string_pretty(&persisted) {
            let _ = crate::fs_atomic::write_atomic(&self.storage_path, json.as_bytes());
        }
    }
}

impl Default for CVarStorage {
    fn default() -> Self {
        Self::new()
    }
}

fn server_mirrored_store_defaults() -> HashMap<String, String> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        return HashMap::new();
    }

    HashMap::from([
        ("bpaystoreenable".to_owned(), "0".to_owned()),
        ("shop2enabled".to_owned(), "1".to_owned()),
    ])
}

fn parse_default_cvars() -> (HashMap<String, String>, HashMap<String, String>) {
    let (mut defaults, mut original_names) = parse_cvar_yaml(include_str!("cvars.yaml"));
    insert_store_mirror_cvars(&mut defaults, &mut original_names);
    insert_collected_journal_cvar(&mut defaults, &mut original_names);
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever {
        // Forever 1.60.1.70707 WM77b native registration.
        defaults.insert("minimaptrackedinfov4".into(), "2162728".into());
    }
    #[cfg(feature = "retail-12-0-0")]
    for name in [
        "minimapTrackedInfov2",
        "useCompactPartyFrames",
        "volumeFogDisableLightScattering",
        "volumeFogDisableNoise",
        "volumeFogDisableShadows",
        "volumeFogUse16BitTexture",
    ] {
        // Fog defaults documented by the 11.2.0 page; other zeros are inferred.
        let key = name.to_lowercase();
        defaults.insert(key.clone(), "0".to_owned());
        original_names.insert(key, name.to_owned());
    }
    #[cfg(feature = "client-wrath")]
    {
        // Blizzard Wrath FrameXML/InterfaceOptionsFrame.lua:319 UI default.
        defaults.insert("targetoftargetmode".into(), "5".into());
        original_names.insert("targetoftargetmode".into(), "targetOfTargetMode".into());
        // User-approved Wrath simulator policy (2026-10-17), not a native default.
        defaults.insert("basemip".into(), "0".into());
        original_names.insert("basemip".into(), "BaseMip".into());
    }
    insert_profile_cvars(&mut defaults, &mut original_names);
    remove_profile_cvars(&mut defaults, &mut original_names);
    (defaults, original_names)
}

/// Retail/PTR server-mirrored store flags, with defaults verified by R20.
fn insert_store_mirror_cvars(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
) {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        return;
    }
    for (name, value) in [("bpayStoreEnable", "0"), ("shop2Enabled", "1")] {
        let key = name.to_lowercase();
        defaults.insert(key.clone(), value.to_owned());
        original_names.insert(key, name.to_owned());
    }
}

/// Collections journal CVar shipped by the PTR and WowForever pins only.
///
/// The pinned CVar dumps list it with `default=1` and `scope=Account`:
/// `data/patch-api/source-cache/legacy-2026-10-09/1.60.1-wikitext.txt:2046`
/// and `data/patch-api/sources/12.1.7-inherited-1.60.1.wikitext:1986`. The
/// Wowless catalogs carry the same account flag on PTR
/// (`crates/sim-data-profile/data/cvar_info_ptr.rs:1153`) and WowForever
/// (`cvar_info_wowforever.rs:1161`). The Retail 12.1.5 catalog omits the name
/// and no Retail pin consumes it, so it stays unregistered on the other
/// profiles.
fn insert_collected_journal_cvar(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
) {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return;
    }
    let name = "onlyShowCollectedItemsInJournal";
    let key = name.to_lowercase();
    original_names.insert(key.clone(), name.to_string());
    defaults.insert(key, "1".to_string());
}

#[cfg(feature = "retail-12-0-5")]
fn insert_profile_cvars(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
) {
    insert_cvar_defaults(defaults, original_names, PATCH_12_0_5_CVARS);
    #[cfg(feature = "retail-12-0-7")]
    insert_cvar_defaults(defaults, original_names, PATCH_12_0_7_CVARS);
    #[cfg(feature = "retail-12-1-0")]
    insert_cvar_defaults(defaults, original_names, PATCH_12_1_CVARS);
}

#[cfg(feature = "client-wowforever")]
fn insert_profile_cvars(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
) {
    for (name, value) in [
        // Native Forever 1.60.1.69913 probe; see docs/specs/forever-gamepad-cvars.md.
        ("GamepadPossessBarOverride", "4"),
        ("GamepadStanceBarOverride", "3"),
        // Selected Forever glue TextSizeManager requires this CVar. Availability
        // is inferred from that consumer; reuse the modeled Retail/PTR scale,
        // not a proven native Forever default.
        ("userFontScaleGlue", "1.0"),
    ] {
        let key = name.to_lowercase();
        original_names.insert(key.clone(), name.to_string());
        defaults.insert(key, value.to_string());
    }
}

#[cfg(not(any(feature = "retail-12-0-5", feature = "client-wowforever")))]
fn insert_profile_cvars(
    _defaults: &mut HashMap<String, String>,
    _original_names: &mut HashMap<String, String>,
) {
}

#[cfg(feature = "retail-12-0-5")]
fn insert_cvar_defaults(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
    values: &[(&str, &str)],
) {
    for &(name, value) in values {
        let key = name.to_lowercase();
        original_names.insert(key.clone(), name.to_string());
        defaults.insert(key, value.to_string());
    }
}

#[cfg(any(
    feature = "retail-12-0-0",
    feature = "retail-12-0-7",
    feature = "retail-12-1-0"
))]
fn remove_profile_cvars(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
) {
    remove_cvar_defaults(defaults, original_names, PATCH_12_0_0_REMOVED_CVARS);
    #[cfg(feature = "retail-12-0-5")]
    remove_cvar_defaults(defaults, original_names, PATCH_12_0_1_REMOVED_CVARS);
    #[cfg(feature = "retail-12-0-5")]
    remove_cvar_defaults(defaults, original_names, PATCH_12_0_5_REMOVED_CVARS);
    #[cfg(feature = "retail-12-0-7")]
    remove_cvar_defaults(defaults, original_names, PATCH_12_0_7_REMOVED_CVARS);
    #[cfg(feature = "retail-12-1-0")]
    remove_cvar_defaults(defaults, original_names, PATCH_12_1_REMOVED_CVARS);
}

#[cfg(not(any(
    feature = "retail-12-0-0",
    feature = "retail-12-0-7",
    feature = "retail-12-1-0"
)))]
fn remove_profile_cvars(
    _defaults: &mut HashMap<String, String>,
    _original_names: &mut HashMap<String, String>,
) {
}

#[cfg(any(
    feature = "retail-12-0-0",
    feature = "retail-12-0-7",
    feature = "retail-12-1-0"
))]
fn remove_cvar_defaults(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
    names: &[&str],
) {
    for key in names.iter().map(|name| name.to_lowercase()) {
        defaults.remove(&key);
        original_names.remove(&key);
    }
}

#[cfg(any(
    feature = "retail-12-0-0",
    feature = "retail-12-0-7",
    feature = "retail-12-1-0"
))]
fn is_profile_removed_cvar_key(key: &str) -> bool {
    if PATCH_12_0_0_REMOVED_CVARS
        .iter()
        .any(|removed| removed.eq_ignore_ascii_case(key))
    {
        return true;
    }

    #[cfg(feature = "retail-12-0-5")]
    if PATCH_12_0_1_REMOVED_CVARS
        .iter()
        .any(|removed| removed.eq_ignore_ascii_case(key))
    {
        return true;
    }

    #[cfg(feature = "retail-12-0-5")]
    if PATCH_12_0_5_REMOVED_CVARS
        .iter()
        .any(|removed| removed.eq_ignore_ascii_case(key))
    {
        return true;
    }

    #[cfg(feature = "retail-12-0-7")]
    if PATCH_12_0_7_REMOVED_CVARS
        .iter()
        .any(|removed| removed.eq_ignore_ascii_case(key))
    {
        return true;
    }

    #[cfg(feature = "retail-12-1-0")]
    if PATCH_12_1_REMOVED_CVARS
        .iter()
        .any(|removed| removed.eq_ignore_ascii_case(key))
    {
        return true;
    }

    false
}

#[cfg(not(any(
    feature = "retail-12-0-0",
    feature = "retail-12-0-7",
    feature = "retail-12-1-0"
)))]
fn is_profile_removed_cvar_key(_key: &str) -> bool {
    false
}

#[cfg(feature = "retail-12-0-0")]
const PATCH_12_0_0_REMOVED_CVARS: &[&str] = &[
    "NamePlateHorizontalScale",
    "NamePlateVerticalScale",
    "ShowClassColorInFriendlyNameplate",
];

// 12.0.5 is the first supported epoch after the 12.0.1 removal.
#[cfg(feature = "retail-12-0-5")]
const PATCH_12_0_1_REMOVED_CVARS: &[&str] = &[
    "minimapTrackedInfov2",
    "useCompactPartyFrames",
    "nameplateLargerScale",
    "nameplatePlayerLargerScale",
    "nameplateSelfAlpha",
];

// Only names cvars.yaml still carries; nameplateShowFriends and
// unlockedExpansionLandingPages return in later patches.
#[cfg(feature = "retail-12-0-5")]
const PATCH_12_0_5_REMOVED_CVARS: &[&str] = &[
    "cameraDistanceFixedValue",
    "endeavorInitiativesLastPoints",
    "secretChallengeModeRestrictionsForced",
    "secretCombatRestrictionsForced",
    "secretEncounterRestrictionsForced",
    "secretMapRestrictionsForced",
    "secretPvPMatchRestrictionsForced",
];

#[cfg(feature = "retail-12-0-5")]
const PATCH_12_0_5_CVARS: &[(&str, &str)] = &[
    ("addonChallengeModeRestrictionsForced", "0"),
    ("addonCombatRestrictionsForced", "0"),
    ("addonEncounterRestrictionsForced", "0"),
    ("addonMapRestrictionsForced", "0"),
    ("addonPvPMatchRestrictionsForced", "0"),
    ("AllowSpectateMode", "1"),
    // INFERRED simulator default; the 12.0.5 page lists no default.
    ("endeavorInitiativesLastPointsMap", ""),
    ("houseExterior_Hide_Decor", "1"),
    ("transmogPreviewedWeaponToggle", "0"),
];

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
const PATCH_12_0_7_REMOVED_CVARS: &[&str] = &[
    "debugGameEvents",
    "frontendMatchingModes_WowLabs",
    "last_matchmaking_party_size",
    "lastCharacterGuid",
    "skipStartGear",
];

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
const PATCH_12_0_7_CVARS: &[(&str, &str)] = &[
    // INFERRED simulator default; the retained 12.0.7 excerpt names this CVar only.
    ("assistedCombatReduceHighlights", "1"),
    ("developerLog", "0"),
    // INFERRED simulator defaults; native historical filter defaults are unproved.
    ("developerLogFilterDebug", "0"),
    ("developerLogFilterError", "1"),
    ("developerLogFilterFatal", "1"),
    ("developerLogFilterNormal", "1"),
    ("developerLogFilterSpam", "0"),
    ("developerLogFilterWarning", "1"),
    ("developerLogWriteToFile", "1"),
    ("housingDecorLightRadiusIndicatorsEnabled", "1"),
    ("housingOtherDecorLightRadiusIndicatorType", "1"),
    ("housingSelectedDecorLightRadiusIndicatorType", "1"),
    ("KioskCanSessionExpire", "1"),
    ("KioskCharacterTemplateSet", "0"),
    ("KioskLobbyKickSeconds", "30"),
    ("ThreadPoolPerThreadAllocator", "1"),
    ("useBLEEP", "0"),
    ("gxWindowedResolution", "auto"),
    // INFERRED simulator default (no landing page unlocked); the page lists none.
    ("unlockedExpansionLandingPages", "0"),
    // {{Test-inline}} rows, published like 12.1.0 AftermathShaderDebug.
    ("Aftermath", "1"),
    ("AftermathCallstacks", "0"),
    ("enableMemoryTrap", "1"),
];

#[cfg(feature = "retail-12-1-0")]
const PATCH_12_1_REMOVED_CVARS: &[&str] = &[
    "auctionDisplayOnCharacter",
    "auctionSortByBuyoutPrice",
    "auctionSortByUnitPrice",
    "lastLockedDelvesCompanionAbilities",
    "SlugSupersampling",
];

#[cfg(feature = "retail-12-1-0")]
const PATCH_12_1_CVARS: &[(&str, &str)] = &[
    ("accessibilityScreenNarrationEnabled", "0"),
    ("accessibilityScreenNarrationSpeechRate", "1.0"),
    ("accessibilityScreenNarrationSpeechVolume", "1.0"),
    ("accessibilityScreenNarrationVoice", ""),
    ("AftermathShaderDebug", "0"),
    ("discordClientEnabled", "0"),
    ("discordDisplayName", ""),
    ("nameplateCheckDistanceForTarget", "60"),
    ("nameplateForceShowUnitName", "0"),
    ("nameplateNotSelectedAlpha", "0.600000"),
    ("nameplatePlayRemovalAnimation", "1"),
    ("nameplateShowAllPersonalAuras", "1"),
    ("nameplateShowFriendlyRealmName", "0"),
    ("nameplateShowFriends", "0"),
    ("pingTarget", "0"),
    ("raidFramesDispelIndicatorOverlayAnimation", "1"),
    ("showPingsOnRaidFrames", "1"),
    ("showScreenNarrationDialog", "1"),
    ("taintLogObjectSecrets", "0"),
    // INFERRED simulator defaults (off); the 12.1.0 page names these CVars only.
    ("tooltipShowAuraSpellIDs", "0"),
    ("userFontScaleGlue", "1.0"),
    ("worldMapShowCursorCoords", "0"),
    ("worldMapShowPlayerCoords", "0"),
];

/// CVars whose value resets every session (lowercase keys). The 12.1.0 page:
/// "This cvar will not persist between sessions."
const SESSION_ONLY_CVAR_KEYS: &[&str] = &["tooltipshowauraspellids"];

fn is_session_only_cvar_key(key: &str) -> bool {
    SESSION_ONLY_CVAR_KEYS.contains(&key)
}

/// Load persisted overrides from disk, dropping session-only CVars.
fn load_overrides(path: &PathBuf) -> HashMap<String, String> {
    let mut overrides: HashMap<String, String> = match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => HashMap::new(),
    };
    overrides.retain(|key, _| !is_session_only_cvar_key(key));
    if let Some(value) = overrides.get_mut("lastlockedtieredentrancecompanionabilities")
        && let Some(migrated) = migrate_legacy_delves_abilities(value)
    {
        *value = migrated;
    }
    overrides
}

/// Older simulator codecs wrote JSON in place of Base64 CBOR. Recognize
/// only the Delves companion's PDE ID → locked node ID → 1 shape in the
/// persisted simulator store. Loading never rewrites the file.
fn migrate_legacy_delves_abilities(value: &str) -> Option<String> {
    fn id(key: &str) -> Option<u32> {
        let value: u32 = key.parse().ok()?;
        (value > 0 && value.to_string() == key).then_some(value)
    }

    if !value.starts_with('{') {
        return None;
    }
    let maps: BTreeMap<String, BTreeMap<String, f64>> = serde_json::from_str(value).ok()?;
    let maps = maps
        .into_iter()
        .map(|(pde, nodes)| {
            let flags = nodes
                .into_iter()
                .map(|(node, flag)| if flag == 1.0 { id(&node) } else { None })
                .collect::<Option<Vec<_>>>()?;
            Some((id(&pde)?, flags))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(crate::c_api::encoding_util::encode_persisted_id_flag_maps(
        &maps,
    ))
}

/// Parse YAML in format `key: 'value'` or `key: value`.
/// Returns (defaults: lowercase_key->value, original_names: lowercase_key->original_key).
fn parse_cvar_yaml(yaml: &str) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut defaults = HashMap::new();
    let mut original_names = HashMap::new();
    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let original_key = key.trim().to_string();
            let lower_key = original_key.to_lowercase();
            let value = value.trim();
            // Strip surrounding quotes and process escapes for double-quoted strings
            let (value, is_double_quoted) = if let Some(inner) =
                value.strip_prefix('"').and_then(|v| v.strip_suffix('"'))
            {
                (inner, true)
            } else if let Some(inner) = value.strip_prefix('\'').and_then(|v| v.strip_suffix('\''))
            {
                (inner, false)
            } else {
                (value, false)
            };
            let value = if is_double_quoted {
                process_yaml_escapes(value)
            } else {
                value.to_string()
            };
            original_names.insert(lower_key.clone(), original_key);
            defaults.insert(lower_key, value);
        }
    }
    (defaults, original_names)
}

/// Process YAML double-quoted string escape sequences (\\xHH hex escapes).
fn process_yaml_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 3 < bytes.len() && bytes[i + 1] == b'x' {
            let hex = &s[i + 2..i + 4];
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                result.push(byte as char);
                i += 4;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
            someVar: '1'
            otherVar: "hello"
            plainVar: 50
        "#;
        let (defaults, original_names) = parse_cvar_yaml(yaml);
        assert_eq!(defaults.get("somevar"), Some(&"1".to_string()));
        assert_eq!(defaults.get("othervar"), Some(&"hello".to_string()));
        assert_eq!(defaults.get("plainvar"), Some(&"50".to_string()));
        // Original case preserved
        assert_eq!(original_names.get("somevar"), Some(&"someVar".to_string()));
        assert_eq!(
            original_names.get("othervar"),
            Some(&"otherVar".to_string())
        );
        assert_eq!(
            original_names.get("plainvar"),
            Some(&"plainVar".to_string())
        );
    }

    #[test]
    fn test_get_set() {
        let storage = CVarStorage::new();
        // Test default
        assert!(storage.get("nameplateShowEnemies").is_some());
        // Test override
        storage.set("nameplateShowEnemies", "0");
        assert_eq!(storage.get("nameplateShowEnemies"), Some("0".to_string()));
        // Test case insensitivity
        assert_eq!(
            storage.get("NAMEPLATESHOWENEMIES"),
            storage.get("nameplateShowEnemies")
        );
    }

    #[test]
    fn store_mirror_cvars_have_the_pinned_client_defaults() {
        use crate::client_profile::{ACTIVE, ClientProfile};

        if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
            return;
        }

        let storage = CVarStorage::new();
        assert_eq!(storage.get_default("bpayStoreEnable").as_deref(), Some("0"));
        assert_eq!(storage.get_default("shop2Enabled").as_deref(), Some("1"));
        assert_eq!(
            storage
                .get_default("seenAsiaCharacterUpgradePopup")
                .as_deref(),
            Some("0")
        );
        assert_eq!(
            storage.get_server_mirrored_bool("bpayStoreEnable"),
            Some(false)
        );
        assert_eq!(storage.get_server_mirrored_bool("shop2Enabled"), Some(true));
    }

    #[test]
    fn test_persistence_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cvars.json");

        // Set overrides and verify they're written to disk
        {
            let storage = CVarStorage::with_path(path.clone());
            storage.set("checkaddonversion", "0");
            storage.set("someCustomVar", "hello");
            assert!(path.exists(), "cvars.json should exist after set()");
        }

        // Load from same path — overrides should survive
        {
            let storage = CVarStorage::with_path(path.clone());
            assert_eq!(
                storage.get("checkaddonversion"),
                Some("0".to_string()),
                "persisted override should take precedence over default"
            );
            // Keys are stored lowercase, so any casing resolves the same value
            assert_eq!(
                storage.get("someCustomVar"),
                Some("hello".to_string()),
                "case-insensitive lookup should find persisted value"
            );
            assert_eq!(
                storage.get("somecustomvar"),
                Some("hello".to_string()),
                "lowercase lookup should also work"
            );
        }
    }

    /// 12.1.0: autoLootDefault is account wide and survives a new session;
    /// tooltipShowAuraSpellIDs resets to its default every session.
    #[cfg(feature = "retail-12-1-0")]
    #[test]
    fn patch_12_1_cvar_scope_survives_or_resets_across_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cvars.json");
        {
            let storage = CVarStorage::with_path(path.clone());
            storage.set("autoLootDefault", "1");
            storage.set("tooltipShowAuraSpellIDs", "1");
            assert_eq!(storage.get("tooltipShowAuraSpellIDs").as_deref(), Some("1"));
        }
        let storage = CVarStorage::with_path(path.clone());
        assert_eq!(storage.get("autoLootDefault").as_deref(), Some("1"));
        assert_eq!(storage.get("tooltipShowAuraSpellIDs").as_deref(), Some("0"));
        assert!(
            !fs::read_to_string(&path)
                .unwrap()
                .contains("tooltipshowauraspellids")
        );
    }

    #[test]
    fn test_persistence_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent").join("cvars.json");

        // No file — should fall back to defaults without error
        let storage = CVarStorage::with_path(path.clone());
        assert_eq!(storage.get("checkaddonversion"), Some("1".to_string()));

        // set() should create parent dirs
        storage.set("checkaddonversion", "0");
        assert!(path.exists());
    }

    #[test]
    fn test_persistence_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cvars.json");
        fs::write(&path, "not valid json {{{").unwrap();

        // Corrupt file — should fall back to defaults
        let storage = CVarStorage::with_path(path);
        assert_eq!(storage.get("checkaddonversion"), Some("1".to_string()));
    }

    #[test]
    fn test_register_exposes_runtime_default_without_override() {
        let storage = CVarStorage::new();
        storage.register("PraiseTheSun", Some("1"));

        assert_eq!(storage.get("PraiseTheSun"), Some("1".to_string()));
        assert_eq!(storage.get_default("PraiseTheSun"), Some("1".to_string()));
        assert!(storage.all_keys().iter().any(|key| key == "PraiseTheSun"));
    }

    #[cfg(feature = "retail-12-0-5")]
    #[test]
    fn patch_12_0_5_renamed_restriction_cvars() {
        let storage = CVarStorage::new();
        assert_eq!(
            storage.get_default("addonCombatRestrictionsForced"),
            Some("0".to_string())
        );
        assert_eq!(storage.get("AllowSpectateMode"), Some("1".to_string()));
        assert_eq!(storage.get("secretCombatRestrictionsForced"), None);
        assert_eq!(storage.get_default("cameraDistanceFixedValue"), None);
        // A removed CVar cannot be re-registered at runtime.
        storage.register("secretCombatRestrictionsForced", Some("0"));
        assert_eq!(storage.get("secretCombatRestrictionsForced"), None);
    }

    #[cfg(feature = "retail-12-0-7")]
    #[test]
    fn patch_12_0_7_cvar_defaults_match_retail() {
        let storage = CVarStorage::new();
        for (name, value) in PATCH_12_0_7_CVARS {
            assert_eq!(storage.get(name), Some((*value).to_string()), "{name}");
            assert_eq!(
                storage.get_default(name),
                Some((*value).to_string()),
                "{name}"
            );
        }
        assert!(storage.get_bool("assistedCombatReduceHighlights"));
        assert!(!storage.get_bool("developerLogFilterDebug"));
        assert_eq!(
            storage.get("gxWindowedResolution"),
            Some("auto".to_string())
        );
        for name in PATCH_12_0_7_REMOVED_CVARS {
            assert_eq!(storage.get(name), None, "{name}");
        }
    }
}
