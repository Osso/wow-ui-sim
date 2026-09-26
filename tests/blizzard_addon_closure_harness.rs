#![cfg(feature = "client-retail")]
mod common;

use std::fs;
use std::path::Path;
use wow_ui_sim::loader::{BlizzardAddonOverride, discover_blizzard_addon_closure_for_screen};
use wow_ui_sim::screen::ScreenKind;

struct TempBlizzardUiDir {
    tempdir: tempfile::TempDir,
}

impl TempBlizzardUiDir {
    fn new(prefix: &str) -> Self {
        let tempdir = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir()
            .expect("failed to create temp dir");
        Self { tempdir }
    }

    fn add_addon(&self, addon_name: &str, toc: &str) {
        let addon_dir = self.path().join(addon_name);
        fs::create_dir_all(&addon_dir).expect("failed to create addon dir");
        fs::write(addon_dir.join(format!("{addon_name}.toc")), toc)
            .expect("failed to write addon toc");
        fs::write(addon_dir.join("Core.lua"), "return true\n").expect("failed to write addon lua");
    }

    fn path(&self) -> &Path {
        self.tempdir.path().as_ref()
    }
}

fn addon_names_for_closure(ui: &TempBlizzardUiDir, roots: &[&str]) -> Vec<String> {
    discover_blizzard_addon_closure_for_screen(ui.path(), ScreenKind::Game, roots)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

fn runtime_loaded_addons(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    addon_names: &[String],
) -> Vec<String> {
    addon_names
        .iter()
        .filter(|addon_name| {
            env.eval::<bool>(&format!("return C_AddOns.IsAddOnLoaded({addon_name:?})"))
                .expect("addon load probe should return")
        })
        .cloned()
        .collect()
}

#[test]
fn blizzard_addon_closure_harness_runs_assertions_after_loading_the_closure() {
    let ui = TempBlizzardUiDir::new("closure-harness");
    ui.add_addon(
        "Blizzard_B",
        r#"
## Title: Blizzard_B
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_C",
        r#"
## Title: Blizzard_C
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_A",
        r#"
## Title: Blizzard_A
## AllowLoad: Both
## Dependencies: Blizzard_B
Core.lua
"#,
    );

    let overrides = &[BlizzardAddonOverride {
        addon: "Blizzard_A",
        extra_roots: &["Blizzard_C"],
    }];

    common::blizzard_addon_harness::with_blizzard_addon_closure_in_dir(
        ui.path(),
        &["Blizzard_A"],
        overrides,
        |env, loaded| {
            assert_eq!(
                loaded,
                ["Blizzard_B", "Blizzard_C", "Blizzard_A"],
                "closure harness should load dependencies and override roots before the target addon",
            );

            let loaded_a: bool = env
                .eval(r#"return C_AddOns.IsAddOnLoaded("Blizzard_A")"#)
                .expect("addon load probe should return");
            let loaded_b: bool = env
                .eval(r#"return C_AddOns.IsAddOnLoaded("Blizzard_B")"#)
                .expect("dependency load probe should return");
            let loaded_c: bool = env
                .eval(r#"return C_AddOns.IsAddOnLoaded("Blizzard_C")"#)
                .expect("override root load probe should return");

            assert!(loaded_a);
            assert!(loaded_b);
            assert!(loaded_c);

            let result: String = env
                .eval(
                    r#"
                    local loaded = {}
                    for _, name in ipairs({"Blizzard_A", "Blizzard_B", "Blizzard_C"}) do
                        loaded[#loaded + 1] = tostring(C_AddOns.IsAddOnLoaded(name))
                    end
                    return table.concat(loaded, ",")
                    "#,
                )
                .expect("lua assertion probe should return");
            assert_eq!(result, "true,true,true");
        },
    );
}

#[test]
fn runtime_load_addon_matches_lod_closure_for_direct_dependencies() {
    let ui = TempBlizzardUiDir::new("runtime-lod-closure-direct");
    ui.add_addon(
        "Blizzard_Dependency",
        r#"
## Title: Blizzard_Dependency
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_Optional",
        r#"
## Title: Blizzard_Optional
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_GlueOnlyOptional",
        r#"
## Title: Blizzard_GlueOnlyOptional
## AllowLoad: Glue
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_LoadOnDemandRoot",
        r#"
## Title: Blizzard_LoadOnDemandRoot
## AllowLoad: Both
## LoadOnDemand: 1
## Dependencies: Blizzard_Dependency
## OptionalDeps: Blizzard_Optional, Blizzard_GlueOnlyOptional
Core.lua
"#,
    );

    let expected = addon_names_for_closure(&ui, &["Blizzard_LoadOnDemandRoot"]);
    assert_eq!(
        expected,
        vec![
            "Blizzard_Dependency".to_string(),
            "Blizzard_Optional".to_string(),
            "Blizzard_LoadOnDemandRoot".to_string(),
        ],
        "dep-graph closure should include game-screen deps/optional deps and exclude glue-only addons",
    );

    let env = common::blizzard_addon_harness::new_blizzard_addon_env(ui.path());
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_LoadOnDemandRoot")"#)
        .expect("LoadAddOn should return");
    assert!(
        loaded,
        "LoadAddOn should succeed for the direct LoD closure test: {reason:?}"
    );

    let mut probe_names = expected.clone();
    probe_names.push("Blizzard_GlueOnlyOptional".to_string());
    let actual_loaded = runtime_loaded_addons(&env, &probe_names);
    assert_eq!(
        actual_loaded, expected,
        "runtime LoadAddOn should load the same addon set as the dep-graph closure for a direct LoD root",
    );
}

#[test]
fn runtime_load_addon_matches_lod_closure_for_transitive_dependencies() {
    let ui = TempBlizzardUiDir::new("runtime-lod-closure-transitive");
    ui.add_addon(
        "Blizzard_BaseDependency",
        r#"
## Title: Blizzard_BaseDependency
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_TransitiveOptional",
        r#"
## Title: Blizzard_TransitiveOptional
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_GlueOnlyOptional",
        r#"
## Title: Blizzard_GlueOnlyOptional
## AllowLoad: Glue
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_IntermediateDependency",
        r#"
## Title: Blizzard_IntermediateDependency
## AllowLoad: Both
## LoadOnDemand: 1
## Dependencies: Blizzard_BaseDependency
## OptionalDeps: Blizzard_TransitiveOptional, Blizzard_GlueOnlyOptional
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_LoadOnDemandRoot",
        r#"
## Title: Blizzard_LoadOnDemandRoot
## AllowLoad: Both
## LoadOnDemand: 1
## Dependencies: Blizzard_IntermediateDependency
Core.lua
"#,
    );

    let expected = addon_names_for_closure(&ui, &["Blizzard_LoadOnDemandRoot"]);
    assert_eq!(
        expected,
        vec![
            "Blizzard_BaseDependency".to_string(),
            "Blizzard_TransitiveOptional".to_string(),
            "Blizzard_IntermediateDependency".to_string(),
            "Blizzard_LoadOnDemandRoot".to_string(),
        ],
        "dep-graph closure should walk transitive LoD dependencies and keep screen-filtered addons out",
    );

    let env = common::blizzard_addon_harness::new_blizzard_addon_env(ui.path());
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_LoadOnDemandRoot")"#)
        .expect("LoadAddOn should return");
    assert!(
        loaded,
        "LoadAddOn should succeed for the transitive LoD closure test: {reason:?}"
    );

    let mut probe_names = expected.clone();
    probe_names.push("Blizzard_GlueOnlyOptional".to_string());
    let actual_loaded = runtime_loaded_addons(&env, &probe_names);
    assert_eq!(
        actual_loaded, expected,
        "runtime LoadAddOn should load the same addon set as the dep-graph closure for a transitive LoD root",
    );
}
