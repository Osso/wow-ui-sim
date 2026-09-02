//! Load smoke for `Blizzard_ArchaeologyUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use crate::common::panel_fixtures::{blizzard_ui_dir, recorded_lua_errors};
use wow_ui_sim::toc::TocFile;

const ROOT: &str = "Blizzard_ArchaeologyUI";
const ROOT_TOC_FILE: &str = "Blizzard_ArchaeologyUI_Mainline.toc";
const DECLARED_DEPENDENCIES: &[&str] = &["Blizzard_FrameXMLUtil", "Blizzard_HelpPlate"];

#[test]
fn archaeology_ui_loads_cleanly_with_no_recorded_lua_errors() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, loaded| {
        assert!(
            loaded.iter().any(|name| name == ROOT),
            "{ROOT} must be present in its dependency closure. Loaded set: {loaded:?}"
        );

        let errors = recorded_lua_errors(env);
        assert!(
            errors.is_empty(),
            "{ROOT} must settle without recorded Lua errors after startup events:\n  {}",
            errors.join("\n  ")
        );
    });
}

#[test]
fn archaeology_ui_declared_toc_dependencies_are_loaded_after_harness_runs() {
    let declared_dependencies = parse_declared_dependencies();

    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, loaded| {
        for dependency in &declared_dependencies {
            assert!(
                loaded.iter().any(|name| name == dependency),
                "{ROOT} dependency closure must include declared TOC dependency `{dependency}`. \
                 Loaded set: {loaded:?}"
            );

            let is_loaded = env
                .eval::<bool>(&format!(
                    r#"return C_AddOns.IsAddOnLoaded("{dependency}") == true"#
                ))
                .expect("C_AddOns.IsAddOnLoaded dependency probe must run cleanly");
            assert!(
                is_loaded,
                "{ROOT} declared TOC dependency `{dependency}` must be marked loaded after the \
                 startup-shape harness runs"
            );
        }
    });
}

fn parse_declared_dependencies() -> Vec<String> {
    let toc_path = blizzard_ui_dir().join(ROOT).join(ROOT_TOC_FILE);
    let toc = TocFile::from_file(&toc_path).unwrap_or_else(|error| {
        panic!(
            "TOC at `{}` must parse cleanly before dependency loading can be checked: {error}",
            toc_path.display()
        )
    });

    let dependencies = toc.dependencies();
    let expected = DECLARED_DEPENDENCIES
        .iter()
        .map(|dependency| (*dependency).to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        dependencies, expected,
        "{ROOT_TOC_FILE} declared dependencies must match this load contract"
    );

    dependencies
}
