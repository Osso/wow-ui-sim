//! Preservation of original Lua-owned locations, not new native topology proof.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]
use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::screen::ScreenKind;

#[test]
fn original_transmog_cache_and_camera_consumer_preserve_distinct_locations() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_name) in [
        (
            "Blizzard_UIPanels_Game",
            "Blizzard_UIPanels_Game_Mainline.toc",
        ),
        ("Blizzard_TransmogShared", "Blizzard_TransmogShared.toc"),
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
        local path = 'Blizzard_TransmogShared/Blizzard_TransmogShared.lua'
        local function original(fn)
            assert(type(fn) == 'function', 'original function missing')
            local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(source:find(path, 1, true), source)
        end
        original(InitializeSlotLocationInfo)
        original(TransmogUtil.GetTransmogLocation)
        original(TransmogUtil.CreateTransmogLocation)
        original(TransmogUtil.GetCameraVariation)
        original(TransmogLocationMixin.IsSecondary)
        -- Initialization already ran in the normal selected source chunk. Do not
        -- re-execute it or supply a synthetic slot catalog to fill the cache.
        local appearance, illusion = Enum.TransmogType.Appearance, Enum.TransmogType.Illusion
        local cases = {
            {'SHOULDERSLOT', appearance, false, false, false},
            {'SHOULDERSLOT', appearance, true, false, false},
            {'MAINHANDSLOT', appearance, false, true, false},
            {'SECONDARYHANDSLOT', appearance, false, true, true},
            {'MAINHANDSLOT', illusion, false, true, false},
            {'SECONDARYHANDSLOT', illusion, false, true, true},
        }
        local first = {}
        for pass=1,2 do
            for index, case in ipairs(cases) do
                local name, kind, secondary, hand, offhand = unpack(case)
                local cached = TransmogUtil.GetTransmogLocation(name, kind, secondary)
                assert(cached, 'normal startup must initialize each requested location')
                assert(TRANSMOG_SLOTS[cached:GetLookupKey()].location == cached, 'original cache identity')
                local constructed = TransmogUtil.CreateTransmogLocation(name, kind, secondary)
                assert(constructed ~= cached and constructed:IsEqual(cached), 'fresh real mixin with same data')
                assert(cached:GetSlotName() == name)
                assert(cached:IsAppearance() == (kind == appearance))
                assert(cached:IsIllusion() == (kind == illusion))
                assert(cached:IsSecondary() == secondary)
                assert(cached:IsEitherHand() == hand and cached:IsOffHand() == offhand)
                -- Original consumer supports explicit checkSecondary (also used
                -- by selected TransmogTemplates). No native item query needed.
                assert(TransmogUtil.GetCameraVariation(cached, true) == (secondary and 0 or 1))
                assert(TransmogUtil.GetCameraVariation(cached, false) == nil)
                if pass == 1 then first[index] = cached else assert(first[index] == cached) end
            end
        end
        for i=1,#first do
            for j=i+1,#first do assert(not first[i]:IsEqual(first[j]), 'distinct locations remain distinct') end
        end
    "#).expect("original transmog cache and camera consumer preserve repeated distinct locations");
}
