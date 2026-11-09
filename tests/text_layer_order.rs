//! Lower-level text must not be deferred past an opaque higher-level sibling.
//! Use the real Lua API, glyph atlas and headless renderer. No Blizzard UI,
//! addon files, CASC textures or reference screenshots are needed.
#![cfg(feature = "gui")]

use crate::common;
use std::cell::RefCell;
use std::rc::Rc;
use wow_ui_sim::iced_app::{RegistryQuadBatchParams, build_quad_batch_for_registry};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::headless::render_to_image;
use wow_ui_sim::render::{GlyphAtlas, WowFontSystem};
use wow_ui_sim::texture::TextureManager;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;

fn render(env: &WowLuaEnv, fonts: &Rc<RefCell<WowFontSystem>>) -> image::RgbaImage {
    let mut glyphs = GlyphAtlas::new();
    let buckets = {
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let batch = {
        let state = env.state().borrow();
        let mut fonts = fonts.borrow_mut();
        build_quad_batch_for_registry(
            RegistryQuadBatchParams::new(&state.widgets, (WIDTH as f32, HEIGHT as f32), &buckets)
                .root_name(Some("LayerHarness"))
                .text_ctx(Some((&mut fonts, &mut glyphs))),
        )
    };
    let (pixels, size, _) = glyphs.texture_data();
    render_to_image(
        &batch,
        &mut TextureManager::new(),
        WIDTH,
        HEIGHT,
        Some((pixels, size)),
    )
}

fn ink_pixels(image: &image::RgbaImage, top: u32, bottom: u32) -> usize {
    // Sample the empty right margin rather than assuming a colour space.
    let background = image.get_pixel(WIDTH - 5, top).0;
    (top..bottom)
        .flat_map(|y| (10..WIDTH - 10).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let pixel = image.get_pixel(x, y).0;
            (0..3).any(|channel| pixel[channel].abs_diff(background[channel]) > 2)
        })
        .count()
}

fn assert_modal_covers_text(wrapper_type: &str) {
    if common::try_create_gpu_device().is_none() {
        eprintln!("Skipping text layer image test: no GPU adapter available");
        return;
    }
    let env = WowLuaEnv::new().expect("create Lua environment");
    let fonts = Rc::new(RefCell::new(WowFontSystem::new_without_casc()));
    env.set_font_system(Rc::clone(&fonts));
    env.set_screen_size(WIDTH as f32, HEIGHT as f32);
    env.exec(&format!("LayerWrapperType = {wrapper_type:?}"))
        .expect("select wrapper type");
    env.exec(
        r#"
        local root = CreateFrame("Frame", "LayerHarness", UIParent)
        root:SetAllPoints(UIParent)
        root:SetFrameStrata("MEDIUM")
        root:SetFrameLevel(100)
        local background = root:CreateTexture(nil, "BACKGROUND")
        background:SetAllPoints()
        background:SetColorTexture(0.05, 0.05, 0.05, 1)

        local wrapper = CreateFrame(LayerWrapperType, nil, root)
        wrapper:SetAllPoints()
        local child = CreateFrame("Frame", nil, wrapper)
        child:SetSize(320, 180)
        if LayerWrapperType == "ScrollFrame" then
            wrapper:SetScrollChild(child)
        else
            child:SetAllPoints()
        end
        local text = child:CreateFontString(nil, "OVERLAY")
        text:SetFont("Fonts\\FRIZQT__.TTF", 24, "")
        text:SetPoint("TOPLEFT", 20, -105)
        text:SetText("UNDERLAY")
        text:SetTextColor(1, 1, 0, 1)

        local cover = CreateFrame("Frame", "LayerCover", root)
        cover:SetFrameLevel(120)
        cover:SetPoint("TOPLEFT", 0, -40)
        cover:SetSize(320, 130)
        local fill = cover:CreateTexture(nil, "BACKGROUND")
        fill:SetAllPoints()
        fill:SetColorTexture(0.12, 0.18, 0.3, 1)
        local caption = cover:CreateFontString(nil, "OVERLAY")
        caption:SetFont("Fonts\\FRIZQT__.TTF", 24, "")
        caption:SetPoint("TOPLEFT", 20, -10)
        caption:SetText("COVER")
        caption:SetTextColor(1, 1, 0, 1)
        assert(child:GetFrameLevel() < cover:GetFrameLevel())
        "#,
    )
    .expect("build layer harness");

    // A missing font or broken text renderer must not make occlusion pass.
    env.exec("LayerCover:Hide()").unwrap();
    let uncovered = render(&env, &fonts);
    assert!(
        ink_pixels(&uncovered, 95, 150) > 30,
        "{wrapper_type}: lower text must be visible without the cover"
    );

    for cycle in 0..2 {
        env.exec("LayerCover:Show()").unwrap();
        let covered = render(&env, &fonts);
        assert_ne!(
            covered.get_pixel(WIDTH - 5, 95),
            uncovered.get_pixel(WIDTH - 5, 95),
            "{wrapper_type}: cover fill must be present"
        );
        assert!(
            ink_pixels(&covered, 45, 85) > 30,
            "{wrapper_type}: cover's own caption must remain visible"
        );
        assert_eq!(
            ink_pixels(&covered, 95, 150),
            0,
            "{wrapper_type}: lower text leaks through opaque cover on cycle {cycle}"
        );
        env.exec("LayerCover:Hide()").unwrap();
        assert!(
            ink_pixels(&render(&env, &fonts), 95, 150) > 30,
            "{wrapper_type}: hiding cover must reveal lower text again"
        );
    }
}

fn assert_same_level_text_after_sibling_background() {
    if common::try_create_gpu_device().is_none() {
        eprintln!("Skipping text layer image test: no GPU adapter available");
        return;
    }
    let env = WowLuaEnv::new().expect("create Lua environment");
    let fonts = Rc::new(RefCell::new(WowFontSystem::new_without_casc()));
    env.set_font_system(Rc::clone(&fonts));
    env.set_screen_size(WIDTH as f32, HEIGHT as f32);
    env.exec(
        r#"
        local root = CreateFrame("Frame", "LayerHarness", UIParent)
        root:SetAllPoints(UIParent)
        root:SetFrameStrata("MEDIUM")
        root:SetFrameLevel(100)

        local wrapper = CreateFrame("Frame", "LayerWrapper", root)
        wrapper:SetAllPoints()
        wrapper:SetFrameLevel(100)
        local textOwner = CreateFrame("Frame", "LayerTextOwner", wrapper)
        textOwner:SetAllPoints()
        textOwner:SetFrameLevel(100)
        local text = textOwner:CreateFontString("LayerLowerText", "OVERLAY")
        text:SetFont("Fonts\\FRIZQT__.TTF", 24, "")
        text:SetPoint("TOPLEFT", 20, -80)
        text:SetText("UNDERLAY")
        text:SetTextColor(1, 1, 0, 1)

        local higher = CreateFrame("Frame", "LayerEmptyHigher", wrapper)
        higher:SetSize(1, 1)
        higher:SetPoint("TOPLEFT")
        higher:SetFrameLevel(110)

        local cover = CreateFrame("Frame", "LayerCover", root)
        cover:SetAllPoints()
        cover:SetFrameLevel(100)
        local fill = cover:CreateTexture("LayerCoverFill", "BACKGROUND")
        fill:SetAllPoints()
        fill:SetColorTexture(0.12, 0.18, 0.3, 1)
        assert(textOwner:GetFrameLevel() == cover:GetFrameLevel())
        "#,
    )
    .expect("build equal-level layer harness");

    let shown = render(&env, &fonts);
    assert!(
        ink_pixels(&shown, 65, 115) > 30,
        "same-level OVERLAY text must render after a sibling BACKGROUND fill"
    );

    env.exec("LayerEmptyHigher:Hide()")
        .expect("hide empty higher-level child");
    let hidden = render(&env, &fonts);
    assert!(
        ink_pixels(&hidden, 65, 115) > 30,
        "hiding an empty higher-level child must not change same-level text ordering"
    );
}

#[test]
fn same_level_overlay_text_stays_above_sibling_background_with_empty_higher_child() {
    assert_same_level_text_after_sibling_background();
}

#[test]
fn opaque_sibling_covers_nested_frame_text() {
    assert_modal_covers_text("Frame");
}

#[test]
fn opaque_sibling_covers_nested_scroll_text() {
    assert_modal_covers_text("ScrollFrame");
}
