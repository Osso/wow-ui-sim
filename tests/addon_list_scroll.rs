//! AddonList ScrollBox behavioral tests.
//!
//! These tests require a large collection of addons to populate the scroll list.
//! They use the local WoW addon directory and skip in CI where it doesn't exist.

use crate::common;

use common::env_with_shared_xml;
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::lua_api::WowLuaEnv;

const USER_ADDONS_PATH: &str = "/home/osso/Projects/wow/Interface/AddOns";
const ADDON_LIST_DEPENDENCY_DIRS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_Colors",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_UIPanelTemplates",
    "Blizzard_GameMenu",
    "Blizzard_UIWidgets",
    "Blizzard_FrameXMLBase",
    "Blizzard_AddOnList",
];

fn has_local_addons() -> bool {
    std::path::Path::new(USER_ADDONS_PATH).exists()
}

fn blizzard_addons_base() -> std::path::PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// Load all Blizzard addons needed for AddonList + scan user addons.
fn env_with_addon_list() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    load_addon_list_dependencies(&env);
    env.scan_and_register_addons(std::path::Path::new(USER_ADDONS_PATH));
    ensure_addon_list_scrollbox_view(&env);
    env
}

fn load_addon_list_dependencies(env: &WowLuaEnv) {
    let base = blizzard_addons_base();
    for dir_name in ADDON_LIST_DEPENDENCY_DIRS {
        let addon_dir = base.join(dir_name);
        let toc_path = find_toc_file(&addon_dir)
            .unwrap_or_else(|| panic!("No TOC found in {}", addon_dir.display()));
        if let Err(e) = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc_path) {
            eprintln!("Warning: Failed to load {}: {}", dir_name, e);
        }
    }
}

fn ensure_addon_list_scrollbox_view(env: &WowLuaEnv) {
    // Re-init the AddonList ScrollBox view. During XML loading, the OnLoad handler's
    // SetView call doesn't persist due to a __newindex ordering issue in method chains.
    env.exec(
        r#"
        if AddonList and AddonList.ScrollBox and not AddonList.ScrollBox:HasView() then
            local view = CreateScrollBoxListTreeListView(20, 5, 5, 5, 5, 8)
            view:SetElementFactory(function(factory, treeNode)
                local elementData = treeNode:GetData()
                if elementData.addonIndex then
                    factory("AddonListEntryTemplate", AddonList_InitAddon)
                elseif elementData.category then
                    factory("AddonListCategoryTemplate", AddonList_InitCategory)
                end
            end)
            ScrollUtil.InitScrollBoxListWithScrollBar(AddonList.ScrollBox, AddonList.ScrollBar, view)
        end
    "#,
    )
    .unwrap();
}

/// Return stable identifiers for visible AddonList rows.
fn get_visible_entry_ids(env: &WowLuaEnv) -> Vec<String> {
    let result: String = env
        .eval(
            r#"
        local rows = {}
        local view = AddonList.ScrollBox:GetView()
        if view then
            local frames = view:GetFrames()
            for _, frame in ipairs(frames) do
                local rowData = frame.GetData and frame:GetData()
                if rowData then
                    if rowData.addonIndex then
                        table.insert(rows, "addon:" .. tostring(C_AddOns.GetAddOnName(rowData.addonIndex)))
                    elseif rowData.category then
                        table.insert(rows, "category:" .. tostring(rowData.category))
                    else
                        table.insert(rows, "unknown")
                    end
                end
            end
        end
        return table.concat(rows, "\030")
    "#,
        )
        .unwrap();
    if result.is_empty() {
        Vec::new()
    } else {
        result.split('\x1e').map(String::from).collect()
    }
}

/// Show AddonList, populate it, and fire OnUpdate ticks.
fn init_addon_list(env: &WowLuaEnv) {
    env.exec("AddonList:Show()").unwrap();
    env.exec("AddonList_Update()").unwrap();
    for _ in 0..5 {
        env.fire_on_update(0.016).unwrap();
    }
}

#[test]
fn test_addon_list_update_defaults_group_metadata_to_addon_name() {
    // Group defaulting belongs to the addon registry, not the modern ScrollBox.
    let env = if cfg!(feature = "client-wrath") {
        WowLuaEnv::new().expect("addon metadata environment")
    } else {
        env_with_addon_list()
    };
    env.exec(r#"A_Admin.RegisterTestAddon("FakeTestAddon")"#)
        .unwrap();

    let group: String = env
        .eval(r#"return C_AddOns.GetAddOnMetadata("FakeTestAddon", "Group") or "" "#)
        .unwrap();
    assert_eq!(
        group, "FakeTestAddon",
        "AddOnList expects every addon to have a Group string, even when no TOC metadata sets one"
    );

    if cfg!(feature = "client-wrath") {
        let legacy_group: String = env
            .eval(r#"return GetAddOnMetadata("FakeTestAddon", "Group")"#)
            .unwrap();
        assert_eq!(legacy_group, group);
    } else {
        env.exec("AddonList:Show()").unwrap();
        env.exec("AddonList_Update()").unwrap();
    }
}

#[test]
fn test_addon_list_scroll_down_changes_entries() {
    if !has_local_addons() {
        return;
    }
    let env = env_with_addon_list();
    init_addon_list(&env);

    let before = get_visible_entry_ids(&env);
    assert!(
        !before.is_empty(),
        "AddonList should have visible entries after init"
    );
    let first_before = before[0].clone();

    for _ in 0..10 {
        env.exec("AddonList.ScrollBox:OnMouseWheel(-1)").unwrap();
        env.fire_on_update(0.016).unwrap();
    }

    let after = get_visible_entry_ids(&env);
    assert!(
        !after.is_empty(),
        "AddonList should still have visible entries after scroll"
    );

    assert_ne!(
        first_before, after[0],
        "First visible entry should change after scrolling down: was '{}', still '{}'",
        first_before, after[0]
    );
}

#[test]
fn test_addon_list_scroll_up_restores_entries() {
    if !has_local_addons() {
        return;
    }
    let env = env_with_addon_list();
    init_addon_list(&env);

    let original = get_visible_entry_ids(&env);
    assert!(
        !original.is_empty(),
        "AddonList should have visible entries"
    );

    for _ in 0..10 {
        env.exec("AddonList.ScrollBox:OnMouseWheel(-1)").unwrap();
        env.fire_on_update(0.016).unwrap();
    }

    let scrolled = get_visible_entry_ids(&env);
    assert_ne!(
        original[0], scrolled[0],
        "Should have scrolled away from initial position"
    );

    for _ in 0..10 {
        env.exec("AddonList.ScrollBox:OnMouseWheel(1)").unwrap();
        env.fire_on_update(0.016).unwrap();
    }

    let restored = get_visible_entry_ids(&env);
    assert_eq!(
        original[0], restored[0],
        "First entry should be restored after scrolling back up: expected '{}', got '{}'",
        original[0], restored[0]
    );
}

#[test]
fn test_addon_list_scroll_percentage_changes() {
    if !has_local_addons() {
        return;
    }
    let env = env_with_addon_list();
    init_addon_list(&env);

    let pct_before: f64 = env
        .eval("return AddonList.ScrollBox:GetScrollPercentage() or 0")
        .unwrap();
    assert!(
        pct_before < 0.01,
        "Initial scroll percentage should be ~0, got {}",
        pct_before
    );

    for _ in 0..5 {
        env.exec("AddonList.ScrollBox:OnMouseWheel(-1)").unwrap();
        env.fire_on_update(0.016).unwrap();
    }

    let pct_after: f64 = env
        .eval("return AddonList.ScrollBox:GetScrollPercentage() or 0")
        .unwrap();
    assert!(
        pct_after > pct_before,
        "Scroll percentage should increase after scrolling down: before={}, after={}",
        pct_before,
        pct_after
    );
}

#[test]
fn test_addon_list_scroll_keeps_rows_initialized_after_update_ticks() {
    if !has_local_addons() {
        return;
    }

    let env = env_with_addon_list();
    init_addon_list(&env);

    for _ in 0..10 {
        env.exec("AddonList.ScrollBox:OnMouseWheel(-1)").unwrap();
        env.fire_on_update(0.016).unwrap();
    }

    let after: String = env
        .eval(
            r#"
            local view = AddonList.ScrollBox:GetView()
            local first = view and view:GetFrames()[1]
            local rowData = first and first.GetData and first:GetData()
            return table.concat({
                tostring(AddonList.ScrollBox:GetDataIndexBegin() or "nil"),
                tostring(AddonList.ScrollBox:GetDataIndexEnd() or "nil"),
                tostring(view and #view:GetFrames() or "nil"),
                tostring(AddonList.ScrollBox:GetFrameCount() or "nil"),
                tostring(first and first.Title and first.Title:GetText() or "nil"),
                tostring(rowData and (rowData.category or rowData.addonIndex) or "nil"),
            }, "|")
        "#,
        )
        .unwrap();

    let after_parts: Vec<_> = after.split('|').collect();
    assert_ne!(
        after_parts[0], "0",
        "scroll begin should stay in range: {after}"
    );
    assert_ne!(
        after_parts[1], "0",
        "scroll end should stay in range: {after}"
    );
    assert_ne!(
        after_parts[2], "0",
        "view should still own visible frames: {after}"
    );
    assert_ne!(
        after_parts[3], "0",
        "scroll box should still report visible frames: {after}"
    );
    assert_ne!(
        after_parts[4], "nil",
        "first visible row should keep its title text after repeated update ticks: {after}"
    );
    assert_ne!(
        after_parts[5], "nil",
        "first visible row should keep its element data after repeated update ticks: {after}"
    );
}

// ============================================================================
// MinimalScrollBar Atlas Texture Tests (requires SharedXML)
// ============================================================================

post_wrath_ui_fixture! {
#[test]
fn test_minimal_scrollbar_atlas_textures() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("EventFrame", "TestMinScrollBarAtlas", UIParent, "MinimalScrollBar")
        sb:SetSize(16, 200)
    "#,
    )
    .unwrap();

    let back_atlas: String = env
        .eval(
            r#"
        local tex = TestMinScrollBarAtlas.Back.Texture
        return tex:GetAtlas() or ""
    "#,
        )
        .unwrap();
    assert_eq!(
        back_atlas, "minimal-scrollbar-arrow-top",
        "Back button texture should have atlas set via OnLoad"
    );

    let back_file: String = env
        .eval(
            r#"
        local tex = TestMinScrollBarAtlas.Back.Texture
        return tex:GetTexture() or ""
    "#,
        )
        .unwrap();
    assert!(
        back_file.contains("minimalscrollbarproportional"),
        "Back texture file should be resolved from atlas: got '{}'",
        back_file
    );

    let forward_atlas: String = env
        .eval(
            r#"
        local tex = TestMinScrollBarAtlas.Forward.Texture
        return tex:GetAtlas() or ""
    "#,
        )
        .unwrap();
    assert_eq!(
        forward_atlas, "minimal-scrollbar-arrow-bottom",
        "Forward button texture should have atlas set via OnLoad"
    );
}

}

// INFERRED from cached profile FrameXML, not native 12340 runtime proof.
#[cfg(feature = "client-wrath")]
#[test]
fn test_minimal_scrollbar_atlas_textures() {
    let env = env_with_shared_xml();
    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "LegacyScrollArt", UIParent, "FauxScrollFrameTemplate")
        sf:SetSize(200, 100)
        sf:SetPoint("CENTER")
        assert(FauxScrollFrame_Update(sf, 20, 5, 20) == 1)
        local bar = LegacyScrollArtScrollBar
        local up = LegacyScrollArtScrollBarScrollUpButton
        local down = LegacyScrollArtScrollBarScrollDownButton
        assert(up:GetParent() == bar and down:GetParent() == bar)
        assert(up:GetNormalTexture():GetTextureFilePath() == "Interface\\Buttons\\UI-ScrollBar-ScrollUpButton-Up")
        assert(down:GetNormalTexture():GetTextureFilePath() == "Interface\\Buttons\\UI-ScrollBar-ScrollDownButton-Up")
        assert(bar:GetThumbTexture():GetTextureFilePath() == "Interface\\Buttons\\UI-ScrollBar-Knob")
        local low, high = bar:GetMinMaxValues()
        assert(low == 0 and high == 300)
        -- 12340 IsEnabled returns 1/0 (FrameXML/UIPanelTemplates.xml:68-96).
        assert(up:IsEnabled() == 0 and down:IsEnabled() == 1)
        bar:SetValue(300)
        FauxScrollFrame_Update(sf, 20, 5, 20)
        assert(up:IsEnabled() == 1 and down:IsEnabled() == 0)
        "#,
    )
    .unwrap();
}
