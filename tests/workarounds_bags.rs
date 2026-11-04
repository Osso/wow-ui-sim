use std::path::PathBuf;
use std::time::Duration;

#[cfg(not(feature = "client-wrath"))]
#[path = "common/token_ui_fixtures.rs"]
mod token_ui_fixtures;

#[cfg(not(feature = "client-wrath"))]
use token_ui_fixtures::load_token_ui;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::{WowLuaEnv, compute_frame_rect};
use wow_ui_sim::startup::{fire_one_on_update_tick, fire_startup_events, process_pending_timers};

const TEST_SCREEN_WIDTH: f32 = 1600.0;
const TEST_SCREEN_HEIGHT: f32 = 1200.0;
const RECT_QUERY_TOLERANCE: f32 = 0.1;

#[derive(Debug)]
struct FrameRectQuery {
    left: f32,
    bottom: f32,
    width: f32,
    height: f32,
}

#[derive(Debug)]
struct FrameRectQueries {
    scaled: FrameRectQuery,
    unscaled: FrameRectQuery,
    effective_scale: f32,
}

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn full_game_env_after_edit_mode_init() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(TEST_SCREEN_WIDTH, TEST_SCREEN_HEIGHT);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        // Wrath FrameXML.toc:105-107 loads ContainerFrame.xml and
        // MainMenuBarBagButtons.xml before split addons. ContainerFrame.lua:56-75
        // opens the original bag frames; no ContainerFrameSettingsManager exists.
        crate::common::load_profile_framexml(&env);
    } else {
        load_all_blizzard_addons(&env);
    }
    settle_env_after_edit_mode_init(&env);

    env
}

fn load_all_blizzard_addons(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    for (name, toc_path) in &discover_blizzard_addons(&ui) {
        if let Err(err) = load_addon(&env.loader_env(), toc_path) {
            panic!("[load {name}] FAILED: {err}");
        }
    }
}

fn settle_env_after_edit_mode_init(env: &WowLuaEnv) {
    env.apply_post_load_workarounds();
    fire_startup_events(env);
    env.apply_post_event_workarounds();
    env.state().borrow_mut().widgets.rebuild_anchor_index();
    process_pending_timers(env);
    fire_one_on_update_tick(env);
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(&*env.rilua());

    std::thread::sleep(Duration::from_secs(2));
    for _ in 0..3 {
        env.state().borrow_mut().ensure_layout_rects();
        fire_one_on_update_tick(env);
        process_pending_timers(env);
    }
}

#[cfg(not(feature = "client-wrath"))]
fn bag_bar_anchor(env: &WowLuaEnv) -> (String, String, String, f32, f32) {
    env.eval(
        r#"
        local point, rel, relPoint, x, y = BagsBar:GetPoint(1)
        return point, rel:GetName(), relPoint, x, y
        "#,
    )
    .unwrap()
}

fn frame_rect(env: &WowLuaEnv, name: &str) -> (f32, f32, f32, f32) {
    let state = env.state().borrow();
    let id = state
        .widgets
        .get_id_by_name(name)
        .unwrap_or_else(|| panic!("Frame '{name}' not found"));
    let rect = compute_frame_rect(&state.widgets, id, TEST_SCREEN_WIDTH, TEST_SCREEN_HEIGHT);
    (rect.x, rect.y, rect.width, rect.height)
}

fn visible_bag_frame_name(env: &WowLuaEnv) -> String {
    env.eval(
        r#"
        if ContainerFrameCombinedBags and ContainerFrameCombinedBags:IsShown() then
            return "ContainerFrameCombinedBags"
        end
        for i = 1, 6 do
            local frame = _G["ContainerFrame" .. i]
            if frame and frame:IsShown() then
                return frame:GetName()
            end
        end
        error("no visible bag frame")
        "#,
    )
    .unwrap()
}

fn queried_frame_rects(env: &WowLuaEnv, name: &str) -> FrameRectQueries {
    let rects: (f32, f32, f32, f32, f32, f32, f32, f32, f32) = env
        .eval(&format!(
            r#"
            local frame = assert(_G[{name:?}], "missing frame: {name}")
            local sl, sb, sw, sh = frame:GetScaledRect()
            local l, b, w, h = frame:GetRect()
            return sl, sb, sw, sh, l, b, w, h, frame:GetEffectiveScale()
            "#
        ))
        .unwrap();

    FrameRectQueries {
        scaled: FrameRectQuery {
            left: rects.0,
            bottom: rects.1,
            width: rects.2,
            height: rects.3,
        },
        unscaled: FrameRectQuery {
            left: rects.4,
            bottom: rects.5,
            width: rects.6,
            height: rects.7,
        },
        effective_scale: rects.8,
    }
}

fn expected_frame_rects(env: &WowLuaEnv, name: &str, effective_scale: f32) -> FrameRectQueries {
    let (render_x, render_y, render_width, render_height) = frame_rect(env, name);
    let render_bottom = TEST_SCREEN_HEIGHT - render_y - render_height;

    FrameRectQueries {
        scaled: FrameRectQuery {
            left: render_x,
            bottom: render_bottom,
            width: render_width,
            height: render_height,
        },
        unscaled: FrameRectQuery {
            left: render_x / effective_scale,
            bottom: render_bottom / effective_scale,
            width: render_width / effective_scale,
            height: render_height / effective_scale,
        },
        effective_scale,
    }
}

fn assert_rect_component_matches(
    name: &str,
    rect_name: &str,
    component: &str,
    expected: f32,
    actual: f32,
) {
    assert!(
        (actual - expected).abs() <= RECT_QUERY_TOLERANCE,
        "{name} {rect_name} {component} mismatch: expected {expected}, got {actual}"
    );
}

fn assert_rect_matches(
    name: &str,
    rect_name: &str,
    actual: &FrameRectQuery,
    expected: &FrameRectQuery,
) {
    assert_rect_component_matches(name, rect_name, "left", expected.left, actual.left);
    assert_rect_component_matches(name, rect_name, "bottom", expected.bottom, actual.bottom);
    assert_rect_component_matches(name, rect_name, "width", expected.width, actual.width);
    assert_rect_component_matches(name, rect_name, "height", expected.height, actual.height);
}

fn assert_rect_queries_match_layout_rect(env: &WowLuaEnv, name: &str) {
    env.state().borrow_mut().ensure_layout_rects();

    let actual = queried_frame_rects(env, name);
    let expected = expected_frame_rects(env, name, actual.effective_scale);

    assert_rect_matches(name, "GetScaledRect", &actual.scaled, &expected.scaled);
    assert_rect_matches(name, "GetRect", &actual.unscaled, &expected.unscaled);
}

// Wrath FrameXML/MainMenuBarBagButtons.xml:79-90 anchors the backpack directly
// to MainMenuBarArtFrame. BagsBar is a Mainline edit-mode frame
// (Blizzard_MainMenuBarBagButtons/Mainline/MainMenuBarBagButtons.xml:44).
#[cfg(not(feature = "client-wrath"))]
#[test]
fn bags_bar_reanchor_replay_moves_the_bar_off_its_correct_rect() {
    let env = full_game_env_after_edit_mode_init();

    let before_anchor = bag_bar_anchor(&env);
    assert_eq!(
        (
            before_anchor.0.clone(),
            before_anchor.1.clone(),
            before_anchor.2.clone(),
        ),
        (
            "TOPRIGHT".to_string(),
            "MicroButtonAndBagsBar".to_string(),
            "TOPRIGHT".to_string(),
        )
    );

    let before_rect = frame_rect(&env, "BagsBar");
    assert_eq!(before_rect, (1386.0, 1104.0, 208.0, 47.0));

    env.exec(
        r#"
        BagsBar:ClearAllPoints()
        BagsBar:SetPoint("TOPRIGHT", MicroButtonAndBagsBar, "TOPRIGHT", 0, 0)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().widgets.rebuild_anchor_index();
    env.state().borrow_mut().ensure_layout_rects();

    let after_anchor = bag_bar_anchor(&env);
    assert_eq!(
        after_anchor,
        (
            "TOPRIGHT".to_string(),
            "MicroButtonAndBagsBar".to_string(),
            "TOPRIGHT".to_string(),
            0.0,
            0.0,
        )
    );
    let after_rect = frame_rect(&env, "BagsBar");
    assert_eq!(after_rect, (1386.0, 1114.0, 208.0, 47.0));
    assert_ne!(after_rect, before_rect);
}

// Wrath FrameXML/MainMenuBarBagButtons.xml:17-76 uses legacy ItemButtonTemplate
// callbacks, not MainMenuBarBagManager or item-context overlays.
#[cfg(not(feature = "client-wrath"))]
#[test]
fn main_menu_bag_buttons_have_correct_item_context_without_overlay_replay() {
    let env = full_game_env_after_edit_mode_init();

    let (
        util_exists,
        bag_match_is_dna,
        backpack_match_is_dna,
        bag_overlay_hidden,
        backpack_overlay_hidden,
    ): (bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local dna = ItemButtonUtil and ItemButtonUtil.ItemContextMatchResult
                and ItemButtonUtil.ItemContextMatchResult.DoesNotApply
            local bag = MainMenuBarBagManager and MainMenuBarBagManager.allBagButtons
                and MainMenuBarBagManager.allBagButtons[1]
            return type(ItemButtonUtil) == "table",
                bag and bag.itemContextMatchResult == dna or false,
                MainMenuBarBackpackButton.itemContextMatchResult == dna,
                bag and bag:GetItemContextOverlayMode() == nil or false,
                MainMenuBarBackpackButton:GetItemContextOverlayMode() == nil
            "#,
        )
        .unwrap();
    assert!(util_exists);
    assert!(bag_match_is_dna);
    assert!(backpack_match_is_dna);
    assert!(bag_overlay_hidden);
    assert!(backpack_overlay_hidden);
}

#[test]
fn bag_frame_rect_queries_match_render_layout_rects() {
    let env = full_game_env_after_edit_mode_init();
    #[cfg(not(feature = "client-wrath"))]
    load_token_ui(&env);

    // Wrath Bindings.xml:629-631 calls OpenAllBags (ContainerFrame.lua:794),
    // not the Mainline ToggleAllBags global. Both paths open original consumers.
    let open_bags = if wow_ui_sim::client_profile::ACTIVE
        == wow_ui_sim::client_profile::ClientProfile::Wrath
    {
        "OpenAllBags()"
    } else {
        "ToggleAllBags()"
    };
    env.exec(open_bags).expect("original bag UI should open");
    env.state().borrow_mut().ensure_layout_rects();

    let visible_bag_frame = visible_bag_frame_name(&env);

    assert_rect_queries_match_layout_rect(&env, "CharacterBag0Slot");
    assert_rect_queries_match_layout_rect(&env, "MainMenuBarBackpackButton");
    assert_rect_queries_match_layout_rect(&env, &visible_bag_frame);
}
