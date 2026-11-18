#![cfg(feature = "gui")]

//! Render order tests: strata bucket ordering and z-order correctness.

use crate::common;
mod render_order_support;

use render_order_support::*;
use wow_ui_sim::iced_app::compute_frame_rect;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::headless::render_to_image;

// ============================================================================
// Higher frame levels render above lower frame levels
// ============================================================================

/// Regions inherit their owning frame's z-order. Draw layers order regions
/// within a frame; they do not let a lower-level sibling cover a higher-level
/// sibling in the same strata.
#[test]
fn higher_level_frame_texture_renders_after_lower_level_content() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local panel = CreateFrame("Frame", "TestPanel", UIParent)
        panel:SetSize(300, 400)
        panel:SetPoint("CENTER")
        panel:Show()

        local background = panel:CreateTexture("TestPanelBg", "BACKGROUND")
        background:SetAllPoints()
        background:SetColorTexture(0.1, 0.1, 0.1, 1)

        local lowFrame = CreateFrame("Frame", "TestLowFrame", panel)
        lowFrame:SetAllPoints()
        lowFrame:SetFrameLevel(1)
        lowFrame:Show()

        local lowTexture = lowFrame:CreateTexture("TestLowTexture", "ARTWORK")
        lowTexture:SetSize(20, 20)
        lowTexture:SetPoint("CENTER")
        lowTexture:SetColorTexture(1, 0, 0, 1)

        local highFrame = CreateFrame("Frame", "TestHighFrame", panel)
        highFrame:SetAllPoints()
        highFrame:SetFrameLevel(100)
        highFrame:Show()

        local highTexture = highFrame:CreateTexture("TestHighTexture", "BORDER")
        highTexture:SetAllPoints()
        highTexture:SetColorTexture(0, 0, 0, 0.8)
    "#,
    )
    .unwrap();

    let buckets = build_strata_buckets(&env);
    let state = env.state().borrow();
    let low_texture_id = state.widgets.get_id_by_name("TestLowTexture").unwrap();
    let high_texture_id = state.widgets.get_id_by_name("TestHighTexture").unwrap();
    let medium_bucket = &buckets[wow_ui_sim::widget::FrameStrata::Medium.as_index()];
    let low_position = medium_bucket
        .iter()
        .position(|&id| id == low_texture_id)
        .expect("low-level texture should be in the MEDIUM strata bucket");
    let high_position = medium_bucket
        .iter()
        .position(|&id| id == high_texture_id)
        .expect("high-level texture should be in the MEDIUM strata bucket");

    assert!(
        high_position > low_position,
        "high-level texture (pos={high_position}) must render after low-level texture \
         (pos={low_position})"
    );
}

#[test]
fn late_created_texture_invalidates_cached_strata_buckets() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "LateBucketParent", UIParent)
        parent:SetSize(80, 80)
        parent:SetPoint("CENTER")
        parent:Show()

        local first = parent:CreateTexture("LateBucketFirstTexture", "ARTWORK")
        first:SetSize(20, 20)
        first:SetPoint("CENTER")
        first:SetColorTexture(1, 0, 0, 1)
    "#,
    )
    .unwrap();

    let _ = build_strata_buckets(&env);

    env.exec(
        r#"
        local second = LateBucketParent:CreateTexture("LateBucketSecondTexture", "OVERLAY")
        second:SetSize(18, 18)
        second:SetPoint("CENTER")
        second:SetColorTexture(0, 1, 0, 1)
    "#,
    )
    .unwrap();

    let buckets = build_strata_buckets(&env);
    let state = env.state().borrow();
    let second_id = state
        .widgets
        .get_id_by_name("LateBucketSecondTexture")
        .expect("late-created texture should exist in widget registry");
    let medium_bucket = &buckets[wow_ui_sim::widget::FrameStrata::Medium.as_index()];

    assert!(
        medium_bucket.contains(&second_id),
        "late-created texture must appear in rebuilt strata bucket after CreateTexture"
    );
}

#[test]
fn late_created_frame_invalidates_cached_strata_buckets() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "LateBucketFrameParent", UIParent)
        parent:SetSize(80, 80)
        parent:SetPoint("CENTER")
        parent:Show()
    "#,
    )
    .unwrap();

    let _ = build_strata_buckets(&env);

    env.exec(
        r#"
        local child = CreateFrame("Frame", "LateBucketChildFrame", LateBucketFrameParent)
        child:SetSize(16, 16)
        child:SetPoint("CENTER")
        child:Show()
    "#,
    )
    .unwrap();

    let buckets = build_strata_buckets(&env);
    let state = env.state().borrow();
    let child_id = state
        .widgets
        .get_id_by_name("LateBucketChildFrame")
        .expect("late-created frame should exist in widget registry");
    let medium_bucket = &buckets[wow_ui_sim::widget::FrameStrata::Medium.as_index()];

    assert!(
        medium_bucket.contains(&child_id),
        "late-created frame must appear in rebuilt strata bucket after CreateFrame"
    );
}

#[test]
fn late_set_draw_layer_invalidates_cached_strata_buckets() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "LateLayerParent", UIParent)
        parent:SetSize(80, 80)
        parent:SetPoint("CENTER")
        parent:Show()

        local circle = parent:CreateTexture("LateLayerCircle", "BACKGROUND")
        circle:SetAllPoints()
        circle:SetColorTexture(1, 0, 0, 1)

        local map = parent:CreateTexture("LateLayerMap", "ARTWORK")
        map:SetAllPoints()
        map:SetColorTexture(0, 1, 0, 1)

        local star = parent:CreateTexture("LateLayerStar", "OVERLAY")
        star:SetAllPoints()
        star:SetColorTexture(0, 0, 1, 1)
    "#,
    )
    .unwrap();

    let _ = build_strata_buckets(&env);

    env.exec(r#"LateLayerCircle:SetDrawLayer("ARTWORK", 1)"#)
        .unwrap();

    let buckets = build_strata_buckets(&env);
    let state = env.state().borrow();
    let medium_bucket = &buckets[wow_ui_sim::widget::FrameStrata::Medium.as_index()];
    let circle_id = state
        .widgets
        .get_id_by_name("LateLayerCircle")
        .expect("circle texture should exist");
    let map_id = state
        .widgets
        .get_id_by_name("LateLayerMap")
        .expect("map texture should exist");
    let star_id = state
        .widgets
        .get_id_by_name("LateLayerStar")
        .expect("star texture should exist");

    let circle_pos = medium_bucket
        .iter()
        .position(|&id| id == circle_id)
        .expect("circle should be in MEDIUM bucket");
    let map_pos = medium_bucket
        .iter()
        .position(|&id| id == map_id)
        .expect("map should be in MEDIUM bucket");
    let star_pos = medium_bucket
        .iter()
        .position(|&id| id == star_id)
        .expect("star should be in MEDIUM bucket");

    assert!(
        map_pos < circle_pos && circle_pos < star_pos,
        "late SetDrawLayer should rebuild region ordering: expected map -> circle -> star, \
         got positions map={map_pos} circle={circle_pos} star={star_pos}"
    );
}

#[test]
fn same_draw_layer_preserves_cached_strata_buckets() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "SameLayerParent", UIParent)
        parent:SetSize(80, 80)
        parent:SetPoint("CENTER")
        parent:Show()

        local tex = parent:CreateTexture("SameLayerTexture", "ARTWORK")
        tex:SetAllPoints()
        tex:SetColorTexture(1, 0, 0, 1)
        tex:SetDrawLayer("OVERLAY", 2)
    "#,
    )
    .unwrap();

    let _ = build_strata_buckets(&env);
    assert!(
        env.state().borrow().strata_buckets.is_some(),
        "building buckets should populate the cache"
    );

    env.exec(r#"SameLayerTexture:SetDrawLayer("OVERLAY", 2)"#)
        .unwrap();

    assert!(
        env.state().borrow().strata_buckets.is_some(),
        "no-op SetDrawLayer should not invalidate cached strata buckets"
    );
}

#[test]
fn same_frame_strata_preserves_cached_strata_buckets() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "SameStrataFrame", UIParent)
        frame:SetSize(80, 80)
        frame:SetPoint("CENTER")
        frame:SetFrameStrata("MEDIUM")
        frame:Show()
    "#,
    )
    .unwrap();

    let _ = build_strata_buckets(&env);
    assert!(
        env.state().borrow().strata_buckets.is_some(),
        "building buckets should populate the cache"
    );

    env.exec(r#"SameStrataFrame:SetFrameStrata("MEDIUM")"#)
        .unwrap();

    assert!(
        env.state().borrow().strata_buckets.is_some(),
        "no-op SetFrameStrata should not invalidate cached strata buckets"
    );
}

#[test]
fn replacing_existing_texture_preserves_cached_strata_buckets() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "ReplaceTextureParent", UIParent)
        parent:SetSize(80, 80)
        parent:SetPoint("CENTER")
        parent:Show()

        local icon = parent:CreateTexture("ReplaceTextureIcon", "BACKGROUND")
        icon:SetAllPoints()
        icon:SetTexture("Interface\\Buttons\\UI-Quickslot2")
    "#,
    )
    .unwrap();

    let _ = build_strata_buckets(&env);
    assert!(
        env.state().borrow().strata_buckets.is_some(),
        "building buckets should populate the cache"
    );

    env.exec(r#"ReplaceTextureIcon:SetTexture("Interface\\Buttons\\UI-Quickslot")"#)
        .unwrap();

    assert!(
        env.state().borrow().strata_buckets.is_some(),
        "replacing an existing texture source should not rebuild strata buckets"
    );
}

#[test]
fn set_texture_moves_region_after_existing_same_layer_siblings() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "SetTextureOrderParent", UIParent)
        parent:SetSize(80, 80)
        parent:SetPoint("CENTER")
        parent:Show()

        local icon = parent:CreateTexture("SetTextureOrderIcon", "BACKGROUND")
        icon:SetAllPoints()

        local slot = parent:CreateTexture("SetTextureOrderSlot", "BACKGROUND")
        slot:SetAllPoints()
        slot:SetColorTexture(0.1, 0.1, 0.1, 1)
    "#,
    )
    .unwrap();

    let _ = build_strata_buckets(&env);

    env.exec(r#"SetTextureOrderIcon:SetTexture("Interface\\Buttons\\UI-Quickslot2")"#)
        .unwrap();

    let buckets = build_strata_buckets(&env);
    let state = env.state().borrow();
    let medium_bucket = &buckets[wow_ui_sim::widget::FrameStrata::Medium.as_index()];
    let icon_id = state
        .widgets
        .get_id_by_name("SetTextureOrderIcon")
        .expect("icon texture should exist");
    let slot_id = state
        .widgets
        .get_id_by_name("SetTextureOrderSlot")
        .expect("slot texture should exist");
    let icon_pos = medium_bucket
        .iter()
        .position(|&id| id == icon_id)
        .expect("icon should be in MEDIUM bucket after SetTexture");
    let slot_pos = medium_bucket
        .iter()
        .position(|&id| id == slot_id)
        .expect("slot should be in MEDIUM bucket");

    assert!(
        icon_pos > slot_pos,
        "SetTexture should move the updated texture after existing same-layer siblings: \
         got icon={icon_pos} slot={slot_pos}"
    );
}

#[test]
fn late_set_frame_level_invalidates_cached_strata_buckets() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "LateLevelParent", UIParent)
        parent:SetSize(80, 80)
        parent:SetPoint("CENTER")
        parent:Show()

        local circleFrame = CreateFrame("Frame", "LateLevelCircleFrame", parent)
        circleFrame:SetAllPoints()
        circleFrame:SetFrameLevel(1)
        circleFrame:Show()

        local circle = circleFrame:CreateTexture("LateLevelCircle", "ARTWORK")
        circle:SetAllPoints()
        circle:SetColorTexture(1, 0, 0, 1)

        local mapFrame = CreateFrame("Frame", "LateLevelMapFrame", parent)
        mapFrame:SetAllPoints()
        mapFrame:SetFrameLevel(2)
        mapFrame:Show()

        local map = mapFrame:CreateTexture("LateLevelMap", "ARTWORK")
        map:SetAllPoints()
        map:SetColorTexture(0, 1, 0, 1)

        local starFrame = CreateFrame("Frame", "LateLevelStarFrame", parent)
        starFrame:SetAllPoints()
        starFrame:SetFrameLevel(4)
        starFrame:Show()

        local star = starFrame:CreateTexture("LateLevelStar", "ARTWORK")
        star:SetAllPoints()
        star:SetColorTexture(0, 0, 1, 1)
    "#,
    )
    .unwrap();

    let _ = build_strata_buckets(&env);

    env.exec(r#"LateLevelCircleFrame:SetFrameLevel(3)"#)
        .unwrap();

    let buckets = build_strata_buckets(&env);
    let state = env.state().borrow();
    let medium_bucket = &buckets[wow_ui_sim::widget::FrameStrata::Medium.as_index()];
    let circle_frame_id = state
        .widgets
        .get_id_by_name("LateLevelCircleFrame")
        .expect("circle frame should exist");
    let map_frame_id = state
        .widgets
        .get_id_by_name("LateLevelMapFrame")
        .expect("map frame should exist");
    let star_frame_id = state
        .widgets
        .get_id_by_name("LateLevelStarFrame")
        .expect("star frame should exist");

    let circle_pos = medium_bucket
        .iter()
        .position(|&id| id == circle_frame_id)
        .expect("circle frame should be in MEDIUM bucket");
    let map_pos = medium_bucket
        .iter()
        .position(|&id| id == map_frame_id)
        .expect("map frame should be in MEDIUM bucket");
    let star_pos = medium_bucket
        .iter()
        .position(|&id| id == star_frame_id)
        .expect("star frame should be in MEDIUM bucket");

    assert!(
        map_pos < circle_pos && circle_pos < star_pos,
        "late SetFrameLevel should rebuild frame ordering: expected map -> circle -> star, \
         got positions map={map_pos} circle={circle_pos} star={star_pos}"
    );
}

post_wrath_ui_fixture! {
#[test]
fn isolated_world_map_stack_opens_and_populates_world_quest_pins() {
    common::with_timeout(120, move || {
        let env = env_with_isolated_world_map();

        std::thread::sleep(std::time::Duration::from_millis(1200));
        wow_ui_sim::startup::process_pending_timers(&env);
        wow_ui_sim::startup::fire_one_on_update_tick(&env);

        let world_map_shown: bool = env
            .eval("return WorldMapFrame ~= nil and WorldMapFrame:IsShown() == true")
            .expect("should query WorldMapFrame visibility");
        assert!(
            world_map_shown,
            "isolated world map stack should show WorldMapFrame"
        );

        let state = env.state().borrow();
        let loaded_addons: Vec<_> = state
            .addons
            .iter()
            .filter(|addon| addon.is_loaded())
            .map(|addon| addon.folder_name.clone())
            .collect();
        let world_quest_pairs = world_quest_pin_pairs(&state);
        eprintln!(
            "isolated world map loaded {} addons: {:?}",
            loaded_addons.len(),
            loaded_addons
        );
        eprintln!(
            "isolated world map visible world quest pin pairs={}",
            world_quest_pairs.len()
        );

        assert!(
            !world_quest_pairs.is_empty(),
            "isolated world map stack should still populate visible world quest pins"
        );
    });
}
}

#[test]
fn isolated_world_map_current_map_does_not_render_fog_of_war_without_db_entry() {
    common::with_timeout(120, move || {
        let env = env_with_isolated_world_map();

        std::thread::sleep(std::time::Duration::from_millis(1200));
        wow_ui_sim::startup::process_pending_timers(&env);
        wow_ui_sim::startup::fire_one_on_update_tick(&env);

        let map_rect = {
            let state = env.state().borrow();
            let world_map_id = state
                .widgets
                .get_id_by_name("WorldMapFrame")
                .expect("isolated world map should create WorldMapFrame");
            let fog_pin_id = state
                .widgets
                .iter_ids()
                .find(|&id| {
                    state.widgets.get(id).is_some_and(|frame| {
                        frame
                            .object_type_name
                            .as_deref()
                            .is_some_and(|name| name.eq_ignore_ascii_case("FogOfWarFrame"))
                            && is_descendant_of(&state.widgets, id, world_map_id)
                    })
                })
                .expect("isolated world map should create a FogOfWarFrame pin");
            compute_frame_rect(&state.widgets, fog_pin_id, 1024.0, 768.0)
        };
        let (explored_right_x, explored_right_y, unexplored_left_x, unexplored_left_y) =
            exploration_sample_points(&env);

        let mut visible_mgr = make_texture_manager();
        let visible_batch = build_screenshot_like_batch(&env, 1024, 768, Some("WorldMapFrame"));
        let visible_render = render_to_image(&visible_batch, &mut visible_mgr, 1024, 768, None);

        env.exec(
            r#"
            local fogPin = WorldMapFrame:EnumeratePinsByTemplate("FogOfWarPinTemplate")()
            assert(fogPin, "missing fog pin")
            fogPin:Hide()
        "#,
        )
        .expect("failed to hide fog pin");
        wow_ui_sim::startup::process_pending_timers(&env);
        wow_ui_sim::startup::fire_one_on_update_tick(&env);

        let mut hidden_mgr = make_texture_manager();
        let hidden_batch = build_screenshot_like_batch(&env, 1024, 768, Some("WorldMapFrame"));
        let hidden_render = render_to_image(&hidden_batch, &mut hidden_mgr, 1024, 768, None);

        let explored_right_pixel = sample_map_pixel(
            &visible_render,
            &hidden_render,
            map_rect,
            explored_right_x,
            explored_right_y,
        );
        let unexplored_left_pixel = sample_map_pixel(
            &visible_render,
            &hidden_render,
            map_rect,
            unexplored_left_x,
            unexplored_left_y,
        );

        let explored_delta = pixel_delta(explored_right_pixel.0, explored_right_pixel.1);
        let unexplored_delta = pixel_delta(unexplored_left_pixel.0, unexplored_left_pixel.1);

        assert!(
            explored_delta <= 24,
            "maps without fog DB data should not darken explored regions: delta={explored_delta} sample=({explored_right_x:.2}, {explored_right_y:.2}) visible={:?} hidden={:?}",
            explored_right_pixel.0,
            explored_right_pixel.1
        );
        assert!(
            unexplored_delta <= 24,
            "maps without fog DB data should not darken unexplored regions either: delta={unexplored_delta} sample=({unexplored_left_x:.2}, {unexplored_left_y:.2}) visible={:?} hidden={:?}",
            unexplored_left_pixel.0,
            unexplored_left_pixel.1
        );
    });
}

fn exploration_sample_points(env: &wow_ui_sim::lua_api::WowLuaEnv) -> (f32, f32, f32, f32) {
    env.eval(
        r#"
        local mapID = C_Map.GetCurrentMapID()
        local exploredRightX, exploredRightY
        local unexploredLeftX, unexploredLeftY

        for yi = 5, 95, 5 do
            local y = yi / 100
            for xi = 5, 95, 5 do
                local x = xi / 100
                local isExplored = #C_MapExplorationInfo.GetExploredAreaIDsAtPosition(
                    mapID,
                    CreateVector2D(x, y)
                ) > 0
                if isExplored and x > 0.55 and not exploredRightX then
                    exploredRightX, exploredRightY = x, y
                end
                if not isExplored and x < 0.45 and not unexploredLeftX then
                    unexploredLeftX, unexploredLeftY = x, y
                end
            end
        end

        assert(exploredRightX and exploredRightY, "missing explored sample point on the right side")
        assert(unexploredLeftX and unexploredLeftY, "missing unexplored sample point on the left side")
        return exploredRightX, exploredRightY, unexploredLeftX, unexploredLeftY
    "#,
    )
    .expect("failed to locate exploration sample points")
}

fn sample_map_pixel(
    visible: &image::RgbaImage,
    hidden: &image::RgbaImage,
    map_rect: wow_ui_sim::LayoutRect,
    normalized_x: f32,
    normalized_y: f32,
) -> ([u8; 4], [u8; 4]) {
    let sample_x = (map_rect.x + map_rect.width * normalized_x)
        .round()
        .clamp(0.0, visible.width().saturating_sub(1) as f32) as u32;
    let sample_y = (map_rect.y + map_rect.height * normalized_y)
        .round()
        .clamp(0.0, visible.height().saturating_sub(1) as f32) as u32;
    (
        visible.get_pixel(sample_x, sample_y).0,
        hidden.get_pixel(sample_x, sample_y).0,
    )
}

fn pixel_delta(lhs: [u8; 4], rhs: [u8; 4]) -> u32 {
    u32::from(lhs[0].abs_diff(rhs[0]))
        + u32::from(lhs[1].abs_diff(rhs[1]))
        + u32::from(lhs[2].abs_diff(rhs[2]))
}

post_wrath_ui_fixture! {
#[test]
fn isolated_world_map_seeded_world_quests_do_not_show_expiration_clock() {
    common::with_timeout(120, move || {
        let env = env_with_isolated_world_map();

        std::thread::sleep(std::time::Duration::from_millis(1200));
        wow_ui_sim::startup::process_pending_timers(&env);
        wow_ui_sim::startup::fire_one_on_update_tick(&env);

        let lua_probe: String = env
            .eval(
                r#"
                local pin = WorldMapFrame and WorldMapFrame:EnumeratePinsByTemplate("WorldMap_WorldQuestPinTemplate")()
                return string.format(
                    "seconds=%s low=%s critical=%s timelow=%s",
                    tostring(C_TaskQuest.GetQuestTimeLeftSeconds(90101)),
                    tostring(QuestUtils_IsQuestWithinLowTimeThreshold(90101)),
                    tostring(QuestUtils_IsQuestWithinCriticalTimeThreshold(90101)),
                    tostring(pin and pin.TimeLowFrame and pin.TimeLowFrame:IsShown())
                )
                "#,
            )
            .expect("lua probe should run");
        eprintln!("{lua_probe}");

        let state = env.state().borrow();
        let visible_clock_icons: Vec<_> = state
            .widgets
            .iter_ids()
            .filter(|&id| {
                let Some(frame) = state.widgets.get(id) else {
                    return false;
                };
                frame.effective_alpha > 0.0
                    && frame.atlas.as_deref() == Some("worldquest-icon-clock")
            })
            .collect();

        assert!(
            visible_clock_icons.is_empty(),
            "seeded world quests default to 120 minutes left, so expiration clocks should stay hidden; visible clock ids={visible_clock_icons:?}"
        );
    });
}
}

post_wrath_ui_fixture! {
#[test]
fn isolated_world_map_world_quest_circle_keeps_atlas_size() {
    common::with_timeout(120, move || {
        let env = env_with_isolated_world_map();

        std::thread::sleep(std::time::Duration::from_millis(1200));
        wow_ui_sim::startup::process_pending_timers(&env);
        wow_ui_sim::startup::fire_one_on_update_tick(&env);

        let state = env.state().borrow();
        let (_circle_id, _icon_id, circle_rect, _circle_request_path, _icon_request_path) =
            world_quest_pin_pairs(&state)
                .into_iter()
                .next()
                .expect("isolated world map should have at least one world quest pair");

        assert!(
            (circle_rect.width - 32.0).abs() <= 0.1 && (circle_rect.height - 32.0).abs() <= 0.1,
            "world quest NormalTexture should keep its 32x32 atlas-sized rect, got {circle_rect:?}"
        );
    });
}
}
