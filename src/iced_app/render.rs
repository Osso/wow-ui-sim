//! Shader rendering implementation.

use iced::mouse;
use iced::widget::shader;
use iced::{Event, Point, Rectangle, Size};

use crate::render::font::WowFontSystem;
use crate::render::glyph::{emit_text_quads, GlyphAtlas};
use crate::render::shader::GLYPH_ATLAS_TEX_INDEX;
use crate::render::texture::UI_SCALE;
use crate::render::{BlendMode, GpuTextureData, QuadBatch, WowUiPrimitive};
use crate::widget::{TextJustify, WidgetType};

use super::app::App;
use super::layout::compute_frame_rect;
use super::state::CanvasMessage;
use super::styles::palette;
use super::Message;

/// Shader program implementation for GPU rendering of WoW frames.
impl shader::Program<Message> for &App {
    type State = ();
    type Primitive = WowUiPrimitive;

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { position } => {
                    if bounds.contains(*position) {
                        let local = Point::new(position.x - bounds.x, position.y - bounds.y);
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseMove(local),
                        )));
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseDown(pos),
                        )));
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseUp(pos),
                        )));
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Middle) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MiddleClick(pos),
                        )));
                    }
                }
                mouse::Event::WheelScrolled { delta } => {
                    let dy = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => *y,
                        mouse::ScrollDelta::Pixels { y, .. } => *y / 30.0,
                    };
                    return Some(shader::Action::publish(Message::Scroll(0.0, dy)));
                }
                _ => {}
            },
            Event::Keyboard(keyboard_event) => {
                use iced::keyboard;
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = keyboard_event {
                    if modifiers.control() && *key == keyboard::Key::Character("r".into()) {
                        return Some(shader::Action::publish(Message::ReloadUI));
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        let start = std::time::Instant::now();

        // Increment frame counter for FPS calculation
        self.frame_count.set(self.frame_count.get() + 1);

        // Update screen size from canvas bounds (used by other functions)
        let size = bounds.size();
        self.screen_size.set(size);
        let mut cache = self.cached_quads.borrow_mut();
        let quads = if self.quads_dirty.get() || cache.as_ref().map(|(s, _)| *s != size).unwrap_or(true) {
            // Rebuild quad batch
            let new_quads = self.build_quad_batch(size);
            *cache = Some((size, new_quads.clone()));
            self.quads_dirty.set(false);
            new_quads
        } else {
            // Use cached quads
            cache.as_ref().unwrap().1.clone()
        };

        // Load ONLY NEW textures (skip ones already uploaded to GPU atlas)
        let mut textures = Vec::new();
        let mut uploaded = self.gpu_uploaded_textures.borrow_mut();
        {
            let mut tex_mgr = self.texture_manager.borrow_mut();
            for request in &quads.texture_requests {
                // Skip if already uploaded to GPU
                if uploaded.contains(&request.path) {
                    continue;
                }
                // Skip if already in this batch
                if textures.iter().any(|t: &GpuTextureData| t.path == request.path) {
                    continue;
                }
                if let Some(tex_data) = tex_mgr.load(&request.path) {
                    textures.push(GpuTextureData {
                        path: request.path.clone(),
                        width: tex_data.width,
                        height: tex_data.height,
                        rgba: tex_data.pixels.clone(),
                    });
                    // Mark as uploaded (will be uploaded in prepare())
                    uploaded.insert(request.path.clone());
                }
            }
        }

        // Update frame time with EMA (alpha = 0.33 for ~5 sample smoothing)
        let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
        self.frame_time_ms.set(elapsed_ms);
        let avg = self.frame_time_avg.get();
        self.frame_time_avg.set(0.33 * elapsed_ms + 0.67 * avg);

        let mut primitive = WowUiPrimitive::with_textures(quads, textures);

        // Include glyph atlas data if it has new glyphs
        {
            let mut ga = self.glyph_atlas.borrow_mut();
            if ga.is_dirty() {
                let (data, size, _) = ga.texture_data();
                primitive.glyph_atlas_data = Some(data.to_vec());
                primitive.glyph_atlas_size = size;
                ga.mark_clean();
            }
        }

        primitive
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

/// Build quads for a Frame widget (backdrop).
pub fn build_frame_quads(batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
    // Draw backdrop if enabled
    if f.backdrop.enabled {
        let bg = &f.backdrop.bg_color;
        batch.push_solid(bounds, [bg.r, bg.g, bg.b, bg.a * f.alpha]);

        // Border
        if f.backdrop.edge_size > 0.0 {
            let bc = &f.backdrop.border_color;
            batch.push_border(bounds, f.backdrop.edge_size.max(1.0), [bc.r, bc.g, bc.b, bc.a * f.alpha]);
        }
    }

    // NineSlice rendering - for now, just draw a placeholder border
    if f.nine_slice_layout.is_some() {
        batch.push_border(bounds, 2.0, [0.6, 0.45, 0.15, f.alpha]);
    }
}

/// Build quads for a Button widget.
pub fn build_button_quads(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    f: &crate::widget::Frame,
    is_pressed: bool,
    is_hovered: bool,
) {
    // If this button has child Texture widgets for NormalTexture/PushedTexture,
    // those children render themselves. Skip rendering here to avoid double-draw.
    let has_normal_child = f.children_keys.contains_key("NormalTexture");
    let has_pushed_child = f.children_keys.contains_key("PushedTexture");

    let (texture_path, tex_coords, skip) = if is_pressed {
        (
            f.pushed_texture.as_ref().or(f.normal_texture.as_ref()),
            f.pushed_tex_coords.or(f.normal_tex_coords),
            if f.pushed_texture.is_some() { has_pushed_child } else { has_normal_child },
        )
    } else {
        (f.normal_texture.as_ref(), f.normal_tex_coords, has_normal_child)
    };

    if !skip {
        emit_button_texture(batch, bounds, texture_path, tex_coords, f.alpha);
    }

    let has_highlight_child = f.children_keys.contains_key("HighlightTexture");
    if is_hovered && !is_pressed && !has_highlight_child {
        emit_button_highlight(batch, bounds, f);
    }
}

/// Render the button's normal/pushed texture (atlas UV or 3-slice).
fn emit_button_texture(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    texture_path: Option<&String>,
    tex_coords: Option<(f32, f32, f32, f32)>,
    alpha: f32,
) {
    let Some(tex_path) = texture_path else { return };
    if let Some((left, right, top, bottom)) = tex_coords {
        let uvs = Rectangle::new(Point::new(left, top), Size::new(right - left, bottom - top));
        batch.push_textured_path_uv(bounds, uvs, tex_path, [1.0, 1.0, 1.0, alpha], BlendMode::Alpha);
    } else {
        const BUTTON_TEX_WIDTH: f32 = 128.0;
        const BUTTON_CAP_WIDTH: f32 = 4.0;
        batch.push_three_slice_h_path(
            bounds, BUTTON_CAP_WIDTH, BUTTON_CAP_WIDTH,
            tex_path, BUTTON_TEX_WIDTH, [1.0, 1.0, 1.0, alpha],
        );
    }
}

/// Render the button highlight overlay on hover.
fn emit_button_highlight(batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
    if let Some(highlight_path) = &f.highlight_texture {
        if let Some((left, right, top, bottom)) = f.highlight_tex_coords {
            let uvs = Rectangle::new(Point::new(left, top), Size::new(right - left, bottom - top));
            batch.push_textured_path_uv(
                bounds, uvs, highlight_path,
                [1.0, 1.0, 1.0, 0.5 * f.alpha], BlendMode::Additive,
            );
        } else {
            const BUTTON_TEX_WIDTH: f32 = 128.0;
            const BUTTON_CAP_WIDTH: f32 = 4.0;
            batch.push_three_slice_h_path_blend(
                bounds, BUTTON_CAP_WIDTH, BUTTON_CAP_WIDTH,
                highlight_path, BUTTON_TEX_WIDTH,
                [1.0, 1.0, 1.0, 0.5 * f.alpha], BlendMode::Additive,
            );
        }
    } else {
        batch.push_quad(
            bounds,
            Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0)),
            [1.0, 0.9, 0.6, 0.15 * f.alpha],
            -1, BlendMode::Additive,
        );
    }
}

/// Build quads for a Texture widget.
pub fn build_texture_quads(batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
    if let Some(color) = f.color_texture {
        batch.push_solid(bounds, [color.r, color.g, color.b, color.a * f.alpha]);
        return;
    }

    let Some(tex_path) = &f.texture else { return };

    if let Some((left, right, top, bottom)) = f.tex_coords {
        let uvs = Rectangle::new(Point::new(left, top), Size::new(right - left, bottom - top));

        if f.horiz_tile || f.vert_tile {
            emit_tiled_texture(batch, bounds, &uvs, tex_path, f);
        } else {
            batch.push_textured_path_uv(bounds, uvs, tex_path, [1.0, 1.0, 1.0, f.alpha], BlendMode::Alpha);
        }
    } else {
        batch.push_textured_path(bounds, tex_path, [1.0, 1.0, 1.0, f.alpha], BlendMode::Alpha);
    }
}

/// Compute tile dimensions from frame size or UV region as fallback.
fn tile_dimensions(f: &crate::widget::Frame, uv_w: f32, uv_h: f32) -> (f32, f32) {
    let tile_w = if f.width > 1.0 { f.width } else { (uv_w * 128.0).max(8.0) };
    let tile_h = if f.height > 1.0 { f.height } else { (uv_h * 128.0).max(8.0) };
    (tile_w, tile_h)
}

/// Emit tiled texture quads (horizontal, vertical, or both).
fn emit_tiled_texture(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    uvs: &Rectangle,
    tex_path: &str,
    f: &crate::widget::Frame,
) {
    let (left, right, top, bottom) = (uvs.x, uvs.x + uvs.width, uvs.y, uvs.y + uvs.height);
    let (tile_w, tile_h) = tile_dimensions(f, right - left, bottom - top);

    if f.horiz_tile && !f.vert_tile {
        emit_horiz_tiles(batch, bounds, uvs, tex_path, tile_w, f.alpha);
    } else if f.vert_tile && !f.horiz_tile {
        emit_vert_tiles(batch, bounds, uvs, tex_path, tile_h, f.alpha);
    } else {
        emit_grid_tiles(batch, bounds, uvs, tex_path, tile_w, tile_h, f.alpha);
    }
}

/// Emit horizontally tiled texture quads.
fn emit_horiz_tiles(batch: &mut QuadBatch, bounds: Rectangle, uvs: &Rectangle, tex_path: &str, tile_w: f32, alpha: f32) {
    let mut x = bounds.x;
    while x < bounds.x + bounds.width {
        let w = (bounds.x + bounds.width - x).min(tile_w);
        let tile_bounds = Rectangle::new(Point::new(x, bounds.y), Size::new(w, bounds.height));
        let uv_w = if w < tile_w { uvs.width * (w / tile_w) } else { uvs.width };
        let tile_uvs = Rectangle::new(uvs.position(), Size::new(uv_w, uvs.height));
        batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, [1.0, 1.0, 1.0, alpha], BlendMode::Alpha);
        x += tile_w;
    }
}

/// Emit vertically tiled texture quads.
fn emit_vert_tiles(batch: &mut QuadBatch, bounds: Rectangle, uvs: &Rectangle, tex_path: &str, tile_h: f32, alpha: f32) {
    let mut y = bounds.y;
    while y < bounds.y + bounds.height {
        let h = (bounds.y + bounds.height - y).min(tile_h);
        let tile_bounds = Rectangle::new(Point::new(bounds.x, y), Size::new(bounds.width, h));
        let uv_h = if h < tile_h { uvs.height * (h / tile_h) } else { uvs.height };
        let tile_uvs = Rectangle::new(uvs.position(), Size::new(uvs.width, uv_h));
        batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, [1.0, 1.0, 1.0, alpha], BlendMode::Alpha);
        y += tile_h;
    }
}

/// Emit grid-tiled texture quads (both horizontal and vertical).
fn emit_grid_tiles(batch: &mut QuadBatch, bounds: Rectangle, uvs: &Rectangle, tex_path: &str, tile_w: f32, tile_h: f32, alpha: f32) {
    let mut y = bounds.y;
    while y < bounds.y + bounds.height {
        let h = (bounds.y + bounds.height - y).min(tile_h);
        let mut x = bounds.x;
        while x < bounds.x + bounds.width {
            let w = (bounds.x + bounds.width - x).min(tile_w);
            let tile_bounds = Rectangle::new(Point::new(x, y), Size::new(w, h));
            let uv_w = if w < tile_w { uvs.width * (w / tile_w) } else { uvs.width };
            let uv_h = if h < tile_h { uvs.height * (h / tile_h) } else { uvs.height };
            let tile_uvs = Rectangle::new(uvs.position(), Size::new(uv_w, uv_h));
            batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, [1.0, 1.0, 1.0, alpha], BlendMode::Alpha);
            x += tile_w;
        }
        y += tile_h;
    }
}

/// Build quads for an EditBox widget.
pub fn build_editbox_quads(batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
    // Skip placeholder if child textures provide the border (e.g. SearchBoxTemplate)
    if !f.children_keys.is_empty() {
        return;
    }
    // Background
    batch.push_solid(bounds, [0.06, 0.06, 0.08, 0.9 * f.alpha]);
    // Border
    batch.push_border(bounds, 1.0, [0.3, 0.25, 0.15, f.alpha]);
}

/// Collect all frame IDs in the subtree rooted at the named frame.
fn collect_subtree_ids(
    registry: &crate::widget::WidgetRegistry,
    root_name: &str,
) -> std::collections::HashSet<u64> {
    let mut ids = std::collections::HashSet::new();
    let root_id = registry.all_ids().into_iter().find(|&id| {
        registry
            .get(id)
            .map(|f| f.name.as_deref() == Some(root_name))
            .unwrap_or(false)
    });
    if let Some(root_id) = root_id {
        let mut queue = vec![root_id];
        while let Some(id) = queue.pop() {
            ids.insert(id);
            if let Some(f) = registry.get(id) {
                queue.extend(f.children.iter().copied());
            }
        }
    }
    ids
}

/// Build set of frame IDs that have a hidden ancestor.
///
/// In WoW, children of hidden frames are not rendered even if their own visible flag is true.
fn build_hidden_ancestors(
    registry: &crate::widget::WidgetRegistry,
) -> std::collections::HashSet<u64> {
    let mut hidden = std::collections::HashSet::new();
    for &id in &registry.all_ids() {
        let mut current = registry.get(id).and_then(|f| f.parent_id);
        while let Some(pid) = current {
            if let Some(parent) = registry.get(pid) {
                if !parent.visible {
                    hidden.insert(id);
                    break;
                }
                current = parent.parent_id;
            } else {
                break;
            }
        }
    }
    hidden
}

/// Collect all frames with computed rects, sorted by strata/level/draw-layer.
fn collect_sorted_frames<'a>(
    registry: &'a crate::widget::WidgetRegistry,
    screen_width: f32,
    screen_height: f32,
) -> Vec<(u64, &'a crate::widget::Frame, crate::LayoutRect)> {
    let mut frames: Vec<_> = registry
        .all_ids()
        .into_iter()
        .filter_map(|id| {
            let f = registry.get(id)?;
            let rect = compute_frame_rect(registry, id, screen_width, screen_height);
            Some((id, f, rect))
        })
        .collect();

    frames.sort_by(|a, b| {
        a.1.frame_strata
            .cmp(&b.1.frame_strata)
            .then_with(|| a.1.frame_level.cmp(&b.1.frame_level))
            .then_with(|| {
                let is_region = |t: &WidgetType| {
                    matches!(t, WidgetType::Texture | WidgetType::FontString)
                };
                match (is_region(&a.1.widget_type), is_region(&b.1.widget_type)) {
                    (true, true) => a.1.draw_layer.cmp(&b.1.draw_layer)
                        .then_with(|| a.1.draw_sub_layer.cmp(&b.1.draw_sub_layer)),
                    (false, true) => std::cmp::Ordering::Less,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, false) => std::cmp::Ordering::Equal,
                }
            })
            .then_with(|| a.0.cmp(&b.0))
    });

    frames
}

/// Emit text quads for a widget, extracting color/shadow from the frame.
fn emit_widget_text_quads(
    batch: &mut QuadBatch,
    font_sys: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    f: &crate::widget::Frame,
    text: &str,
    text_bounds: Rectangle,
    justify_h: TextJustify,
    justify_v: TextJustify,
    word_wrap: bool,
    max_lines: u32,
) {
    let color = [
        f.text_color.r,
        f.text_color.g,
        f.text_color.b,
        f.text_color.a * f.alpha,
    ];
    let shadow = if f.shadow_color.a > 0.0 {
        Some([f.shadow_color.r, f.shadow_color.g, f.shadow_color.b, f.shadow_color.a * f.alpha])
    } else {
        None
    };
    emit_text_quads(
        batch, font_sys, glyph_atlas, text, text_bounds,
        f.font.as_deref(), f.font_size, color,
        justify_h, justify_v,
        GLYPH_ATLAS_TEX_INDEX,
        shadow, f.shadow_offset,
        f.font_outline,
        word_wrap, max_lines,
    );
}

/// Emit quads for a single visible frame based on its widget type.
fn emit_frame_quads(
    batch: &mut QuadBatch,
    id: u64,
    f: &crate::widget::Frame,
    bounds: Rectangle,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
) {
    match f.widget_type {
        WidgetType::Frame => build_frame_quads(batch, bounds, f),
        WidgetType::Button => {
            build_button_quads(batch, bounds, f, pressed_frame == Some(id), hovered_frame == Some(id));
            if let Some((fs, ga)) = text_ctx {
                if let Some(ref txt) = f.text {
                    emit_widget_text_quads(batch, fs, ga, f, txt, bounds, TextJustify::Center, TextJustify::Center, false, 0);
                }
            }
        }
        WidgetType::Texture => build_texture_quads(batch, bounds, f),
        WidgetType::FontString => {
            if let Some((fs, ga)) = text_ctx {
                if let Some(ref txt) = f.text {
                    emit_widget_text_quads(batch, fs, ga, f, txt, bounds, f.justify_h, f.justify_v, f.word_wrap, f.max_lines);
                }
            }
        }
        WidgetType::CheckButton => {
            build_button_quads(batch, bounds, f, pressed_frame == Some(id), hovered_frame == Some(id));
            if let Some((fs, ga)) = text_ctx {
                if let Some(ref txt) = f.text {
                    let label_bounds = Rectangle::new(
                        Point::new(bounds.x + 20.0, bounds.y),
                        Size::new(bounds.width - 20.0, bounds.height),
                    );
                    emit_widget_text_quads(batch, fs, ga, f, txt, label_bounds, TextJustify::Left, TextJustify::Center, false, 0);
                }
            }
        }
        WidgetType::EditBox => {
            build_editbox_quads(batch, bounds, f);
            if let Some((fs, ga)) = text_ctx {
                if let Some(ref txt) = f.text {
                    let padding = 4.0;
                    let text_bounds = Rectangle::new(
                        Point::new(bounds.x + padding, bounds.y),
                        Size::new(bounds.width - padding * 2.0, bounds.height),
                    );
                    emit_widget_text_quads(batch, fs, ga, f, txt, text_bounds, TextJustify::Left, TextJustify::Center, false, 0);
                }
            }
        }
        _ => {}
    }
}

/// Build a QuadBatch from a WidgetRegistry without needing an App instance.
///
/// This contains the sorting/filtering logic from `App::build_quad_batch` but
/// takes a `WidgetRegistry` directly.
///
/// When `text_ctx` is provided, FontString and button/editbox/checkbox text is
/// rendered as glyph quads interleaved with texture quads (correct draw order).
/// When `None`, text is skipped (legacy behavior for callers without fonts).
pub fn build_quad_batch_for_registry(
    registry: &crate::widget::WidgetRegistry,
    screen_size: (f32, f32),
    root_name: Option<&str>,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
    mut text_ctx: Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
) -> QuadBatch {
    let mut batch = QuadBatch::with_capacity(1000);
    let (screen_width, screen_height) = screen_size;
    let size = Size::new(screen_width, screen_height);

    // Add background quad
    batch.push_solid(
        Rectangle::new(Point::ORIGIN, size),
        [palette::BG_DARK.r, palette::BG_DARK.g, palette::BG_DARK.b, 1.0],
    );

    let visible_ids = root_name.map(|name| collect_subtree_ids(registry, name));
    let hidden_ancestors = build_hidden_ancestors(registry);
    let frames = collect_sorted_frames(registry, screen_width, screen_height);

    for (id, f, rect) in frames {
        if let Some(ref ids) = visible_ids {
            if !ids.contains(&id) {
                continue;
            }
        }
        if !f.visible || hidden_ancestors.contains(&id) {
            continue;
        }
        // Skip frames with no dimensions, but allow FontStrings with width=0
        // (they auto-size to text content during rendering)
        let is_fontstring = matches!(f.widget_type, WidgetType::FontString);
        if rect.height <= 0.0 || (rect.width <= 0.0 && !is_fontstring) {
            continue;
        }

        let bounds = Rectangle::new(
            Point::new(rect.x * UI_SCALE, rect.y * UI_SCALE),
            Size::new(rect.width * UI_SCALE, rect.height * UI_SCALE),
        );
        emit_frame_quads(&mut batch, id, f, bounds, pressed_frame, hovered_frame, &mut text_ctx);
    }

    batch
}

impl App {
    /// Build a QuadBatch for GPU shader rendering.
    pub(crate) fn build_quad_batch(&self, size: Size) -> QuadBatch {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let mut font_sys = self.font_system.borrow_mut();
        let mut glyph_atlas = self.glyph_atlas.borrow_mut();
        build_quad_batch_for_registry(
            &state.widgets,
            (size.width, size.height),
            Some("AddonList"),
            self.pressed_frame,
            self.hovered_frame,
            Some((&mut font_sys, &mut glyph_atlas)),
        )
    }

}
