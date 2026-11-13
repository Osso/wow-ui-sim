//! Retail original-consumer coverage through the application startup loader.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::screen::ScreenKind;

#[test]
fn original_container_xml_parent_array_drives_repeated_bag_enumeration() {
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
        local path = 'Blizzard_UIPanels_Game/Mainline/ContainerFrame.lua'
        local function original(fn)
            local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(source:find(path, 1, true), source)
        end
        original(ContainerFrameUtil_EnumerateContainerFrames)
        original(ContainerFrameUtil_EnumerateBagFrames)
        original(ContainerFrameSettingsManager.OnCombinedBagSettingChanged)
        local owner = ContainerFrameContainer
        assert(owner:GetParent() == UIParent, 'original XML owner parent')
        local members = owner.ContainerFrames
        assert(#members == 6, 'six original XML bag instances')
        for i=1,6 do
            assert(members[i] == _G['ContainerFrame'..i], 'original XML parentArray identity/order')
            assert(members[i]:GetParent() == owner, 'bag parent is original container')
            assert(members[i]:GetScript('OnLoad'), 'XML installed bag OnLoad')
        end
        -- Original manager invalidates its private enumerator caches. No table
        -- mutation, fake bag, manual event or native container data is supplied.
        for _, combined in ipairs({false, true, false, true, false}) do
            ContainerFrameSettingsManager:OnCombinedBagSettingChanged(combined)
            local observed = {}
            for _, frame in ContainerFrameUtil_EnumerateContainerFrames() do
                observed[#observed+1] = frame
            end
            local offset = combined and 1 or 0
            assert(#observed == 6 + offset, 'container membership changes and returns')
            if combined then assert(observed[1] == ContainerFrameCombinedBags) end
            for i=1,6 do assert(observed[i+offset] == members[i], 'original distinct bag identity') end
            local bags = {}
            for _, frame in ContainerFrameUtil_EnumerateBagFrames() do bags[#bags+1] = frame end
            if combined then
                assert(#bags == 1 and bags[1] == ContainerFrameCombinedBags, 'combined bag membership')
            else
                assert(#bags == math.min(Constants.InventoryConstants.NumBagSlots, #members))
                for i, frame in ipairs(bags) do assert(frame == members[i], 'individual bag membership') end
            end
            assert(ContainerFrameContainer == owner and owner.ContainerFrames == members,
                'original XML owner/array preserved across preference transitions')
        end
    "#).expect("original XML parent array drives real repeated bag enumerators");
}
