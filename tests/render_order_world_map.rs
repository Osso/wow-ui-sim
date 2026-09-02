#![cfg(feature = "gui")]

#[path = "common/blizzard_addon_manifest.rs"]
mod blizzard_addon_manifest;
use crate::common;
mod render_order_support;

use render_order_support::*;
use std::collections::HashSet;
use wow_ui_sim::iced_app::compute_frame_rect;
use wow_ui_sim::render::headless::render_to_image;
use wow_ui_sim::screen::ScreenKind;

#[test]
fn opening_world_map_does_not_darken_the_strip_above_the_panel() {
    let env = env_with_isolated_world_map_ui();

    // The first time the map opens, WorldMapTutorialMixin:CheckAndShowTooltip
    // shows the HelpPlate tutorial tooltip above the "?" button -- that is
    // client behaviour, and it lands in the strip this test watches. Startup
    // in the full simulator has already flipped the bit; do the same here so
    // the comparison is about the map, not about first-run help.
    env.exec(r#"SetCVarBitfield("closedInfoFrames", LE_FRAME_TUTORIAL_WORLD_MAP_FRAME, true)"#)
        .expect("closing the world map tutorial");

    let baseline_batch = build_screenshot_like_batch(&env, 1024, 768, None);
    let mut baseline_mgr = make_texture_manager();
    let baseline_render = render_to_image(&baseline_batch, &mut baseline_mgr, 1024, 768, None);

    open_world_map(&env);

    let world_map_batch = build_screenshot_like_batch(&env, 1024, 768, None);
    let mut world_map_mgr = make_texture_manager();
    let world_map_render = render_to_image(&world_map_batch, &mut world_map_mgr, 1024, 768, None);

    let strip_rect = (80, 0, 820, 80);
    let diffs = diff_pixels_in_rect(&baseline_render, &world_map_render, strip_rect, 8);

    let texture_matches: Vec<_> = world_map_batch
        .texture_requests
        .iter()
        .filter(|request| {
            request_overlaps_rect(
                &world_map_batch,
                request,
                (
                    strip_rect.0 as f32,
                    strip_rect.1 as f32,
                    strip_rect.2 as f32,
                    strip_rect.3 as f32,
                ),
            )
        })
        .map(|request| {
            (
                request.path.as_str(),
                quad_bounds(&world_map_batch, request),
            )
        })
        .collect();

    let solid_matches: Vec<_> = world_map_batch
        .vertices
        .chunks_exact(4)
        .enumerate()
        .filter_map(|(quad_idx, verts)| {
            if verts[0].tex_index != -1 {
                return None;
            }
            let vertex_start = quad_idx * 4;
            let bounds = vertex_range_bounds(&world_map_batch, vertex_start, 4);
            bounds_overlap_rect(
                bounds,
                (
                    strip_rect.0 as f32,
                    strip_rect.1 as f32,
                    strip_rect.2 as f32,
                    strip_rect.3 as f32,
                ),
            )
            .then_some((quad_idx, bounds, verts[0].color, verts[0].flags))
        })
        .collect();

    assert!(
        diffs.is_empty(),
        "world map should not change the strip above its panel; diff_count={} first_diff={:?} textures={texture_matches:#?} solids={solid_matches:#?}",
        diffs.len(),
        diffs.first()
    );
}

#[test]
fn world_map_quest_track_checkboxes_use_high_res_checkbox_atlas() {
    let env = env_with_isolated_world_map();
    let state = env.state().borrow();
    let quest_map_id = state
        .widgets
        .get_id_by_name("QuestMapFrame")
        .expect("QuestMapFrame should exist after opening the world map");

    let checkbox_ids: Vec<u64> = state
        .widgets
        .iter_ids()
        .filter(|&id| {
            let Some(frame) = state.widgets.get(id) else {
                return false;
            };
            frame.atlas.as_deref() == Some("questlog-icon-ticksquare")
                && is_descendant_of(&state.widgets, id, quest_map_id)
        })
        .collect();

    assert!(
        !checkbox_ids.is_empty(),
        "world map quest log should contain questlog-icon-ticksquare textures"
    );

    for id in checkbox_ids {
        let frame = state
            .widgets
            .get(id)
            .expect("checkbox texture should still exist");
        let texture = frame
            .texture
            .as_deref()
            .expect("checkbox texture should resolve to a texture path");
        assert!(
            texture.to_ascii_lowercase().contains("questlogframe2x"),
            "quest log checkbox atlas should prefer the 2x texture path, got {texture}"
        );
        assert_eq!(
            frame.width, 14.0,
            "checkbox texture width should stay logical 14px"
        );
        assert_eq!(
            frame.height, 14.0,
            "checkbox texture height should stay logical 14px"
        );
    }
}

#[test]
fn world_map_quest_track_checkbox_quads_match_texture_layout_bounds() {
    let env = env_with_isolated_world_map();
    let batch = build_screenshot_like_batch(&env, 1024, 768, None);
    let state = env.state().borrow();
    let quest_map_id = state
        .widgets
        .get_id_by_name("QuestMapFrame")
        .expect("QuestMapFrame should exist after opening the world map");

    let checkbox_ids: Vec<u64> = state
        .widgets
        .iter_ids()
        .filter(|&id| {
            let Some(frame) = state.widgets.get(id) else {
                return false;
            };
            frame.atlas.as_deref() == Some("questlog-icon-ticksquare")
                && is_descendant_of(&state.widgets, id, quest_map_id)
        })
        .collect();

    assert!(
        !checkbox_ids.is_empty(),
        "world map quest log should contain questlog-icon-ticksquare textures"
    );

    for id in checkbox_ids {
        let frame = state
            .widgets
            .get(id)
            .expect("checkbox texture should still exist");
        let expected_path = request_path_for_frame_texture(
            frame
                .texture
                .as_deref()
                .expect("checkbox texture should resolve to a texture path"),
            frame.atlas_tex_coords,
        );
        let rect = compute_frame_rect(&state.widgets, id, 1024.0, 768.0);
        let expected_bounds = (rect.x, rect.y, rect.x + rect.width, rect.y + rect.height);

        let matching_bounds: Vec<_> = batch
            .texture_requests
            .iter()
            .filter(|request| request.path == expected_path)
            .map(|request| quad_bounds(&batch, request))
            .filter(|bounds| {
                let width_matches = (bounds.2 - bounds.0 - rect.width).abs() < 0.1;
                let height_matches = (bounds.3 - bounds.1 - rect.height).abs() < 0.1;
                let overlaps = bounds_overlap_rect(
                    *bounds,
                    (rect.x, rect.y, rect.width.max(0.1), rect.height.max(0.1)),
                );
                width_matches && height_matches && overlaps
            })
            .collect();

        assert!(
            !matching_bounds.is_empty(),
            "checkbox quad should match texture layout bounds; id={id} expected_path={expected_path} expected_bounds={expected_bounds:?}"
        );
    }
}

#[test]
fn isolated_world_map_dependency_closure_loads_declared_dependencies() {
    let ui = blizzard_ui_dir();
    let addons =
        discover_blizzard_addon_closure_for_screen(&ui, ScreenKind::Game, &["Blizzard_Channels"]);
    let loaded: HashSet<_> = addons.iter().map(|(name, _)| name.as_str()).collect();

    assert!(
        loaded.contains("Blizzard_SocialToast"),
        "dependency closure should include Blizzard_SocialToast when Blizzard_Channels is requested; loaded={loaded:?}"
    );
}

#[test]
fn voice_chat_prompt_renders_below_world_map_panel_in_combined_stack() {
    let env = env_with_isolated_world_map_ui();
    open_world_map(&env);
    env.exec(
        r#"
        VoiceChatPromptActivateChannel:ClearAllPoints();
        VoiceChatPromptActivateChannel:SetPoint("CENTER", WorldMapFrame, "CENTER", 0, 0);
        VoiceChatPromptActivateChannel:SetAlpha(1);
        VoiceChatPromptActivateChannel:Show();
    "#,
    )
    .expect("failed to show voice chat prompt over world map");
    wow_ui_sim::startup::process_pending_timers(&env);
    wow_ui_sim::startup::fire_one_on_update_tick(&env);

    let buckets = build_strata_buckets(&env);
    let flattened: Vec<u64> = buckets.iter().flatten().copied().collect();
    let state = env.state().borrow();

    let prompt_id = state
        .widgets
        .get_id_by_name("VoiceChatPromptActivateChannel")
        .expect("voice prompt should exist");
    let world_map_id = state
        .widgets
        .get_id_by_name("WorldMapFrame")
        .expect("world map should exist");
    let border_id = state
        .widgets
        .get(world_map_id)
        .and_then(|frame| frame.children_keys.get("BorderFrame"))
        .copied()
        .expect("world map border frame should exist");

    let prompt = state.widgets.get(prompt_id).unwrap();
    let border = state.widgets.get(border_id).unwrap();

    assert_eq!(prompt.frame_strata.as_str(), "LOW");
    assert_eq!(border.frame_strata.as_str(), "HIGH");

    let prompt_pos = flattened
        .iter()
        .position(|&id| id == prompt_id)
        .expect("voice prompt should be in render list");
    let border_pos = flattened
        .iter()
        .position(|&id| id == border_id)
        .expect("world map border should be in render list");

    assert!(
        prompt_pos < border_pos,
        "voice prompt should render before world map border when both overlap; prompt_pos={prompt_pos}, border_pos={border_pos}"
    );
}

#[test]
fn chat_frame_voice_button_overlaps_world_map_but_renders_below_panel_border() {
    let env = env_with_isolated_world_map_ui();
    open_world_map(&env);

    let buckets = build_strata_buckets(&env);
    let flattened: Vec<u64> = buckets.iter().flatten().copied().collect();
    let state = env.state().borrow();

    let world_map_id = state
        .widgets
        .get_id_by_name("WorldMapFrame")
        .expect("world map should exist");
    let border_id = state
        .widgets
        .get(world_map_id)
        .and_then(|frame| frame.children_keys.get("BorderFrame"))
        .copied()
        .expect("world map border frame should exist");
    let voice_button_id = state
        .widgets
        .get_id_by_name("ChatFrameChannelButton")
        .expect("chat voice button should exist");
    let voice_icon_id = state
        .widgets
        .get(voice_button_id)
        .and_then(|frame| frame.children_keys.get("Icon"))
        .copied()
        .expect("chat voice button icon should exist");

    let world_map_rect = compute_frame_rect(&state.widgets, world_map_id, 1024.0, 768.0);
    let voice_button_rect = compute_frame_rect(&state.widgets, voice_button_id, 1024.0, 768.0);
    let voice_icon = state.widgets.get(voice_icon_id).unwrap();
    let border = state.widgets.get(border_id).unwrap();

    assert_eq!(
        voice_icon.atlas.as_deref(),
        Some("chatframe-button-icon-voicechat")
    );
    assert_eq!(border.frame_strata.as_str(), "HIGH");

    let overlaps_horizontally = voice_button_rect.x < world_map_rect.x + world_map_rect.width
        && voice_button_rect.x + voice_button_rect.width > world_map_rect.x;
    let overlaps_vertically = voice_button_rect.y < world_map_rect.y + world_map_rect.height
        && voice_button_rect.y + voice_button_rect.height > world_map_rect.y;
    assert!(
        overlaps_horizontally && overlaps_vertically,
        "chat voice button should overlap world map bounds at 1024x768; button={voice_button_rect:?} map={world_map_rect:?}"
    );

    let voice_button_pos = flattened
        .iter()
        .position(|&id| id == voice_button_id)
        .expect("chat voice button should be in render list");
    let border_pos = flattened
        .iter()
        .position(|&id| id == border_id)
        .expect("world map border should be in render list");

    assert!(
        voice_button_pos < border_pos,
        "chat voice button should render before world map border even though the layouts overlap; button_pos={voice_button_pos}, border_pos={border_pos}"
    );
}
