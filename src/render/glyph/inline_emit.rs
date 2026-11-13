//! Text with inline texture and atlas escapes (`|T...|t`, `|A...|a`).
//!
//! The client draws these escapes as images in the line and makes room for
//! them, so text after an icon moves right by the icon's width. Text runs and
//! icons are laid out here piece by piece: text pieces are shaped and drawn by
//! [`emit_text_quads`], icons become textured quads centered on their line.

use crate::render::inline_markup::{
    InlineIcon, InlineIconSource, InlinePiece, split_inline_markup,
};
use iced::{Point, Rectangle, Size};

use super::GlyphAtlas;
use super::text_emit::emit_text_quads;
use crate::render::font::{WowFontSystem, line_height_for_font_size};
use crate::render::shader::{BlendMode, QuadBatch};
use crate::widget::TextJustify;

/// Text in one color, escapes included.
pub struct InlineTextRun<'a> {
    pub text: &'a str,
    pub color: [f32; 4],
}

/// Everything [`emit_inline_markup_quads`] needs besides the runs.
pub struct InlineTextRequest<'a> {
    pub runs: &'a [InlineTextRun<'a>],
    pub bounds: Rectangle,
    pub font_path: Option<&'a str>,
    /// Font size in screen pixels.
    pub font_size: f32,
    /// Screen pixels per UI unit, for the icon sizes and offsets the escapes
    /// give in UI units.
    pub icon_scale: f32,
    pub justify_h: TextJustify,
    pub justify_v: TextJustify,
    pub word_wrap: bool,
    pub max_lines: u32,
    pub spacing: f32,
    pub smooth_scaling: bool,
    pub glyph_tex_index: i32,
    pub shadow_color: Option<[f32; 4]>,
    pub shadow_offset: (f32, f32),
    pub outline: crate::widget::TextOutline,
    /// Alpha the icons are drawn with.
    pub alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InlineItem {
    Text { text: String, color: [f32; 4] },
    Icon(InlineIcon),
}

/// A laid-out item: its x within the line and its size in screen pixels.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlacedItem {
    pub(crate) item: InlineItem,
    pub(crate) x: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct InlineLine {
    pub(crate) items: Vec<PlacedItem>,
    pub(crate) width: f32,
}

/// Lay the runs out in lines and emit their glyph and icon quads.
///
/// R88e.6 verifies taller-icon line-height growth. The default vertical mode
/// and the +0.5/floor coordinate's axis and drawing path remain UNKNOWN.
pub fn emit_inline_markup_quads(
    batch: &mut QuadBatch,
    font_system: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    request: &InlineTextRequest<'_>,
) {
    let Some(line_height) = line_height_for_font_size(request.font_size, request.smooth_scaling)
    else {
        return;
    };
    let mut lines = layout_inline_runs(request, |text| {
        font_system.measure_text_width(text, request.font_path, request.font_size)
    });
    if request.max_lines > 0 {
        lines.truncate(request.max_lines as usize);
    }
    let heights = line_box_heights(&lines, line_height);
    let total_height =
        heights.iter().sum::<f32>() + lines.len().saturating_sub(1) as f32 * request.spacing;
    let top = request.bounds.y
        + match request.justify_v {
            TextJustify::Left => 0.0,
            TextJustify::Center => (request.bounds.height - total_height) / 2.0,
            TextJustify::Right => request.bounds.height - total_height,
        };
    let mut y = top;
    for (line, &height) in lines.iter().zip(&heights) {
        let x = line_start_x(request.bounds, request.justify_h, line.width);
        for placed in &line.items {
            let item_x = x + placed.x;
            match &placed.item {
                InlineItem::Text { text, color } => {
                    let bounds = Rectangle::new(
                        Point::new(item_x, y + (height - line_height) / 2.0),
                        Size::new(placed.width.max(1.0), line_height),
                    );
                    emit_text_piece(
                        batch,
                        font_system,
                        glyph_atlas,
                        request,
                        text,
                        bounds,
                        *color,
                    );
                }
                InlineItem::Icon(icon) => {
                    // INFERRED simulator placement: mode 1 centers on the caller metric,
                    // but the native default mode and the 0/1/2 mapping are UNKNOWN.
                    let bounds = Rectangle::new(
                        Point::new(
                            item_x + icon.offset_x * request.icon_scale,
                            y + (height - placed.height) / 2.0 - icon.offset_y * request.icon_scale,
                        ),
                        Size::new(placed.width, placed.height),
                    );
                    push_inline_icon(batch, icon, bounds, request.alpha);
                }
            }
        }
        y += height + request.spacing;
    }
}

/// Height of each laid-out line: the text's line height, or its tallest
/// icon when that is taller. R88e.6 verifies this height increase. The
/// centering here is simulator policy; the native default vertical mode is UNKNOWN.
pub(crate) fn line_box_heights(lines: &[InlineLine], line_height: f32) -> Vec<f32> {
    lines
        .iter()
        .map(|line| {
            line.items
                .iter()
                .filter(|placed| matches!(placed.item, InlineItem::Icon(_)))
                .fold(line_height, |height, placed| height.max(placed.height))
        })
        .collect()
}

/// Break the runs into lines. A line ends at a line break and, with word
/// wrap, before a word or icon that would cross the right edge of the
/// bounds. R88e.6 verifies that accepted icon width enters the wrap check.
/// `measure` gives the pixel width of escape-free text.
pub(crate) fn layout_inline_runs(
    request: &InlineTextRequest<'_>,
    mut measure: impl FnMut(&str) -> f32,
) -> Vec<InlineLine> {
    let max_width = request.bounds.width;
    let line_height = line_height_for_font_size(request.font_size, request.smooth_scaling)
        .unwrap_or(request.font_size)
        / request.icon_scale;
    let mut lines = vec![InlineLine::default()];
    for run in request.runs {
        for piece in split_inline_markup(run.text) {
            match piece {
                InlinePiece::Text(text) => {
                    for (index, paragraph) in text.split('\n').enumerate() {
                        if index > 0 {
                            lines.push(InlineLine::default());
                        }
                        for chunk in text_chunks(paragraph, request.word_wrap) {
                            let width = measure(chunk);
                            let item = InlineItem::Text {
                                text: chunk.to_string(),
                                color: run.color,
                            };
                            let breakable = !chunk.trim().is_empty();
                            place(&mut lines, request, max_width, breakable, item, width, 0.0);
                        }
                    }
                }
                InlinePiece::Icon(icon) => {
                    let (width, height) = icon.size(line_height);
                    let (width, height) = (width * request.icon_scale, height * request.icon_scale);
                    place(
                        &mut lines,
                        request,
                        max_width,
                        true,
                        InlineItem::Icon(icon),
                        width,
                        height,
                    );
                }
            }
        }
    }
    lines
}

fn text_chunks(text: &str, word_wrap: bool) -> Vec<&str> {
    if text.is_empty() {
        Vec::new()
    } else if word_wrap {
        text.split_inclusive(char::is_whitespace).collect()
    } else {
        vec![text]
    }
}

fn place(
    lines: &mut Vec<InlineLine>,
    request: &InlineTextRequest<'_>,
    max_width: f32,
    breakable: bool,
    item: InlineItem,
    width: f32,
    height: f32,
) {
    let line = lines.last_mut().expect("layout keeps one line");
    if request.word_wrap && breakable && line.width > 0.0 && line.width + width > max_width {
        lines.push(InlineLine::default());
    }
    let line = lines.last_mut().expect("layout keeps one line");
    line.items.push(PlacedItem {
        item,
        x: line.width,
        width,
        height,
    });
    line.width += width;
}

fn line_start_x(bounds: Rectangle, justify_h: TextJustify, line_width: f32) -> f32 {
    match justify_h {
        TextJustify::Left => bounds.x,
        TextJustify::Center => bounds.x + ((bounds.width - line_width) / 2.0).max(0.0),
        TextJustify::Right => bounds.x + (bounds.width - line_width).max(0.0),
    }
}

fn emit_text_piece(
    batch: &mut QuadBatch,
    font_system: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    request: &InlineTextRequest<'_>,
    text: &str,
    bounds: Rectangle,
    color: [f32; 4],
) {
    emit_text_quads(
        batch,
        font_system,
        glyph_atlas,
        text,
        bounds,
        request.font_path,
        request.font_size,
        color,
        TextJustify::Left,
        TextJustify::Center,
        request.glyph_tex_index,
        request.shadow_color,
        request.shadow_offset,
        request.outline,
        false,
        0,
        0.0,
        request.smooth_scaling,
        Some(text),
        None,
    );
}

/// Push the icon's textured quad. Its texture loads like any other texture
/// path; unknown atlas and file IDs draw no quad in this simulator, while
/// their native result remains UNKNOWN (R88d). R88e.6 verifies one post-size
/// coordinate snap, but its axis and drawing path are UNKNOWN.
pub(crate) fn push_inline_icon(
    batch: &mut QuadBatch,
    icon: &InlineIcon,
    bounds: Rectangle,
    alpha: f32,
) {
    let [r, g, b] = icon.vertex_color.unwrap_or([1.0, 1.0, 1.0]);
    let tint = [r, g, b, alpha];
    match &icon.source {
        InlineIconSource::File(path) => push_file_icon(batch, path, icon.tex_coords, bounds, tint),
        InlineIconSource::FileId(id) => {
            if let Some(path) = crate::manifest_interface_data::get_texture_path(*id) {
                let path = format!("Interface\\{}", path.replace('/', "\\"));
                push_file_icon(batch, &path, icon.tex_coords, bounds, tint);
            }
        }
        InlineIconSource::Atlas(name) => {
            if let Some(lookup) = crate::atlas::get_render_atlas_info(name) {
                let info = lookup.info;
                // The atlas region gets a texture of its own, as for an atlas
                // set on a Texture, so filtering cannot bleed in its neighbours.
                let path = format!(
                    "{}@crop:{:.6},{:.6},{:.6},{:.6}",
                    info.file,
                    info.left_tex_coord,
                    info.right_tex_coord,
                    info.top_tex_coord,
                    info.bottom_tex_coord
                );
                batch.push_textured_path(bounds, &path, tint, BlendMode::Alpha);
            }
        }
    }
}

fn push_file_icon(
    batch: &mut QuadBatch,
    path: &str,
    tex_coords: Option<(f32, f32, f32, f32)>,
    bounds: Rectangle,
    tint: [f32; 4],
) {
    match tex_coords {
        Some((left, right, top, bottom)) => {
            let uvs = Rectangle::new(Point::new(left, top), Size::new(right - left, bottom - top));
            batch.push_textured_path_uv(bounds, uvs, path, tint, BlendMode::Alpha);
        }
        None => batch.push_textured_path(bounds, path, tint, BlendMode::Alpha),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request<'a>(
        runs: &'a [InlineTextRun<'a>],
        width: f32,
        word_wrap: bool,
    ) -> InlineTextRequest<'a> {
        InlineTextRequest {
            runs,
            bounds: Rectangle::new(Point::ORIGIN, Size::new(width, 100.0)),
            font_path: None,
            font_size: 24.0,
            icon_scale: 2.0,
            justify_h: TextJustify::Left,
            justify_v: TextJustify::Left,
            word_wrap,
            max_lines: 0,
            spacing: 0.0,
            smooth_scaling: false,
            glyph_tex_index: 0,
            shadow_color: None,
            shadow_offset: (0.0, 0.0),
            outline: crate::widget::TextOutline::None,
            alpha: 1.0,
        }
    }

    /// Five pixels per character.
    fn measure(text: &str) -> f32 {
        text.chars().count() as f32 * 5.0
    }

    fn summary(lines: &[InlineLine]) -> Vec<Vec<(String, f32, f32)>> {
        lines
            .iter()
            .map(|line| {
                line.items
                    .iter()
                    .map(|placed| {
                        let label = match &placed.item {
                            InlineItem::Text { text, .. } => text.clone(),
                            InlineItem::Icon(_) => "<icon>".to_string(),
                        };
                        (label, placed.x, placed.width)
                    })
                    .collect()
            })
            .collect()
    }

    // R88e.6 verifies width participation; the explicit-size outputs are INFERRED.
    #[test]
    fn icons_take_room_in_the_line() {
        let runs = [InlineTextRun {
            text: "|A:hud-backpack:14:14|a 3 |A:banker:0:0|a 2",
            color: [1.0; 4],
        }];
        let lines = layout_inline_runs(&request(&runs, 500.0, false), measure);
        // 14 UI units at two pixels per unit; size 0 is the 29 px line of
        // the 24 px font.
        assert_eq!(
            summary(&lines),
            vec![vec![
                ("<icon>".into(), 0.0, 28.0),
                (" 3 ".into(), 28.0, 15.0),
                ("<icon>".into(), 43.0, 29.0),
                (" 2".into(), 72.0, 10.0),
            ]]
        );
        assert_eq!(lines[0].width, 82.0);
        let InlineItem::Icon(_) = &lines[0].items[0].item else {
            panic!("icon first");
        };
        assert_eq!(lines[0].items[0].height, 28.0);
    }

    // R88e.6 verifies line-height growth; the exact 32-unit size is INFERRED.
    #[test]
    fn a_tall_icon_raises_its_line() {
        let runs = [InlineTextRun {
            text: "a|T133784:32|t|nb",
            color: [1.0; 4],
        }];
        let lines = layout_inline_runs(&request(&runs, 500.0, false), measure);
        assert_eq!(line_box_heights(&lines, 29.0), vec![64.0, 29.0]);
    }

    // R88e.6 verifies accepted icon width enters the wrap check; exact size is INFERRED.
    #[test]
    fn word_wrap_moves_an_icon_to_the_next_line() {
        let runs = [InlineTextRun {
            text: "abcd |T133784:10|t",
            color: [1.0; 4],
        }];
        let lines = layout_inline_runs(&request(&runs, 30.0, true), measure);
        assert_eq!(
            summary(&lines),
            vec![
                vec![("abcd ".into(), 0.0, 25.0)],
                vec![("<icon>".into(), 0.0, 20.0)]
            ]
        );
    }

    #[test]
    fn line_breaks_and_colored_runs_keep_their_order() {
        let runs = [
            InlineTextRun {
                text: "a|A:x:5:5|a",
                color: [1.0, 0.0, 0.0, 1.0],
            },
            InlineTextRun {
                text: "b\nc",
                color: [0.0, 1.0, 0.0, 1.0],
            },
        ];
        let lines = layout_inline_runs(&request(&runs, 500.0, false), measure);
        assert_eq!(
            summary(&lines),
            vec![
                vec![
                    ("a".into(), 0.0, 5.0),
                    ("<icon>".into(), 5.0, 10.0),
                    ("b".into(), 15.0, 5.0)
                ],
                vec![("c".into(), 0.0, 5.0)],
            ]
        );
    }

    #[test]
    fn atlas_icon_uses_its_own_texture_region() {
        let mut batch = QuadBatch::new();
        let icon = split_inline_markup("|A:hud-backpack:14:14|a").remove(0);
        let InlinePiece::Icon(icon) = icon else {
            panic!("icon expected");
        };
        push_inline_icon(
            &mut batch,
            &icon,
            Rectangle::new(Point::ORIGIN, Size::new(14.0, 14.0)),
            1.0,
        );
        let lookup =
            crate::atlas::get_render_atlas_info("hud-backpack").expect("atlas in the table");
        assert_eq!(batch.texture_requests.len(), 1);
        assert!(
            batch.texture_requests[0]
                .path
                .starts_with(&format!("{}@crop:", lookup.info.file)),
            "{}",
            batch.texture_requests[0].path
        );
    }

    #[test]
    fn unknown_atlas_draws_nothing() {
        let mut batch = QuadBatch::new();
        let InlinePiece::Icon(icon) =
            split_inline_markup("|A:no-such-atlas-anywhere:14:14|a").remove(0)
        else {
            panic!("icon expected");
        };
        push_inline_icon(
            &mut batch,
            &icon,
            Rectangle::new(Point::ORIGIN, Size::new(14.0, 14.0)),
            1.0,
        );
        assert!(batch.vertices.is_empty());
    }
}
