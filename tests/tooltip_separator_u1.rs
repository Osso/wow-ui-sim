#![cfg(all(feature = "gui", feature = "client-retail"))]

use wow_ui_sim::iced_app::{
    RegistryQuadBatchParams, build_quad_batch_for_registry, compute_frame_rect,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const DIVIDER: &str = "Interface\\Common\\UI-TooltipDivider-Transparent";

fn load_original_social_helper(env: &WowLuaEnv) {
    // Original cached consumer, not a reconstructed/extracted helper:
    // retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:138-152.
    // :140-142 supplies height=10, All, margins=0,0,-2,-2;
    // :145-150 inserts ONE wrapped blank row, :152 attaches the path.
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Retail Blizzard UI cache must already be synced");
    let path = addons.join("Blizzard_SocialUIShared/SocialUIUtil.lua");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read original consumer {}: {error}", path.display()));
    env.exec(&source)
        .unwrap_or_else(|error| panic!("execute original consumer {}: {error}", path.display()));
}

/// Emitted divider quads as (min x, min y, max x, max y) and their alphas.
fn emitted_dividers(env: &WowLuaEnv) -> Vec<([f32; 4], Vec<f32>)> {
    let mut state = env.state().borrow_mut();
    state.ensure_layout_rects();
    let buckets = state.get_strata_buckets().expect("render buckets").clone();
    let batch = build_quad_batch_for_registry(
        RegistryQuadBatchParams::new(&state.widgets, (1024.0, 768.0), &buckets)
            .root_name(Some("U1Tooltip")),
    );
    batch
        .texture_requests
        .iter()
        .filter(|request| request.path == DIVIDER)
        .map(|request| {
            let start = request.vertex_start as usize;
            let vertices = &batch.vertices[start..start + request.vertex_count as usize];
            let fold = |axis: usize, pick: fn(f32, f32) -> f32, init: f32| {
                vertices
                    .iter()
                    .map(|vertex| vertex.position[axis])
                    .fold(init, pick)
            };
            (
                [
                    fold(0, f32::min, f32::INFINITY),
                    fold(1, f32::min, f32::INFINITY),
                    fold(0, f32::max, f32::NEG_INFINITY),
                    fold(1, f32::max, f32::NEG_INFINITY),
                ],
                vertices.iter().map(|vertex| vertex.color[3]).collect(),
            )
        })
        .collect()
}

#[test]
fn retail_u1_original_separator_spans_its_blank_row() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(1024.0, 768.0);
    load_original_social_helper(&env);
    env.exec(
        r#"
        U1Tooltip = CreateFrame("GameTooltip", "U1Tooltip", UIParent)
        U1Tooltip:SetOwner(UIParent, "ANCHOR_NONE")
        U1Tooltip:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
        U1Tooltip:SetAlpha(0.4)
        U1Tooltip:AddLine("Header makes the tooltip wider than the divider")
        SocialUIUtil.AddSeparatorToTooltip(U1Tooltip)
        U1Tooltip:AddLine("Footer")
        U1Tooltip:Show()
        assert(U1Tooltip:NumLines() == 3, "attachment must not append a row")
        assert(U1Tooltip:GetLeftLine(2):GetText() == " ")
        assert(U1Tooltip:GetLeftLine(3):GetText() == "Footer")
        -- All with zero horizontal margins and negative vertical margins
        -- reserves nothing (WM84 section 1).
        local blank = U1Tooltip:GetLeftLine(2)
        assert(math.abs(select(4, blank:GetPoint(2)) - 10) < 0.01, "no left reservation")
        assert(math.abs(select(5, U1Tooltip:GetLeftLine(3):GetPoint(1)) + 2) < 0.01, "no overflow")
        local h = 0
        for row = 1, 3 do h = h + U1Tooltip:GetLeftLine(row):GetHeight() end
        assert(math.abs(U1Tooltip:GetHeight() - (h + 4 + 20.48)) < 0.01, "three text rows")
        assert(math.abs(blank:GetWidth() - (U1Tooltip:GetWidth() - 20.48)) < 0.01,
            "the wrapped blank row spans the content width")
        "#,
    )
    .unwrap();

    let dividers = emitted_dividers(&env);
    assert_eq!(dividers.len(), 1, "original helper must emit one divider");
    let (bounds, alphas) = &dividers[0];
    assert!(alphas.iter().all(|alpha| (alpha - 0.4).abs() < 0.001));
    // The divider spans the blank FontString's layout rect under one scale.
    let row = {
        let state = env.state().borrow();
        let id = state.widgets.get_id_by_name("U1Tooltip").unwrap();
        let blank = state.tooltips[&id].left_line_ids[1];
        compute_frame_rect(&state.widgets, blank, 1024.0, 768.0)
    };
    let scale = (bounds[2] - bounds[0]) / row.width;
    for (actual, expected) in [
        (bounds[0], row.x * scale),
        (bounds[1], row.y * scale),
        (bounds[3] - bounds[1], row.height * scale),
    ] {
        assert!(
            (actual - expected).abs() < 0.01,
            "actual={actual}, expected={expected}"
        );
    }

    env.exec("U1Tooltip:ClearLines(); U1Tooltip:AddLine('Replacement'); U1Tooltip:Show()")
        .unwrap();
    assert!(emitted_dividers(&env).is_empty(), "cleared attachment is not drawn");
}

#[test]
fn retail_u1_numeric_add_texture_attaches_without_rows() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local tooltip = CreateFrame("GameTooltip", "U1Numeric", UIParent)
        tooltip:AddLine("Header")
        tooltip:AddTexture(136243)
        assert(tooltip:NumLines() == 1)
        tooltip:AddTexture("136244", { height = 10, anchor = Enum.TooltipTextureAnchor.All })
        assert(tooltip:NumLines() == 1)
        tooltip:AddLine("Footer")
        assert(tooltip:NumLines() == 2)
        assert(tooltip:GetLeftLine(1):GetText() == "Header")
        assert(tooltip:GetLeftLine(2):GetText() == "Footer")
        "#,
    )
    .unwrap();
}
