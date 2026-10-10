//! Source-backed classic cast colors; native client RGBA parity remains unproven.
#![cfg(feature = "client-mists")]

use wow_ui_sim::loader::{discover_blizzard_startup_addons_for_screen, load_startup_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

#[test]
fn mists_player_casting_bar_source_callback_uses_classic_colors() {
    crate::common::with_timeout(90, || {
        let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
            .expect("resolve Mists Blizzard UI cache");
        assert!(
            ui.ends_with("mists/AddOns"),
            "wrong cache: {}",
            ui.display()
        );
        let env = WowLuaEnv::new().expect("initialize Mists environment");
        env.set_screen_size(1024.0, 768.0);
        env.set_screen_mode(ScreenKind::Game);
        env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
        wow_ui_sim::xml::register_intrinsic_templates();
        for addon in discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game) {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
                .unwrap_or_else(|error| panic!("load {}: {error}", addon.name));
        }
        env.apply_post_load_workarounds();
        fire_startup_events_for_screen(&env, ScreenKind::Game);
        env.exec(
            r#"
            assert(PlayerCastingBarFrame:GetObjectType() == "StatusBar")
            assert(PlayerCastingBarFrame.classicStyleCastBar == true)
            assert(PlayerCastingBarFrame.UpdateBarFillTexture == CastingBarMixin.UpdateBarFillTexture)
            assert(type(PlayerCastingBarFrame:GetScript("OnEvent")) == "function")
            "#,
        )
        .expect("real XML-created classic PlayerCastingBarFrame and source callback");

        // Literal current CastingBarTypeInfo mappings, GlobalColor rows 395–398.
        for (bar_type, is_full, expected) in [
            ("Standard", false, (1.0, 179.0 / 255.0, 0.0)),
            ("Standard", true, (0.0, 1.0, 0.0)),
            ("Channel", false, (0.0, 1.0, 0.0)),
            (
                "Uninterruptable",
                false,
                (179.0 / 255.0, 179.0 / 255.0, 179.0 / 255.0),
            ),
            ("Interrupted", false, (1.0, 0.0, 0.0)),
            ("Interrupted", true, (1.0, 0.0, 0.0)),
        ] {
            let actual: (f64, f64, f64) = env
                .eval(&format!(
                    r#"
                    PlayerCastingBarFrame.barType = CastingBarType.{bar_type}
                    PlayerCastingBarFrame:UpdateBarFillTexture({is_full})
                    local r, g, b = PlayerCastingBarFrame:GetStatusBarColor()
                    return r, g, b
                    "#,
                ))
                .unwrap_or_else(|error| {
                    panic!("source UpdateBarFillTexture({bar_type}, full={is_full}): {error}")
                });
            for (actual, expected) in [
                (actual.0, expected.0),
                (actual.1, expected.1),
                (actual.2, expected.2),
            ] {
                assert!(
                    (actual - expected).abs() < 1e-7,
                    "{bar_type}, full={is_full}: {actual} != {expected}"
                );
            }
        }
    });
}

#[test]
fn mists_named_cast_colors_preserve_literal_argb_channels() {
    let env = WowLuaEnv::new().expect("initialize Mists environment");
    for (name, expected) in [
        ("CASTBAR_CLASSIC_YELLOW", (1.0, 179.0 / 255.0, 0.0)),
        ("CASTBAR_CLASSIC_GREEN", (0.0, 1.0, 0.0)),
        (
            "CASTBAR_CLASSIC_GRAY",
            (179.0 / 255.0, 179.0 / 255.0, 179.0 / 255.0),
        ),
        ("CASTBAR_CLASSIC_RED", (1.0, 0.0, 0.0)),
    ] {
        let rgb: (f64, f64, f64) = env
            .eval(&format!("return {name}:GetRGB()"))
            .unwrap_or_else(|error| panic!("{name}:GetRGB(): {error}"));
        assert_eq!(rgb, expected, "{name} integer-normalized RGB");
        let alpha: f64 = env
            .eval(&format!("local r, g, b, a = {name}:GetRGBA(); return a"))
            .expect("read literal encoded alpha, not native parity");
        assert_eq!(alpha, 0.0, "{name} encoded ARGB alpha");
    }
}
