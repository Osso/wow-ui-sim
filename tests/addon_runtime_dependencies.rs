//! Runtime and startup addon dependency behavior across Game and Glue screens.

use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;
use wow_ui_sim::loader::{
    StartupAddonLoadKind, discover_blizzard_startup_addons_for_screen, load_startup_addon,
};
use wow_ui_sim::lua_api::{AddonInfo, WowLuaEnv};
use wow_ui_sim::screen::ScreenKind;

struct AddonTree {
    root: TempDir,
    env: WowLuaEnv,
}

impl AddonTree {
    fn new(screen: ScreenKind) -> Self {
        let root = tempfile::tempdir().unwrap();
        let env = WowLuaEnv::new().unwrap();
        env.state().borrow_mut().addon_base_paths = vec![root.path().to_path_buf()];
        env.set_screen_mode(screen);
        Self { root, env }
    }

    fn add(&self, name: &str, headers: &str, source: &str) -> PathBuf {
        let addon_dir = self.root.path().join(name);
        fs::create_dir_all(&addon_dir).unwrap();
        let toc_path = addon_dir.join(format!("{name}.toc"));
        fs::write(
            &toc_path,
            format!("## Title: {name}\n{headers}\n{name}.lua\n"),
        )
        .unwrap();
        fs::write(addon_dir.join(format!("{name}.lua")), source).unwrap();
        toc_path
    }

    fn load(&self, name: &str) -> (bool, Option<String>) {
        self.env
            .eval(&format!("return C_AddOns.LoadAddOn({name:?})"))
            .unwrap()
    }

    fn ran(&self, name: &str) -> bool {
        self.env.eval(&format!("return {name} == true")).unwrap()
    }

    fn register_enabled(&self, name: &str) {
        self.env.register_addon(AddonInfo {
            folder_name: name.to_owned(),
            title: name.to_owned(),
            enabled: true,
            ..Default::default()
        });
    }

    fn load_startup_addons(&self, screen: ScreenKind) -> Vec<String> {
        let addons = discover_blizzard_startup_addons_for_screen(self.root.path(), screen);
        let names = addons.iter().map(|addon| addon.name.clone()).collect();
        for addon in addons {
            load_startup_addon(&self.env.loader_env(), &addon.toc_path, addon.kind, None)
                .unwrap_or_else(|error| panic!("load startup {}: {error}", addon.name));
        }
        names
    }
}

#[test]
fn game_load_of_required_glue_dependency_reports_wrong_interface() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.add(
        "GameRuntimeProbe",
        "## RequiredDeps: Blizzard_GlueOnlyProbe",
        "GameRuntimeProbeRan = true",
    );
    tree.add(
        "Blizzard_GlueOnlyProbe",
        "## AllowLoad: Glue",
        "GlueOnlyProbeRan = true",
    );

    let (loaded, reason) = tree.load("GameRuntimeProbe");

    assert!(
        !loaded,
        "the parent must not load across a required screen mismatch"
    );
    assert_eq!(reason.as_deref(), Some("DEP_WRONG_ACTIVE_INTERFACE"));
    assert!(!tree.ran("GameRuntimeProbeRan"));
    assert!(!tree.ran("GlueOnlyProbeRan"));
}

#[test]
fn addon_queries_ignore_screen_while_required_load_fails() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.add(
        "GameQueryProbe",
        "## RequiredDeps: Blizzard_GlueQueryProbe",
        "GameQueryProbeRan = true",
    );
    tree.add(
        "Blizzard_GlueQueryProbe",
        "## AllowLoad: Glue",
        "GlueQueryProbeRan = true",
    );
    tree.register_enabled("GameQueryProbe");

    let (info_name, _, _, info_loadable, info_reason): (
        String,
        String,
        Option<String>,
        bool,
        Option<String>,
    ) = tree
        .env
        .eval("return C_AddOns.GetAddOnInfo('GameQueryProbe')")
        .unwrap();
    let (parent_loadable, parent_reason): (bool, Option<String>) = tree
        .env
        .eval("return C_AddOns.IsAddOnLoadable('GameQueryProbe')")
        .unwrap();
    let (target_loadable, target_reason): (bool, Option<String>) = tree
        .env
        .eval("return C_AddOns.IsAddOnLoadable('Blizzard_GlueQueryProbe')")
        .unwrap();

    assert_eq!(info_name, "GameQueryProbe");
    assert!(info_loadable);
    assert_eq!(info_reason, None);
    assert!(
        parent_loadable,
        "queries do not recursively validate dependencies"
    );
    assert_eq!(parent_reason, None);
    assert!(target_loadable, "query mode skips the active-screen test");
    assert_eq!(target_reason, None);

    let (loaded, reason) = tree.load("GameQueryProbe");
    assert!(!loaded);
    assert_eq!(reason.as_deref(), Some("DEP_WRONG_ACTIVE_INTERFACE"));
}

#[test]
fn direct_load_of_glue_only_addon_reports_wrong_interface() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.add(
        "Blizzard_DirectGlueProbe",
        "## AllowLoad: Glue",
        "DirectGlueProbeRan = true",
    );

    let (loaded, reason) = tree.load("Blizzard_DirectGlueProbe");

    assert!(!loaded);
    assert_eq!(reason.as_deref(), Some("WRONG_ACTIVE_INTERFACE"));
    assert!(!tree.ran("DirectGlueProbeRan"));
}

#[test]
fn both_required_dependency_loads_on_game_and_glue_and_stays_required() {
    for screen in [ScreenKind::Game, ScreenKind::Login] {
        let tree = AddonTree::new(screen);
        tree.add(
            "Blizzard_BothParentProbe",
            "## AllowLoad: Both\n## RequiredDeps: Blizzard_BothDependencyProbe",
            "BothParentProbeRan = true",
        );
        tree.add(
            "Blizzard_BothDependencyProbe",
            "## AllowLoad: Both",
            "BothDependencyProbeRan = true",
        );

        assert_eq!(tree.load("Blizzard_BothParentProbe"), (true, None));
        assert!(tree.ran("BothParentProbeRan"));
        assert!(tree.ran("BothDependencyProbeRan"));
    }

    let tree = AddonTree::new(ScreenKind::Game);
    tree.add(
        "Blizzard_DisabledBothParentProbe",
        "## RequiredDeps: Blizzard_DisabledBothProbe",
        "DisabledBothParentProbeRan = true",
    );
    tree.add(
        "Blizzard_DisabledBothProbe",
        "## AllowLoad: Both",
        "DisabledBothProbeRan = true",
    );
    tree.env.register_addon(AddonInfo {
        folder_name: "Blizzard_DisabledBothProbe".to_owned(),
        enabled: false,
        ..Default::default()
    });

    let (loaded, reason) = tree.load("Blizzard_DisabledBothParentProbe");
    assert!(!loaded);
    assert_eq!(reason.as_deref(), Some("DEP_DISABLED"));
    assert!(!tree.ran("DisabledBothParentProbeRan"));
}

#[test]
fn glue_dependency_annotation_filters_the_whole_startup_line() {
    for (screen, expect_dependencies) in [(ScreenKind::Game, false), (ScreenKind::Login, true)] {
        let tree = AddonTree::new(screen);
        tree.add(
            "Blizzard_LineConditionParentProbe",
            "## AllowLoad: Both\n## RequiredDeps: Blizzard_LineConditionA, Blizzard_LineConditionB [AllowLoad glue]",
            "LineConditionParentRan = true",
        );
        for name in ["Blizzard_LineConditionA", "Blizzard_LineConditionB"] {
            tree.add(
                name,
                "## AllowLoad: Both\n## LoadOnDemand: 1",
                &format!("{name}Ran = true"),
            );
        }

        let loaded = tree.load_startup_addons(screen);
        assert!(loaded.contains(&"Blizzard_LineConditionParentProbe".to_owned()));
        assert_eq!(tree.ran("Blizzard_LineConditionARan"), expect_dependencies);
        assert_eq!(tree.ran("Blizzard_LineConditionBRan"), expect_dependencies);
    }
}

#[test]
fn startup_rejects_an_unannotated_required_dependency_for_another_screen() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.add(
        "Blizzard_StartupGameProbe",
        "## Dependencies: Blizzard_StartupGlueOnlyProbe",
        "StartupGameProbeRan = true",
    );
    tree.add(
        "Blizzard_StartupGlueOnlyProbe",
        "## AllowLoad: Glue",
        "StartupGlueOnlyProbeRan = true",
    );

    let addons = discover_blizzard_startup_addons_for_screen(tree.root.path(), ScreenKind::Game);
    let parent = addons
        .iter()
        .find(|addon| addon.name == "Blizzard_StartupGameProbe")
        .expect("the Game parent is a startup addon");
    let result = load_startup_addon(
        &tree.env.loader_env(),
        &parent.toc_path,
        StartupAddonLoadKind::Full,
        None,
    );

    assert_eq!(
        result.unwrap_err().to_string(),
        "DEP_WRONG_ACTIVE_INTERFACE"
    );
    assert!(!tree.ran("StartupGameProbeRan"));
    assert!(!tree.ran("StartupGlueOnlyProbeRan"));
}

#[test]
fn reverse_required_dependency_fails_unless_game_line_is_excluded_in_glue() {
    let tree = AddonTree::new(ScreenKind::Login);
    tree.add(
        "Blizzard_GlueParentProbe",
        "## AllowLoad: Glue\n## RequiredDeps: Blizzard_GameOnlyProbe",
        "GlueParentProbeRan = true",
    );
    tree.add(
        "Blizzard_GameOnlyProbe",
        "## AllowLoad: Game",
        "GameOnlyProbeRan = true",
    );

    let (loaded, reason) = tree.load("Blizzard_GlueParentProbe");
    assert!(!loaded);
    assert_eq!(reason.as_deref(), Some("DEP_WRONG_ACTIVE_INTERFACE"));
    assert!(!tree.ran("GlueParentProbeRan"));

    let tree = AddonTree::new(ScreenKind::Login);
    tree.add(
        "Blizzard_GlueGameAnnotatedProbe",
        "## AllowLoad: Glue\n## RequiredDeps: Blizzard_GameOnlyAnnotatedProbe [AllowLoad game]",
        "GlueGameAnnotatedProbeRan = true",
    );
    tree.add(
        "Blizzard_GameOnlyAnnotatedProbe",
        "## AllowLoad: Game",
        "GameOnlyAnnotatedProbeRan = true",
    );
    assert_eq!(tree.load("Blizzard_GlueGameAnnotatedProbe"), (true, None));
    assert!(tree.ran("GlueGameAnnotatedProbeRan"));
    assert!(!tree.ran("GameOnlyAnnotatedProbeRan"));

    let tree = AddonTree::new(ScreenKind::Login);
    tree.add(
        "Blizzard_GlueGlueAnnotatedProbe",
        "## AllowLoad: Glue\n## RequiredDeps: Blizzard_GameOnlyGlueAnnotatedProbe [AllowLoad glue]",
        "GlueGlueAnnotatedProbeRan = true",
    );
    tree.add(
        "Blizzard_GameOnlyGlueAnnotatedProbe",
        "## AllowLoad: Game",
        "GameOnlyGlueAnnotatedProbeRan = true",
    );
    let (loaded, reason) = tree.load("Blizzard_GlueGlueAnnotatedProbe");
    assert!(!loaded);
    assert_eq!(reason.as_deref(), Some("DEP_WRONG_ACTIVE_INTERFACE"));
    assert!(!tree.ran("GlueGlueAnnotatedProbeRan"));
}

#[test]
fn optional_dependency_failure_does_not_fail_parent_load() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.add(
        "OptionalFailureParentProbe",
        "## OptionalDeps: Blizzard_OptionalFailureProbe",
        "OptionalFailureParentRan = true",
    );
    tree.add(
        "Blizzard_OptionalFailureProbe",
        "## AllowLoad: Both\n## RequiredDeps: Blizzard_OptionalGlueOnlyProbe",
        "OptionalFailureProbeRan = true",
    );
    tree.add(
        "Blizzard_OptionalGlueOnlyProbe",
        "## AllowLoad: Glue",
        "OptionalGlueOnlyProbeRan = true",
    );

    assert_eq!(tree.load("OptionalFailureParentProbe"), (true, None));
    assert!(tree.ran("OptionalFailureParentRan"));
    assert!(!tree.ran("OptionalFailureProbeRan"));
    assert!(!tree.ran("OptionalGlueOnlyProbeRan"));
}

#[test]
fn startup_optional_dependency_loads_before_required_dependency() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.env.exec("startupDependencyOrder = {}").unwrap();
    tree.add(
        "Blizzard_A_ParentProbe",
        "## AllowLoad: Both\n## Dependencies: Blizzard_B_RequiredProbe\n## OptionalDeps: Blizzard_Z_OptionalProbe",
        "table.insert(startupDependencyOrder, 'parent')",
    );
    tree.add(
        "Blizzard_B_RequiredProbe",
        "## AllowLoad: Both",
        "table.insert(startupDependencyOrder, 'required')",
    );
    tree.add(
        "Blizzard_Z_OptionalProbe",
        "## AllowLoad: Both",
        "table.insert(startupDependencyOrder, 'optional')",
    );

    let addons = tree.load_startup_addons(ScreenKind::Game);
    assert_eq!(
        addons.iter().map(String::as_str).collect::<Vec<_>>(),
        [
            "Blizzard_Z_OptionalProbe",
            "Blizzard_B_RequiredProbe",
            "Blizzard_A_ParentProbe",
        ]
    );
    tree.env
        .exec("assert(table.concat(startupDependencyOrder, ',') == 'optional,required,parent')")
        .unwrap();
}

#[test]
fn load_managers_run_before_their_startup_owner_files() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.env.exec("startupManagerOrder = {}").unwrap();
    tree.add(
        "Blizzard_A_ManagerOwnerProbe",
        "## AllowLoad: Both\n## LoadManagers: Blizzard_Z_LodManagerProbe",
        "table.insert(startupManagerOrder, 'owner')",
    );
    tree.add(
        "Blizzard_Z_LodManagerProbe",
        "## AllowLoad: Both\n## LoadOnDemand: 1",
        "table.insert(startupManagerOrder, 'manager')",
    );

    let addons = tree.load_startup_addons(ScreenKind::Game);
    let manager_position = addons
        .iter()
        .position(|name| name == "Blizzard_Z_LodManagerProbe")
        .expect("LoadManagers entry must be in the startup sequence");
    let owner_position = addons
        .iter()
        .position(|name| name == "Blizzard_A_ManagerOwnerProbe")
        .expect("owner must be in the startup sequence");
    assert!(manager_position < owner_position);
    tree.env
        .exec("assert(table.concat(startupManagerOrder, ',') == 'manager,owner')")
        .unwrap();
}

#[test]
fn startup_load_with_follower_runs_after_target_files() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.env.exec("startupLoadWithOrder = {}").unwrap();
    tree.add(
        "Blizzard_Z_LoadWithTargetProbe",
        "## AllowLoad: Both",
        "table.insert(startupLoadWithOrder, 'target')",
    );
    tree.add(
        "Blizzard_A_LoadWithFollowerProbe",
        "## AllowLoad: Both\n## LoadOnDemand: 1\n## LoadWith: Blizzard_Z_LoadWithTargetProbe",
        "table.insert(startupLoadWithOrder, 'follower')",
    );

    tree.load_startup_addons(ScreenKind::Game);

    tree.env
        .exec("assert(table.concat(startupLoadWithOrder, ',') == 'target,follower')")
        .unwrap();
}

/// Preservation: explicitly loading a `LoadWith` follower does not submit its target.
#[test]
fn loading_load_with_follower_does_not_load_target() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.add(
        "Blizzard_LoadWithTargetProbe",
        "## AllowLoad: Both\n## LoadOnDemand: 1",
        "LoadWithTargetRan = true",
    );
    tree.add(
        "Blizzard_LoadWithFollowerProbe",
        "## AllowLoad: Both\n## LoadOnDemand: 1\n## LoadWith: Blizzard_LoadWithTargetProbe",
        "LoadWithFollowerRan = true",
    );

    assert_eq!(tree.load("Blizzard_LoadWithFollowerProbe"), (true, None));
    assert!(tree.ran("LoadWithFollowerRan"));
    assert!(!tree.ran("LoadWithTargetRan"));
}

#[test]
fn runtime_loading_target_runs_load_with_follower_after_target_files() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.env.exec("runtimeLoadWithOrder = {}").unwrap();
    tree.add(
        "Blizzard_RuntimeLoadWithTargetProbe",
        "## AllowLoad: Both\n## LoadOnDemand: 1",
        "table.insert(runtimeLoadWithOrder, 'target')",
    );
    tree.add(
        "Blizzard_RuntimeLoadWithFollowerProbe",
        "## AllowLoad: Both\n## LoadOnDemand: 1\n## LoadWith: Blizzard_RuntimeLoadWithTargetProbe\n## OptionalDeps: RuntimeLoadWithOptionalProbe\n## RequiredDeps: RuntimeLoadWithRequiredProbe",
        "table.insert(runtimeLoadWithOrder, 'follower')",
    );
    tree.add(
        "RuntimeLoadWithOptionalProbe",
        "",
        "table.insert(runtimeLoadWithOrder, 'optional')",
    );
    tree.add(
        "RuntimeLoadWithRequiredProbe",
        "",
        "table.insert(runtimeLoadWithOrder, 'required')",
    );

    assert_eq!(
        tree.load("Blizzard_RuntimeLoadWithTargetProbe"),
        (true, None)
    );
    tree.env
        .exec("assert(table.concat(runtimeLoadWithOrder, ',') == 'target,optional,required,follower')")
        .unwrap();
}

/// Preservation: a target that exits during dependency checks submits no followers.
#[test]
fn runtime_target_early_exit_does_not_load_load_with_followers() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.add(
        "Blizzard_EarlyExitLoadWithTargetProbe",
        "## AllowLoad: Both\n## LoadOnDemand: 1\n## RequiredDeps: Blizzard_GlueOnlyLoadWithDependencyProbe",
        "EarlyExitLoadWithTargetRan = true",
    );
    tree.add(
        "Blizzard_GlueOnlyLoadWithDependencyProbe",
        "## AllowLoad: Glue",
        "GlueOnlyLoadWithDependencyRan = true",
    );
    tree.add(
        "Blizzard_EarlyExitLoadWithFollowerProbe",
        "## AllowLoad: Both\n## LoadOnDemand: 1\n## LoadWith: Blizzard_EarlyExitLoadWithTargetProbe",
        "EarlyExitLoadWithFollowerRan = true",
    );

    let (loaded, reason) = tree.load("Blizzard_EarlyExitLoadWithTargetProbe");

    assert!(!loaded);
    assert_eq!(reason.as_deref(), Some("DEP_WRONG_ACTIVE_INTERFACE"));
    assert!(!tree.ran("EarlyExitLoadWithTargetRan"));
    assert!(!tree.ran("EarlyExitLoadWithFollowerRan"));
}

#[test]
fn insecure_toc_allowload_keeps_the_game_default() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.add(
        "UserAllowLoadProbe",
        "## AllowLoad: Glue",
        "UserAllowLoadProbeRan = true",
    );
    assert_eq!(tree.load("UserAllowLoadProbe"), (true, None));
    assert!(tree.ran("UserAllowLoadProbeRan"));

    let tree = AddonTree::new(ScreenKind::Login);
    tree.add(
        "UserAllowLoadProbe",
        "## AllowLoad: Glue",
        "UserAllowLoadProbeRan = true",
    );
    tree.register_enabled("UserAllowLoadProbe");
    let (loadable, reason): (bool, Option<String>) = tree
        .env
        .eval("return C_AddOns.IsAddOnLoadable('UserAllowLoadProbe')")
        .unwrap();
    assert!(loadable, "loadability queries skip the screen test");
    assert_eq!(reason, None);
    let (loaded, reason) = tree.load("UserAllowLoadProbe");
    assert!(!loaded);
    assert_eq!(reason.as_deref(), Some("WRONG_ACTIVE_INTERFACE"));
}

#[test]
fn allowload_header_condition_runs_before_directive_dispatch() {
    let tree = AddonTree::new(ScreenKind::Game);
    tree.add(
        "Blizzard_ConditionalAllowLoadProbe",
        "## AllowLoad: Both [AllowLoad game]",
        "ConditionalAllowLoadProbeRan = true",
    );
    assert_eq!(
        tree.load("Blizzard_ConditionalAllowLoadProbe"),
        (true, None)
    );
    assert!(tree.ran("ConditionalAllowLoadProbeRan"));

    let tree = AddonTree::new(ScreenKind::Login);
    tree.add(
        "Blizzard_ConditionalAllowLoadProbe",
        "## AllowLoad: Both [AllowLoad game]",
        "ConditionalAllowLoadProbeRan = true",
    );
    let (loaded, reason) = tree.load("Blizzard_ConditionalAllowLoadProbe");
    assert!(!loaded);
    assert_eq!(reason.as_deref(), Some("WRONG_ACTIVE_INTERFACE"));
    assert!(!tree.ran("ConditionalAllowLoadProbeRan"));
}

#[test]
fn invalid_allowload_dependency_conditions_discard_the_line_on_both_screens() {
    for condition in ["nonsense", "glue game"] {
        for screen in [ScreenKind::Game, ScreenKind::Login] {
            let tree = AddonTree::new(screen);
            tree.add(
                "Blizzard_InvalidConditionProbe",
                &format!(
                    "## AllowLoad: Both\n## RequiredDeps: MissingConditionA, MissingConditionB [AllowLoad {condition}]"
                ),
                "InvalidConditionProbeRan = true",
            );
            assert_eq!(tree.load("Blizzard_InvalidConditionProbe"), (true, None));
            assert!(tree.ran("InvalidConditionProbeRan"));
        }
    }
}

#[test]
fn required_dependency_names_split_only_on_ascii_space_and_comma() {
    let tree = AddonTree::new(ScreenKind::Game);
    let dependency_name = "Tab\tSeparatedProbe";
    tree.add(
        "GameTabDependencyParent",
        &format!("## RequiredDeps: {dependency_name}"),
        "GameTabDependencyParentRan = true",
    );
    tree.add(dependency_name, "", "TabSeparatedProbeRan = true");

    assert_eq!(tree.load("GameTabDependencyParent"), (true, None));
    assert!(tree.ran("GameTabDependencyParentRan"));
    assert!(tree.ran("TabSeparatedProbeRan"));
}
