//! Retail PRESERVATION of original constructors through normal WorldMap loading.
//! Not waypoint callback acceptance: waypoint APIs remain a separate modeling prerequisite.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::screen::ScreenKind;

#[test]
fn original_worldmap_load_preserves_uimap_point_ownership() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_name) in [
        (
            "Blizzard_UIPanels_Game",
            "Blizzard_UIPanels_Game_Mainline.toc",
        ),
        ("Blizzard_TokenUI", "Blizzard_TokenUI.toc"),
    ] {
        let addon = startup
            .iter()
            .find(|a| a.name == name)
            .expect("selected game addon");
        assert_eq!(addon.toc_path.file_name().unwrap().to_str(), Some(toc_name));
    }
    // Match wow_sim/addon_loading.rs: register LoadWith, normal startup
    // discovery/kinds/order, skip already-loaded followers, restore cleanup.
    // No manual events, post-load workaround, source override or fake frame.
    for addon in startup {
        let loaded = env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|a| a.folder_name == addon.name && a.loaded);
        if !loaded {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
                .unwrap_or_else(|e| panic!("startup {}: {e}", addon.name));
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.sync_string_metatable_to_global_string();
    eprintln!(
        "unfiltered startup diagnostics: {:?}",
        env.state().borrow().lua_errors
    );
    env.exec(r#"
        assert(C_AddOns.LoadAddOn('Blizzard_WorldMap'))
        assert(type(WorldMapFrame) == 'table')
        local function owner(fn)
            return debug.getinfo(fn, 'S').source:gsub('\\', '/')
        end
        assert(owner(UiMapPoint.CreateFromCoordinates):find('Blizzard_ObjectAPI/Mainline/UiMapPoint.lua',1,true))
        assert(owner(UiMapPoint.CreateFromVector2D):find('Blizzard_ObjectAPI/Mainline/UiMapPoint.lua',1,true))
        local coordinatesOwner = UiMapPoint.CreateFromCoordinates
        local vectorOwner = UiMapPoint.CreateFromVector2D
        for _, coordinates in ipairs({{1,0.2,0.7,9},{2248,0.8,0.3,4},{1,0.2,0.7,9}}) do
            local id,x,y,z=unpack(coordinates)
            local point=UiMapPoint.CreateFromCoordinates(id,x,y,z)
            assert(point.uiMapID==id and point.z==z)
            local px,py=point.position:GetXY()
            assert(px==x and py==y)
            local vector=CreateVector2D(x,y)
            local shared=UiMapPoint.CreateFromVector2D(id,vector,z)
            assert(shared.position==vector and shared.z==z and shared.uiMapID==id)
            vector:SetXY(y,x)
            local sx,sy=shared.position:GetXY()
            assert(sx==y and sy==x)
            local unchangedX,unchangedY=point.position:GetXY()
            assert(unchangedX==x and unchangedY==y)
        end
        assert(C_AddOns.LoadAddOn('Blizzard_WorldMap'))
        assert(UiMapPoint.CreateFromCoordinates==coordinatesOwner)
        assert(UiMapPoint.CreateFromVector2D==vectorOwner)
    "#).expect("original WorldMap-load constructor ownership PRESERVATION, not waypoint callback acceptance");
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "unfiltered errors: {:?}",
        env.state().borrow().lua_errors
    );
}
