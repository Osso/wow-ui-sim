//! H1 consumer tests. Cache files are inputs, never edited; no native oracle.
use super::test_support::build_test_app;
#[cfg(feature = "client-retail")]
use super::test_support::rebuild_hittable_cache;
use super::*;
use crate::render::{GlyphAtlas, QuadBatch, WowFontSystem};
use crate::screen::ScreenKind;
use iced::{Point, Rectangle, Size};

fn cached_source(profile: &str, relative: &str) -> String {
    // Production cache root (`%LOCALAPPDATA%`-based on Windows), not a POSIX home path.
    let root = crate::blizzard_ui_sync::default_cache_root_path().unwrap_or_else(|error| {
        panic!("Blizzard UI cache root unavailable: {error}; run `wow-cli casc sync-blizzard-ui`")
    });
    let path = root.join(profile).join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn load_consumer(app: &App, retail: bool) {
    let env = app.env.borrow();
    // Supplied dependency fixtures, not replacements for the consumer functions.
    env.exec(
        r#"
        for _, entry in ipairs({
            {"ItemTextFontNormal", 12}, {"UserScaledFontGlueNormalLarge", 24},
            {"UserScaledFontGlueHighlight", 18}
        }) do
            local font = CreateFont(entry[1]); font:SetFont("Fonts\\FRIZQT__.TTF", entry[2])
        end
        NarrationHTMLMixin = {}
        h1Calls = {}
        LaunchURL = function(link) table.insert(h1Calls, link) end
    "#,
    )
    .unwrap();
    let (profile, path) = if retail {
        ("retail", "AddOns/Blizzard_StaticPopup_Glue/GlueDialog.xml")
    } else {
        ("wrath", "FrameXML/ItemTextFrame.xml")
    };
    let source = cached_source(profile, path);
    // Execute the original region declaration, including its fonts and callback.
    let start = source
        .find("<SimpleHTML ")
        .expect("original SimpleHTML region");
    let end = start + source[start..].find("</SimpleHTML>").unwrap() + "</SimpleHTML>".len();
    let fragment = &source[start..end];
    let xml = format!(
        "<Ui><Frame name=\"UserScaledFrameTemplate\" virtual=\"true\"/><Frame name=\"H1FixtureRoot\"><Frames>{fragment}</Frames></Frame></Ui>"
    );
    let temp = tempfile::tempdir().unwrap();
    let xml_path = temp.path().join("consumer.xml");
    std::fs::write(&xml_path, xml).unwrap();
    let toc_path = temp.path().join("H1Fixture.toc");
    std::fs::write(&toc_path, "## AllowLoad: Both\nconsumer.xml\n").unwrap();
    let loaded = crate::loader::load_addon(&env.loader_env(), &toc_path).unwrap();
    assert!(
        loaded.warnings.is_empty(),
        "fixture XML: {:?}",
        loaded.warnings
    );
    let name = {
        let state = env.state().borrow();
        state.widgets.iter_ids().find_map(|id| {
            let frame = state.widgets.get(id).unwrap();
            (frame.widget_type == crate::widget::WidgetType::SimpleHTML)
                .then(|| frame.name.clone())
                .flatten()
        })
    }
    .expect("original SimpleHTML child must be instantiated");
    env.exec(&format!("H1HTML = _G[{name:?}]; assert(H1HTML ~= nil)"))
        .unwrap();
    env.exec(
        r#"
        H1FixtureRoot:SetAllPoints(UIParent)
        H1HTML:ClearAllPoints(); H1HTML:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 20, -20)
        H1HTML:SetSize(270, 304); H1HTML:Show(); H1HTML:EnableMouse(true)
        H1HTML:SetTextColor("h1", 1, 0, 0)
        H1HTML:SetTextColor("h2", 0, 0, 1)
        H1HTML:SetTextColor("h3", 1, 1, 0)
        H1HTML:SetTextColor("p", 0, 1, 0)
    "#,
    )
    .unwrap();
    if !retail {
        // Original ITEM_TEXT_READY handler: ItemTextFrame.lua:37-94, SetText :41-43.
        env.exec(&cached_source("wrath", "FrameXML/ItemTextFrame.lua"))
            .unwrap();
        env.exec(r#"
            ItemTextPageText = H1HTML
            for _, name in ipairs({"ItemTextFrame", "ItemTextScrollFrame", "ItemTextCurrentPage",
                "ItemTextStatusBar", "ItemTextPrevPageButton", "ItemTextNextPageButton",
                "ItemTextMaterialTopLeft", "ItemTextMaterialTopRight", "ItemTextMaterialBotLeft",
                "ItemTextMaterialBotRight"}) do _G[name] = CreateFrame("Frame", name, UIParent) end
            ItemTextScrollFrameScrollBar = CreateFrame("Slider", nil, UIParent)
            ItemTextGetCreator = function() return nil end
            ItemTextGetText = function() return '<h1>Title</h1><p>First paragraph</p><h2>Section</h2><p>Last paragraph</p>' end
            ItemTextGetPage = function() return 1 end
            ItemTextHasNextPage = function() return false end
            ItemTextGetMaterial = function() return 'Parchment' end
            ShowUIPanel = function(frame) frame:Show() end
            ItemTextFrame_OnEvent(ItemTextFrame, "ITEM_TEXT_READY")
        "#).unwrap();
    }
}

fn rendered_html(app: &App) -> QuadBatch {
    let env = app.env.borrow();
    let mut state = env.state().borrow_mut();
    state.ensure_layout_rects();
    let id = state
        .widgets
        .iter_ids()
        .find(|id| {
            state.widgets.get(*id).unwrap().widget_type == crate::widget::WidgetType::SimpleHTML
        })
        .unwrap();
    let widget = state.widgets.get(id).unwrap();
    let rect = widget.layout_rect.unwrap();
    let mut batch = QuadBatch::new();
    let mut fonts = app.font_system.borrow_mut();
    let mut atlas = GlyphAtlas::new();
    crate::iced_app::quad_builders::emit_frame_quads(
        &mut batch,
        &mut Some((&mut fonts, &mut atlas)),
        crate::iced_app::quad_builders::FrameQuadEmit {
            id,
            widget,
            bounds: Rectangle::new(
                Point::new(rect.x, rect.y),
                Size::new(rect.width, rect.height),
            ),
            clip_bounds: None,
            bar_fill: None,
            pressed_frame: None,
            hovered_frame: None,
            message_frames: None,
            tooltip_data: None,
            quest_blobs: None,
            registry: &state.widgets,
            elapsed_secs: 0.0,
            eff_alpha: 1.0,
        },
    );
    batch
}

fn color_span(batch: &QuadBatch, color: [f32; 4]) -> (f32, f32) {
    let ys: Vec<_> = batch
        .vertices
        .iter()
        .filter(|v| v.tex_index == crate::render::shader::GLYPH_ATLAS_TEX_INDEX && v.color == color)
        .map(|v| v.position[1])
        .collect();
    assert!(!ys.is_empty(), "missing configured block color {color:?}");
    (
        ys.iter().copied().fold(f32::INFINITY, f32::min),
        ys.iter().copied().fold(f32::NEG_INFINITY, f32::max),
    )
}

#[cfg(feature = "client-wrath")]
#[test]
fn h1_wrath_original_item_text_preserves_block_geometry() {
    let mut app = build_test_app(ScreenKind::Game);
    app.font_system = std::rc::Rc::new(std::cell::RefCell::new(WowFontSystem::new_without_casc()));
    app.env
        .borrow()
        .set_font_system(std::rc::Rc::clone(&app.font_system));
    load_consumer(&app, false);
    let batch = rendered_html(&app);
    let heading = color_span(&batch, [1.0, 0.0, 0.0, 1.0]);
    let body = color_span(&batch, [0.0, 1.0, 0.0, 1.0]);
    let section = color_span(&batch, [0.0, 0.0, 1.0, 1.0]);
    assert!(heading.1 < body.0);
    assert!(section.0 > heading.1 && section.1 < body.1);
    assert!(
        batch
            .vertices
            .iter()
            .filter(|v| v.color == [0.0, 1.0, 0.0, 1.0])
            .all(|v| v.position[1] < section.0 || v.position[1] > section.1),
        "both paragraph glyph rectangles must stay outside the intervening header"
    );
    let height: f32 = app
        .env
        .borrow()
        .eval("return H1HTML:GetContentHeight()")
        .unwrap();
    assert!(height >= body.1 - heading.0);
    app.env.borrow().exec("H1HTML:SetWidth(45)").unwrap();
    let narrow: f32 = app
        .env
        .borrow()
        .eval("return H1HTML:GetContentHeight()")
        .unwrap();
    assert!(
        narrow > height,
        "content height must track wrapping of separate blocks"
    );
}

#[cfg(feature = "client-retail")]
fn click(app: &mut App, point: Point) {
    app.handle_mouse_down(point);
    app.handle_mouse_up(point);
}

#[cfg(feature = "client-retail")]
#[test]
fn h1_retail_original_glue_callback_from_wrapped_link_glyphs() {
    let mut app = build_test_app(ScreenKind::Login);
    app.font_system = std::rc::Rc::new(std::cell::RefCell::new(WowFontSystem::new_without_casc()));
    app.env
        .borrow()
        .set_font_system(std::rc::Rc::clone(&app.font_system));
    load_consumer(&app, true);
    app.env.borrow().exec(r#"
        H1HTML:SetWidth(90)
        H1HTML:SetText('<h1>Title</h1><p><a href="h1:inert">First second third fourth fifth sixth</a></p><h2>Section</h2><h3>End</h3>')
        H1HTML:HookScript("OnHyperlinkClick", function(self, link, text, button)
            h1Payload = {self == H1HTML, link, text, button}
        end)
    "#).unwrap();
    rebuild_hittable_cache(&app);
    let batch = rendered_html(&app);
    let heading = color_span(&batch, [1.0, 0.0, 0.0, 1.0]);
    let body = color_span(&batch, [0.0, 1.0, 0.0, 1.0]);
    let section = color_span(&batch, [0.0, 0.0, 1.0, 1.0]);
    let end = color_span(&batch, [1.0, 1.0, 0.0, 1.0]);
    assert!(heading.1 < body.0 && body.1 < section.0 && section.1 < end.0);
    assert!(
        heading.1 - heading.0 > section.1 - section.0,
        "original XML font roles must affect glyph geometry"
    );
    let quads: Vec<_> = batch
        .vertices
        .chunks_exact(4)
        .filter(|q| q[0].color == [0.0, 1.0, 0.0, 1.0])
        .map(|q| {
            Point::new(
                (q[0].position[0] + q[2].position[0]) / 2.0,
                (q[0].position[1] + q[2].position[1]) / 2.0,
            )
        })
        .collect();
    assert!(
        quads.iter().any(|p| p.y > quads[0].y + 24.0),
        "link must wrap"
    );
    let html_name: String = app.env.borrow().eval("return H1HTML:GetName()").unwrap();
    let html_id = app
        .env
        .borrow()
        .state()
        .borrow()
        .widgets
        .get_id_by_name(&html_name)
        .unwrap();
    assert_eq!(
        app.hit_test_mouse_button(quads[0], "LeftButton", true),
        Some(html_id),
        "original region must participate in actual mouse routing"
    );
    // Every emitted link glyph must be clickable, including continuation lines.
    for point in &quads {
        click(&mut app, *point);
    }
    let count: i64 = app.env.borrow().eval("return #h1Calls").unwrap();
    assert_eq!(count, i64::try_from(quads.len()).unwrap());
    let payload: (bool, String, String, String) = app
        .env
        .borrow()
        .eval("return h1Payload[1], h1Payload[2], h1Payload[3], h1Payload[4]")
        .unwrap();
    assert_eq!(
        payload,
        (
            true,
            "h1:inert".into(),
            "First second third fourth fifth sixth".into(),
            "LeftButton".into()
        )
    );
    click(&mut app, Point::new(109.0, body.1 + 2.0));
    click(&mut app, Point::new(25.0, heading.0 + 2.0));
    app.env
        .borrow()
        .exec("H1HTML:SetHyperlinksEnabled(false)")
        .unwrap();
    click(&mut app, quads[0]);
    app.env
        .borrow()
        .exec("H1HTML:SetHyperlinksEnabled(true); H1HTML:SetText('<p>replacement</p>')")
        .unwrap();
    click(&mut app, quads[0]);
    assert_eq!(
        app.env.borrow().eval::<i64>("return #h1Calls").unwrap(),
        count
    );
}
