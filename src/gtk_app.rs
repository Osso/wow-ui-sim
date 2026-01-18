//! GTK4/relm4-based UI for rendering WoW frames.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use gtk::cairo::{self, ImageSurface, Format};
use gtk::gdk;
use gtk::glib;
use pangocairo::functions as pango_cairo;
use glib::ControlFlow;
use relm4::prelude::*;

use crate::lua_api::WowLuaEnv;
use crate::LayoutRect;
use crate::texture::TextureManager;
use crate::widget::{TextJustify, WidgetType};
use gtk_layout_inspector::server::{self as debug_server, Command as DebugCommand, ScreenshotData};
use gtk_layout_inspector::{dump_widget_tree, find_button_by_label, find_entry_by_placeholder};

/// Custom CSS for WoW-style theming.
const STYLE_CSS: &str = include_str!("style.css");

/// Default path to wow-ui-textures repository.
const DEFAULT_TEXTURES_PATH: &str = "/home/osso/Repos/wow-ui-textures";

/// Default path to WoW Interface directory (extracted game files).
const DEFAULT_INTERFACE_PATH: &str = "/home/osso/Projects/wow/Interface";

/// Default path to addons directory.
const DEFAULT_ADDONS_PATH: &str = "/home/osso/Projects/wow/reference-addons";

/// UI scale factor (1.0 = pixel-perfect, no scaling).
const UI_SCALE: f64 = 1.0;

/// Load WoW fonts from a directory into fontconfig.
/// This makes fonts available to Pango without requiring system-wide installation.
fn load_fonts_from_dir(fonts_dir: &Path) {
    if !fonts_dir.exists() {
        eprintln!("[Fonts] Directory not found: {}", fonts_dir.display());
        return;
    }

    // Use fontconfig FFI to add the fonts directory
    #[link(name = "fontconfig")]
    unsafe extern "C" {
        fn FcConfigGetCurrent() -> *mut std::ffi::c_void;
        fn FcConfigAppFontAddDir(config: *mut std::ffi::c_void, dir: *const i8) -> i32;
    }

    let dir_cstr = match CString::new(fonts_dir.to_string_lossy().as_bytes()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Fonts] Invalid path: {}", e);
            return;
        }
    };

    unsafe {
        let config = FcConfigGetCurrent();
        if config.is_null() {
            eprintln!("[Fonts] Failed to get fontconfig");
            return;
        }
        let result = FcConfigAppFontAddDir(config, dir_cstr.as_ptr());
        if result != 0 {
            println!("[Fonts] Loaded fonts from: {}", fonts_dir.display());
        } else {
            eprintln!("[Fonts] Failed to add fonts dir: {}", fonts_dir.display());
        }
    }
}

/// Run the GTK UI with the given Lua environment.
pub fn run_gtk_ui(env: WowLuaEnv) -> Result<(), Box<dyn std::error::Error>> {
    run_gtk_ui_with_textures(env, PathBuf::from(DEFAULT_TEXTURES_PATH))
}

/// Run the GTK UI with the given Lua environment and textures path.
pub fn run_gtk_ui_with_textures(
    env: WowLuaEnv,
    textures_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load WoW fonts from project fonts/ directory
    let fonts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts");
    load_fonts_from_dir(&fonts_dir);

    let app = RelmApp::new("com.osso.wow-ui-sim");
    app.run::<App>((env, textures_path));
    Ok(())
}

/// Fire the standard WoW startup events.
fn fire_startup_events(env: &Rc<RefCell<WowLuaEnv>>) {
    let env = env.borrow();

    println!("[Startup] Firing ADDON_LOADED");
    if let Err(e) = env.fire_event_with_args(
        "ADDON_LOADED",
        &[mlua::Value::String(env.lua().create_string("WoWUISim").unwrap())],
    ) {
        eprintln!("Error firing ADDON_LOADED: {}", e);
    }

    println!("[Startup] Firing PLAYER_LOGIN");
    if let Err(e) = env.fire_event("PLAYER_LOGIN") {
        eprintln!("Error firing PLAYER_LOGIN: {}", e);
    }

    println!("[Startup] Firing PLAYER_ENTERING_WORLD");
    if let Err(e) = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[
            mlua::Value::Boolean(true),  // isInitialLogin
            mlua::Value::Boolean(false), // isReloadingUi
        ],
    ) {
        eprintln!("Error firing PLAYER_ENTERING_WORLD: {}", e);
    }
}

// Note: FrameInfo will be used in Phase 2 for texture rendering

/// Shared UI state accessible from draw functions.
#[derive(Default)]
struct SharedUiState {
    /// Currently hovered frame ID.
    hovered_frame: Option<u64>,
    /// Currently pressed frame ID (mouse down).
    pressed_frame: Option<u64>,
    /// Current scroll offset in pixels.
    scroll_offset: f64,
}

/// App state
struct App {
    env: Rc<RefCell<WowLuaEnv>>,
    log_messages: Vec<String>,
    /// Current text in the command input field.
    command_input: String,
    /// Drawing area for WoW frames canvas.
    drawing_area: gtk::DrawingArea,
    /// Frames list sidebar content.
    frames_box: gtk::Box,
    /// Console output label.
    console_label: gtk::Label,
    /// Shared UI state for draw functions.
    ui_state: Rc<RefCell<SharedUiState>>,
    /// Frame that received mouse down (for click detection).
    mouse_down_frame: Option<u64>,
    /// Scroll offset for scrollable content (in pixels).
    scroll_offset: f64,
}

/// Cairo texture cache for GPU-ready surfaces.
type CairoTextureCache = HashMap<String, Rc<ImageSurface>>;

/// Convert RGBA pixels to Cairo ARGB32 format (premultiplied, BGRA byte order on little-endian).
fn rgba_to_cairo_argb32(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut argb = Vec::with_capacity((width * height * 4) as usize);
    for chunk in rgba.chunks_exact(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;
        let a = chunk[3] as f32 / 255.0;
        // Premultiply and convert to BGRA
        argb.push((b * a) as u8); // B
        argb.push((g * a) as u8); // G
        argb.push((r * a) as u8); // R
        argb.push(chunk[3]);       // A
    }
    argb
}

/// Load a texture and convert to Cairo ImageSurface.
fn load_cairo_surface(
    texture_manager: &mut TextureManager,
    cache: &mut CairoTextureCache,
    path: &str,
) -> Option<Rc<ImageSurface>> {
    // Skip empty paths
    if path.is_empty() {
        return None;
    }

    // Check cache first
    if let Some(surface) = cache.get(path) {
        return Some(Rc::clone(surface));
    }

    // Load from texture manager
    if let Some(tex_data) = texture_manager.load(path) {
        let argb = rgba_to_cairo_argb32(&tex_data.pixels, tex_data.width, tex_data.height);
        let stride = cairo::Format::ARgb32.stride_for_width(tex_data.width).unwrap();

        // Cairo expects rows to be stride-aligned
        let mut aligned_data = vec![0u8; (stride * tex_data.height as i32) as usize];
        for y in 0..tex_data.height {
            let src_start = (y * tex_data.width * 4) as usize;
            let src_end = src_start + (tex_data.width * 4) as usize;
            let dst_start = (y as i32 * stride) as usize;
            aligned_data[dst_start..dst_start + (tex_data.width * 4) as usize]
                .copy_from_slice(&argb[src_start..src_end]);
        }

        if let Ok(surface) = ImageSurface::create_for_data(
            aligned_data,
            Format::ARgb32,
            tex_data.width as i32,
            tex_data.height as i32,
            stride,
        ) {
            let rc = Rc::new(surface);
            cache.insert(path.to_string(), Rc::clone(&rc));
            return Some(rc);
        }
    }
    None
}

/// Draw a texture scaled to fit the target rectangle.
fn draw_scaled_texture(
    cr: &cairo::Context,
    surface: &ImageSurface,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    alpha: f64,
) {
    let tex_w = surface.width() as f64;
    let tex_h = surface.height() as f64;

    if tex_w <= 0.0 || tex_h <= 0.0 || w <= 0.0 || h <= 0.0 {
        return;
    }

    cr.save().ok();

    // Clip to target rectangle
    cr.rectangle(x, y, w, h);
    cr.clip();

    // Translate to position, then scale to fit
    cr.translate(x, y);
    cr.scale(w / tex_w, h / tex_h);

    // Now draw the surface at origin - it will be scaled to fill the clipped area
    cr.set_source_surface(surface, 0.0, 0.0).ok();

    // Use EXTEND_PAD to avoid edge artifacts
    let pattern = cr.source();
    pattern.set_extend(cairo::Extend::Pad);
    pattern.set_filter(cairo::Filter::Bilinear);

    if alpha < 1.0 {
        cr.paint_with_alpha(alpha).ok();
    } else {
        cr.paint().ok();
    }

    cr.restore().ok();
}

/// Draw a texture with TexCoords (only uses a portion of the texture).
/// Uses additive blending for highlight textures.
fn draw_texture_with_texcoords(
    cr: &cairo::Context,
    surface: &ImageSurface,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    tex_right: f64,
    tex_bottom: f64,
    alpha: f64,
    additive: bool,
) {
    let full_tex_w = surface.width() as f64;
    let full_tex_h = surface.height() as f64;

    // Calculate the source region from TexCoords
    let src_w = full_tex_w * tex_right;
    let src_h = full_tex_h * tex_bottom;

    cr.save().ok();

    // Set additive blending if requested
    if additive {
        cr.set_operator(cairo::Operator::Add);
    }

    // Clip to destination
    cr.rectangle(x, y, w, h);
    cr.clip();

    // Translate to position, then scale to fit destination
    cr.translate(x, y);
    cr.scale(w / src_w, h / src_h);

    // Draw the surface - only the src_w x src_h portion will be visible due to scaling
    cr.set_source_surface(surface, 0.0, 0.0).ok();

    let pattern = cr.source();
    pattern.set_extend(cairo::Extend::Pad);
    pattern.set_filter(cairo::Filter::Bilinear);

    cr.paint_with_alpha(alpha).ok();

    cr.restore().ok();
}

/// Draw a texture tiled to fill the target rectangle.
fn draw_tiled_texture(
    cr: &cairo::Context,
    surface: &ImageSurface,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    alpha: f64,
) {
    let tex_w = surface.width() as f64;
    let tex_h = surface.height() as f64;

    cr.save().ok();

    // Clip to target rectangle
    cr.rectangle(x, y, w, h);
    cr.clip();

    // Draw tiles
    let mut ty = y;
    while ty < y + h {
        let mut tx = x;
        while tx < x + w {
            cr.set_source_surface(surface, tx, ty).ok();
            cr.paint_with_alpha(alpha).ok();
            tx += tex_w;
        }
        ty += tex_h;
    }

    cr.restore().ok();
}

/// Draw a texture using 9-slice scaling (corners fixed, edges stretch).
/// `inset` is the size of the corner regions in texture pixels.
#[allow(dead_code)]
fn draw_nine_slice_texture(
    cr: &cairo::Context,
    surface: &ImageSurface,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    inset: f64,
    alpha: f64,
) {
    let tex_w = surface.width() as f64;
    let tex_h = surface.height() as f64;

    // Clamp inset to half the texture size
    let inset_x = inset.min(tex_w / 2.0);
    let inset_y = inset.min(tex_h / 2.0);

    // If target is smaller than corners, just scale the whole thing
    if w < inset_x * 2.0 || h < inset_y * 2.0 {
        draw_scaled_texture(cr, surface, x, y, w, h, alpha);
        return;
    }

    cr.save().ok();

    // Helper to draw a region of the texture
    let draw_region = |sx: f64, sy: f64, sw: f64, sh: f64, dx: f64, dy: f64, dw: f64, dh: f64| {
        if sw <= 0.0 || sh <= 0.0 || dw <= 0.0 || dh <= 0.0 {
            return;
        }
        cr.save().ok();
        cr.rectangle(dx, dy, dw, dh);
        cr.clip();
        cr.translate(dx, dy);
        cr.scale(dw / sw, dh / sh);
        cr.set_source_surface(surface, -sx, -sy).ok();
        cr.paint_with_alpha(alpha).ok();
        cr.restore().ok();
    };

    let center_tex_w = tex_w - inset_x * 2.0;
    let center_tex_h = tex_h - inset_y * 2.0;
    let center_dst_w = w - inset_x * 2.0;
    let center_dst_h = h - inset_y * 2.0;

    // Top-left corner
    draw_region(0.0, 0.0, inset_x, inset_y, x, y, inset_x, inset_y);
    // Top edge
    draw_region(inset_x, 0.0, center_tex_w, inset_y, x + inset_x, y, center_dst_w, inset_y);
    // Top-right corner
    draw_region(tex_w - inset_x, 0.0, inset_x, inset_y, x + w - inset_x, y, inset_x, inset_y);

    // Left edge
    draw_region(0.0, inset_y, inset_x, center_tex_h, x, y + inset_y, inset_x, center_dst_h);
    // Center
    draw_region(inset_x, inset_y, center_tex_w, center_tex_h, x + inset_x, y + inset_y, center_dst_w, center_dst_h);
    // Right edge
    draw_region(tex_w - inset_x, inset_y, inset_x, center_tex_h, x + w - inset_x, y + inset_y, inset_x, center_dst_h);

    // Bottom-left corner
    draw_region(0.0, tex_h - inset_y, inset_x, inset_y, x, y + h - inset_y, inset_x, inset_y);
    // Bottom edge
    draw_region(inset_x, tex_h - inset_y, center_tex_w, inset_y, x + inset_x, y + h - inset_y, center_dst_w, inset_y);
    // Bottom-right corner
    draw_region(tex_w - inset_x, tex_h - inset_y, inset_x, inset_y, x + w - inset_x, y + h - inset_y, inset_x, inset_y);

    cr.restore().ok();
}

/// Draw a texture using horizontal 3-slice (left cap, stretchable middle, right cap).
/// - `left_cap_ratio`: left cap as ratio of texture width (e.g., 0.09375 = 12/128)
/// - `right_cap_start`: where right cap starts as ratio (e.g., 0.53125)
/// - `tex_right`: right edge of used texture region (e.g., 0.625 = 80/128)
/// - `tex_bottom`: bottom edge of used texture region (e.g., 0.6875 = 22/32)
fn draw_horizontal_slice_texture(
    cr: &cairo::Context,
    surface: &ImageSurface,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    left_cap_ratio: f64,
    right_cap_start: f64,
    tex_right: f64,
    tex_bottom: f64,
    alpha: f64,
) {
    let full_tex_w = surface.width() as f64;
    let full_tex_h = surface.height() as f64;

    // Calculate source regions in pixels from ratios
    let src_left_cap_w = full_tex_w * left_cap_ratio;
    let src_right_cap_x = full_tex_w * right_cap_start;
    let src_right_edge = full_tex_w * tex_right;
    let src_right_cap_w = src_right_edge - src_right_cap_x;
    let src_middle_w = src_right_cap_x - src_left_cap_w;
    let src_h = full_tex_h * tex_bottom;

    // Destination cap widths match source (1:1 pixels for caps)
    let dst_left_cap_w = src_left_cap_w * UI_SCALE;
    let dst_right_cap_w = src_right_cap_w * UI_SCALE;
    let dst_middle_w = w - dst_left_cap_w - dst_right_cap_w;

    // If target is smaller than caps, just scale the whole thing
    if dst_middle_w < 0.0 {
        return;
    }

    cr.save().ok();

    // Helper to draw a region of the texture
    // Source coords (sx, sy, sw, sh) are in texture pixels
    // Dest coords (dx, dy, dw, dh) are in screen pixels
    let draw_region = |sx: f64, sy: f64, sw: f64, sh: f64, dx: f64, dy: f64, dw: f64, dh: f64| {
        if sw <= 0.0 || sh <= 0.0 || dw <= 0.0 || dh <= 0.0 {
            return;
        }
        cr.save().ok();
        cr.translate(dx, dy);
        cr.rectangle(0.0, 0.0, dw, dh);
        cr.clip();
        cr.scale(dw / sw, dh / sh);
        cr.set_source_surface(surface, -sx, -sy).ok();
        cr.paint_with_alpha(alpha).ok();
        cr.restore().ok();
    };

    // Left cap: src (0, 0, left_cap, src_h) -> dst (x, y, left_cap, h)
    draw_region(0.0, 0.0, src_left_cap_w, src_h, x, y, dst_left_cap_w, h);

    // Middle: src (left_cap, 0, middle, src_h) -> dst (x+left_cap, y, dst_middle, h)
    draw_region(src_left_cap_w, 0.0, src_middle_w, src_h, x + dst_left_cap_w, y, dst_middle_w, h);

    // Right cap: src (right_cap_x, 0, right_cap_w, src_h) -> dst (x+w-right_cap, y, right_cap, h)
    draw_region(src_right_cap_x, 0.0, src_right_cap_w, src_h, x + w - dst_right_cap_w, y, dst_right_cap_w, h);

    cr.restore().ok();
}

/// Map WoW font paths to system font families.
/// Fonts installed to ~/.local/share/fonts/wow/ are available by their registered names.
fn wow_font_to_family(font_path: Option<&str>) -> &'static str {
    match font_path {
        Some(path) => {
            let path_upper = path.to_uppercase();
            if path_upper.contains("FRIZQT") && path_upper.contains("CYR") {
                // WoW Cyrillic font
                "FrizQuadrataCTT"
            } else if path_upper.contains("FRIZQT") {
                // WoW's main UI font - Friz Quadrata
                "Friz Quadrata TT"
            } else if path_upper.contains("ARIALN") {
                // WoW's narrow font
                "Arial Narrow"
            } else if path_upper.contains("SKURRI") {
                // WoW's fantasy font - fallback to serif (no TTF available)
                "Serif"
            } else if path_upper.contains("MORPHEUS") {
                // WoW's title font - use Trajan Pro as similar alternative
                "Trajan Pro 3"
            } else if path_upper.contains("FIRA") {
                // WeakAuras Fira fonts
                "Fira Sans"
            } else if path_upper.contains("MONO") {
                "Monospace"
            } else {
                "Sans"
            }
        }
        None => "Friz Quadrata TT", // Default to WoW font
    }
}

/// Draw text using Pango for proper font rendering.
fn draw_pango_text(
    cr: &cairo::Context,
    text: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    font_size: f64,
    justify_h: TextJustify,
    justify_v: TextJustify,
    color: (f64, f64, f64, f64),
) {
    draw_pango_text_with_font(cr, text, x, y, w, h, font_size, justify_h, justify_v, color, None, false)
}

/// Strip WoW texture/atlas markup from text (e.g., "|TInterface\ICONS\...:20:20:0:0|t")
/// Preserves plain text content while removing inline texture tags.
fn strip_wow_markup(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '|' {
            // Check for texture/atlas markup: |T...|t or |A...|a
            if let Some(&next) = chars.peek() {
                if next == 'T' || next == 'A' {
                    // Skip until we find |t or |a
                    let end_marker = if next == 'T' { 't' } else { 'a' };
                    chars.next(); // consume T or A
                    while let Some(ch) = chars.next() {
                        if ch == '|' {
                            if let Some(&marker) = chars.peek() {
                                if marker == end_marker {
                                    chars.next(); // consume the end marker
                                    break;
                                }
                            }
                        }
                    }
                    continue;
                }
                // Handle color markup: |c...|r - strip the |c...| and |r but keep content
                if next == 'c' {
                    // |cXXXXXXXX - skip 9 characters after |c
                    chars.next(); // consume 'c'
                    for _ in 0..8 {
                        chars.next();
                    }
                    continue;
                }
                if next == 'r' {
                    chars.next(); // consume 'r'
                    continue;
                }
            }
        }
        result.push(c);
    }

    result
}

/// Draw text using Pango with custom font.
fn draw_pango_text_with_font(
    cr: &cairo::Context,
    text: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    font_size: f64,
    justify_h: TextJustify,
    justify_v: TextJustify,
    color: (f64, f64, f64, f64),
    font_path: Option<&str>,
    word_wrap: bool,
) {
    let layout = pango_cairo::create_layout(cr);

    // Set up font description
    let mut font_desc = pango::FontDescription::new();
    font_desc.set_family(wow_font_to_family(font_path));
    font_desc.set_size((font_size * pango::SCALE as f64) as i32);
    layout.set_font_description(Some(&font_desc));

    // Set text and alignment
    layout.set_text(text);
    layout.set_width((w * pango::SCALE as f64) as i32);
    layout.set_alignment(match justify_h {
        TextJustify::Left => pango::Alignment::Left,
        TextJustify::Center => pango::Alignment::Center,
        TextJustify::Right => pango::Alignment::Right,
    });
    // Word wrapping based on frame setting
    if word_wrap {
        layout.set_wrap(pango::WrapMode::Word);
    }
    // Ellipsize text that overflows
    layout.set_ellipsize(pango::EllipsizeMode::End);

    // Get ink extents for proper visual vertical centering
    // pixel_size() returns logical bounds (includes leading), ink gives actual painted area
    let (ink_rect, _logical_rect) = layout.pixel_extents();
    let ink_height = ink_rect.height() as f64;
    let ink_y_offset = ink_rect.y() as f64; // Offset from baseline to ink top
    let text_y = match justify_v {
        TextJustify::Left => y - ink_y_offset, // TOP: align ink top to y
        TextJustify::Center => y + (h - ink_height) / 2.0 - ink_y_offset, // MIDDLE: center ink
        TextJustify::Right => y + h - ink_height - ink_y_offset, // BOTTOM: align ink bottom
    };

    // Set color and draw
    cr.set_source_rgba(color.0, color.1, color.2, color.3);
    cr.move_to(x, text_y);
    pango_cairo::show_layout(cr, &layout);
}

#[derive(Debug)]
enum Msg {
    FireEvent(String),
    MouseMove(f64, f64),
    MousePress(f64, f64),
    MouseRelease(f64, f64),
    Scroll(f64, f64), // (dx, dy) scroll delta
    ReloadUI,
    CommandInputChanged(String),
    ExecuteCommand,
    ProcessTimers,
    #[allow(dead_code)]
    Redraw,
}

#[relm4::component]
impl Component for App {
    type Init = (WowLuaEnv, PathBuf);
    type Input = Msg;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::ApplicationWindow {
            set_title: Some("WoW UI Simulator"),
            set_default_size: (1024, 768),

            add_controller = gtk::EventControllerKey {
                set_propagation_phase: gtk::PropagationPhase::Capture,
                connect_key_pressed[sender] => move |_, key, _, modifiers| {
                    if modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
                        if key == gdk::Key::r {
                            sender.input(Msg::ReloadUI);
                            return glib::Propagation::Stop;
                        }
                    }
                    glib::Propagation::Proceed
                }
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 7,

                // Title
                gtk::Label {
                    set_label: "WoW UI Simulator",
                    set_halign: gtk::Align::Start,
                    add_css_class: "title-2",
                },

                // Main content area: Canvas + Sidebar
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,
                    set_vexpand: true,

                    // WoW Canvas
                    gtk::Frame {
                        set_hexpand: true,
                        add_css_class: "view",

                        #[local_ref]
                        drawing_area -> gtk::DrawingArea {
                            set_hexpand: true,
                            set_vexpand: true,
                            set_content_width: 600,
                            set_content_height: 450,
                        },
                    },

                    // Frames sidebar
                    gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_min_content_width: 180,

                        #[local_ref]
                        frames_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 2,
                            set_margin_all: 6,
                        },
                    },
                },

                // Event buttons
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,

                    gtk::Button {
                        set_label: "ADDON_LOADED",
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::FireEvent("ADDON_LOADED".to_string()));
                        },
                    },

                    gtk::Button {
                        set_label: "PLAYER_LOGIN",
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::FireEvent("PLAYER_LOGIN".to_string()));
                        },
                    },

                    gtk::Button {
                        set_label: "PLAYER_ENTERING_WORLD",
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::FireEvent("PLAYER_ENTERING_WORLD".to_string()));
                        },
                    },
                },

                // Command input row
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,

                    gtk::Entry {
                        set_hexpand: true,
                        set_placeholder_text: Some("/command"),
                        #[watch]
                        set_text: &model.command_input,
                        connect_changed[sender] => move |entry| {
                            sender.input(Msg::CommandInputChanged(entry.text().to_string()));
                        },
                        connect_activate[sender] => move |_| {
                            sender.input(Msg::ExecuteCommand);
                        },
                    },

                    gtk::Button {
                        set_label: "Run",
                        add_css_class: "suggested-action",
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::ExecuteCommand);
                        },
                    },
                },

                // Console output
                gtk::ScrolledWindow {
                    set_min_content_height: 160,
                    set_max_content_height: 200,
                    set_hscrollbar_policy: gtk::PolicyType::Never,

                    #[local_ref]
                    console_label -> gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_valign: gtk::Align::Start,
                        set_wrap: true,
                        set_selectable: true,
                        add_css_class: "monospace",
                    },
                },
            },
        }
    }

    fn init(
        (env, textures_path): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Load custom CSS with USER priority to override theme
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_data(STYLE_CSS);
        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );

        // Force dark color scheme
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);

        let env_rc = Rc::new(RefCell::new(env));

        // Fire startup events
        fire_startup_events(&env_rc);

        // Collect console output from startup
        let mut log_messages = vec!["UI loaded. Press Ctrl+R to reload.".to_string()];
        {
            let env = env_rc.borrow();
            let mut state = env.state().borrow_mut();
            log_messages.append(&mut state.console_output);
        }

        // Create widgets
        let drawing_area = gtk::DrawingArea::new();
        let frames_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let console_label = gtk::Label::new(None);

        let texture_manager = Rc::new(RefCell::new(
            TextureManager::new(textures_path)
                .with_interface_path(DEFAULT_INTERFACE_PATH)
                .with_addons_path(DEFAULT_ADDONS_PATH)
        ));
        let texture_cache: Rc<RefCell<CairoTextureCache>> = Rc::new(RefCell::new(HashMap::new()));

        let ui_state: Rc<RefCell<SharedUiState>> = Rc::new(RefCell::new(SharedUiState::default()));

        let model = App {
            env: env_rc.clone(),
            log_messages,
            command_input: String::new(),
            drawing_area: drawing_area.clone(),
            frames_box: frames_box.clone(),
            console_label: console_label.clone(),
            ui_state: ui_state.clone(),
            mouse_down_frame: None,
            scroll_offset: 0.0,
        };

        // Update console label
        model.update_console_label();

        // Set up drawing area
        let env_for_draw = env_rc.clone();
        let tex_mgr_for_draw = Rc::clone(&texture_manager);
        let tex_cache_for_draw = Rc::clone(&texture_cache);
        let ui_state_for_draw = Rc::clone(&ui_state);
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            draw_wow_frames(
                &env_for_draw,
                &tex_mgr_for_draw,
                &tex_cache_for_draw,
                &ui_state_for_draw,
                cr,
                width,
                height,
            );
        });

        // Force initial draw
        drawing_area.queue_draw();

        // Mouse motion controller
        let motion_controller = gtk::EventControllerMotion::new();
        let sender_motion = sender.clone();
        motion_controller.connect_motion(move |_, x, y| {
            sender_motion.input(Msg::MouseMove(x, y));
        });
        drawing_area.add_controller(motion_controller);

        // Mouse click controller
        let click_controller = gtk::GestureClick::new();
        let sender_press = sender.clone();
        let sender_release = sender.clone();
        click_controller.connect_pressed(move |_, _n, x, y| {
            sender_press.input(Msg::MousePress(x, y));
        });
        click_controller.connect_released(move |_, _n, x, y| {
            sender_release.input(Msg::MouseRelease(x, y));
        });
        drawing_area.add_controller(click_controller);

        // Scroll controller for mouse wheel
        let scroll_controller = gtk::EventControllerScroll::new(
            gtk::EventControllerScrollFlags::VERTICAL | gtk::EventControllerScrollFlags::DISCRETE
        );
        let sender_scroll = sender.clone();
        scroll_controller.connect_scroll(move |_, dx, dy| {
            sender_scroll.input(Msg::Scroll(dx, dy));
            glib::Propagation::Stop
        });
        drawing_area.add_controller(scroll_controller);

        let widgets = view_output!();

        // Update frames sidebar
        model.update_frames_sidebar();

        // Initialize debug server
        let (mut cmd_rx, _guard) = debug_server::init();
        eprintln!("[wow-ui-sim] Debug server at {}", debug_server::socket_path().display());

        // Keep guard alive
        let guard = Rc::new(RefCell::new(Some(_guard)));
        let guard_clone = guard.clone();

        // Store window reference and sender for debug commands
        let window_weak = root.downgrade();
        let debug_sender = sender.clone();

        // Poll for debug commands
        glib::timeout_add_local(Duration::from_millis(50), move || {
            let _guard = guard_clone.borrow();

            while let Ok(cmd) = cmd_rx.try_recv() {
                let Some(window) = window_weak.upgrade() else {
                    continue;
                };

                match cmd {
                    DebugCommand::Dump { respond } => {
                        let dump = dump_widget_tree(&window);
                        let _ = respond.send(dump.to_string());
                    }
                    DebugCommand::DumpJson { respond } => {
                        let dump = dump_widget_tree(&window);
                        let _ = respond.send(dump.to_json());
                    }
                    DebugCommand::Click { label, respond } => {
                        if let Some(button) = find_button_by_label(&window, &label) {
                            button.emit_clicked();
                            let _ = respond.send(Ok(()));
                        } else {
                            let _ = respond.send(Err(format!("Button '{}' not found", label)));
                        }
                    }
                    DebugCommand::Input { field, value, respond } => {
                        if let Some(entry) = find_entry_by_placeholder(&window, &field) {
                            entry.set_text(&value);
                            let _ = respond.send(Ok(()));
                        } else {
                            let _ = respond.send(Err(format!("Entry '{}' not found", field)));
                        }
                    }
                    DebugCommand::Submit { respond } => {
                        use gtk::prelude::GtkWindowExt;
                        if let Some(focus) = GtkWindowExt::focus(&window) {
                            focus.activate();
                            let _ = respond.send(Ok(()));
                        } else {
                            let _ = respond.send(Err("No focused widget".to_string()));
                        }
                    }
                    DebugCommand::KeyPress { key, respond } => {
                        let msg = match key.as_str() {
                            "ctrl+r" | "Ctrl+r" => Some(Msg::ReloadUI),
                            "Return" => Some(Msg::ExecuteCommand),
                            _ => None,
                        };
                        if let Some(m) = msg {
                            debug_sender.input(m);
                            let _ = respond.send(Ok(()));
                        } else {
                            let _ = respond.send(Err(format!("Unknown key '{}'", key)));
                        }
                    }
                    DebugCommand::Screenshot { respond } => {
                        // Use capture_with_retry for reliability (3 attempts, 50ms delay)
                        let result = ScreenshotData::capture_with_retry(&window, 3, 50);
                        let _ = respond.send(result);
                    }
                }
            }
            ControlFlow::Continue
        });

        // Timer processing loop - check every 33ms (~30fps) for pending timers
        let timer_sender = sender.clone();
        glib::timeout_add_local(Duration::from_millis(33), move || {
            timer_sender.input(Msg::ProcessTimers);
            ControlFlow::Continue
        });

        // Schedule initial redraw after the main loop starts (when widget is mapped)
        let redraw_sender = sender.clone();
        glib::idle_add_local_once(move || {
            redraw_sender.input(Msg::Redraw);
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Msg, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            Msg::FireEvent(event) => {
                {
                    let env = self.env.borrow();
                    if let Err(e) = env.fire_event(&event) {
                        self.log_messages.push(format!("Event error: {}", e));
                    } else {
                        self.log_messages.push(format!("Fired: {}", event));
                    }
                }
                self.drain_console();
                self.drawing_area.queue_draw();
                self.update_frames_sidebar();
                self.update_console_label();
            }
            Msg::MouseMove(x, y) => {
                let new_hovered = self.hit_test(x, y);
                let old_hovered = self.ui_state.borrow().hovered_frame;
                if new_hovered != old_hovered {
                    // Fire OnLeave/OnEnter
                    let env = self.env.borrow();
                    if let Some(old_id) = old_hovered {
                        let _ = env.fire_script_handler(old_id, "OnLeave", vec![]);
                    }
                    if let Some(new_id) = new_hovered {
                        let _ = env.fire_script_handler(new_id, "OnEnter", vec![]);
                    }
                    drop(env);
                    self.ui_state.borrow_mut().hovered_frame = new_hovered;
                    self.drain_console();
                    self.drawing_area.queue_draw();
                    self.update_console_label();
                }
            }
            Msg::MousePress(x, y) => {
                if let Some(frame_id) = self.hit_test(x, y) {
                    self.mouse_down_frame = Some(frame_id);
                    self.ui_state.borrow_mut().pressed_frame = Some(frame_id);
                    let env = self.env.borrow();
                    let button_val = mlua::Value::String(env.lua().create_string("LeftButton").unwrap());
                    let _ = env.fire_script_handler(frame_id, "OnMouseDown", vec![button_val]);
                    drop(env);
                    self.drain_console();
                    self.drawing_area.queue_draw();
                    self.update_console_label();
                }
            }
            Msg::MouseRelease(x, y) => {
                let released_on = self.hit_test(x, y);
                if let Some(frame_id) = released_on {
                    let env = self.env.borrow();
                    let button_val = mlua::Value::String(env.lua().create_string("LeftButton").unwrap());

                    // Fire OnClick if released on same frame as pressed
                    if self.mouse_down_frame == Some(frame_id) {
                        let down_val = mlua::Value::Boolean(false);
                        let _ = env.fire_script_handler(frame_id, "OnClick", vec![button_val.clone(), down_val]);
                    }

                    let _ = env.fire_script_handler(frame_id, "OnMouseUp", vec![button_val]);
                    drop(env);
                    self.drain_console();
                    self.drawing_area.queue_draw();
                    self.update_console_label();
                }
                self.mouse_down_frame = None;
                self.ui_state.borrow_mut().pressed_frame = None;
            }
            Msg::Scroll(_dx, dy) => {
                // Scroll the addon list content
                // dy > 0 means scrolling down (content moves up, offset increases)
                let scroll_speed = 30.0; // pixels per scroll tick
                self.scroll_offset += dy * scroll_speed;

                // Clamp scroll offset to valid range
                // Calculate max based on content height vs viewport height
                // For now, use rough estimates: ~127 addons * ~24px per row = ~3048px content
                // Viewport ~450px, so max scroll ~2600px
                let max_scroll = 2600.0;
                self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);

                // Store scroll offset in ui_state for rendering
                self.ui_state.borrow_mut().scroll_offset = self.scroll_offset;
                self.drawing_area.queue_draw();
            }
            Msg::ReloadUI => {
                self.log_messages.push("Reloading UI...".to_string());
                {
                    let env = self.env.borrow();

                    if let Ok(s) = env.lua().create_string("WoWUISim") {
                        let _ = env.fire_event_with_args("ADDON_LOADED", &[mlua::Value::String(s)]);
                    }

                    let _ = env.fire_event("PLAYER_LOGIN");

                    let _ = env.fire_event_with_args(
                        "PLAYER_ENTERING_WORLD",
                        &[
                            mlua::Value::Boolean(false), // isInitialLogin
                            mlua::Value::Boolean(true),  // isReloadingUi
                        ],
                    );
                }
                self.drain_console();
                self.log_messages.push("UI reloaded.".to_string());
                self.drawing_area.queue_draw();
                self.update_frames_sidebar();
                self.update_console_label();
            }
            Msg::CommandInputChanged(input) => {
                self.command_input = input;
            }
            Msg::ExecuteCommand => {
                let cmd = self.command_input.clone();
                if !cmd.is_empty() {
                    self.log_messages.push(format!("> {}", cmd));

                    // Handle /frames specially
                    let cmd_lower = cmd.to_lowercase();
                    if cmd_lower == "/frames" || cmd_lower == "/f" {
                        let env = self.env.borrow();
                        let dump = env.dump_frames();
                        eprintln!("{}", dump);
                        let line_count = dump.lines().count();
                        self.log_messages.push(format!("Dumped {} frames to stderr", line_count / 2));
                    } else {
                        let env = self.env.borrow();
                        match env.dispatch_slash_command(&cmd) {
                            Ok(true) => {}
                            Ok(false) => {
                                self.log_messages.push(format!("Unknown command: {}", cmd));
                            }
                            Err(e) => {
                                self.log_messages.push(format!("Command error: {}", e));
                            }
                        }
                    }
                    self.drain_console();
                    self.command_input.clear();
                    self.drawing_area.queue_draw();
                    self.update_frames_sidebar();
                    self.update_console_label();
                }
            }
            Msg::ProcessTimers => {
                let env = self.env.borrow();
                match env.process_timers() {
                    Ok(count) if count > 0 => {
                        // Timer callbacks may have changed the UI
                        drop(env);
                        self.drain_console();
                        self.drawing_area.queue_draw();
                        self.update_frames_sidebar();
                        self.update_console_label();
                    }
                    Err(e) => {
                        eprintln!("Timer error: {}", e);
                    }
                    _ => {}
                }
            }
            Msg::Redraw => {
                self.drawing_area.queue_draw();
            }
        }
    }
}

impl App {
    /// Drain console output from Lua and add to log messages.
    fn drain_console(&mut self) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        self.log_messages.append(&mut state.console_output);
    }

    /// Update the console label with recent messages.
    fn update_console_label(&self) {
        let text: String = self.log_messages
            .iter()
            .rev()
            .take(10)
            .rev()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        self.console_label.set_label(&text);
    }

    /// Update the frames sidebar.
    fn update_frames_sidebar(&self) {
        // Clear existing children
        while let Some(child) = self.frames_box.first_child() {
            self.frames_box.remove(&child);
        }

        // Add "Frames" header
        let header = gtk::Label::new(Some("Frames"));
        header.set_halign(gtk::Align::Start);
        header.add_css_class("heading");
        self.frames_box.append(&header);

        // Get frame list
        let env = self.env.borrow();
        let state = env.state().borrow();

        let mut count = 0;
        for id in state.widgets.all_ids() {
            if let Some(frame) = state.widgets.get(id) {
                // Filter out anonymous and internal frames
                let name = match &frame.name {
                    Some(n) => n.as_str(),
                    None => continue,
                };
                if name.starts_with("__") || name.starts_with("DBM") || name.starts_with("Details")
                    || name.starts_with("Avatar") || name.starts_with("Plater")
                    || name.starts_with("WeakAuras") || name.starts_with("UIWidget")
                    || name.starts_with("GameMenu")
                {
                    continue;
                }
                if frame.width <= 0.0 || frame.height <= 0.0 {
                    continue;
                }

                let visible = if frame.visible { "visible" } else { "hidden" };
                let text = format!(
                    "{} [{}] {}x{} ({})",
                    name,
                    frame.widget_type.as_str(),
                    frame.width as i32,
                    frame.height as i32,
                    visible
                );

                // Truncate long names
                let display = if text.len() > 30 {
                    format!("{}...", &text[..27])
                } else {
                    text
                };

                let label = gtk::Label::new(Some(&display));
                label.set_halign(gtk::Align::Start);
                label.add_css_class("dim-label");
                self.frames_box.append(&label);

                count += 1;
                if count >= 15 {
                    break;
                }
            }
        }
    }

    /// Hit test to find frame at given canvas coordinates.
    fn hit_test(&self, x: f64, y: f64) -> Option<u64> {
        let env = self.env.borrow();
        let state = env.state().borrow();

        let scale_x = UI_SCALE as f32;
        let scale_y = UI_SCALE as f32;

        // Get screen dimensions from drawing area
        let screen_width = self.drawing_area.width() as f32;
        let screen_height = self.drawing_area.height() as f32;

        // Collect frames and sort by z-order
        let mut frames: Vec<_> = state.widgets.all_ids()
            .into_iter()
            .filter_map(|id| {
                let frame = state.widgets.get(id)?;
                if !frame.visible || !frame.mouse_enabled {
                    return None;
                }
                if matches!(frame.name.as_deref(), Some("UIParent") | Some("Minimap") | Some("WorldFrame") | Some("DEFAULT_CHAT_FRAME") | Some("ChatFrame1") | Some("EventToastManagerFrame") | Some("EditModeManagerFrame")) {
                    return None;
                }
                let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);
                Some((id, frame.frame_strata, frame.frame_level, rect))
            })
            .collect();

        frames.sort_by(|a, b| {
            a.1.cmp(&b.1)
                .then_with(|| a.2.cmp(&b.2))
                .then_with(|| a.0.cmp(&b.0))
        });

        // Check in reverse (topmost first)
        for (id, _, _, rect) in frames.iter().rev() {
            let scaled_x = rect.x * scale_x;
            let scaled_y = rect.y * scale_y;
            let scaled_w = rect.width * scale_x;
            let scaled_h = rect.height * scale_y;

            if x >= scaled_x as f64 && x <= (scaled_x + scaled_w) as f64
                && y >= scaled_y as f64 && y <= (scaled_y + scaled_h) as f64
            {
                return Some(*id);
            }
        }
        None
    }
}

/// Draw WoW frames using Cairo.
fn draw_wow_frames(
    env: &Rc<RefCell<WowLuaEnv>>,
    texture_manager: &Rc<RefCell<TextureManager>>,
    texture_cache: &Rc<RefCell<CairoTextureCache>>,
    ui_state: &Rc<RefCell<SharedUiState>>,
    cr: &gtk::cairo::Context,
    width: i32,
    height: i32,
) {
    let env = env.borrow();
    let state = env.state().borrow();

    // Dark background
    cr.set_source_rgb(0.05, 0.05, 0.08);
    cr.paint().ok();

    let scale_x = UI_SCALE;
    let scale_y = UI_SCALE;
    let screen_width = width as f32;
    let screen_height = height as f32;

    // Get UI state for hover/press detection
    let ui_state_ref = ui_state.borrow();
    let hovered_frame = ui_state_ref.hovered_frame;
    let pressed_frame = ui_state_ref.pressed_frame;
    let scroll_offset = ui_state_ref.scroll_offset;
    drop(ui_state_ref);

    // Find AddonList frame and collect all its descendant IDs
    let mut addonlist_ids = std::collections::HashSet::new();
    let addonlist_id = state.widgets.all_ids()
        .into_iter()
        .find(|&id| {
            state.widgets.get(id)
                .map(|f| f.name.as_deref() == Some("AddonList"))
                .unwrap_or(false)
        });
    // Debug: check if we found the AddonList frame (always log on first 5 renders)
    static RENDER_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let render_num = RENDER_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if render_num < 5 {
        eprintln!("[Render #{}] AddonList frame found: {:?}", render_num, addonlist_id);
    }
    if let Some(root_id) = addonlist_id {
        // BFS to collect all descendants
        let mut queue = vec![root_id];
        while let Some(id) = queue.pop() {
            addonlist_ids.insert(id);
            if let Some(frame) = state.widgets.get(id) {
                queue.extend(frame.children.iter().copied());
            }
        }
        // Debug: log how many AddonList children we collected
        static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            eprintln!("[Render] AddonList root_id={}, collected {} descendants", root_id, addonlist_ids.len());
            // List some children with their types
            for &id in addonlist_ids.iter().take(20) {
                if let Some(f) = state.widgets.get(id) {
                    eprintln!("  - id={} type={:?} text={:?}", id, f.widget_type, f.text.as_deref().unwrap_or("-"));
                }
            }
        }
    }

    // Collect and sort frames
    let mut frames: Vec<_> = state.widgets.all_ids()
        .into_iter()
        .filter_map(|id| {
            let frame = state.widgets.get(id)?;
            let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);
            // Get checked state for CheckButton widgets
            let checked = if let Some(crate::widget::AttributeValue::Boolean(c)) = frame.attributes.get("__checked") {
                *c
            } else {
                false
            };
            Some((
                id,
                frame.frame_strata,
                frame.frame_level,
                frame.widget_type,
                frame.visible,
                frame.alpha,
                frame.text.clone(),
                frame.text_color,
                frame.backdrop.clone(),
                frame.name.clone(),
                frame.texture.clone(),
                frame.color_texture,
                frame.font_size,
                frame.font.clone(),
                frame.justify_h,
                frame.justify_v,
                frame.word_wrap,
                frame.normal_texture.clone(),
                frame.pushed_texture.clone(),
                frame.highlight_texture.clone(),
                checked,
                rect,
            ))
        })
        .collect();

    // Sort by strata, level, type, id
    frames.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| {
                let type_order = |t: &WidgetType| match t {
                    WidgetType::Texture => 0,
                    WidgetType::FontString => 1,
                    WidgetType::Frame => 2,
                    WidgetType::ScrollFrame => 3,
                    WidgetType::Button => 4,
                    WidgetType::CheckButton => 5,
                    WidgetType::EditBox => 6,
                    WidgetType::Slider => 7,
                    WidgetType::StatusBar => 8,
                    WidgetType::Cooldown => 9,
                    WidgetType::Model | WidgetType::PlayerModel => 10,
                    WidgetType::ColorSelect => 11,
                    WidgetType::MessageFrame => 12,
                    WidgetType::SimpleHTML => 13,
                };
                type_order(&a.3).cmp(&type_order(&b.3))
            })
            .then_with(|| a.0.cmp(&b.0))
    });

    // Get mutable borrows for texture loading
    let mut tex_mgr = texture_manager.borrow_mut();
    let mut tex_cache = texture_cache.borrow_mut();

    for (id, _strata, _level, widget_type, visible, alpha, text, text_color, backdrop, name, texture_path, color_texture, font_size, font_path, justify_h, justify_v, word_wrap, normal_texture, pushed_texture, highlight_texture, checked, rect) in frames {
        // ONLY show AddonList frame and its children for now
        if !addonlist_ids.contains(&id) {
            continue;
        }

        if !visible {
            continue;
        }
        if rect.width <= 0.0 || rect.height <= 0.0 {
            continue;
        }


        let x = (rect.x as f64) * scale_x;
        let mut y = (rect.y as f64) * scale_y;
        let w = (rect.width as f64) * scale_x;
        let h = (rect.height as f64) * scale_y;

        // Apply scroll offset to scrollable content (AddonList children, not the main frame)
        // Check if this is the AddonList frame itself - don't scroll it
        let is_addonlist_root = addonlist_id == Some(id);
        if !is_addonlist_root && addonlist_ids.contains(&id) {
            // This is a child of AddonList, apply scroll
            y -= scroll_offset;

            // Skip rendering if scrolled out of view (simple clipping)
            // AddonList content area is roughly y=260 to y=700 (height ~440px)
            let content_top = 260.0; // Approximate top of scrollable area
            let content_bottom = 700.0; // Approximate bottom
            if y + h < content_top || y > content_bottom {
                continue; // Clipped out of view
            }
        }

        match widget_type {
            WidgetType::Frame => {
                // Frames only render their backdrop if enabled
                // The background textures come from child Texture widgets
                if backdrop.enabled {
                    // Try to draw backdrop texture
                    let mut drew_texture = false;
                    if let Some(ref bg_file) = backdrop.bg_file {
                        if let Some(surface) = load_cairo_surface(&mut tex_mgr, &mut tex_cache, bg_file) {
                            draw_tiled_texture(cr, &surface, x, y, w, h, alpha as f64);
                            drew_texture = true;
                        }
                    }

                    if !drew_texture {
                        // Fallback: draw colored background
                        let bg = &backdrop.bg_color;
                        cr.set_source_rgba(
                            bg.r as f64,
                            bg.g as f64,
                            bg.b as f64,
                            bg.a as f64 * alpha as f64,
                        );
                        cr.rectangle(x, y, w, h);
                        cr.fill().ok();
                    }

                    // Draw border
                    let bc = &backdrop.border_color;
                    cr.set_source_rgba(
                        bc.r as f64,
                        bc.g as f64,
                        bc.b as f64,
                        bc.a as f64 * alpha as f64,
                    );
                    cr.set_line_width(backdrop.edge_size.max(1.0) as f64);
                    cr.rectangle(x, y, w, h);
                    cr.stroke().ok();
                }

                // Draw title bar for AddonList frame
                if name.as_deref() == Some("AddonList") {
                    let title_height = 24.0;

                    // Title bar background (dark gradient)
                    cr.set_source_rgba(0.15, 0.12, 0.08, 0.95 * alpha as f64);
                    cr.rectangle(x, y, w, title_height);
                    cr.fill().ok();

                    // Title bar bottom border (gold)
                    cr.set_source_rgba(0.8, 0.6, 0.2, alpha as f64);
                    cr.set_line_width(2.0);
                    cr.move_to(x, y + title_height);
                    cr.line_to(x + w, y + title_height);
                    cr.stroke().ok();

                    // Title text "Addons"
                    draw_pango_text(
                        cr, "Addons", x + 10.0, y, w - 60.0, title_height, 16.0,
                        TextJustify::Left,
                        TextJustify::Center,
                        (1.0, 0.85, 0.4, alpha as f64),
                    );

                    // Close button (X) in top right
                    let close_size = 18.0;
                    let close_x = x + w - close_size - 6.0;
                    let close_y = y + (title_height - close_size) / 2.0;

                    // Close button background
                    cr.set_source_rgba(0.5, 0.2, 0.2, 0.8 * alpha as f64);
                    cr.rectangle(close_x, close_y, close_size, close_size);
                    cr.fill().ok();

                    // Close button X
                    cr.set_source_rgba(1.0, 0.9, 0.7, alpha as f64);
                    cr.set_line_width(2.0);
                    cr.set_line_cap(cairo::LineCap::Round);
                    let margin = 4.0;
                    cr.move_to(close_x + margin, close_y + margin);
                    cr.line_to(close_x + close_size - margin, close_y + close_size - margin);
                    cr.move_to(close_x + close_size - margin, close_y + margin);
                    cr.line_to(close_x + margin, close_y + close_size - margin);
                    cr.stroke().ok();
                }
            }
            WidgetType::Button => {
                let is_pressed = pressed_frame == Some(id);

                // Choose texture based on button state (pressed or normal)
                let mut drew_background = false;
                let button_texture = if is_pressed {
                    pushed_texture.as_ref().or(normal_texture.as_ref())
                } else {
                    normal_texture.as_ref()
                };

                if let Some(tex_path) = button_texture {
                    if let Some(surface) = load_cairo_surface(&mut tex_mgr, &mut tex_cache, tex_path) {
                        // WoW buttons use 3-slice horizontal stretching
                        // From Blizzard TexCoords (same for Up/Down/Disabled):
                        //   Left:   0.0     - 0.09375  (12px cap)
                        //   Middle: 0.09375 - 0.53125  (56px stretchable)
                        //   Right:  0.53125 - 0.625    (12px cap)
                        //   Height: 0.0     - 0.6875   (22px of 32px)
                        draw_horizontal_slice_texture(
                            cr, &surface, x, y, w, h,
                            0.09375,  // left_cap_ratio
                            0.53125,  // right_cap_start
                            0.625,    // tex_right
                            0.6875,   // tex_bottom
                            alpha as f64,
                        );
                        drew_background = true;
                    }
                }

                // Fallback to backdrop if no button texture
                if !drew_background && backdrop.enabled {
                    if let Some(ref bg_file) = backdrop.bg_file {
                        if let Some(surface) = load_cairo_surface(&mut tex_mgr, &mut tex_cache, bg_file) {
                            draw_scaled_texture(cr, &surface, x, y, w, h, alpha as f64);
                            drew_background = true;
                        }
                    }
                }

                if !drew_background {
                    // Default button styling (dark red gradient-like)
                    cr.set_source_rgba(0.15, 0.05, 0.05, 0.95 * alpha as f64);
                    cr.rectangle(x, y, w, h);
                    cr.fill().ok();

                    cr.set_source_rgba(0.6, 0.45, 0.15, alpha as f64);
                    cr.set_line_width(1.5);
                    cr.rectangle(x, y, w, h);
                    cr.stroke().ok();
                }

                // Draw highlight texture on hover (not when pressed)
                if hovered_frame == Some(id) && !is_pressed {
                    if let Some(ref tex_path) = highlight_texture {
                        if let Some(surface) = load_cairo_surface(&mut tex_mgr, &mut tex_cache, tex_path) {
                            // From UIPanelButtonHighlightTexture:
                            //   TexCoords: right=0.625, bottom=0.6875
                            //   alphaMode="ADD" (additive blending)
                            draw_texture_with_texcoords(
                                cr, &surface, x, y, w, h,
                                0.625,  // tex_right
                                0.6875, // tex_bottom
                                alpha as f64,
                                true,   // additive blending
                            );
                        }
                    }
                }

                // Draw button text (centered) using Pango
                if let Some(ref txt) = text {
                    draw_pango_text_with_font(
                        cr, txt, x, y, w, h, font_size as f64,
                        TextJustify::Center, // Buttons always center text horizontally
                        TextJustify::Center, // Buttons always center text vertically
                        (text_color.r as f64, text_color.g as f64, text_color.b as f64, text_color.a as f64 * alpha as f64),
                        font_path.as_deref(),
                        false, // Buttons don't wrap
                    );
                }
            }
            WidgetType::Texture => {
                // Try to render actual texture
                let mut drew_texture = false;

                // First check for color texture (SetColorTexture)
                if let Some(color) = color_texture {
                    cr.set_source_rgba(
                        color.r as f64,
                        color.g as f64,
                        color.b as f64,
                        color.a as f64 * alpha as f64,
                    );
                    cr.rectangle(x, y, w, h);
                    cr.fill().ok();
                    drew_texture = true;
                }

                // Then try file texture
                if !drew_texture {
                    if let Some(ref path) = texture_path {
                        if let Some(surface) = load_cairo_surface(&mut tex_mgr, &mut tex_cache, path) {
                            draw_scaled_texture(cr, &surface, x, y, w, h, alpha as f64);
                            drew_texture = true;
                        }
                    }
                }

                if !drew_texture {
                    // Fallback: draw placeholder
                    cr.set_source_rgba(0.4, 0.35, 0.3, 0.7 * alpha as f64);
                    cr.rectangle(x, y, w, h);
                    cr.fill().ok();

                    // Diagonal lines to indicate placeholder
                    cr.set_source_rgba(1.0, 1.0, 1.0, 0.2 * alpha as f64);
                    cr.set_line_width(1.0);
                    cr.move_to(x, y);
                    cr.line_to(x + w, y + h);
                    cr.stroke().ok();
                    cr.move_to(x + w, y);
                    cr.line_to(x, y + h);
                    cr.stroke().ok();
                }
            }
            WidgetType::FontString => {
                // Draw FontString text using Pango with frame's justification and font
                if let Some(ref txt) = text {
                    // Strip WoW markup (texture tags, color codes) for display
                    let clean_text = strip_wow_markup(txt);

                    // Debug: print text color for addon list items (first few only)
                    static DEBUG_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
                    let count = DEBUG_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if count < 30 && clean_text.len() > 3 {
                        eprintln!("[FontString] '{}' color: ({:.2}, {:.2}, {:.2}, {:.2})",
                            &clean_text[..clean_text.len().min(25)],
                            text_color.r, text_color.g, text_color.b, text_color.a);
                    }

                    draw_pango_text_with_font(
                        cr, &clean_text, x, y, w, h, font_size as f64,
                        justify_h,
                        justify_v,
                        (text_color.r as f64, text_color.g as f64, text_color.b as f64, text_color.a as f64 * alpha as f64),
                        font_path.as_deref(),
                        word_wrap,
                    );
                }
            }
            WidgetType::EditBox => {
                // Draw EditBox as a text input field
                // Background
                cr.set_source_rgba(0.08, 0.08, 0.1, 0.9 * alpha as f64);
                cr.rectangle(x, y, w, h);
                cr.fill().ok();

                // Border (slightly inset look)
                cr.set_source_rgba(0.3, 0.3, 0.35, alpha as f64);
                cr.set_line_width(1.0);
                cr.rectangle(x + 0.5, y + 0.5, w - 1.0, h - 1.0);
                cr.stroke().ok();

                // Draw text content (left-aligned, vertically centered)
                if let Some(ref txt) = text {
                    // Add padding
                    let padding = 4.0;
                    draw_pango_text(
                        cr, txt,
                        x + padding, y, w - padding * 2.0, h,
                        font_size as f64,
                        TextJustify::Left,
                        TextJustify::Center,
                        (text_color.r as f64, text_color.g as f64, text_color.b as f64, text_color.a as f64 * alpha as f64),
                    );
                }
            }
            WidgetType::ScrollFrame => {
                // ScrollFrame renders like a Frame with clipping area
                if backdrop.enabled {
                    let bg = &backdrop.bg_color;
                    cr.set_source_rgba(
                        bg.r as f64,
                        bg.g as f64,
                        bg.b as f64,
                        bg.a as f64 * alpha as f64,
                    );
                    cr.rectangle(x, y, w, h);
                    cr.fill().ok();

                    // Draw border
                    let bc = &backdrop.border_color;
                    cr.set_source_rgba(
                        bc.r as f64,
                        bc.g as f64,
                        bc.b as f64,
                        bc.a as f64 * alpha as f64,
                    );
                    cr.set_line_width(backdrop.edge_size.max(1.0) as f64);
                    cr.rectangle(x, y, w, h);
                    cr.stroke().ok();
                }
            }
            WidgetType::Slider => {
                // Draw slider track
                let track_height = 4.0;
                let track_y = y + (h - track_height) / 2.0;
                cr.set_source_rgba(0.2, 0.2, 0.25, 0.9 * alpha as f64);
                cr.rectangle(x, track_y, w, track_height);
                cr.fill().ok();

                // Draw slider thumb (centered for now - would use slider value)
                let thumb_width = 12.0;
                let thumb_height = 16.0;
                let thumb_x = x + (w - thumb_width) / 2.0;
                let thumb_y = y + (h - thumb_height) / 2.0;
                cr.set_source_rgba(0.6, 0.5, 0.3, alpha as f64);
                cr.rectangle(thumb_x, thumb_y, thumb_width, thumb_height);
                cr.fill().ok();

                // Thumb border
                cr.set_source_rgba(0.8, 0.7, 0.4, alpha as f64);
                cr.set_line_width(1.0);
                cr.rectangle(thumb_x, thumb_y, thumb_width, thumb_height);
                cr.stroke().ok();
            }
            WidgetType::CheckButton => {
                // Draw checkbox - use full widget size for positioning
                let box_size = h.min(w).min(20.0);
                let box_x = x + (w - box_size) / 2.0;
                let box_y = y + (h - box_size) / 2.0;

                // Checkbox background
                cr.set_source_rgba(0.12, 0.12, 0.15, 0.9 * alpha as f64);
                cr.rectangle(box_x, box_y, box_size, box_size);
                cr.fill().ok();

                // Checkbox border (gold)
                cr.set_source_rgba(0.8, 0.6, 0.2, alpha as f64);
                cr.set_line_width(1.5);
                cr.rectangle(box_x, box_y, box_size, box_size);
                cr.stroke().ok();

                // Draw checkmark if checked
                if checked {
                    // Draw a gold checkmark
                    cr.set_source_rgba(1.0, 0.8, 0.2, alpha as f64);
                    cr.set_line_width(2.5);
                    cr.set_line_cap(cairo::LineCap::Round);
                    cr.set_line_join(cairo::LineJoin::Round);

                    // Checkmark path (starting from bottom-left, through center-bottom, to top-right)
                    let margin = box_size * 0.2;
                    let check_x1 = box_x + margin;
                    let check_y1 = box_y + box_size * 0.5;
                    let check_x2 = box_x + box_size * 0.4;
                    let check_y2 = box_y + box_size - margin;
                    let check_x3 = box_x + box_size - margin;
                    let check_y3 = box_y + margin;

                    cr.move_to(check_x1, check_y1);
                    cr.line_to(check_x2, check_y2);
                    cr.line_to(check_x3, check_y3);
                    cr.stroke().ok();
                }

                // Draw label text (to the right of checkbox)
                if let Some(ref txt) = text {
                    let text_x = x + w + 6.0;
                    let text_w = 200.0; // Fixed width for label
                    draw_pango_text(
                        cr, txt, text_x, y, text_w, h, font_size as f64,
                        TextJustify::Left,
                        TextJustify::Center,
                        (text_color.r as f64, text_color.g as f64, text_color.b as f64, text_color.a as f64 * alpha as f64),
                    );
                }
            }
            WidgetType::StatusBar => {
                // Draw status bar track
                cr.set_source_rgba(0.1, 0.1, 0.12, 0.9 * alpha as f64);
                cr.rectangle(x, y, w, h);
                cr.fill().ok();

                // Draw filled portion (50% for now - would use actual value)
                let fill_width = w * 0.5;
                cr.set_source_rgba(0.2, 0.6, 0.2, alpha as f64);
                cr.rectangle(x, y, fill_width, h);
                cr.fill().ok();

                // Border
                cr.set_source_rgba(0.4, 0.35, 0.2, alpha as f64);
                cr.set_line_width(1.0);
                cr.rectangle(x, y, w, h);
                cr.stroke().ok();
            }
            WidgetType::Cooldown => {
                // Draw cooldown overlay (semi-transparent radial sweep)
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.5 * alpha as f64);
                cr.rectangle(x, y, w, h);
                cr.fill().ok();
            }
            WidgetType::Model | WidgetType::PlayerModel => {
                // Draw placeholder for 3D model
                cr.set_source_rgba(0.15, 0.15, 0.2, 0.8 * alpha as f64);
                cr.rectangle(x, y, w, h);
                cr.fill().ok();

                // Draw "3D" indicator
                draw_pango_text(
                    cr, "3D Model", x, y, w, h, 10.0,
                    TextJustify::Center,
                    TextJustify::Center,
                    (0.5, 0.5, 0.5, alpha as f64),
                );
            }
            WidgetType::ColorSelect => {
                // Draw color picker placeholder
                cr.set_source_rgba(0.5, 0.5, 0.5, alpha as f64);
                cr.rectangle(x, y, w, h);
                cr.fill().ok();

                // Rainbow gradient hint
                cr.set_source_rgba(1.0, 0.0, 0.0, 0.3 * alpha as f64);
                cr.rectangle(x, y, w / 3.0, h);
                cr.fill().ok();
                cr.set_source_rgba(0.0, 1.0, 0.0, 0.3 * alpha as f64);
                cr.rectangle(x + w / 3.0, y, w / 3.0, h);
                cr.fill().ok();
                cr.set_source_rgba(0.0, 0.0, 1.0, 0.3 * alpha as f64);
                cr.rectangle(x + 2.0 * w / 3.0, y, w / 3.0, h);
                cr.fill().ok();
            }
            WidgetType::MessageFrame | WidgetType::SimpleHTML => {
                // Draw message frame (text display area)
                if backdrop.enabled {
                    let bg = &backdrop.bg_color;
                    cr.set_source_rgba(bg.r as f64, bg.g as f64, bg.b as f64, bg.a as f64 * alpha as f64);
                    cr.rectangle(x, y, w, h);
                    cr.fill().ok();
                }

                // Draw text if present
                if let Some(ref txt) = text {
                    draw_pango_text(
                        cr, txt, x + 4.0, y, w - 8.0, h, font_size as f64,
                        justify_h,
                        TextJustify::Left, // Messages typically start at top
                        (text_color.r as f64, text_color.g as f64, text_color.b as f64, text_color.a as f64 * alpha as f64),
                    );
                }
            }
        }
    }

    // Draw center crosshair
    let cx = width as f64 / 2.0;
    let cy = height as f64 / 2.0;
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.3);
    cr.set_line_width(1.0);
    cr.move_to(cx - 20.0, cy);
    cr.line_to(cx + 20.0, cy);
    cr.stroke().ok();
    cr.move_to(cx, cy - 20.0);
    cr.line_to(cx, cy + 20.0);
    cr.stroke().ok();
}

/// Compute frame rect with anchor resolution.
fn compute_frame_rect(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    screen_width: f32,
    screen_height: f32,
) -> LayoutRect {
    let frame = match registry.get(id) {
        Some(f) => f,
        None => return LayoutRect::default(),
    };

    let parent_rect = if let Some(parent_id) = frame.parent_id {
        compute_frame_rect(registry, parent_id, screen_width, screen_height)
    } else {
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
        }
    };

    // If no anchors, default to center of parent
    if frame.anchors.is_empty() {
        return LayoutRect {
            x: parent_rect.x + (parent_rect.width - frame.width) / 2.0,
            y: parent_rect.y + (parent_rect.height - frame.height) / 2.0,
            width: frame.width,
            height: frame.height,
        };
    }

    // Check for multi-point anchoring to compute position and possibly size from anchors
    if frame.anchors.len() >= 2 {
        use crate::widget::AnchorPoint;

        // Track left/right x-coordinates and top/bottom y-coordinates separately
        // This handles LEFT+RIGHT anchors correctly (for width) and TOP+BOTTOM (for height)
        let mut left_x: Option<f32> = None;
        let mut right_x: Option<f32> = None;
        let mut top_y: Option<f32> = None;
        let mut bottom_y: Option<f32> = None;

        for anchor in &frame.anchors {
            // Get the relative frame's rect (use parent if relative_to_id is None)
            let relative_rect = if let Some(rel_id) = anchor.relative_to_id {
                compute_frame_rect(registry, rel_id as u64, screen_width, screen_height)
            } else {
                parent_rect
            };

            let (anchor_x, anchor_y) = anchor_position(
                anchor.relative_point,
                relative_rect.x,
                relative_rect.y,
                relative_rect.width,
                relative_rect.height,
            );
            let target_x = anchor_x + anchor.x_offset;
            let target_y = anchor_y - anchor.y_offset;

            // Record edge positions based on which edge this anchor affects
            match anchor.point {
                AnchorPoint::TopLeft => {
                    left_x = Some(target_x);
                    top_y = Some(target_y);
                }
                AnchorPoint::TopRight => {
                    right_x = Some(target_x);
                    top_y = Some(target_y);
                }
                AnchorPoint::BottomLeft => {
                    left_x = Some(target_x);
                    bottom_y = Some(target_y);
                }
                AnchorPoint::BottomRight => {
                    right_x = Some(target_x);
                    bottom_y = Some(target_y);
                }
                AnchorPoint::Top => {
                    top_y = Some(target_y);
                }
                AnchorPoint::Bottom => {
                    bottom_y = Some(target_y);
                }
                AnchorPoint::Left => {
                    left_x = Some(target_x);
                }
                AnchorPoint::Right => {
                    right_x = Some(target_x);
                }
                AnchorPoint::Center => {}
            }
        }

        // Compute width from left/right anchors only if stored width is 0
        let computed_width = if frame.width == 0.0 {
            if let (Some(lx), Some(rx)) = (left_x, right_x) {
                Some((rx - lx).max(0.0))
            } else {
                None
            }
        } else {
            None
        };

        // Compute height from top/bottom anchors only if stored height is 0
        let computed_height = if frame.height == 0.0 {
            if let (Some(ty), Some(by)) = (top_y, bottom_y) {
                Some((by - ty).max(0.0))
            } else {
                None
            }
        } else {
            None
        };

        // Determine final dimensions - prefer stored values
        let final_width = if frame.width > 0.0 { frame.width } else { computed_width.unwrap_or(0.0) };
        let final_height = if frame.height > 0.0 { frame.height } else { computed_height.unwrap_or(0.0) };

        // Determine position from available anchors
        let final_x = left_x.unwrap_or_else(|| {
            right_x.map(|rx| rx - final_width).unwrap_or(parent_rect.x)
        });
        let final_y = top_y.unwrap_or_else(|| {
            bottom_y.map(|by| by - final_height).unwrap_or(parent_rect.y)
        });

        return LayoutRect {
            x: final_x,
            y: final_y,
            width: final_width,
            height: final_height,
        };
    }

    // Single anchor case - use explicit width/height
    let anchor = &frame.anchors[0];
    let width = frame.width;
    let height = frame.height;

    let (parent_anchor_x, parent_anchor_y) = anchor_position(
        anchor.relative_point,
        parent_rect.x,
        parent_rect.y,
        parent_rect.width,
        parent_rect.height,
    );

    let target_x = parent_anchor_x + anchor.x_offset;
    let target_y = parent_anchor_y - anchor.y_offset;

    let (frame_x, frame_y) =
        frame_position_from_anchor(anchor.point, target_x, target_y, width, height);

    LayoutRect {
        x: frame_x,
        y: frame_y,
        width,
        height,
    }
}

fn anchor_position(
    point: crate::widget::AnchorPoint,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    use crate::widget::AnchorPoint;
    match point {
        AnchorPoint::TopLeft => (x, y),
        AnchorPoint::Top => (x + w / 2.0, y),
        AnchorPoint::TopRight => (x + w, y),
        AnchorPoint::Left => (x, y + h / 2.0),
        AnchorPoint::Center => (x + w / 2.0, y + h / 2.0),
        AnchorPoint::Right => (x + w, y + h / 2.0),
        AnchorPoint::BottomLeft => (x, y + h),
        AnchorPoint::Bottom => (x + w / 2.0, y + h),
        AnchorPoint::BottomRight => (x + w, y + h),
    }
}

fn frame_position_from_anchor(
    point: crate::widget::AnchorPoint,
    anchor_x: f32,
    anchor_y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    use crate::widget::AnchorPoint;
    match point {
        AnchorPoint::TopLeft => (anchor_x, anchor_y),
        AnchorPoint::Top => (anchor_x - w / 2.0, anchor_y),
        AnchorPoint::TopRight => (anchor_x - w, anchor_y),
        AnchorPoint::Left => (anchor_x, anchor_y - h / 2.0),
        AnchorPoint::Center => (anchor_x - w / 2.0, anchor_y - h / 2.0),
        AnchorPoint::Right => (anchor_x - w, anchor_y - h / 2.0),
        AnchorPoint::BottomLeft => (anchor_x, anchor_y - h),
        AnchorPoint::Bottom => (anchor_x - w / 2.0, anchor_y - h),
        AnchorPoint::BottomRight => (anchor_x - w, anchor_y - h),
    }
}

/// Check if a frame is effectively visible (itself and all ancestors are visible).
#[allow(dead_code)]
fn is_effectively_visible(registry: &crate::widget::WidgetRegistry, id: u64) -> bool {
    let mut current_id = Some(id);
    while let Some(cid) = current_id {
        match registry.get(cid) {
            Some(frame) => {
                if !frame.visible {
                    return false;
                }
                current_id = frame.parent_id;
            }
            None => break,
        }
    }
    true
}
