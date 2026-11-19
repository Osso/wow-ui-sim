#![cfg(all(feature = "client-wowforever", feature = "optional-tests"))]

use wow_ui_sim::{lua_api::WowLuaEnv, screen::ScreenKind, startup::settle_headless_startup};

fn load_tooltip_screen(screen: ScreenKind) -> WowLuaEnv {
    use wow_ui_sim::loader::{discover_all_blizzard_addons, register_load_with_addons};
    let env = WowLuaEnv::new().expect("create tooltip startup environment");
    let ui = crate::common::blizzard_ui_dir();
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(screen);
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    if screen == ScreenKind::Game {
        crate::common::supply_empty_forever_spellbook_outfits(&env);
        crate::common::load_game_startup_inventory(&env);
    } else {
        use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
        for (name, toc) in discover_blizzard_addons_for_screen(&ui, screen) {
            load_addon(&env.loader_env(), &toc)
                .unwrap_or_else(|error| panic!("load {name}: {error}"));
        }
    }
    env
}

fn verify_native_border(screen: ScreenKind, tooltip: &str) {
    let env = load_tooltip_screen(screen);
    let native = format!("assert({tooltip}.NineSlice, 'missing XML child before workaround')");
    env.exec(&native).expect("template should create the child");
    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env.exec(&format!(r#"
        local tooltip = assert({tooltip})
        local child = assert(tooltip.NineSlice)
        local source = debug.getinfo(SharedTooltip_SetBackdropStyle, 'S')
        assert(source.source:gsub('\\', '/'):find('Blizzard_SharedXML/SharedTooltipTemplates.lua', 1, true))
        assert(source.linedefined == 47)
        assert(child:GetParent() == tooltip)
        assert(child:IsUsingParentLevel() and child:GetFrameLevel() == tooltip:GetFrameLevel())
        SharedTooltip_SetBackdropStyle(tooltip, nil, true)
        assert(not child:IsShown())
        SharedTooltip_SetBackdropStyle(tooltip, nil, false)
        assert(child:IsShown() and tooltip.NineSlice == child)
        local corner = assert(child.TopLeftCorner)
        assert(corner:GetNumPoints() > 0, 'original layout must anchor its corner')
    "#)).expect("original tooltip backdrop should lay out and show its inherited child");
}

#[test]
fn forever_game_tooltip_uses_template_border() {
    verify_native_border(ScreenKind::Game, "GameTooltip");
}

#[test]
fn forever_glue_tooltip_uses_template_border() {
    verify_native_border(ScreenKind::CharacterSelect, "GlueTooltip");
}
