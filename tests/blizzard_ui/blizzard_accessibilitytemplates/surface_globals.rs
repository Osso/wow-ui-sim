//! Surface-level globals pinned by `Blizzard_AccessibilityTemplates`.
//!
//! Five tables are published at file scope when the addon's Lua bodies
//! run during the smoke-shape closure:
//!
//! | Global                  | Defining file (under `Blizzard_AccessibilityTemplates/`)        |
//! |-------------------------|------------------------------------------------------------------|
//! | `UIThemeContainerMixin` | `Mainline/AccessibilityTemplates.lua` (`= {}` at line 1)         |
//! | `QuestTextContrast`     | `QuestTextContrast.lua` (`= {}` at line 1, mainline-gated)       |
//! | `TextSizeManagerBase`   | `TextSizeManager.lua` (`= {}` at line 1)                         |
//! | `TextSizeManager`       | `TextSizeManagerGame.lua` (`= CreateFromMixins(...)`, game-only) |
//! | `UserScaledElementMixin`| `UserScaledElementTemplates.lua` (`= {}` at line 1)              |
//!
//! The harness loads the closure with `ScreenKind::Game`, which honours
//! both `[AllowLoadGameType mainline]` (QuestTextContrast) and
//! `[AllowLoad game]` (TextSizeManagerGame). If either gate were
//! mis-evaluated, `QuestTextContrast` or `TextSizeManager` would stay
//! nil and downstream consumers (`UIThemeManager`, font-size sliders)
//! would silently break — pinning the table type guards both the file
//! ordering AND the gate semantics with a single assertion per global.
//!
//! `QuestTextContrast.GetBackgroundAtlas(0..4)` is also pinned here
//! because the contrast → atlas mapping lives in a file-local table
//! (`questBackgroundAtlas`) that downstream consumers can only reach
//! through the `GetBackgroundAtlas` accessor. The five atlases drive
//! the parchment background swap on QuestFrame / GossipFrame, and a
//! regression in either the keys or the values would shift the rendered
//! background silently — the assertion list catches both classes of
//! regression by pinning value strings keyed on integer settings.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccessibilityTemplates";

const PUBLISHED_TABLES: &[&str] = &[
    "UIThemeContainerMixin",
    "QuestTextContrast",
    "TextSizeManagerBase",
    "TextSizeManager",
    "UserScaledElementMixin",
];

const BACKGROUND_ATLASES: &[(i32, &str)] = &[
    (0, "QuestBG-Parchment"),
    (1, "QuestBG-Parchment-Accessibility"),
    (2, "QuestBG-Parchment-Accessibility2"),
    (3, "QuestBG-Parchment-Accessibility3"),
    (4, "QuestBG-Parchment-Accessibility4"),
];

#[cfg(feature = "optional-tests")]
#[test]
fn accessibility_templates_publishes_expected_global_tables() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for global in PUBLISHED_TABLES {
            let probe = format!("return type(_G[{global:?}])");
            let actual_type: String = env
                .eval(&probe)
                .unwrap_or_else(|error| panic!("failed to probe `{global}` type: {error}"));
            assert_eq!(
                actual_type, "table",
                "Expected `{global}` to publish as a table after `{ROOT}` loads, got `{actual_type}`. \
                 If this regresses, check that the defining file (see module docs) actually ran — \
                 a load-gate mis-evaluation will leave the global at nil rather than at a different type."
            );
        }
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn quest_text_contrast_get_background_atlas_maps_settings_to_atlases() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (setting, expected_atlas) in BACKGROUND_ATLASES {
            let probe = format!("return QuestTextContrast.GetBackgroundAtlas({setting})");
            let actual_atlas: String = env.eval(&probe).unwrap_or_else(|error| {
                panic!("failed to call `QuestTextContrast.GetBackgroundAtlas({setting})`: {error}")
            });
            assert_eq!(
                &actual_atlas, expected_atlas,
                "Expected `QuestTextContrast.GetBackgroundAtlas({setting})` to return \
                 `{expected_atlas}` (per the file-local `questBackgroundAtlas` map in \
                 `QuestTextContrast.lua`), got `{actual_atlas}`. Downstream parchment \
                 background swaps on QuestFrame / GossipFrame route through this accessor."
            );
        }
    });
}
