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

#[test]
fn store_right_inset_relative_key_resolves_to_left_inset() {
    let env = load_store_ui_tree();
    let resolved: bool = env
        .eval(
            r#"
            local point, rel, relPoint, x, y = StoreFrame.RightInset:GetPoint(2)
            local same_rect = false
            if rel and StoreFrame.LeftInset then
                same_rect =
                    rel:GetLeft() == StoreFrame.LeftInset:GetLeft()
                    and rel:GetBottom() == StoreFrame.LeftInset:GetBottom()
                    and rel:GetWidth() == StoreFrame.LeftInset:GetWidth()
                    and rel:GetHeight() == StoreFrame.LeftInset:GetHeight()
            end
            return point == "BOTTOMLEFT"
                and rel ~= nil
                and same_rect
                and relPoint == "BOTTOMRIGHT"
                and x == 2
                and y == 0
            "#,
        )
        .expect("RightInset anchor should be queryable");

    assert!(
        resolved,
        "StoreFrame.RightInset second anchor should resolve $parent.LeftInset"
    );
}

#[cfg(feature = "client-retail")]
#[test]
fn store_cards_stay_within_store_frame_after_product_layout() {
    let env = load_store_ui_tree();
    crate::common::store_catalog_fixture::publish_featured_catalog(&env);
    let metrics: (f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            StoreFrame:Show()
            C_StoreSecure.GetProductList()

            local storeLeft, _, storeWidth = StoreFrame:GetLeft(), StoreFrame:GetBottom(), StoreFrame:GetWidth()
            local maxRight = storeLeft + storeWidth
            local cardLeft, cardRight
            for card in StoreFrame.productCardPoolCollection:EnumerateActive() do
                local left = card:GetLeft()
                local right = left + card:GetWidth()
                cardLeft = cardLeft and math.min(cardLeft, left) or left
                cardRight = cardRight and math.max(cardRight, right) or right
            end

            return storeLeft or 0, storeWidth or 0, maxRight or 0, cardLeft or 0, cardRight or 0
            "#,
        )
        .expect("store card bounds should be queryable");

    let (store_left, store_width, max_right, card_left, card_right) = metrics;
    assert!(store_width > 0.0, "store width should be positive");
    assert!(
        card_left >= store_left,
        "leftmost card should not render left of StoreFrame: card_left={card_left}, store_left={store_left}"
    );
    assert!(
        card_right <= max_right,
        "rightmost card should stay within StoreFrame: card_right={card_right}, max_right={max_right}"
    );
}
