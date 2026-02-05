//! Shader and canvas rendering implementations.

use iced::mouse;
use iced::widget::canvas::{self, Geometry, Path, Stroke};
use iced::widget::image::Handle as ImageHandle;
use iced::widget::shader;
use iced::{Color, Event, Font, Point, Rectangle, Size, Theme};

use crate::render::text::{strip_wow_markup, wow_font_to_iced, TextRenderer};
use crate::render::texture::{draw_horizontal_slice_texture, UI_SCALE};
use crate::render::{BlendMode, GpuTextureData, QuadBatch, WowUiPrimitive};
use crate::widget::{TextJustify, WidgetType};

use super::app::App;
use super::layout::{anchor_position, compute_frame_rect};
use super::state::{CanvasMessage, TextOverlay};
use super::styles::palette;
use super::Message;

/// Canvas program implementation for text overlay.
///
/// This renders only FontString widgets with a transparent background,
/// allowing it to be layered on top of the shader which handles textures.
impl canvas::Program<Message> for TextOverlay<'_> {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // Text overlay doesn't handle events - the shader layer handles them
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.app.text_cache.draw(renderer, bounds.size(), |frame| {
            // Transparent background - let shader show through
            // Pass canvas origin offset so coordinates can be adjusted
            self.app.draw_text_overlay(frame, bounds.size());
        });

        vec![geometry]
    }
}

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

        WowUiPrimitive::with_textures(quads, textures)
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
    // Determine which texture and tex_coords to use based on state
    let (texture_path, tex_coords) = if is_pressed {
        (
            f.pushed_texture.as_ref().or(f.normal_texture.as_ref()),
            f.pushed_tex_coords.or(f.normal_tex_coords),
        )
    } else {
        (f.normal_texture.as_ref(), f.normal_tex_coords)
    };

    // Render button texture or fallback to solid color
    if let Some(tex_path) = texture_path {
        if let Some((left, right, top, bottom)) = tex_coords {
            // Atlas texture - use sub-region UV coordinates
            let uvs = Rectangle::new(
                Point::new(left, top),
                Size::new(right - left, bottom - top),
            );
            batch.push_textured_path_uv(
                bounds,
                uvs,
                tex_path,
                [1.0, 1.0, 1.0, f.alpha],
                BlendMode::Alpha,
            );
        } else {
            // WoW button textures are 128x32 with thin gold borders (~3px)
            // Use 3-slice rendering to preserve the end caps while stretching the middle
            const BUTTON_TEX_WIDTH: f32 = 128.0;
            const BUTTON_CAP_WIDTH: f32 = 4.0;
            batch.push_three_slice_h_path(
                bounds,
                BUTTON_CAP_WIDTH,
                BUTTON_CAP_WIDTH,
                tex_path,
                BUTTON_TEX_WIDTH,
                [1.0, 1.0, 1.0, f.alpha],
            );
        }
    }
    // In WoW, buttons without NormalTexture are transparent - their visuals come
    // from child Texture widgets (e.g. MinimalScrollBar steppers, ThreeSliceButton Left/Right/Center).

    // Highlight texture overlay on hover
    if is_hovered && !is_pressed {
        if let Some(ref highlight_path) = f.highlight_texture {
            if let Some((left, right, top, bottom)) = f.highlight_tex_coords {
                // Atlas-based highlight
                let uvs = Rectangle::new(
                    Point::new(left, top),
                    Size::new(right - left, bottom - top),
                );
                batch.push_textured_path_uv(
                    bounds,
                    uvs,
                    highlight_path,
                    [1.0, 1.0, 1.0, 0.5 * f.alpha],
                    BlendMode::Additive,
                );
            } else {
                // Non-atlas highlight (3-slice)
                const BUTTON_TEX_WIDTH: f32 = 128.0;
                const BUTTON_CAP_WIDTH: f32 = 4.0;
                batch.push_three_slice_h_path_blend(
                    bounds,
                    BUTTON_CAP_WIDTH,
                    BUTTON_CAP_WIDTH,
                    highlight_path,
                    BUTTON_TEX_WIDTH,
                    [1.0, 1.0, 1.0, 0.5 * f.alpha],
                    BlendMode::Additive,
                );
            }
        } else {
            // Fallback highlight
            batch.push_quad(
                bounds,
                Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0)),
                [1.0, 0.9, 0.6, 0.15 * f.alpha],
                -1,
                BlendMode::Additive,
            );
        }
    }
}

/// Build quads for a Texture widget.
pub fn build_texture_quads(batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
    // Color texture
    if let Some(color) = f.color_texture {
        batch.push_solid(bounds, [color.r, color.g, color.b, color.a * f.alpha]);
        return;
    }

    // File texture
    if let Some(ref tex_path) = f.texture {
        // Check if we have tex_coords (from SetAtlas or SetTexCoord)
        if let Some((left, right, top, bottom)) = f.tex_coords {
            // Atlas texture - use sub-region UV coordinates
            let uvs = Rectangle::new(
                Point::new(left, top),
                Size::new(right - left, bottom - top),
            );

            // Handle tiling for edge pieces
            if f.horiz_tile || f.vert_tile {
                // Get tile size from frame dimensions (set by SetAtlas with useAtlasSize)
                let tile_w = f.width.max(1.0);
                let tile_h = f.height.max(1.0);

                if f.horiz_tile && !f.vert_tile {
                    // Horizontal tiling only
                    let mut x = bounds.x;
                    while x < bounds.x + bounds.width {
                        let w = (bounds.x + bounds.width - x).min(tile_w);
                        let tile_bounds = Rectangle::new(Point::new(x, bounds.y), Size::new(w, bounds.height));
                        // Adjust UV for partial tiles
                        let uv_w = if w < tile_w { uvs.width * (w / tile_w) } else { uvs.width };
                        let tile_uvs = Rectangle::new(uvs.position(), Size::new(uv_w, uvs.height));
                        batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, [1.0, 1.0, 1.0, f.alpha], BlendMode::Alpha);
                        x += tile_w;
                    }
                } else if f.vert_tile && !f.horiz_tile {
                    // Vertical tiling only
                    let mut y = bounds.y;
                    while y < bounds.y + bounds.height {
                        let h = (bounds.y + bounds.height - y).min(tile_h);
                        let tile_bounds = Rectangle::new(Point::new(bounds.x, y), Size::new(bounds.width, h));
                        // Adjust UV for partial tiles
                        let uv_h = if h < tile_h { uvs.height * (h / tile_h) } else { uvs.height };
                        let tile_uvs = Rectangle::new(uvs.position(), Size::new(uvs.width, uv_h));
                        batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, [1.0, 1.0, 1.0, f.alpha], BlendMode::Alpha);
                        y += tile_h;
                    }
                } else {
                    // Both directions - grid tiling
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
                            batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, [1.0, 1.0, 1.0, f.alpha], BlendMode::Alpha);
                            x += tile_w;
                        }
                        y += tile_h;
                    }
                }
            } else {
                // No tiling - render once
                batch.push_textured_path_uv(
                    bounds,
                    uvs,
                    tex_path,
                    [1.0, 1.0, 1.0, f.alpha],
                    BlendMode::Alpha,
                );
            }
        } else {
            // Full texture - use default UVs
            batch.push_textured_path(
                bounds,
                tex_path,
                [1.0, 1.0, 1.0, f.alpha],
                BlendMode::Alpha,
            );
        }
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

/// Build a QuadBatch from a WidgetRegistry without needing an App instance.
///
/// This contains the sorting/filtering logic from `App::build_quad_batch` but
/// takes a `WidgetRegistry` directly.
pub fn build_quad_batch_for_registry(
    registry: &crate::widget::WidgetRegistry,
    screen_size: (f32, f32),
    root_name: Option<&str>,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
) -> QuadBatch {
    let mut batch = QuadBatch::with_capacity(1000);
    let (screen_width, screen_height) = screen_size;
    let size = Size::new(screen_width, screen_height);

    // Add background quad
    batch.push_solid(
        Rectangle::new(Point::ORIGIN, size),
        [palette::BG_DARK.r, palette::BG_DARK.g, palette::BG_DARK.b, 1.0],
    );

    // Determine which frames to render
    let mut visible_ids = std::collections::HashSet::new();
    if let Some(name) = root_name {
        // Filter to subtree rooted at the named frame
        let root_id = registry.all_ids().into_iter().find(|&id| {
            registry
                .get(id)
                .map(|f| f.name.as_deref() == Some(name))
                .unwrap_or(false)
        });
        if let Some(root_id) = root_id {
            let mut queue = vec![root_id];
            while let Some(id) = queue.pop() {
                visible_ids.insert(id);
                if let Some(f) = registry.get(id) {
                    queue.extend(f.children.iter().copied());
                }
            }
        }
    }
    let filter_to_subtree = root_name.is_some();

    // Collect and sort frames
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

    for (id, f, rect) in frames {
        if filter_to_subtree && !visible_ids.contains(&id) {
            continue;
        }
        if !f.visible {
            continue;
        }
        if rect.width <= 0.0 || rect.height <= 0.0 {
            continue;
        }

        let x = rect.x * UI_SCALE;
        let y = rect.y * UI_SCALE;
        let w = rect.width * UI_SCALE;
        let h = rect.height * UI_SCALE;
        let bounds = Rectangle::new(Point::new(x, y), Size::new(w, h));

        match f.widget_type {
            WidgetType::Frame => {
                build_frame_quads(&mut batch, bounds, f);
            }
            WidgetType::Button => {
                let is_pressed = pressed_frame == Some(id);
                let is_hovered = hovered_frame == Some(id);
                build_button_quads(&mut batch, bounds, f, is_pressed, is_hovered);
            }
            WidgetType::Texture => {
                build_texture_quads(&mut batch, bounds, f);
            }
            WidgetType::FontString => {
                // Text is handled separately (not in quad batch)
            }
            WidgetType::CheckButton => {
                let is_pressed = pressed_frame == Some(id);
                let is_hovered = hovered_frame == Some(id);
                build_button_quads(&mut batch, bounds, f, is_pressed, is_hovered);
            }
            WidgetType::EditBox => {
                build_editbox_quads(&mut batch, bounds, f);
            }
            _ => {}
        }
    }

    batch
}

impl App {
    /// Build a QuadBatch for GPU shader rendering.
    pub(crate) fn build_quad_batch(&self, size: Size) -> QuadBatch {
        let env = self.env.borrow();
        let state = env.state().borrow();
        build_quad_batch_for_registry(
            &state.widgets,
            (size.width, size.height),
            Some("AddonList"),
            self.pressed_frame,
            self.hovered_frame,
        )
    }

    /// Draw text elements and debug overlays (borders, anchor points).
    pub(crate) fn draw_text_overlay(&self, frame: &mut canvas::Frame, size: Size) {
        let env = self.env.borrow();
        let state = env.state().borrow();

        let screen_width = size.width;
        let screen_height = size.height;

        // Find AddonList frame and collect descendant IDs
        let mut addonlist_ids = std::collections::HashSet::new();
        let addonlist_id = state.widgets.all_ids().into_iter().find(|&id| {
            state
                .widgets
                .get(id)
                .map(|f| f.name.as_deref() == Some("AddonList"))
                .unwrap_or(false)
        });

        if let Some(root_id) = addonlist_id {
            let mut queue = vec![root_id];
            while let Some(id) = queue.pop() {
                addonlist_ids.insert(id);
                if let Some(f) = state.widgets.get(id) {
                    queue.extend(f.children.iter().copied());
                }
            }
        }

        // Collect and sort frames
        let mut frames: Vec<_> = state
            .widgets
            .all_ids()
            .into_iter()
            .filter_map(|id| {
                let f = state.widgets.get(id)?;
                let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);
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
                        (true, true) => {
                            a.1.draw_layer
                                .cmp(&b.1.draw_layer)
                                .then_with(|| a.1.draw_sub_layer.cmp(&b.1.draw_sub_layer))
                        }
                        (false, true) => std::cmp::Ordering::Less,
                        (true, false) => std::cmp::Ordering::Greater,
                        (false, false) => std::cmp::Ordering::Equal,
                    }
                })
                .then_with(|| a.0.cmp(&b.0))
        });

        // Draw only text elements
        for (id, f, rect) in frames {
            if !addonlist_ids.contains(&id) {
                continue;
            }
            if !f.visible {
                continue;
            }
            if rect.width <= 0.0 || rect.height <= 0.0 {
                continue;
            }

            let x = rect.x * UI_SCALE;
            let y = rect.y * UI_SCALE;
            let w = rect.width * UI_SCALE;
            let h = rect.height * UI_SCALE;
            let bounds = Rectangle::new(Point::new(x, y), Size::new(w, h));

            match f.widget_type {
                WidgetType::FontString => {
                    self.draw_fontstring_widget(frame, bounds, f);
                }
                WidgetType::Button => {
                    self.draw_button_text(frame, bounds, f);
                }
                WidgetType::EditBox => {
                    self.draw_editbox_text(frame, bounds, f);
                }
                WidgetType::CheckButton => {
                    self.draw_checkbutton_text(frame, bounds, f);
                }
                _ => {}
            }

            // Draw debug border if enabled
            if self.debug_borders {
                let border_path = Path::rectangle(bounds.position(), bounds.size());
                frame.stroke(
                    &border_path,
                    Stroke::default()
                        .with_color(Color::from_rgb(1.0, 0.0, 0.0))
                        .with_width(1.0),
                );
            }

            // Draw debug anchor points if enabled
            if self.debug_anchors {
                let anchor_color = Color::from_rgb(0.0, 1.0, 0.0);
                let dot_radius = 4.0;

                for anchor in &f.anchors {
                    let (ax, ay) = anchor_position(
                        anchor.point,
                        bounds.x,
                        bounds.y,
                        bounds.width,
                        bounds.height,
                    );
                    let dot = Path::circle(Point::new(ax, ay), dot_radius);
                    frame.fill(&dot, anchor_color);
                }
            }
        }
    }

    /// Draw only the text portion of a button.
    fn draw_button_text(&self, frame: &mut canvas::Frame, bounds: Rectangle, f: &crate::widget::Frame) {
        if let Some(ref txt) = f.text {
            let clean_text = strip_wow_markup(txt);
            TextRenderer::draw_justified_text(
                frame,
                &clean_text,
                bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                wow_font_to_iced(f.font.as_deref()),
                TextJustify::Center,
                TextJustify::Center,
            );
        }
    }

    /// Draw only the text portion of an editbox.
    fn draw_editbox_text(&self, frame: &mut canvas::Frame, bounds: Rectangle, f: &crate::widget::Frame) {
        if let Some(ref txt) = f.text {
            let padding = 4.0;
            let text_bounds = Rectangle::new(
                Point::new(bounds.x + padding, bounds.y),
                Size::new(bounds.width - padding * 2.0, bounds.height),
            );
            TextRenderer::draw_justified_text(
                frame,
                txt,
                text_bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                Font::DEFAULT,
                TextJustify::Left,
                TextJustify::Center,
            );
        }
    }

    /// Draw only the text portion of a checkbox.
    fn draw_checkbutton_text(&self, frame: &mut canvas::Frame, bounds: Rectangle, f: &crate::widget::Frame) {
        if let Some(ref txt) = f.text {
            let clean_text = strip_wow_markup(txt);
            let label_x = bounds.x + 20.0;
            let label_bounds = Rectangle::new(
                Point::new(label_x, bounds.y),
                Size::new(bounds.width - 20.0, bounds.height),
            );
            TextRenderer::draw_justified_text(
                frame,
                &clean_text,
                label_bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                wow_font_to_iced(f.font.as_deref()),
                TextJustify::Left,
                TextJustify::Center,
            );
        }
    }

    fn draw_fontstring_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
    ) {
        if let Some(ref txt) = f.text {
            let clean_text = strip_wow_markup(txt);

            TextRenderer::draw_justified_text(
                frame,
                &clean_text,
                bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                wow_font_to_iced(f.font.as_deref()),
                f.justify_h,
                f.justify_v,
            );
        }
    }

    /// Load a texture and cache its ImageHandle.
    pub(crate) fn get_or_load_texture(&self, wow_path: &str) -> Option<ImageHandle> {
        // Check cache first
        {
            let cache = self.image_handles.borrow();
            if let Some(handle) = cache.get(wow_path) {
                return Some(handle.clone());
            }
        }

        // Load texture
        let mut tex_mgr = self.texture_manager.borrow_mut();
        if let Some(tex_data) = tex_mgr.load(wow_path) {
            let handle = ImageHandle::from_rgba(
                tex_data.width,
                tex_data.height,
                tex_data.pixels.clone(),
            );
            self.image_handles
                .borrow_mut()
                .insert(wow_path.to_string(), handle.clone());
            Some(handle)
        } else {
            None
        }
    }

    /// Load a sub-region of a texture (for atlas textures with tex_coords).
    pub(crate) fn get_or_load_texture_region(
        &self,
        wow_path: &str,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Option<ImageHandle> {
        // Create a cache key for this specific region
        let cache_key = format!("{}#{}_{}_{}_{}", wow_path, x, y, width, height);

        // Check cache first
        {
            let cache = self.image_handles.borrow();
            if let Some(handle) = cache.get(&cache_key) {
                return Some(handle.clone());
            }
        }

        // Load sub-region from texture manager
        let mut tex_mgr = self.texture_manager.borrow_mut();
        if let Some(tex_data) = tex_mgr.load_sub_region(wow_path, x, y, width, height) {
            let handle = ImageHandle::from_rgba(
                tex_data.width,
                tex_data.height,
                tex_data.pixels.clone(),
            );
            self.image_handles
                .borrow_mut()
                .insert(cache_key, handle.clone());
            Some(handle)
        } else {
            None
        }
    }

    /// Get the dimensions of a cached texture.
    pub(crate) fn get_texture_size(&self, path: &str) -> Option<(f32, f32)> {
        self.texture_manager
            .borrow()
            .get_texture_size(path)
            .map(|(w, h)| (w as f32, h as f32))
    }

    /// Load an atlas texture by name, extracting the region from the atlas file.
    pub(crate) fn get_atlas_texture(&self, atlas_name: &str) -> Option<ImageHandle> {
        use crate::atlas::ATLAS_DB;

        let info = ATLAS_DB.get(atlas_name)?;

        // Get the full texture first
        let _ = self.get_or_load_texture(info.file);
        if let Some((tex_w, tex_h)) = self.get_texture_size(info.file) {
            // Calculate pixel coordinates from normalized tex_coords
            let x = (info.left_tex_coord * tex_w).round() as u32;
            let y = (info.top_tex_coord * tex_h).round() as u32;
            let w = ((info.right_tex_coord - info.left_tex_coord) * tex_w).round() as u32;
            let h = ((info.bottom_tex_coord - info.top_tex_coord) * tex_h).round() as u32;

            self.get_or_load_texture_region(info.file, x, y, w, h)
        } else {
            None
        }
    }

    /// Resolve texture path for a button state by checking both direct fields and child textures.
    #[allow(dead_code)]
    pub(crate) fn resolve_button_texture(
        &self,
        button: &crate::widget::Frame,
        key: &str,
        registry: &crate::widget::WidgetRegistry,
    ) -> Option<String> {
        // First check child texture (may have atlas set via SetAtlas)
        if let Some(&tex_id) = button.children_keys.get(key) {
            if let Some(tex) = registry.get(tex_id) {
                if tex.texture.is_some() {
                    return tex.texture.clone();
                }
            }
        }

        // Fall back to button's direct fields
        match key {
            "NormalTexture" => button.normal_texture.clone(),
            "PushedTexture" => button.pushed_texture.clone(),
            "HighlightTexture" => button.highlight_texture.clone(),
            "DisabledTexture" => button.disabled_texture.clone(),
            _ => None,
        }
    }

    /// Resolve tex_coords for a button texture from child texture (set via atlas).
    #[allow(dead_code)]
    pub(crate) fn resolve_button_tex_coords(
        &self,
        button: &crate::widget::Frame,
        key: &str,
        registry: &crate::widget::WidgetRegistry,
    ) -> Option<(f32, f32, f32, f32)> {
        if let Some(&tex_id) = button.children_keys.get(key) {
            if let Some(tex) = registry.get(tex_id) {
                return tex.tex_coords;
            }
        }
        None
    }

    /// Draw a texture tiled across the given bounds.
    #[allow(dead_code)]
    fn draw_tiled_texture(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        handle: &ImageHandle,
        tex_width: f32,
        tex_height: f32,
        horiz_tile: bool,
        vert_tile: bool,
        alpha: f32,
    ) {
        let tile_width = if horiz_tile { tex_width } else { bounds.width };
        let tile_height = if vert_tile { tex_height } else { bounds.height };

        // Calculate how many tiles we need
        let cols = if horiz_tile {
            ((bounds.width / tile_width).ceil() as i32).max(1)
        } else {
            1
        };
        let rows = if vert_tile {
            ((bounds.height / tile_height).ceil() as i32).max(1)
        } else {
            1
        };

        // Clip all tile drawing to the bounds
        frame.with_clip(bounds, |frame| {
            for row in 0..rows {
                for col in 0..cols {
                    let x = bounds.x + col as f32 * tile_width;
                    let y = bounds.y + row as f32 * tile_height;

                    let full_tile =
                        Rectangle::new(Point::new(x, y), Size::new(tile_width, tile_height));
                    let mut img = canvas::Image::new(handle.clone());
                    if alpha < 1.0 {
                        img = img.opacity(alpha);
                    }
                    frame.draw_image(full_tile, img);
                }
            }
        });
    }

    // Legacy drawing methods (kept for compatibility)
    #[allow(dead_code)]
    fn draw_frame_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
    ) {
        // Draw NineSlice border if this is a NineSlice frame
        if let Some(ref layout) = f.nine_slice_layout {
            self.draw_nine_slice_border(frame, bounds, layout, f.alpha);
        }

        if f.backdrop.enabled {
            // Draw backdrop background
            let bg = &f.backdrop.bg_color;
            frame.fill_rectangle(
                bounds.position(),
                bounds.size(),
                Color::from_rgba(bg.r, bg.g, bg.b, bg.a * f.alpha),
            );

            // Draw border
            let bc = &f.backdrop.border_color;
            frame.stroke(
                &Path::rectangle(bounds.position(), bounds.size()),
                Stroke::default()
                    .with_color(Color::from_rgba(bc.r, bc.g, bc.b, bc.a * f.alpha))
                    .with_width(f.backdrop.edge_size.max(1.0)),
            );
        }
    }

    /// Draw NineSlice panel border using atlas textures.
    #[allow(dead_code)]
    fn draw_nine_slice_border(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        _layout: &str,
        alpha: f32,
    ) {
        let corner_size = 32.0;
        let edge_thickness = 32.0;

        // Atlas names for ButtonFrameTemplateNoPortrait
        let tl_atlas = "ui-frame-metal-cornertopleft-2x";
        let tr_atlas = "ui-frame-metal-cornertopright-2x";
        let bl_atlas = "ui-frame-metal-cornerbottomleft-2x";
        let br_atlas = "ui-frame-metal-cornerbottomright-2x";
        let top_atlas = "_ui-frame-metal-edgetop-2x";
        let bottom_atlas = "_ui-frame-metal-edgebottom-2x";
        let left_atlas = "!ui-frame-metal-edgeleft-2x";
        let right_atlas = "!ui-frame-metal-edgeright-2x";

        // Draw corners
        if let Some(handle) = self.get_atlas_texture(tl_atlas) {
            let corner_bounds = Rectangle::new(
                Point::new(bounds.x - 8.0, bounds.y - 8.0),
                Size::new(corner_size, corner_size),
            );
            frame.draw_image(corner_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        if let Some(handle) = self.get_atlas_texture(tr_atlas) {
            let corner_bounds = Rectangle::new(
                Point::new(bounds.x + bounds.width - corner_size + 8.0, bounds.y - 8.0),
                Size::new(corner_size, corner_size),
            );
            frame.draw_image(corner_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        if let Some(handle) = self.get_atlas_texture(bl_atlas) {
            let corner_bounds = Rectangle::new(
                Point::new(bounds.x - 8.0, bounds.y + bounds.height - corner_size + 8.0),
                Size::new(corner_size, corner_size),
            );
            frame.draw_image(corner_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        if let Some(handle) = self.get_atlas_texture(br_atlas) {
            let corner_bounds = Rectangle::new(
                Point::new(bounds.x + bounds.width - corner_size + 8.0, bounds.y + bounds.height - corner_size + 8.0),
                Size::new(corner_size, corner_size),
            );
            frame.draw_image(corner_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        // Draw edges
        if let Some(handle) = self.get_atlas_texture(top_atlas) {
            let edge_bounds = Rectangle::new(
                Point::new(bounds.x + corner_size - 8.0, bounds.y - 8.0),
                Size::new(bounds.width - corner_size * 2.0 + 16.0, edge_thickness),
            );
            frame.draw_image(edge_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        if let Some(handle) = self.get_atlas_texture(bottom_atlas) {
            let edge_bounds = Rectangle::new(
                Point::new(bounds.x + corner_size - 8.0, bounds.y + bounds.height - edge_thickness + 8.0),
                Size::new(bounds.width - corner_size * 2.0 + 16.0, edge_thickness),
            );
            frame.draw_image(edge_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        if let Some(handle) = self.get_atlas_texture(left_atlas) {
            let edge_bounds = Rectangle::new(
                Point::new(bounds.x - 8.0, bounds.y + corner_size - 8.0),
                Size::new(edge_thickness, bounds.height - corner_size * 2.0 + 16.0),
            );
            frame.draw_image(edge_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        if let Some(handle) = self.get_atlas_texture(right_atlas) {
            let edge_bounds = Rectangle::new(
                Point::new(bounds.x + bounds.width - edge_thickness + 8.0, bounds.y + corner_size - 8.0),
                Size::new(edge_thickness, bounds.height - corner_size * 2.0 + 16.0),
            );
            frame.draw_image(edge_bounds, canvas::Image::new(handle).opacity(alpha));
        }
    }

    #[allow(dead_code)]
    fn draw_button_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
        frame_id: u64,
        normal_tex: Option<&str>,
        normal_coords: Option<(f32, f32, f32, f32)>,
        pushed_tex: Option<&str>,
        pushed_coords: Option<(f32, f32, f32, f32)>,
        highlight_tex: Option<&str>,
        highlight_coords: Option<(f32, f32, f32, f32)>,
    ) {
        let is_pressed = self.pressed_frame == Some(frame_id);
        let is_hovered = self.hovered_frame == Some(frame_id);

        // Try 3-slice button rendering first (gold button style)
        if let (Some(left_path), Some(middle_path), Some(right_path)) =
            (&f.left_texture, &f.middle_texture, &f.right_texture)
        {
            let left_handle = self.get_or_load_texture(left_path);
            let middle_handle = self.get_or_load_texture(middle_path);
            let right_handle = self.get_or_load_texture(right_path);

            if left_handle.is_some() && middle_handle.is_some() && right_handle.is_some() {
                draw_horizontal_slice_texture(
                    frame,
                    bounds,
                    left_handle.as_ref().unwrap(),
                    left_handle.as_ref(),
                    middle_handle.as_ref(),
                    right_handle.as_ref(),
                    64.0,
                    32.0,
                    f.alpha,
                );

                if let Some(ref txt) = f.text {
                    TextRenderer::draw_centered_text(
                        frame,
                        txt,
                        bounds,
                        f.font_size,
                        Color::from_rgba(
                            f.text_color.r,
                            f.text_color.g,
                            f.text_color.b,
                            f.text_color.a * f.alpha,
                        ),
                        wow_font_to_iced(f.font.as_deref()),
                    );
                }
                return;
            }
        }

        // Select texture and tex_coords based on button state
        let (button_texture, button_coords) = if is_pressed {
            (pushed_tex.or(normal_tex), pushed_coords.or(normal_coords))
        } else {
            (normal_tex, normal_coords)
        };

        // Try single texture (pushed or normal)
        let mut drew_background = false;
        if let Some(tex_path) = button_texture {
            let handle = if let Some((left, right, top, bottom)) = button_coords {
                let _ = self.get_or_load_texture(tex_path);
                if let Some((tex_w, tex_h)) = self.get_texture_size(tex_path) {
                    let x = (left * tex_w).round() as u32;
                    let y = (top * tex_h).round() as u32;
                    let w = ((right - left) * tex_w).round() as u32;
                    let h = ((bottom - top) * tex_h).round() as u32;
                    self.get_or_load_texture_region(tex_path, x, y, w, h)
                } else {
                    self.get_or_load_texture(tex_path)
                }
            } else if tex_path.to_lowercase().contains("ui-panel-button") {
                self.get_or_load_texture_region(tex_path, 0, 0, 80, 22)
            } else {
                self.get_or_load_texture(tex_path)
            };

            if let Some(handle) = handle {
                frame.draw_image(bounds, canvas::Image::new(handle));
                drew_background = true;
            }
        }

        // Fallback: default button styling
        if !drew_background {
            let bg_color = if is_pressed {
                Color::from_rgba(0.20, 0.08, 0.08, 0.95 * f.alpha)
            } else if is_hovered {
                Color::from_rgba(0.18, 0.07, 0.07, 0.95 * f.alpha)
            } else {
                Color::from_rgba(0.15, 0.05, 0.05, 0.95 * f.alpha)
            };

            frame.fill_rectangle(bounds.position(), bounds.size(), bg_color);

            let border_color = if is_hovered || is_pressed {
                Color::from_rgba(0.8, 0.6, 0.2, f.alpha)
            } else {
                Color::from_rgba(0.6, 0.45, 0.15, f.alpha)
            };

            frame.stroke(
                &Path::rectangle(bounds.position(), bounds.size()),
                Stroke::default().with_color(border_color).with_width(1.5),
            );
        }

        // Draw highlight texture overlay when hovered
        if is_hovered && !is_pressed {
            if let Some(highlight_path) = highlight_tex {
                let handle = if let Some((left, right, top, bottom)) = highlight_coords {
                    let _ = self.get_or_load_texture(highlight_path);
                    if let Some((tex_w, tex_h)) = self.get_texture_size(highlight_path) {
                        let x = (left * tex_w).round() as u32;
                        let y = (top * tex_h).round() as u32;
                        let w = ((right - left) * tex_w).round() as u32;
                        let h = ((bottom - top) * tex_h).round() as u32;
                        self.get_or_load_texture_region(highlight_path, x, y, w, h)
                    } else {
                        self.get_or_load_texture(highlight_path)
                    }
                } else if highlight_path.to_lowercase().contains("ui-panel-button") {
                    self.get_or_load_texture_region(highlight_path, 0, 0, 80, 22)
                } else {
                    self.get_or_load_texture(highlight_path)
                };

                if let Some(handle) = handle {
                    let mut img = canvas::Image::new(handle);
                    img = img.opacity(0.5 * f.alpha);
                    frame.draw_image(bounds, img);
                }
            } else if drew_background {
                frame.fill_rectangle(
                    bounds.position(),
                    bounds.size(),
                    Color::from_rgba(1.0, 0.9, 0.6, 0.15 * f.alpha),
                );
            }
        }

        // Draw button text (centered)
        if let Some(ref txt) = f.text {
            TextRenderer::draw_centered_text(
                frame,
                txt,
                bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                wow_font_to_iced(f.font.as_deref()),
            );
        }
    }

    #[allow(dead_code)]
    fn draw_texture_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
    ) {
        // Try color texture first
        if let Some(color) = f.color_texture {
            frame.fill_rectangle(
                bounds.position(),
                bounds.size(),
                Color::from_rgba(color.r, color.g, color.b, color.a * f.alpha),
            );
            return;
        }

        // Try to load and render the texture file
        if let Some(ref tex_path) = f.texture {
            let handle_opt = if let Some((left, right, top, bottom)) = f.tex_coords {
                let _ = self.get_or_load_texture(tex_path);

                if let Some(tex_size) = self.get_texture_size(tex_path) {
                    let x = (left * tex_size.0).round() as u32;
                    let y = (top * tex_size.1).round() as u32;
                    let w = ((right - left) * tex_size.0).round() as u32;
                    let h = ((bottom - top) * tex_size.1).round() as u32;
                    self.get_or_load_texture_region(tex_path, x, y, w, h)
                } else {
                    self.get_or_load_texture(tex_path)
                }
            } else {
                self.get_or_load_texture(tex_path)
            };

            if let Some(handle) = handle_opt {
                if f.horiz_tile || f.vert_tile {
                    let tex_size = if f.tex_coords.is_some() {
                        (bounds.width, bounds.height)
                    } else {
                        self.get_texture_size(tex_path).unwrap_or((256.0, 256.0))
                    };

                    self.draw_tiled_texture(
                        frame,
                        bounds,
                        &handle,
                        tex_size.0,
                        tex_size.1,
                        f.horiz_tile,
                        f.vert_tile,
                        f.alpha,
                    );
                } else {
                    frame.draw_image(bounds, canvas::Image::new(handle));
                }
            }
        }
    }

    #[allow(dead_code)]
    fn draw_editbox_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
    ) {
        // Background
        frame.fill_rectangle(
            bounds.position(),
            bounds.size(),
            Color::from_rgba(0.08, 0.08, 0.1, 0.9 * f.alpha),
        );

        // Border
        frame.stroke(
            &Path::rectangle(bounds.position(), bounds.size()),
            Stroke::default()
                .with_color(Color::from_rgba(0.3, 0.3, 0.35, f.alpha))
                .with_width(1.0),
        );

        // Text
        if let Some(ref txt) = f.text {
            let padding = 4.0;
            let text_bounds = Rectangle::new(
                Point::new(bounds.x + padding, bounds.y),
                Size::new(bounds.width - padding * 2.0, bounds.height),
            );
            TextRenderer::draw_justified_text(
                frame,
                txt,
                text_bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                Font::DEFAULT,
                TextJustify::Left,
                TextJustify::Center,
            );
        }
    }

    #[allow(dead_code)]
    fn draw_slider_widget(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        let track_height = 4.0;
        let track_y = bounds.y + (bounds.height - track_height) / 2.0;

        frame.fill_rectangle(
            Point::new(bounds.x, track_y),
            Size::new(bounds.width, track_height),
            Color::from_rgba(0.2, 0.2, 0.25, 0.9),
        );

        let thumb_width = 12.0;
        let thumb_height = 16.0;
        let thumb_x = bounds.x + (bounds.width - thumb_width) / 2.0;
        let thumb_y = bounds.y + (bounds.height - thumb_height) / 2.0;

        frame.fill_rectangle(
            Point::new(thumb_x, thumb_y),
            Size::new(thumb_width, thumb_height),
            Color::from_rgb(0.6, 0.5, 0.3),
        );

        frame.stroke(
            &Path::rectangle(
                Point::new(thumb_x, thumb_y),
                Size::new(thumb_width, thumb_height),
            ),
            Stroke::default()
                .with_color(Color::from_rgb(0.8, 0.7, 0.4))
                .with_width(1.0),
        );
    }
}
