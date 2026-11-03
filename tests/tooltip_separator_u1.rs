#![cfg(all(feature = "gui", feature = "client-retail"))]

use iced::{Point, Rectangle, Size};
use wow_ui_sim::iced_app::tooltip::{
    TooltipRender, build_tooltip_quads, collect_tooltip_data, prepare_tooltip_layout,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::{GlyphAtlas, QuadBatch, WowFontSystem};

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

fn render_tooltip(env: &WowLuaEnv, scale: f32) -> (QuadBatch, Rectangle) {
    let mut fonts = WowFontSystem::new();
    let mut glyphs = GlyphAtlas::new();
    let mut state = env.state().borrow_mut();
    prepare_tooltip_layout(&mut state, &mut fonts);
    let id = state.widgets.get_id_by_name("U1Tooltip").unwrap();
    let frame = state.widgets.get(id).unwrap();
    let bounds = Rectangle::new(
        Point::new(100.0, 200.0),
        Size::new(frame.width * scale, frame.height * scale),
    );
    let data = collect_tooltip_data(&state);
    let mut batch = QuadBatch::new();
    build_tooltip_quads(
        TooltipRender {
            batch: &mut batch,
            bounds,
            tooltip_data: Some(&data),
            id,
            eff_alpha: 0.4,
            eff_scale: scale,
            draw_background: false,
        },
        &mut Some((&mut fonts, &mut glyphs)),
    );
    (batch, bounds)
}

#[test]
fn retail_u1_original_separator_attaches_and_emits_scaled_divider() {
    let env = WowLuaEnv::new().unwrap();
    load_original_social_helper(&env);
    env.exec(
        r#"
        U1Tooltip = CreateFrame("GameTooltip", "U1Tooltip", UIParent)
        U1Tooltip:SetOwner(UIParent, "ANCHOR_NONE")
        U1Tooltip:SetAlpha(0.4)
        U1Tooltip:AddLine("Header makes the tooltip wider than the divider")
        SocialUIUtil.AddSeparatorToTooltip(U1Tooltip)
        U1Tooltip:AddLine("Footer")
        U1Tooltip:Show()
        assert(U1Tooltip:NumLines() == 3, "attachment must not append a row")
        assert(U1Tooltip:GetLeftLine(2):GetText() == " ")
        assert(U1Tooltip:GetLeftLine(3):GetText() == "Footer")
        "#,
    )
    .unwrap();

    for scale in [1.0, 0.5] {
        let (batch, bounds) = render_tooltip(&env, scale);
        let requests: Vec<_> = batch
            .texture_requests
            .iter()
            .filter(|request| request.path == DIVIDER)
            .collect();
        assert_eq!(requests.len(), 1, "original helper must emit one divider");
        let request = requests[0];
        let start = request.vertex_start as usize;
        let vertices = &batch.vertices[start..start + request.vertex_count as usize];
        let min_x = vertices
            .iter()
            .map(|v| v.position[0])
            .fold(f32::INFINITY, f32::min);
        let min_y = vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::INFINITY, f32::min);
        let max_x = vertices
            .iter()
            .map(|v| v.position[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let max_y = vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        let nine_slice = wow_ui_sim::atlas::get_nine_slice_atlas_info("Tooltip").unwrap();
        let left = 12.0 + (nine_slice.corner_tl.width as f32 - 4.0).max(0.0);
        let right = 12.0 + (nine_slice.corner_tr.width as f32 - 4.0).max(0.0);
        let top = 12.0 + (nine_slice.corner_tl.height as f32 - 4.0).max(0.0);
        let bottom = 12.0 + (nine_slice.corner_bl.height as f32 - 4.0).max(0.0);
        let expected_height = (17.0 + 15.0 + 15.0 + 4.0 + top + bottom) * scale;
        assert!(
            (bounds.height - expected_height).abs() < 0.01,
            "Header, attached row, Footer and two spacings must determine height"
        );
        // Accepted U1 policy: retain the 15-unit blank row, center the
        // 10-unit image (2.5-unit offset), and keep both 2-unit spacings.
        let expected = [
            bounds.x + left * scale,
            bounds.y + (top + 17.0 + 2.0 + 2.5) * scale,
            bounds.x + bounds.width - right * scale,
            10.0 * scale,
        ];
        for (actual, expected) in [min_x, min_y, max_x, max_y - min_y]
            .into_iter()
            .zip(expected)
        {
            assert!(
                (actual - expected).abs() < 0.01,
                "actual={actual}, expected={expected}"
            );
        }
        assert!(
            vertices
                .iter()
                .all(|vertex| (vertex.color[3] - 0.4).abs() < 0.001)
        );
    }

    env.exec("U1Tooltip:ClearLines(); U1Tooltip:AddLine('Replacement'); U1Tooltip:Show()")
        .unwrap();
    let (batch, _) = render_tooltip(&env, 1.0);
    assert!(
        batch
            .texture_requests
            .iter()
            .all(|request| request.path != DIVIDER)
    );
}

#[test]
fn retail_u1_numeric_add_texture_still_appends_independent_rows() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local tooltip = CreateFrame("GameTooltip", "U1Numeric", UIParent)
        tooltip:AddLine("Header")
        tooltip:AddTexture(136243)
        assert(tooltip:NumLines() == 2)
        tooltip:AddTexture("136244", { height = 10, anchor = Enum.TooltipTextureAnchor.All })
        assert(tooltip:NumLines() == 3)
        tooltip:AddLine("Footer")
        assert(tooltip:NumLines() == 4)
        assert(tooltip:GetLeftLine(1):GetText() == "Header")
        assert(tooltip:GetLeftLine(4):GetText() == "Footer")
        "#,
    )
    .unwrap();
}
