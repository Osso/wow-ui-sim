#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const RESTRICTED_ENV_IMPORT_PROBE: &str = r#"
    local _, helpfulSpell = C_Item.GetItemSpell(5512)
    local _, harmfulSpell = C_Item.GetItemSpell(6948)
    A_Admin.SetSpellTargeting(helpfulSpell, "helpful")
    A_Admin.SetSpellTargeting(harmfulSpell, "harmful")
    A_Admin.SetActionSlot(1, 123456)
    local header = CreateFrame("Frame", nil, UIParent, "SecureHandlerBaseTemplate")
    SecureHandlerExecute(header, [[
        self:SetAttribute("helpful", IsHelpfulItem(5512))
        self:SetAttribute("harmful", IsHarmfulItem(6948))
        self:SetAttribute("not-helpful", IsHelpfulItem(6948))
        self:SetAttribute("not-harmful", IsHarmfulItem(5512))
        self:SetAttribute("has-action", HasAction(1))
        self:SetAttribute("no-action", HasAction(2))
    ]])
    return header:GetAttribute("helpful"), header:GetAttribute("harmful"),
        header:GetAttribute("not-helpful"), header:GetAttribute("not-harmful"),
        header:GetAttribute("has-action"), header:GetAttribute("no-action")
"#;

#[test]
fn restricted_environment_secure_snippet_imports_item_and_action_apis() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_RestrictedAddOnEnvironment"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_RestrictedAddOnEnvironment"));
    assert_restricted_environment_imports(&env);
}

fn assert_restricted_environment_imports(env: &WowLuaEnv) {
    let result: (bool, bool, bool, bool, bool, bool) = env
        .eval(RESTRICTED_ENV_IMPORT_PROBE)
        .expect("original restricted snippet should return API results");
    assert_eq!(result, (true, true, false, false, true, false));
}
