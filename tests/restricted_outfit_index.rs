#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::c_api::c_transmog_outfit_info::OutfitEntry;
use wow_ui_sim::lua_api::WowLuaEnv;

fn loaded_env() -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_RestrictedAddOnEnvironment"],
        &[],
    );
    env.state().borrow_mut().transmog_outfit_catalog.entries = vec![outfit(91, 7), outfit(305, 42)];
    env.exec(r#"
        assert(issecure())
        header = CreateFrame('Frame', nil, UIParent, 'SecureHandlerBaseTemplate')
        assert(select(2, header:IsProtected()))
        SecureHandlerExecute(header,
            'self:SetAttribute("first-index", GetTransmogOutfitIndex(91)); self:SetAttribute("second-index", GetTransmogOutfitIndex(305)); self:SetAttribute("missing-index", GetTransmogOutfitIndex(999))')
        assert(header:GetAttribute('first-index') == 7, 'real restricted helper must resolve ID91 to index7')
        assert(header:GetAttribute('second-index') == 42, 'indices are not IDs or array positions')
        assert(header:GetAttribute('missing-index') == nil)
    "#).unwrap();
    env
}

fn outfit(id: i64, index: i64) -> OutfitEntry {
    OutfitEntry {
        outfit_id: id,
        name: format!("Outfit {id}"),
        situation_categories: vec![],
        icon: 135771,
        is_event_outfit: false,
        is_disabled: false,
        player_facing_outfit_index: index,
    }
}

#[test]
fn real_restricted_helper_reads_live_catalog_and_is_environment_local() {
    let first = loaded_env();
    let second = loaded_env();
    first.state().borrow_mut().transmog_outfit_catalog.entries[0].player_facing_outfit_index = 19;
    first
        .exec(
            r#"
        SecureHandlerExecute(header, 'self:SetAttribute("first-index", GetTransmogOutfitIndex(91))')
        assert(header:GetAttribute('first-index') == 19)
    "#,
        )
        .unwrap();
    second
        .exec(
            r#"
        SecureHandlerExecute(header, 'self:SetAttribute("first-index", GetTransmogOutfitIndex(91))')
        assert(header:GetAttribute('first-index') == 7)
    "#,
        )
        .unwrap();
    first
        .state()
        .borrow_mut()
        .transmog_outfit_catalog
        .entries
        .clear();
    first.exec(r#"
        SecureHandlerExecute(header, 'self:SetAttribute("first-index", GetTransmogOutfitIndex(91))')
        assert(header:GetAttribute('first-index') == nil, 'cached closure must not cache catalog results')
    "#).unwrap();
    assert!(first.state().borrow().lua_errors.is_empty());
    assert!(second.state().borrow().lua_errors.is_empty());
}

#[test]
fn real_restricted_helper_output_drives_existing_outfit_macro_consumer() {
    let env = loaded_env();
    env.exec(
        r#"
        SecureHandlerExecute(header,
            'self:SetAttribute("macrotext", "/outfit !" .. GetTransmogOutfitIndex(305))')
        local text = header:GetAttribute('macrotext')
        assert(text == '/outfit !42')
        C_Macro.RunMacroText(text)
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 305)
        C_Macro.RunMacroText(text)
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 305)
        SecureHandlerExecute(header,
            'self:SetAttribute("macrotext", "/outfit " .. GetTransmogOutfitIndex(91))')
        C_Macro.RunMacroText(header:GetAttribute('macrotext'))
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 91)
        C_Macro.RunMacroText(header:GetAttribute('macrotext'))
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 0)
    "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().active_transmog_outfit_id, None);
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn real_cached_helper_preserves_original_getter_secret_boundary() {
    let env = loaded_env();
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let source = std::fs::read_to_string(
        ui.join("Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua"),
    )
    .expect("required cached retail RestrictedEnvironment.lua");
    // Load the entire unchanged file again with its documented addon-table export.
    // This exposes the real wrapper for a getter-boundary test, NOT a replacement
    // helper or restricted execution implementation. Ordinary snippets above use
    // the real TOC-loaded managed environment and CallRestrictedClosure pipeline.
    assert!(!source.contains("]====]"));
    env.exec(&format!(r#"
        local exports = {{}}
        local chunk = assert(loadstring([====[{source}]====], '@RestrictedEnvironment-outfit-boundary'))
        setfenv(chunk, __secureenv)
        chunk('Blizzard_RestrictedAddOnEnvironment', exports)
        actualCachedOutfitHelper = exports.RESTRICTED_FUNCTIONS_SCOPE.GetTransmogOutfitIndex
        assert(type(actualCachedOutfitHelper) == 'function')
    "#)).unwrap();
    env.exec(
        r#"
        local helper = actualCachedOutfitHelper
        assert(helper(secretwrap(91)) == 7)
        assert(helper(secretwrap(305)) == 42)
        assert(helper(999) == nil and select('#', helper(999)) == 1)
        local secret = secretwrap(91)
        local function addon() return helper(secret) end
        debug.setobjecttaint(addon, 'RestrictedOutfitProbe')
        assert(not pcall(addon), 'real helper must not bypass original getter authentication')
        local function ordinary() return helper(305) end
        debug.setobjecttaint(ordinary, 'RestrictedOutfitProbe')
        local ok, index = pcall(ordinary)
        assert(ok and index == 42)
        assert(issecure())
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn real_restricted_environment_keeps_direct_tables_forbidden() {
    let env = loaded_env();
    env.exec(
        r#"
        local working = GetManagedEnvironment(header, false)
        local handle = GetFrameHandle(header, true)
        local ok, failure = pcall(CallRestrictedClosure, header, 'self', working,
            handle, 'return {}', handle)
        assert(not ok and tostring(failure):find('Direct table creation is not permitted', 1, true))
        SecureHandlerExecute(header, 'self:SetAttribute("first-index", GetTransmogOutfitIndex(91))')
        assert(header:GetAttribute('first-index') == 7, 'rejection must release execution context')
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
