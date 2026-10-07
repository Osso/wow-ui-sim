//! Tests for addon API functions (addon_api.rs).

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;
use std::sync::Mutex;

use wow_ui_sim::lua_api::AddonInfo;
use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "addon_api/dependencies.rs"]
mod addon_api_dependencies;
#[path = "addon_api/beta_policy.rs"]
mod addon_api_beta_policy;
#[path = "addon_api/profiler.rs"]
mod addon_api_profiler;

fn env_with_addons() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    {
        let mut state = env.state().borrow_mut();
        state.addons.push(AddonInfo {
            folder_name: "MyAddon".into(),
            title: "My Addon Title".into(),
            notes: "A test addon".into(),
            enabled: true,
            loaded: true,
            load_on_demand: false,
            load_time_secs: 0.0,
            metadata: HashMap::from([
                ("X-Prefix".to_string(), "MyPrefix".to_string()),
                ("X-Acronym".to_string(), "MA".to_string()),
                ("Version".to_string(), "1.2.3".to_string()),
                ("X-oUF".to_string(), "ElvUF".to_string()),
            ]),
            ..Default::default()
        });
        state.addons.push(AddonInfo {
            folder_name: "LODAddon".into(),
            title: "LOD Addon".into(),
            notes: "".into(),
            enabled: false,
            loaded: false,
            load_on_demand: true,
            load_time_secs: 0.0,
            ..Default::default()
        });
    }
    env
}

// ============================================================================
// C_AddOns.GetNumAddOns
// ============================================================================

#[test]
fn test_get_num_addons() {
    let env = env_with_addons();
    // 2 test addons + 1 __BuiltIn pseudo-addon from init_builtin_frames
    let count: i32 = env.eval("return C_AddOns.GetNumAddOns()").unwrap();
    assert_eq!(count, 3);
}

// ============================================================================
// C_AddOns.GetAddOnInfo
// ============================================================================

#[test]
fn test_get_addon_info_by_index() {
    let env = env_with_addons();
    // Index 1 is __BuiltIn, test addons start at index 2
    let (name, title, notes, loadable): (String, String, String, bool) =
        env.eval("return C_AddOns.GetAddOnInfo(2)").unwrap();
    assert_eq!(name, "MyAddon");
    assert_eq!(title, "My Addon Title");
    assert_eq!(notes, "A test addon");
    assert!(loadable);
}

#[test]
fn test_get_addon_info_by_name() {
    let env = env_with_addons();
    let (name, title): (String, String) =
        env.eval("return C_AddOns.GetAddOnInfo('MyAddon')").unwrap();
    assert_eq!(name, "MyAddon");
    assert_eq!(title, "My Addon Title");
}

#[test]
fn test_get_addon_info_not_found() {
    let env = env_with_addons();
    let is_nil: bool = env
        .eval("local n = C_AddOns.GetAddOnInfo(999); return n == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_get_addon_info_missing_blizzard_addon_reports_reason() {
    let env = env_with_addons();
    let (name, loadable, reason): (String, bool, String) = env
        .eval(
            r#"
            local name, _, _, loadable, reason =
                C_AddOns.GetAddOnInfo("Blizzard_DefinitelyMissing")
            return name, loadable, reason
            "#,
        )
        .unwrap();
    assert_eq!(name, "Blizzard_DefinitelyMissing");
    assert!(!loadable);
    assert_eq!(reason, "MISSING");
}

#[test]
fn test_load_missing_blizzard_addon_reports_missing_reason() {
    let env = env_with_addons();
    let (loaded, reason): (bool, String) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_DefinitelyMissing")"#)
        .unwrap();
    assert!(!loaded);
    assert_eq!(reason, "MISSING");
}

// ============================================================================
// C_AddOns.IsAddOnLoaded
// ============================================================================

#[test]
fn test_is_addon_loaded_by_name() {
    let env = env_with_addons();
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('MyAddon')")
        .unwrap();
    assert!(loaded);
    let not_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('LODAddon')")
        .unwrap();
    assert!(!not_loaded);
}

#[test]
fn test_is_addon_loaded_by_index() {
    let env = env_with_addons();
    // Index 2 = MyAddon (loaded), Index 3 = LODAddon (not loaded)
    let loaded: bool = env.eval("return C_AddOns.IsAddOnLoaded(2)").unwrap();
    assert!(loaded);
    let not_loaded: bool = env.eval("return C_AddOns.IsAddOnLoaded(3)").unwrap();
    assert!(!not_loaded);
}

// ============================================================================
// C_AddOns.IsAddOnLoadOnDemand
// ============================================================================

#[test]
fn test_is_addon_load_on_demand() {
    let env = env_with_addons();
    let lod: bool = env
        .eval("return C_AddOns.IsAddOnLoadOnDemand('LODAddon')")
        .unwrap();
    assert!(lod);
    let not_lod: bool = env
        .eval("return C_AddOns.IsAddOnLoadOnDemand('MyAddon')")
        .unwrap();
    assert!(!not_lod);
}

#[test]
fn test_is_addon_loadable_reports_enabled_and_disabled_states() {
    let env = env_with_addons();

    let (enabled_loadable, enabled_reason): (bool, Option<String>) = env
        .eval("return C_AddOns.IsAddOnLoadable('MyAddon')")
        .unwrap();
    assert!(enabled_loadable);
    assert_eq!(enabled_reason, None);

    let (disabled_loadable, disabled_reason): (bool, String) = env
        .eval("return C_AddOns.IsAddOnLoadable('LODAddon')")
        .unwrap();
    assert!(!disabled_loadable);
    assert_eq!(disabled_reason, "DISABLED");
}

// ============================================================================
// C_AddOns.EnableAddOn / DisableAddOn
// ============================================================================

#[test]
fn test_enable_disable_addon_by_name() {
    let env = env_with_addons();
    // LODAddon starts disabled
    let state_before: i32 = env
        .eval("return C_AddOns.GetAddOnEnableState('LODAddon')")
        .unwrap();
    assert_eq!(state_before, 0);

    env.eval::<()>("C_AddOns.EnableAddOn('LODAddon')").unwrap();
    let state_after: i32 = env
        .eval("return C_AddOns.GetAddOnEnableState('LODAddon')")
        .unwrap();
    assert_eq!(state_after, 2);

    env.eval::<()>("C_AddOns.DisableAddOn('LODAddon')").unwrap();
    let state_disabled: i32 = env
        .eval("return C_AddOns.GetAddOnEnableState('LODAddon')")
        .unwrap();
    assert_eq!(state_disabled, 0);
}

#[test]
fn test_enable_disable_addon_by_index() {
    let env = env_with_addons();
    // Index 2 = MyAddon
    env.eval::<()>("C_AddOns.DisableAddOn(2)").unwrap();
    let state: i32 = env.eval("return C_AddOns.GetAddOnEnableState(2)").unwrap();
    assert_eq!(state, 0);

    env.eval::<()>("C_AddOns.EnableAddOn(2)").unwrap();
    let state: i32 = env.eval("return C_AddOns.GetAddOnEnableState(2)").unwrap();
    assert_eq!(state, 2);
}

// ============================================================================
// C_AddOns.EnableAllAddOns / DisableAllAddOns
// ============================================================================

#[test]
fn test_enable_all_disable_all() {
    let env = env_with_addons();
    env.eval::<()>("C_AddOns.DisableAllAddOns()").unwrap();
    // Index 2 = MyAddon, Index 3 = LODAddon
    let s1: i32 = env.eval("return C_AddOns.GetAddOnEnableState(2)").unwrap();
    let s2: i32 = env.eval("return C_AddOns.GetAddOnEnableState(3)").unwrap();
    assert_eq!(s1, 0);
    assert_eq!(s2, 0);

    env.eval::<()>("C_AddOns.EnableAllAddOns()").unwrap();
    let s1: i32 = env.eval("return C_AddOns.GetAddOnEnableState(2)").unwrap();
    let s2: i32 = env.eval("return C_AddOns.GetAddOnEnableState(3)").unwrap();
    assert_eq!(s1, 2);
    assert_eq!(s2, 2);
}

// ============================================================================
// C_AddOns.GetAddOnMetadata
// ============================================================================

#[test]
fn test_get_addon_metadata() {
    let env = env_with_addons();
    let title: String = env
        .eval("return C_AddOns.GetAddOnMetadata('MyAddon', 'Title')")
        .unwrap();
    assert_eq!(title, "My Addon Title");

    let notes: String = env
        .eval("return C_AddOns.GetAddOnMetadata('MyAddon', 'Notes')")
        .unwrap();
    assert_eq!(notes, "A test addon");

    let version: String = env
        .eval("return C_AddOns.GetAddOnMetadata('MyAddon', 'Version')")
        .unwrap();
    assert_eq!(version, "1.2.3");

    let lowercase_version: String = env
        .eval("return C_AddOns.GetAddOnMetadata('MyAddon', 'version')")
        .unwrap();
    assert_eq!(lowercase_version, "1.2.3");

    let prefix: String = env
        .eval("return C_AddOns.GetAddOnMetadata('MyAddon', 'X-Prefix')")
        .unwrap();
    assert_eq!(prefix, "MyPrefix");

    let acronym: String = env
        .eval("return C_AddOns.GetAddOnMetadata('MyAddon', 'X-Acronym')")
        .unwrap();
    assert_eq!(acronym, "MA");

    let custom: String = env
        .eval("return C_AddOns.GetAddOnMetadata('MyAddon', 'X-oUF')")
        .unwrap();
    assert_eq!(custom, "ElvUF");
}

#[test]
fn test_get_addon_metadata_unknown_addon() {
    let env = env_with_addons();
    // For unknown addons, Title returns the addon name itself
    let title: String = env
        .eval("return C_AddOns.GetAddOnMetadata('Unknown', 'Title')")
        .unwrap();
    assert_eq!(title, "Unknown");
}

// ============================================================================
// C_AddOns.DoesAddOnExist
// ============================================================================

#[test]
fn test_does_addon_exist() {
    let env = env_with_addons();
    let exists: bool = env
        .eval("return C_AddOns.DoesAddOnExist('MyAddon')")
        .unwrap();
    assert!(exists);
    let not_exists: bool = env
        .eval("return C_AddOns.DoesAddOnExist('Nonexistent')")
        .unwrap();
    assert!(!not_exists);
}

// ============================================================================
// C_AddOns.GetAddOnName / GetAddOnTitle / GetAddOnNotes
// ============================================================================

#[test]
fn test_get_addon_name_title_notes() {
    let env = env_with_addons();
    // Index 2 = MyAddon
    let name: String = env.eval("return C_AddOns.GetAddOnName(2)").unwrap();
    assert_eq!(name, "MyAddon");
    let title: String = env.eval("return C_AddOns.GetAddOnTitle(2)").unwrap();
    assert_eq!(title, "My Addon Title");
    let notes: String = env.eval("return C_AddOns.GetAddOnNotes(2)").unwrap();
    assert_eq!(notes, "A test addon");
}

#[test]
fn test_get_addon_notes_empty() {
    let env = env_with_addons();
    // Index 3 = LODAddon (empty notes)
    let is_nil: bool = env.eval("return C_AddOns.GetAddOnNotes(3) == nil").unwrap();
    assert!(is_nil, "Empty notes should return nil");
}

// ============================================================================
// C_AddOns.GetAddOnSecurity
// ============================================================================

#[test]
fn test_get_addon_security() {
    let env = env_with_addons();
    // Index 2 = MyAddon
    let sec: String = env.eval("return C_AddOns.GetAddOnSecurity(2)").unwrap();
    assert_eq!(sec, "INSECURE");
}

// ============================================================================
// C_AddOns.IsAddonVersionCheckEnabled / SetAddonVersionCheck
// ============================================================================

#[test]
fn test_version_check_toggle() {
    let env = env_with_addons();
    env.eval::<()>("C_AddOns.SetAddonVersionCheck(true)")
        .unwrap();
    let enabled: bool = env
        .eval("return C_AddOns.IsAddonVersionCheckEnabled()")
        .unwrap();
    assert!(enabled);

    env.eval::<()>("C_AddOns.SetAddonVersionCheck(false)")
        .unwrap();
    let disabled: bool = env
        .eval("return C_AddOns.IsAddonVersionCheckEnabled()")
        .unwrap();
    assert!(!disabled);
}

#[test]
fn test_save_reset_addons_restores_version_check_state() {
    let env = env_with_addons();

    env.eval::<()>(
        r#"
        C_AddOns.SetAddonVersionCheck(false)
        C_AddOns.SaveAddOns()
        C_AddOns.SetAddonVersionCheck(true)
        C_AddOns.ResetAddOns()
    "#,
    )
    .unwrap();

    let restored: bool = env
        .eval("return C_AddOns.IsAddonVersionCheckEnabled()")
        .unwrap();
    assert!(
        !restored,
        "ResetAddOns should restore the version-check state saved by SaveAddOns"
    );
}

// ============================================================================
// Legacy global functions
// ============================================================================

#[test]
#[cfg(not(any(feature = "client-retail", feature = "client-ptr")))]
fn test_legacy_get_num_addons() {
    let env = env_with_addons();
    // 2 test addons + 1 __BuiltIn
    let count: i32 = env.eval("return GetNumAddOns()").unwrap();
    assert_eq!(count, 3);
}

#[test]
#[cfg(not(any(feature = "client-retail", feature = "client-ptr")))]
fn test_legacy_is_addon_loaded() {
    let env = env_with_addons();
    let loaded: bool = env.eval("return IsAddOnLoaded('MyAddon')").unwrap();
    assert!(loaded);
}

#[test]
fn test_legacy_get_addon_metadata() {
    let env = env_with_addons();
    let title: String = env
        .eval("return GetAddOnMetadata('MyAddon', 'Title')")
        .unwrap();
    assert_eq!(title, "My Addon Title");

    let version: String = env
        .eval("return GetAddOnMetadata('MyAddon', 'version')")
        .unwrap();
    assert_eq!(version, "1.2.3");
}

// ============================================================================
// Global constants
// ============================================================================

#[test]
fn test_addon_actions_blocked_table() {
    let env = env_with_addons();
    let is_table: bool = env
        .eval("return type(ADDON_ACTIONS_BLOCKED) == 'table'")
        .unwrap();
    assert!(is_table);
}

/// 10.2.0 moved these five globals to C_AddOns; current retail Blizzard Lua has
/// no bare callers or deprecation aliases, so retail publishes only C_AddOns.
#[test]
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
fn test_retail_omits_legacy_addon_globals_but_keeps_c_addons() {
    let env = env_with_addons();
    let missing: bool = env
        .eval(
            "return GetNumAddOns == nil and IsAddOnLoaded == nil and GetAddOnEnableState == nil \
             and IsAddOnLoadOnDemand == nil and LoadAddOn == nil",
        )
        .unwrap();
    assert!(missing, "retail must not publish the 10.2.0 legacy addon globals");
    let count: i32 = env.eval("return C_AddOns.GetNumAddOns()").unwrap();
    assert_eq!(count, 3);
    let loaded: bool = env.eval("return C_AddOns.IsAddOnLoaded('MyAddon')").unwrap();
    assert!(loaded);
    let lod: bool = env
        .eval("return C_AddOns.IsAddOnLoadOnDemand('LODAddon')")
        .unwrap();
    assert!(lod);
}

// ============================================================================
// Legacy GetAddOnEnableState (always returns 2)
// ============================================================================

#[test]
#[cfg(not(any(feature = "client-retail", feature = "client-ptr")))]
fn test_legacy_get_addon_enable_state_always_returns_2() {
    let env = env_with_addons();
    // Even for disabled addons, legacy GetAddOnEnableState always returns 2
    let state: i32 = env
        .eval("return GetAddOnEnableState(2, 'LODAddon')")
        .unwrap();
    assert_eq!(
        state, 2,
        "Legacy GetAddOnEnableState should always return 2"
    );
}

// ============================================================================
// Legacy IsAddOnLoadOnDemand
// ============================================================================

#[test]
#[cfg(not(any(feature = "client-retail", feature = "client-ptr")))]
fn test_legacy_is_addon_load_on_demand() {
    let env = env_with_addons();
    let lod: bool = env.eval("return IsAddOnLoadOnDemand('LODAddon')").unwrap();
    assert!(lod);
    let not_lod: bool = env.eval("return IsAddOnLoadOnDemand('MyAddon')").unwrap();
    assert!(!not_lod);
}

// ── C_AddOns.IsAddOnDefaultEnabled ────────────────────────────────────────────

fn env_with_default_state_addons() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    {
        let mut state = env.state().borrow_mut();
        state.addons.push(AddonInfo {
            folder_name: "EnabledAddon".into(),
            title: "Enabled".into(),
            enabled: true,
            default_enabled: true,
            ..Default::default()
        });
        state.addons.push(AddonInfo {
            folder_name: "ShipsDisabledAddon".into(),
            title: "Disabled".into(),
            enabled: false,
            default_enabled: false,
            ..Default::default()
        });
    }
    env
}

#[test]
fn test_is_addon_default_enabled_true_for_factory_enabled() {
    let env = env_with_default_state_addons();
    let result: bool = env
        .eval("return C_AddOns.IsAddOnDefaultEnabled('EnabledAddon')")
        .unwrap();
    assert!(
        result,
        "addon without DefaultState should default to enabled"
    );
}

#[test]
fn test_is_addon_default_enabled_false_for_ships_disabled() {
    let env = env_with_default_state_addons();
    let result: bool = env
        .eval("return C_AddOns.IsAddOnDefaultEnabled('ShipsDisabledAddon')")
        .unwrap();
    assert!(
        !result,
        "addon with DefaultState=disabled should return false"
    );
}

#[test]
fn test_is_addon_default_enabled_independent_of_runtime_enabled() {
    // Toggling AddonInfo.enabled at runtime must not affect the factory default —
    // that is the whole point of the API for the reset-to-default path.
    let env = env_with_default_state_addons();
    {
        let mut state = env.state().borrow_mut();
        if let Some(a) = state
            .addons
            .iter_mut()
            .find(|a| a.folder_name == "EnabledAddon")
        {
            a.enabled = false;
        }
    }
    let result: bool = env
        .eval("return C_AddOns.IsAddOnDefaultEnabled('EnabledAddon')")
        .unwrap();
    assert!(
        result,
        "factory default must be independent of current state"
    );
}

#[test]
fn test_is_addon_default_enabled_by_index() {
    let env = env_with_default_state_addons();
    let result: bool = env
        .eval(
            r#"
            local idx
            for i = 1, C_AddOns.GetNumAddOns() do
                if C_AddOns.GetAddOnName(i) == "ShipsDisabledAddon" then idx = i end
            end
            return C_AddOns.IsAddOnDefaultEnabled(idx)
            "#,
        )
        .unwrap();
    assert!(!result);
}

#[test]
fn test_is_addon_default_enabled_unknown_returns_false() {
    let env = env_with_default_state_addons();
    let by_name: bool = env
        .eval("return C_AddOns.IsAddOnDefaultEnabled('NoSuchAddon')")
        .unwrap();
    assert!(!by_name);

    let by_index: bool = env
        .eval("return C_AddOns.IsAddOnDefaultEnabled(9999)")
        .unwrap();
    assert!(!by_index);
}

// ── C_AddOns.SaveAddOns / ResetAddOns ─────────────────────────────────────────

static ADDONS_TXT_ENV_LOCK: Mutex<()> = Mutex::new(());

struct ScopedAddonsTxtEnv {
    previous: Option<OsString>,
}

impl ScopedAddonsTxtEnv {
    fn set(path: &Path) -> Self {
        let previous = std::env::var_os("WOW_SIM_ADDONS_TXT");
        unsafe {
            std::env::set_var("WOW_SIM_ADDONS_TXT", path);
        }
        Self { previous }
    }
}

impl Drop for ScopedAddonsTxtEnv {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(previous) => unsafe {
                std::env::set_var("WOW_SIM_ADDONS_TXT", previous);
            },
            None => unsafe {
                std::env::remove_var("WOW_SIM_ADDONS_TXT");
            },
        }
    }
}

fn env_with_two_user_addons() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    {
        let mut state = env.state().borrow_mut();
        state.addons.push(AddonInfo {
            folder_name: "AddonA".into(),
            title: "A".into(),
            enabled: true,
            ..Default::default()
        });
        state.addons.push(AddonInfo {
            folder_name: "AddonB".into(),
            title: "B".into(),
            enabled: true,
            ..Default::default()
        });
    }
    env
}

fn enabled_state(env: &WowLuaEnv, name: &str) -> bool {
    let state = env.state().borrow();
    state
        .addons
        .iter()
        .find(|a| a.folder_name == name)
        .expect("addon registered")
        .enabled
}

#[test]
fn test_save_addons_writes_local_addons_txt() {
    let _env_lock = ADDONS_TXT_ENV_LOCK.lock().expect("env lock");
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("local").join("AddOns.txt");
    let _env = ScopedAddonsTxtEnv::set(&path);
    let env = env_with_two_user_addons();

    env.exec("C_AddOns.DisableAddOn('AddonB')").unwrap();
    env.exec("C_AddOns.SaveAddOns()").unwrap();

    let text = std::fs::read_to_string(&path).expect("read written AddOns.txt");
    assert_eq!(text, "AddonA: enabled\nAddonB: disabled\n");
}

#[test]
fn test_save_addons_followed_by_reset_reverts_pending_disable() {
    let env = env_with_two_user_addons();
    env.exec("C_AddOns.SaveAddOns()").unwrap();
    env.exec("C_AddOns.DisableAddOn('AddonA')").unwrap();
    assert!(!enabled_state(&env, "AddonA"));

    env.exec("C_AddOns.ResetAddOns()").unwrap();
    assert!(
        enabled_state(&env, "AddonA"),
        "ResetAddOns must revert AddonA to its saved-enabled baseline"
    );
}

#[test]
fn test_save_addons_followed_by_reset_reverts_pending_enable() {
    let env = env_with_two_user_addons();
    env.exec("C_AddOns.DisableAddOn('AddonA')").unwrap();
    env.exec("C_AddOns.SaveAddOns()").unwrap();
    env.exec("C_AddOns.EnableAddOn('AddonA')").unwrap();
    assert!(enabled_state(&env, "AddonA"));

    env.exec("C_AddOns.ResetAddOns()").unwrap();
    assert!(
        !enabled_state(&env, "AddonA"),
        "ResetAddOns must revert AddonA to its saved-disabled baseline"
    );
}

#[test]
fn test_save_addons_baseline_includes_all_addons() {
    // Toggle both addons, save, toggle back the other way, reset — each
    // addon must end up at the value it had at Save time, not at any of the
    // intermediate transitions.
    let env = env_with_two_user_addons();
    env.exec("C_AddOns.DisableAddOn('AddonA')").unwrap();
    env.exec("C_AddOns.DisableAddOn('AddonB')").unwrap();
    env.exec("C_AddOns.SaveAddOns()").unwrap();

    env.exec("C_AddOns.EnableAllAddOns()").unwrap();
    assert!(enabled_state(&env, "AddonA"));
    assert!(enabled_state(&env, "AddonB"));

    env.exec("C_AddOns.ResetAddOns()").unwrap();
    assert!(!enabled_state(&env, "AddonA"));
    assert!(!enabled_state(&env, "AddonB"));
}

#[test]
fn test_save_addons_snapshot_is_keyed_by_addon_name() {
    let env = env_with_two_user_addons();
    env.exec("C_AddOns.DisableAddOn('AddonB')").unwrap();
    env.exec("C_AddOns.SaveAddOns()").unwrap();

    {
        let mut state = env.state().borrow_mut();
        let addon_b = state
            .addons
            .iter()
            .position(|addon| addon.folder_name == "AddonB")
            .expect("AddonB registered");
        let addon_b = state.addons.remove(addon_b);
        state.addons.insert(0, addon_b);
    }

    env.exec("C_AddOns.EnableAllAddOns()").unwrap();
    env.exec("C_AddOns.ResetAddOns()").unwrap();

    assert!(
        enabled_state(&env, "AddonA"),
        "ResetAddOns must not apply AddonB's saved disabled state by index"
    );
    assert!(
        !enabled_state(&env, "AddonB"),
        "ResetAddOns must restore the disabled addon by folder name"
    );
}

#[test]
fn test_reset_addons_no_op_when_nothing_saved() {
    // Without a prior SaveAddOns, ResetAddOns must not clobber the live
    // state — there's no baseline to revert to.
    let env = env_with_two_user_addons();
    env.exec("C_AddOns.DisableAddOn('AddonA')").unwrap();
    env.exec("C_AddOns.ResetAddOns()").unwrap();
    assert!(
        !enabled_state(&env, "AddonA"),
        "ResetAddOns without a snapshot should leave live state alone"
    );
}

#[test]
fn test_save_addons_overwrites_previous_baseline() {
    // The baseline is the *most recent* save, not a stack. After save→toggle
    // →save, Reset should revert to the second save's state.
    let env = env_with_two_user_addons();
    env.exec("C_AddOns.SaveAddOns()").unwrap();
    env.exec("C_AddOns.DisableAddOn('AddonA')").unwrap();
    env.exec("C_AddOns.SaveAddOns()").unwrap();
    env.exec("C_AddOns.EnableAddOn('AddonA')").unwrap();

    env.exec("C_AddOns.ResetAddOns()").unwrap();
    assert!(
        !enabled_state(&env, "AddonA"),
        "ResetAddOns should revert to the second SaveAddOns snapshot, not the first"
    );
}
