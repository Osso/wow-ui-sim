//! Exercise the cached Forever restricted-execution consumer, not a compiler shim.
#![cfg(feature = "client-wowforever")]

#[test]
fn forever_restricted_closures_keep_header_state_and_reject_table_literals() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_RestrictedAddOnEnvironment"],
        &[],
    );
    env.exec(
        r#"
        assert(issecure(), 'probe must begin in the ordinary secure harness context')
        local first = CreateFrame('Frame', nil, UIParent, 'SecureHandlerBaseTemplate')
        local second = CreateFrame('Frame', nil, UIParent, 'SecureHandlerBaseTemplate')
        assert(select(2, first:IsProtected()), 'template must produce a protected header')
        SecureHandlerExecute(first, 'value = 10; self:SetAttribute("probe", value)')
        assert(first:GetAttribute('probe') == 10)
        SecureHandlerExecute(first, 'value = value + 1; self:SetAttribute("probe", value)')
        assert(first:GetAttribute('probe') == 11)
        SecureHandlerExecute(second, 'value = 20; self:SetAttribute("probe", value)')
        assert(second:GetAttribute('probe') == 20 and first:GetAttribute('probe') == 11)
        local working = GetManagedEnvironment(first, false)
        assert(IsWritableRestrictedTable(working))
        local ok, failure = pcall(CallRestrictedClosure, first, 'self', working,
            GetFrameHandle(first, true), 'return {}', GetFrameHandle(first, true))
        assert(not ok and tostring(failure):find('Direct table creation is not permitted', 1, true),
            tostring(failure))
        SecureHandlerExecute(first, 'value = value + 1; self:SetAttribute("probe", value)')
        assert(first:GetAttribute('probe') == 12, 'failed closure must release execution state')
        "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
