//! World-map floor navigation must use supplied map-group membership.

#[path = "common/blizzard_addon_closure.rs"]
mod blizzard_addon_closure;

use wow_ui_sim::lua_api::WowLuaEnv;

const FLOOR_MENU_LUA: &str = r#"
    local function floor_menu(mapID)
        WorldMapFrame:SetMapID(mapID)
        local dropdown
        for _, frame in ipairs(WorldMapFrame.overlayFrames or {}) do
            if type(frame.RefreshMenu) == "function" then
                dropdown = frame
                break
            end
        end
        assert(dropdown, "WorldMap floor-navigation frame must exist")
        local shown = dropdown:RefreshMenu(mapID)
        local rows = {}
        for _, entry in dropdown:GetMenuDescription():EnumerateElementDescriptions() do
            local member = entry:GetData()
            table.insert(rows, member.mapID .. ":" .. member.name)
        end
        return shown, table.concat(rows, ",")
    end
    local firstShown, firstRows = floor_menu(84)
    local secondShown, secondRows = floor_menu(2248)
    return firstShown, firstRows, secondShown, secondRows
"#;

fn load_world_map_consumer() -> WowLuaEnv {
    let ui = blizzard_addon_closure::blizzard_ui_dir();
    let env = blizzard_addon_closure::new_blizzard_addon_env(&ui);
    let loaded = blizzard_addon_closure::load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_WorldMap"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_FrameXML"));
    assert!(loaded.iter().any(|name| name == "Blizzard_WorldMap"));
    env
}

fn supply_map_group_members(env: &WowLuaEnv) {
    env.exec(
        r#"
        A_Admin.SetMapGroupMember(84, 7001, 1)
        A_Admin.SetMapGroupMember(1, 7001, 2)
        A_Admin.SetMapGroupMember(2248, 7002, 1)
        A_Admin.SetMapGroupMember(2200, 7002, 2)
        "#,
    )
    .expect("map-group fixtures should be supplied through A_Admin");
}

fn read_floor_menu_outputs(env: &WowLuaEnv) -> (bool, String, bool, String) {
    env.eval(FLOOR_MENU_LUA)
        .expect("WorldMap floor-navigation consumer should build both menus")
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn world_map_floor_navigation_uses_each_supplied_map_group() {
    let env = load_world_map_consumer();
    supply_map_group_members(&env);
    let result = read_floor_menu_outputs(&env);

    assert_eq!(
        result,
        (
            true,
            "84:Stormwind City,1:Dun Morogh".into(),
            true,
            "2248:Isle of Dorn,2200:Emerald Dream".into()
        )
    );
}
