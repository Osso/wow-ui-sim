use std::collections::HashSet;
use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addon_closure_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn load_store_ui_tree() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    load_store_addons(&env);

    env.apply_post_load_workarounds();
    env
}

fn load_store_addons(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addon_closure_for_screen(
        &ui,
        ScreenKind::Game,
        &["Blizzard_FrameXMLBase", "Blizzard_StoreUI"],
    );
    for (name, toc_path) in addons {
        load_addon(&env.loader_env(), &toc_path).unwrap_or_else(|err| {
            panic!("[load {name}] FAILED: {err}");
        });
    }
}

fn assert_no_child_cycle(
    widgets: &wow_ui_sim::widget::WidgetRegistry,
    id: u64,
    path: &mut Vec<u64>,
    seen: &mut HashSet<u64>,
) {
    assert!(
        !path.contains(&id),
        "cycle detected in child graph: {:?} -> {}",
        path,
        id
    );
    if !seen.insert(id) {
        return;
    }

    path.push(id);
    let children = widgets
        .get(id)
        .map(|f| f.children.clone())
        .unwrap_or_default();
    for child_id in children {
        assert_no_child_cycle(widgets, child_id, path, seen);
    }
    path.pop();
}

post_wrath_ui_fixture! {
#[test]
fn store_frame_tree_contains_expected_panels_and_controls() {
    let env = load_store_ui_tree();
    let state = env.state().borrow();

    let store_id = state
        .widgets
        .get_id_by_name("StoreFrame")
        .expect("StoreFrame should exist");
    let ui_parent_id = state
        .widgets
        .get_id_by_name("UIParent")
        .expect("UIParent should exist");
    let store = state
        .widgets
        .get(store_id)
        .expect("StoreFrame widget should exist");

    assert_eq!(
        store.parent_id,
        Some(ui_parent_id),
        "StoreFrame should be parented to UIParent"
    );
    drop(state);

    for name in [
        "StoreConfirmationFrame",
        "StoreVASValidationFrame",
        "StoreTooltip",
    ] {
        let state = env.state().borrow();
        let id = state
            .widgets
            .get_id_by_name(name)
            .unwrap_or_else(|| panic!("{name} should exist"));
        let frame = state
            .widgets
            .get(id)
            .unwrap_or_else(|| panic!("{name} widget missing"));
        assert_eq!(
            frame.parent_id,
            Some(store_id),
            "{name} should be parented to StoreFrame"
        );
    }

    let tree_exists: bool = env
        .eval(
            r#"
            return StoreFrame ~= nil
                and StoreFrame.LeftInset ~= nil
                and StoreFrame.RightInset ~= nil
                and StoreFrame.LeftDisplay ~= nil
                and StoreFrame.RightDisplay ~= nil
                and StoreFrame.Cover ~= nil
                and StoreFrame.Notice ~= nil
                and StoreFrame.Notice.Icon ~= nil
                and StoreFrame.Notice.Line ~= nil
                and StoreFrame.Notice.Title ~= nil
                and StoreFrame.Notice.Description ~= nil
                and StoreFrame.ErrorFrame ~= nil
                and StoreFrame.ErrorFrame.Icon ~= nil
                and StoreFrame.ErrorFrame.Line ~= nil
                and StoreFrame.ErrorFrame.Title ~= nil
                and StoreFrame.ErrorFrame.Description ~= nil
                and StoreFrame.ErrorFrame.WebsiteWarning ~= nil
                and StoreFrame.ErrorFrame.AcceptButton ~= nil
                and StoreFrame.ErrorFrame.WebsiteButton ~= nil
                and StoreFrame.BuyButton ~= nil
                and StoreFrame.PrevPageButton ~= nil
                and StoreFrame.NextPageButton ~= nil
                and StoreFrame.CloseButton ~= nil
                and StoreFrame.CategoryTreeScrollContainer ~= nil
                and StoreFrame.CategoryTreeScrollContainer.ScrollBox ~= nil
                and StoreFrame.CategoryTreeScrollContainer.ScrollBar ~= nil
            "#,
        )
        .expect("store tree should be queryable from Lua");
    assert!(
        tree_exists,
        "StoreFrame Lua subtree should expose the expected store controls"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn store_frame_can_be_shown_after_minimal_store_load() {
    let env = load_store_ui_tree();
    let shown: bool = env
        .eval(
            r#"
            StoreFrame:Show()
            return StoreFrame:IsShown() == true
            "#,
        )
        .expect("StoreFrame:Show() should return");
    assert!(
        shown,
        "StoreFrame should show after minimal store-only load"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn store_frame_action_attribute_shows_frame_after_minimal_store_load() {
    let env = load_store_ui_tree();
    let (has_handler, shown, isshown_attr, global_type, secure_type): (
        bool,
        bool,
        bool,
        String,
        String,
    ) = env
        .eval(
            r#"
            StoreFrame:SetAttribute("action", "Show")
            return StoreFrame:GetScript("OnAttributeChanged") ~= nil,
                StoreFrame:IsShown() == true,
                StoreFrame:GetAttribute("isshown") == true,
                type(StoreFrame_OnAttributeChanged),
                type(__secureenv and __secureenv.StoreFrame_OnAttributeChanged)
            "#,
        )
        .expect("StoreFrame action attribute should be processed");
    assert!(
        has_handler,
        "StoreFrame should have OnAttributeChanged in minimal load (global={global_type}, secure={secure_type})"
    );
    assert!(shown, "action=Show should show the store in minimal load");
    assert!(
        isshown_attr,
        "OnShow should set isshown=true in minimal load"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn store_catalog_exposes_fake_products_after_product_refresh() {
    let env = load_store_ui_tree();
    let (group_count, selected_category, product_count, first_name): (i32, i32, i32, String) = env
        .eval(
            r#"
            StoreFrame:Show()
            C_StoreSecure.GetProductList()
            local selectedCategoryGetter = StoreFrame_GetSelectedCategoryID or (__secureenv and __secureenv.StoreFrame_GetSelectedCategoryID)
            local selectedCategory = selectedCategoryGetter and selectedCategoryGetter()
            local products = C_StoreSecure.GetProducts(selectedCategory)
            local firstEntry = products and products[1] and C_StoreSecure.GetEntryInfo(products[1])
            return #C_StoreSecure.GetProductGroups(),
                selectedCategory or 0,
                products and #products or 0,
                firstEntry and firstEntry.sharedData and firstEntry.sharedData.name or ""
            "#,
        )
        .expect("store catalog should be queryable after product refresh");

    assert!(
        group_count > 0,
        "fake store catalog should expose at least one category"
    );
    assert!(
        selected_category > 0,
        "product refresh should select a default category"
    );
    assert!(
        product_count > 0,
        "selected category should expose at least one product"
    );
    assert_eq!(first_name, "Apprentice Rider Bundle");
}
}

prefork_full_ui_case! {
fn store_open_does_not_record_store_state_errors_or_connecting_notice(env: &WowLuaEnv) {
    let notice: (bool, Option<String>, Option<String>) = env
        .eval(
            r#"
            StoreFrame:Show()
            C_StoreSecure.GetProductList()
            return StoreFrame.Notice:IsShown(),
                StoreFrame.Notice.Title:GetText(),
                StoreFrame.Notice.Description:GetText()
            "#,
        )
        .expect("store open should be inspectable");

    let errors = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "store open should not record Lua errors: {errors:#?}"
    );
    assert!(
        !notice.0,
        "plain store open should not show notice: title={:?} desc={:?}",
        notice.1, notice.2
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn store_catalog_uses_blizzard_card_pool_instead_of_debug_card() {
    let env = load_store_ui_tree();
    let first_card_name: Option<String> = env
        .eval(
            r#"
            StoreFrame:Show()
            C_StoreSecure.GetProductList()
            for activeCard in StoreFrame.productCardPoolCollection:EnumerateActive() do
                return activeCard:GetName()
            end
            return nil
            "#,
        )
        .expect("store card pool should be inspectable");

    assert_ne!(
        first_card_name.as_deref(),
        Some("WowStoreSimCard1"),
        "store catalog should use Blizzard card pool output, not the simulator debug card"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn store_catalog_card_geometry_stays_bounded() {
    let env = load_store_ui_tree();
    let metrics: (f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, String) = env
        .eval(
            r#"
            StoreFrame:Show()
            C_StoreSecure.GetProductList()
            local card
            for activeCard in StoreFrame.productCardPoolCollection:EnumerateActive() do
                card = activeCard
                break
            end
            assert(card, "expected at least one active store card")

            local cardW, cardH = card:GetSize()
            local cardLeft, cardBottom, cardRectW, cardRectH = card:GetRect()
            local buyW, buyH = 0, 0
            local buyRectW, buyRectH = 0, 0
            if card.BuyButton then
                buyW, buyH = card.BuyButton:GetSize()
                local _, _, bw, bh = card.BuyButton:GetRect()
                buyRectW, buyRectH = bw or 0, bh or 0
            end
            local iconW, iconH = card.Icon:GetSize()
            local texW, texH = card.Card:GetSize()
            local _, _, texRectW, texRectH = card.Card:GetRect()
            return cardW, cardH, cardRectW, cardRectH, buyW, buyH, buyRectW, buyRectH, texW, texH, texRectW, texRectH, card:GetName() or ""
            "#,
        )
        .expect("store card geometry should be queryable");

    let (
        card_w,
        card_h,
        rect_w,
        rect_h,
        buy_w,
        buy_h,
        buy_rect_w,
        buy_rect_h,
        tex_w,
        tex_h,
        tex_rect_w,
        tex_rect_h,
        name,
    ) = metrics;
    assert!(card_w <= 600.0, "card width too large: {card_w} for {name}");
    assert!(
        card_h <= 500.0,
        "card height too large: {card_h} for {name}"
    );
    assert!(
        rect_w <= 600.0,
        "card rect width too large: {rect_w} for {name}"
    );
    assert!(
        rect_h <= 500.0,
        "card rect height too large: {rect_h} for {name}"
    );
    if buy_w > 0.0 || buy_h > 0.0 {
        assert!(
            buy_w <= 250.0,
            "buy button width too large: {buy_w} for {name}"
        );
        assert!(
            buy_h <= 60.0,
            "buy button height too large: {buy_h} for {name}"
        );
        assert!(
            buy_rect_w <= 250.0,
            "buy button rect width too large: {buy_rect_w} for {name}"
        );
        assert!(
            buy_rect_h <= 60.0,
            "buy button rect height too large: {buy_rect_h} for {name}"
        );
    }
    assert!(
        tex_w <= 600.0,
        "card texture width too large: {tex_w} for {name}"
    );
    assert!(
        tex_h <= 500.0,
        "card texture height too large: {tex_h} for {name}"
    );
    assert!(
        tex_rect_w <= 600.0,
        "card texture rect width too large: {tex_rect_w} for {name}"
    );
    assert!(
        tex_rect_h <= 500.0,
        "card texture rect height too large: {tex_rect_h} for {name}"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn store_catalog_button_slice_geometry_stays_bounded() {
    let env = load_store_ui_tree();
    let metrics: (f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, String) = env
        .eval(
            r#"
            StoreFrame:Show()
            C_StoreSecure.GetProductList()
            local card
            for activeCard in StoreFrame.productCardPoolCollection:EnumerateActive() do
                if activeCard.BuyButton then
                    card = activeCard
                    break
                end
            end
            assert(card, "expected at least one active store card with a buy button")

            local left = card.BuyButton.Left
            local middle = card.BuyButton.Middle
            local right = card.BuyButton.Right
            assert(left and middle and right, "expected store buy button slices")

            local _, _, leftW, leftH = left:GetRect()
            local _, _, middleW, middleH = middle:GetRect()
            local _, _, rightW, rightH = right:GetRect()
            local _, _, iconW, iconH = card.Icon:GetRect()

            return left:GetWidth(), left:GetHeight(), leftW or 0, leftH or 0,
                middle:GetWidth(), middle:GetHeight(), middleW or 0, middleH or 0,
                right:GetWidth(), right:GetHeight(), rightW or 0, rightH or 0,
                string.format("iconRect=%.1fx%.1f card=%s", iconW or 0, iconH or 0, card:GetName() or "")
            "#,
        )
        .expect("store button slice geometry should be queryable");

    let (
        left_w,
        left_h,
        left_rect_w,
        left_rect_h,
        middle_w,
        middle_h,
        middle_rect_w,
        middle_rect_h,
        right_w,
        right_h,
        right_rect_w,
        right_rect_h,
        label,
    ) = metrics;

    for (name, w, h) in [
        ("left", left_w, left_h),
        ("left rect", left_rect_w, left_rect_h),
        ("middle", middle_w, middle_h),
        ("middle rect", middle_rect_w, middle_rect_h),
        ("right", right_w, right_h),
        ("right rect", right_rect_w, right_rect_h),
    ] {
        assert!(w <= 250.0, "{name} width too large: {w} ({label})");
        assert!(h <= 80.0, "{name} height too large: {h} ({label})");
    }
}
}

prefork_full_ui_case! {
fn store_notice_show_updates_cover(env: &WowLuaEnv) {
    env.exec(r#"
        StoreFrame:Show()
        StoreFrame.Notice:Hide()
        StoreFrame.Notice:Show()
        assert(StoreFrame.Cover:IsShown(), "notice must show the store cover")
    "#).expect("store notice show updates the cover");
    let errors = env.state().borrow().lua_errors.clone();
    assert!(errors.is_empty(), "store notice must not record Lua errors: {errors:#?}");
}
}

prefork_full_ui_case! {
fn store_product_card_buy_button_click_completes_without_store_upvalue_error(env: &WowLuaEnv) {
    let result: (bool, Option<String>) = env
        .eval(
            r#"
            StoreFrame:Show()
            C_StoreSecure.GetProductList()
            local button
            for activeCard in StoreFrame.productCardPoolCollection:EnumerateActive() do
                if activeCard.BuyButton then
                    button = activeCard.BuyButton
                    break
                end
            end
            assert(button, "expected at least one active store card buy button")

            return pcall(function()
                button:Click()
            end)
            "#,
        )
        .expect("store product card buy button click should return pcall result");

    assert!(
        result.0,
        "store product card buy button click should not throw: {:?}",
        result.1
    );
    let errors = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "store product card buy button click should not record Lua errors: {errors:#?}"
    );
}
}

prefork_full_ui_case! {
fn secure_env_retains_nineslice_disable_sharpening_in_full_game_ui(env: &WowLuaEnv) {
    let (global_disable, secure_disable): (String, String) = env
        .eval(
            r#"
            return type(NineSliceUtil and NineSliceUtil.DisableSharpening),
                type(__secureenv and __secureenv.NineSliceUtil and __secureenv.NineSliceUtil.DisableSharpening)
            "#,
        )
        .expect("NineSliceUtil should be queryable");

    assert_eq!(
        global_disable, "function",
        "global NineSliceUtil.DisableSharpening should exist"
    );
    assert_eq!(
        secure_disable, "function",
        "secureenv should retain NineSliceUtil.DisableSharpening"
    );
}
}

prefork_full_ui_case! {
fn secure_env_preserves_secure_store_free_check_function(env: &WowLuaEnv) {
    let (global_type, secure_type, same_func): (String, String, bool) = env
        .eval(
            r#"
            return type(StoreFrame_CheckForFree),
                type(__secureenv and __secureenv.StoreFrame_CheckForFree),
                __secureenv and __secureenv.StoreFrame_CheckForFree == StoreFrame_CheckForFree
            "#,
        )
        .expect("StoreFrame_CheckForFree should be queryable");

    assert!(
        global_type == "nil" || global_type == "function",
        "global inbound StoreFrame_CheckForFree should be absent or callable, got {global_type}"
    );
    assert_eq!(
        secure_type, "function",
        "secure StoreFrame_CheckForFree should remain bound in secureenv"
    );
    assert!(
        global_type == "nil" || !same_func,
        "secureenv StoreFrame_CheckForFree should not be overwritten by the inbound wrapper"
    );
}
}

prefork_full_ui_case! {
fn store_button_full_game_path_shows_store_frame(env: &WowLuaEnv) {
    let (ok, err, store_shown, isshown_attr, action): (
        bool,
        Option<String>,
        bool,
        bool,
        Option<String>,
    ) = env
        .eval(
            r#"
            local ok, err = pcall(function()
                StoreMicroButton:GetScript("OnClick")(StoreMicroButton, "LeftButton", false)
            end)
            return ok, err, StoreFrame:IsShown() == true, StoreFrame:GetAttribute("isshown") == true, StoreFrame:GetAttribute("action")
            "#,
        )
        .expect("store micro button click should return");

    assert_eq!(action.as_deref(), Some("Show"));
    assert!(store_shown, "StoreFrame should be shown after the store button click");
    assert!(isshown_attr, "OnShow should set isshown=true after the store button click");
    assert!(ok, "store button path should return cleanly now: err={err:?}");
}
}

prefork_full_ui_case! {
fn store_button_full_game_path_fires_onshow(env: &WowLuaEnv) {
    let (store_shown, onshow_count, onhide_count, isshown_attr, action): (
        bool,
        i32,
        i32,
        bool,
        Option<String>,
    ) = env
        .eval(
            r#"
            local onShowCount, onHideCount = 0, 0
            StoreFrame:HookScript("OnShow", function() onShowCount = onShowCount + 1 end)
            StoreFrame:HookScript("OnHide", function() onHideCount = onHideCount + 1 end)

            StoreMicroButton:GetScript("OnClick")(StoreMicroButton, "LeftButton", false)

            return StoreFrame:IsShown() == true,
                onShowCount,
                onHideCount,
                StoreFrame:GetAttribute("isshown") == true,
                StoreFrame:GetAttribute("action")
            "#,
        )
        .expect("store micro button click should return");

    assert_eq!(action.as_deref(), Some("Show"));
    assert!(
        onshow_count >= 1,
        "OnShow should fire when the store button successfully shows the frame"
    );
    assert_eq!(onhide_count, 0, "OnHide should not have fired in the broken path");
    assert!(isshown_attr, "isshown attribute should be true after showing the store");
    assert!(store_shown, "store should be visible after the button click");
}
}

prefork_full_ui_case! {
fn store_frame_set_shown_full_game_path_shows_frame(env: &WowLuaEnv) {
    let result: bool = env
        .eval(
            r#"
            StoreFrame_SetShown(true, "StoreMicroButton")
            return StoreFrame:GetAttribute("action") == "Show" and StoreFrame:IsShown() == true and StoreFrame:GetAttribute("isshown") == true
            "#,
        )
        .expect("direct StoreFrame_SetShown should return cleanly");
    assert!(result, "StoreFrame_SetShown should now show the frame in the full game path");
}
}

prefork_full_ui_case! {
fn store_frame_subtree_has_no_cycles_in_full_game_ui(env: &WowLuaEnv) {
    let state = env.state().borrow();
    let store_id = state.widgets.get_id_by_name("StoreFrame").expect("StoreFrame should exist");
    let mut path = Vec::new();
    let mut seen = HashSet::new();
    assert_no_child_cycle(&state.widgets, store_id, &mut path, &mut seen);
}
}

#[test]
fn store_free_upgrade_check_handles_account_without_upgrade() {
    let env = load_store_ui_tree();
    env.exec(r#"
        assert(type(__secureenv.StoreFrame_CheckForFree) == 'function')
        assert(C_SharedCharacterServices.HasFreePromotionalUpgrade() == false)
        local frame = {}
        __secureenv.StoreFrame_CheckForFree(frame, 'VARIABLES_LOADED')
        __secureenv.StoreFrame_CheckForFree(frame, 'PRODUCT_DISTRIBUTIONS_UPDATED')
        assert(frame.variablesLoaded and frame.distributionsUpdated)
    "#).expect("store free upgrade check should accept an account without an upgrade");
}
