#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn communities_secure_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_CommunitiesSecure/Blizzard_CommunitiesSecure.toc")
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn blizzard_communities_secure_toc_declares_secure_environment_and_required_deps() {
    let toc = TocFile::from_file(&communities_secure_toc())
        .expect("Blizzard_CommunitiesSecure TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_CommunitiesSecure is a non-LOD addon (it should auto-load on Game screen)"
    );
    assert!(
        toc.is_secure_env(),
        "Blizzard_CommunitiesSecure declares `## UseSecureEnvironment: 1`, so its mixins \
         (CommunitiesAddDialogMixin, CommunitiesCreateDialogMixin) and the Outbound bridge table \
         live in __secureenv"
    );
    let deps = toc.dependencies();
    for required in ["Blizzard_SharedXML"] {
        assert!(
            deps.contains(&required.to_string()),
            "Blizzard_CommunitiesSecure should declare `## RequiredDep: {required}`, got {deps:?}"
        );
    }
}

#[test]
fn blizzard_communities_secure_appears_in_game_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_CommunitiesSecure");
    assert!(
        in_game,
        "Blizzard_CommunitiesSecure (non-LOD, no AllowLoadGameType restriction) should appear in \
         Game-screen auto-discovery"
    );
}

prefork_full_ui_case! {
fn blizzard_communities_secure_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_CommunitiesSecure")
                || message.contains("CommunitiesAddDialog")
                || message.contains("CommunitiesAddDialogOutbound")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_CommunitiesSecure emitted Lua errors during load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_communities_secure_dialog_frames_are_defined(env: &WowLuaEnv) {

    let frames_present: bool = env
        .eval(
            "return type(_G.CommunitiesAddDialog) == 'table' \
                and CommunitiesAddDialog:GetParent() == UIParent \
                and type(_G.CommunitiesCreateDialog) == 'table' \
                and CommunitiesCreateDialog:GetParent() == UIParent",
        )
        .expect("toplevel frame query should succeed");
    assert!(
        frames_present,
        "Blizzard_CommunitiesSecure should define CommunitiesAddDialog and CommunitiesCreateDialog \
         (both `<Frame parent=\"UIParent\" frameStrata=\"DIALOG\" hidden=\"true\">` from \
         CommunitiesAddDialog.xml under a `<ScopedModifier forbidden=\"true\">` block)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_communities_secure_mixins_are_defined_in_secureenv(env: &WowLuaEnv) {

    let mixins_present: bool = env
        .eval(
            "local function lookup(name) \
               return _G[name] or (__secureenv and rawget(__secureenv, name)) \
             end; \
             local add = lookup('CommunitiesAddDialogMixin'); \
             local create = lookup('CommunitiesCreateDialogMixin'); \
             return type(add) == 'table' \
                and type(add.OnShow) == 'function' \
                and type(add.OnAttributeChanged) == 'function' \
                and type(add.OnHide) == 'function' \
                and type(create) == 'table' \
                and type(create.ClearText) == 'function' \
                and type(create.OnShow) == 'function' \
                and type(create.SetClubType) == 'function' \
                and type(create.GetClubType) == 'function' \
                and type(create.SetAvatarId) == 'function' \
                and type(create.GetAvatarId) == 'function' \
                and type(create.OnAttributeChanged) == 'function' \
                and type(create.OnHide) == 'function' \
                and type(create.CreateCommunity) == 'function' \
                and type(create.UpdateCreateButton) == 'function'",
        )
        .expect("mixin query should succeed");
    assert!(
        mixins_present,
        "CommunitiesAddDialogMixin (3 methods: OnShow/OnAttributeChanged/OnHide) and \
         CommunitiesCreateDialogMixin (10 methods: ClearText/OnShow/SetClubType/GetClubType/\
         SetAvatarId/GetAvatarId/OnAttributeChanged/OnHide/CreateCommunity/UpdateCreateButton) \
         should be defined in __secureenv after CommunitiesAddDialog.lua loads"
    );
}
}

prefork_full_ui_case! {
fn blizzard_communities_secure_outbound_bridge_is_in_secureenv(env: &WowLuaEnv) {

    let outbound_present: bool = env
        .eval(
            "local function lookup(name) \
               return _G[name] or (__secureenv and rawget(__secureenv, name)) \
             end; \
             local outbound = lookup('CommunitiesOutbound'); \
             return type(outbound) == 'table' \
                and type(outbound.ShowGameTooltip) == 'function' \
                and type(outbound.HideGameTooltip) == 'function' \
                and type(outbound.ShowAvatarPicker) == 'function' \
                and type(outbound.HideAvatarPicker) == 'function'",
        )
        .expect("CommunitiesOutbound query should succeed");
    assert!(
        outbound_present,
        "CommunitiesOutbound should be reachable through __secureenv with its four bridge \
         functions (ShowGameTooltip / HideGameTooltip / ShowAvatarPicker / HideAvatarPicker) — \
         CommunitiesAddDialogOutbound.lua creates the table in the global env via \
         SwapToGlobalEnvironment, then assigns `secureEnv.CommunitiesOutbound = CommunitiesOutbound`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_communities_secure_global_button_handlers_are_defined(env: &WowLuaEnv) {

    let handlers_present: bool = env
        .eval(
            "local function lookup(name) \
               return _G[name] or (__secureenv and rawget(__secureenv, name)) \
             end; \
             return type(lookup('CommunitiesAddDialogWoWButton_OnEnter')) == 'function' \
                and type(lookup('CommunitiesAddDialogWoWButton_OnLeave')) == 'function' \
                and type(lookup('CommunitiesAddDialogWoWButton_OnClick')) == 'function' \
                and type(lookup('CommunitiesAddDialogBattleNetButton_OnEnter')) == 'function' \
                and type(lookup('CommunitiesAddDialogBattleNetButton_OnClick')) == 'function' \
                and type(lookup('CommunitiesAddDialogJoinButton_OnClick')) == 'function' \
                and type(lookup('CommunitiesCreateDialogChangeAvatarButton_OnClick')) == 'function' \
                and type(lookup('CommunitiesCreateDialogCancelButton_OnClick')) == 'function' \
                and type(lookup('CommunitiesCreateDialogCreateButton_OnClick')) == 'function' \
                and type(lookup('CommunitiesCreateDialogCreateButton_OnEnter')) == 'function' \
                and type(lookup('CommunitiesCreateDialogCreateButton_OnLeave')) == 'function'",
        )
        .expect("button handler query should succeed");
    assert!(
        handlers_present,
        "Blizzard_CommunitiesSecure should define its 11 inline XML script handlers \
         (CommunitiesAddDialogWoWButton_OnEnter/_OnLeave/_OnClick, \
         CommunitiesAddDialogBattleNetButton_OnEnter/_OnClick, \
         CommunitiesAddDialogJoinButton_OnClick, \
         CommunitiesCreateDialogChangeAvatarButton_OnClick, \
         CommunitiesCreateDialogCancelButton_OnClick, \
         CommunitiesCreateDialogCreateButton_OnClick/_OnEnter/_OnLeave) — these run from \
         secure-env XML script tags, so they live in __secureenv"
    );
}
}

prefork_full_ui_case! {
fn blizzard_communities_secure_dialog_attribute_setshown_toggles_visibility(env: &WowLuaEnv) {

    let attr_routes_to_show: bool = env
        .eval(
            "CommunitiesAddDialog:Hide(); \
             local before = CommunitiesAddDialog:IsShown(); \
             CommunitiesAddDialog:SetAttribute('setshown', true); \
             local after_show = CommunitiesAddDialog:IsShown(); \
             CommunitiesAddDialog:SetAttribute('setshown', false); \
             local after_hide = CommunitiesAddDialog:IsShown(); \
             return (not before) and after_show and (not after_hide)",
        )
        .expect("setshown attribute round-trip should succeed");
    assert!(
        attr_routes_to_show,
        "CommunitiesAddDialogMixin:OnAttributeChanged should route the `setshown` attribute to \
         self:SetShown(value), so insecure callers can toggle the dialog's visibility through the \
         attribute bridge without touching the protected frame directly"
    );
}
}
