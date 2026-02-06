//! Tests for addon API functions (addon_api.rs).

use wow_ui_sim::lua_api::AddonInfo;
use wow_ui_sim::lua_api::WowLuaEnv;

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
        });
        state.addons.push(AddonInfo {
            folder_name: "LODAddon".into(),
            title: "LOD Addon".into(),
            notes: "".into(),
            enabled: false,
            loaded: false,
            load_on_demand: true,
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
    let count: i32 = env.eval("return C_AddOns.GetNumAddOns()").unwrap();
    assert_eq!(count, 2);
}

// ============================================================================
// C_AddOns.GetAddOnInfo
// ============================================================================

#[test]
fn test_get_addon_info_by_index() {
    let env = env_with_addons();
    let (name, title, notes, loadable): (String, String, String, bool) = env
        .eval("return C_AddOns.GetAddOnInfo(1)")
        .unwrap();
    assert_eq!(name, "MyAddon");
    assert_eq!(title, "My Addon Title");
    assert_eq!(notes, "A test addon");
    assert!(loadable);
}

#[test]
fn test_get_addon_info_by_name() {
    let env = env_with_addons();
    let (name, title): (String, String) = env
        .eval("return C_AddOns.GetAddOnInfo('MyAddon')")
        .unwrap();
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
    let loaded: bool = env.eval("return C_AddOns.IsAddOnLoaded(1)").unwrap();
    assert!(loaded);
    let not_loaded: bool = env.eval("return C_AddOns.IsAddOnLoaded(2)").unwrap();
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
    env.eval::<()>("C_AddOns.DisableAddOn(1)").unwrap();
    let state: i32 = env
        .eval("return C_AddOns.GetAddOnEnableState(1)")
        .unwrap();
    assert_eq!(state, 0);

    env.eval::<()>("C_AddOns.EnableAddOn(1)").unwrap();
    let state: i32 = env
        .eval("return C_AddOns.GetAddOnEnableState(1)")
        .unwrap();
    assert_eq!(state, 2);
}

// ============================================================================
// C_AddOns.EnableAllAddOns / DisableAllAddOns
// ============================================================================

#[test]
fn test_enable_all_disable_all() {
    let env = env_with_addons();
    env.eval::<()>("C_AddOns.DisableAllAddOns()").unwrap();
    let s1: i32 = env
        .eval("return C_AddOns.GetAddOnEnableState(1)")
        .unwrap();
    let s2: i32 = env
        .eval("return C_AddOns.GetAddOnEnableState(2)")
        .unwrap();
    assert_eq!(s1, 0);
    assert_eq!(s2, 0);

    env.eval::<()>("C_AddOns.EnableAllAddOns()").unwrap();
    let s1: i32 = env
        .eval("return C_AddOns.GetAddOnEnableState(1)")
        .unwrap();
    let s2: i32 = env
        .eval("return C_AddOns.GetAddOnEnableState(2)")
        .unwrap();
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
    assert_eq!(version, "@project-version@");
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
    let name: String = env.eval("return C_AddOns.GetAddOnName(1)").unwrap();
    assert_eq!(name, "MyAddon");
    let title: String = env.eval("return C_AddOns.GetAddOnTitle(1)").unwrap();
    assert_eq!(title, "My Addon Title");
    let notes: String = env.eval("return C_AddOns.GetAddOnNotes(1)").unwrap();
    assert_eq!(notes, "A test addon");
}

#[test]
fn test_get_addon_notes_empty() {
    let env = env_with_addons();
    let is_nil: bool = env
        .eval("return C_AddOns.GetAddOnNotes(2) == nil")
        .unwrap();
    assert!(is_nil, "Empty notes should return nil");
}

// ============================================================================
// C_AddOns.GetAddOnSecurity
// ============================================================================

#[test]
fn test_get_addon_security() {
    let env = env_with_addons();
    let sec: String = env.eval("return C_AddOns.GetAddOnSecurity(1)").unwrap();
    assert_eq!(sec, "INSECURE");
}

// ============================================================================
// C_AddOns.IsAddonVersionCheckEnabled / SetAddonVersionCheck
// ============================================================================

#[test]
fn test_version_check_toggle() {
    let env = env_with_addons();
    env.eval::<()>("C_AddOns.SetAddonVersionCheck(true)").unwrap();
    let enabled: bool = env
        .eval("return C_AddOns.IsAddonVersionCheckEnabled()")
        .unwrap();
    assert!(enabled);

    env.eval::<()>("C_AddOns.SetAddonVersionCheck(false)").unwrap();
    let disabled: bool = env
        .eval("return C_AddOns.IsAddonVersionCheckEnabled()")
        .unwrap();
    assert!(!disabled);
}

// ============================================================================
// Legacy global functions
// ============================================================================

#[test]
fn test_legacy_get_num_addons() {
    let env = env_with_addons();
    let count: i32 = env.eval("return GetNumAddOns()").unwrap();
    assert_eq!(count, 2);
}

#[test]
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
