//! Supplied eligibility through the selected generic ItemUtil, not the upgrade window.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

// Pinned ItemUpgradeDocumentation.lua: R/P/F11–24; M/E/A11–23.
// Generic ItemUtil calls: R147, P/F217, M/E/A142 (rechecked 2026-10-26).
fn load_original_item_util() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let toc_path = find_toc_file(&ui.join("Blizzard_FrameXMLUtil")).unwrap();
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    assert!(toc.files.iter().any(|entry| entry == "ItemUtil.lua"));
    eprintln!(
        "selected generic ItemUtil: {} {:?}",
        toc_path.display(),
        toc.metadata
    );
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_ObjectAPI", "Blizzard_FrameXMLUtil"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_FrameXMLUtil"));
    eprintln!(
        "unfiltered closure diagnostics: {:?}",
        env.state().borrow().lua_errors
    );
    env.exec(r#"
        local info = debug.getinfo(ItemButtonUtil.GetItemContextMatchResultForItem, 'S')
        local suffix = 'Blizzard_FrameXMLUtil/ItemUtil.lua'
        local source = info.source:gsub('\\', '/')
        assert(info.what == 'Lua' and source:sub(-#suffix) == suffix, 'generic consumer provenance')
        print('generic ItemUtil provenance', source, info.linedefined, info.lastlinedefined)
        -- Context input only. Era/Anniversary do not select an upgrade window.
        -- No Blizzard function is replaced and no upgrade-window behavior is claimed.
        assert(not ItemUpgradeFrame, 'closure must not load the upgrade window')
        ItemUpgradeFrame = CreateFrame('Frame', nil, UIParent)
        ItemUpgradeFrame:Show()
        assert(ItemButtonUtil.GetItemContext() == ItemButtonUtil.ItemContextEnum.UpgradableItem)
        A_Admin.ClearBags()
        A_Admin.AddBagItem(0, 1, 6948, 1)
        A_Admin.AddBagItem(0, 2, 19019, 1)
        firstLocation = ItemLocation:CreateFromBagAndSlot(0, 1)
        secondLocation = ItemLocation:CreateFromBagAndSlot(0, 2)
        equipmentLocation = ItemLocation:CreateFromEquipmentSlot(16)
        function CheckUpgrade(location, expected)
            assert(C_Item.DoesItemExist(location), 'fixture location must resolve')
            local results = ItemButtonUtil.ItemContextMatchResult
            local result = ItemButtonUtil.GetItemContextMatchResultForItem(location)
            assert(result == (expected and results.Match or results.Mismatch), 'original ItemUtil supplied match')
            local function checkResult(...)
                assert(select('#', ...) == 1 and type((...)) == 'boolean', 'documented boolean arity')
                assert((...) == expected, 'supplied direct eligibility')
            end
            checkResult(C_ItemUpgrade.CanUpgradeItem(location))
        end
        function CheckUpgradeDefault(location)
            local function checkResult(...)
                assert(select('#', ...) == 1 and (...) == false, 'unsupplied location defaults to false')
            end
            checkResult(C_ItemUpgrade.CanUpgradeItem(location))
        end
        function CheckUpgradeInvalid(location)
            local ok, reason = pcall(C_ItemUpgrade.CanUpgradeItem, location)
            assert(not ok and tostring(reason):find('itemLocation must be', 1, true), tostring(reason))
        end
    "#).expect("original generic consumer and real two-location fixture");
    env
}

#[test]
fn original_item_util_tracks_supplied_eligibility_movement_removal_and_isolation() {
    crate::common::with_timeout(90, || {
        let first = load_original_item_util();
        let second = load_original_item_util();
        first.exec(r#"
            A_Admin.SetItemUpgradeEligibility(firstLocation, true)
            A_Admin.SetItemUpgradeEligibility(secondLocation, false)
            CheckUpgrade(firstLocation, true)
            CheckUpgrade(secondLocation, false)
            A_Admin.SetItemUpgradeEligibility(firstLocation, false)
            A_Admin.SetItemUpgradeEligibility(secondLocation, true)
            CheckUpgrade(firstLocation, false)
            CheckUpgrade(secondLocation, true)
            -- Swap two different items through the real cursor path.
            C_Container.PickupContainerItem(0, 1)
            C_Container.PickupContainerItem(0, 2)
            C_Container.PickupContainerItem(0, 1)
            CheckUpgrade(firstLocation, true)
            CheckUpgrade(secondLocation, false)
            -- Bag -> equipment -> bag retains the observation, not slot policy.
            C_Container.PickupContainerItem(0, 1)
            PickupInventoryItem(16)
            C_Container.PickupContainerItem(0, 1)
            CheckUpgrade(equipmentLocation, true)
            PickupInventoryItem(16)
            C_Container.PickupContainerItem(0, 3)
            movedLocation = ItemLocation:CreateFromBagAndSlot(0, 3)
            CheckUpgrade(movedLocation, true)
            A_Admin.RemoveBagItem(0, 3)
            assert(ItemButtonUtil.GetItemContextMatchResultForItem(movedLocation) == ItemButtonUtil.ItemContextMatchResult.DoesNotApply)
            CheckUpgradeDefault(movedLocation)
            A_Admin.AddBagItem(0, 3, 19019, 1)
            CheckUpgradeDefault(movedLocation)
            A_Admin.SetItemUpgradeEligibility(movedLocation, false)
            CheckUpgrade(movedLocation, false)
            A_Admin.ClearItemUpgradeEligibility(movedLocation)
            CheckUpgradeDefault(movedLocation)
            A_Admin.ClearItemUpgradeEligibility()
            CheckUpgradeDefault(secondLocation)
            A_Admin.SetItemUpgradeEligibility(secondLocation, true)
            A_Admin.RemoveBagItem(0, 1)
            C_Container.PickupContainerItem(0, 2)
            PutItemInBackpack()
            CheckUpgrade(firstLocation, true)
            C_Container.PickupContainerItem(0, 1)
            DeleteCursorItem()
            A_Admin.AddBagItem(0, 1, 6948, 1)
            CheckUpgradeDefault(firstLocation)
            A_Admin.SetItemUpgradeEligibility(firstLocation, true)
            CheckUpgrade(firstLocation, true)
        "#).expect("supplied location eligibility must drive original ItemUtil through lifecycle");
        second
            .exec(
                r#"
            CheckUpgradeDefault(firstLocation)
            A_Admin.SetItemUpgradeEligibility(firstLocation, false)
            A_Admin.SetItemUpgradeEligibility(secondLocation, true)
            CheckUpgrade(firstLocation, false)
            CheckUpgrade(secondLocation, true)
        "#,
            )
            .expect("second environment has independent supplied eligibility");
        first
            .exec("CheckUpgrade(firstLocation, true); CheckUpgradeDefault(secondLocation)")
            .unwrap();
        second.exec(r#"
            A_Admin.SetItemUpgradeEligibility(firstLocation, true)
            CheckUpgrade(firstLocation, true)
            A_Admin.RemoveBagItem(0, 1)
            assert(ItemButtonUtil.GetItemContextMatchResultForItem(firstLocation) == ItemButtonUtil.ItemContextMatchResult.DoesNotApply)
        "#).expect("second environment changes and removes its own observation");
        first.exec("CheckUpgrade(firstLocation, true)").unwrap();
        assert!(first.state().borrow().lua_errors.is_empty());
        assert!(second.state().borrow().lua_errors.is_empty());
    });
}

#[test]
fn item_upgrade_unsupplied_locations_default_false_and_invalid_inputs_error() {
    crate::common::with_timeout(90, || {
        let env = load_original_item_util();
        env.exec(r#"
            CheckUpgradeDefault(firstLocation)
            CheckUpgradeDefault(ItemLocation:CreateFromBagAndSlot(0, 9))
            CheckUpgradeInvalid({itemID = 19019})
            CheckUpgradeInvalid(nil)
            CheckUpgradeInvalid('not a location')
            local ok = pcall(A_Admin.SetItemUpgradeEligibility, ItemLocation:CreateFromBagAndSlot(0, 9), true)
            assert(not ok, 'admin cannot supply an absent item')
            A_Admin.SetItemUpgradeEligibility(firstLocation, true)
            CheckUpgrade(firstLocation, true)
            A_Admin.AddBagItem(0, 1, 6948, 1)
            CheckUpgradeDefault(firstLocation)
            A_Admin.SetItemUpgradeEligibility(firstLocation, false)
            CheckUpgrade(firstLocation, false)
            A_Admin.ClearBags()
            CheckUpgradeDefault(firstLocation)
            A_Admin.AddBagItem(0, 1, 6948, 1)
            CheckUpgradeDefault(firstLocation)
            A_Admin.ClearBags()
            A_Admin.EquipItem(16, 19019)
            A_Admin.SetItemUpgradeEligibility(equipmentLocation, true)
            CheckUpgrade(equipmentLocation, true)
            A_Admin.UnequipItem(16)
            CheckUpgradeDefault(equipmentLocation)
            A_Admin.EquipItem(16, 19019)
            CheckUpgradeDefault(equipmentLocation)
        "#).expect("unsupplied locations use the best-guess false default");
    });
}
