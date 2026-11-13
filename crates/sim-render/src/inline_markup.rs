//! Inline texture (`|T...|t`) and atlas (`|A...|a`) escapes in text.
//!
//! The client draws these escapes as images inside a line of text and
//! reserves their width when it measures the string. This module splits a
//! string into text and icon pieces and sizes the icons; drawing them is up
//! to the renderer.

use crate::strip_wow_markup;

/// What an inline icon draws.
#[derive(Debug, Clone, PartialEq)]
pub enum InlineIconSource {
    /// A texture path, e.g. `Interface\Icons\INV_Misc_QuestionMark`.
    File(String),
    /// A texture file data ID, written as a number in place of the path.
    FileId(u32),
    /// An atlas name.
    Atlas(String),
}

/// One `|T...|t` or `|A...|a` escape.
///
/// `|Tpath:height:width:offsetX:offsetY:texWidth:texHeight:left:right:top:bottom:r:g:b|t`
/// and `|A:atlas:height:width:offsetX:offsetY:r:g:b|a` (the colon after `|A`
/// is optional). Every field after the path or atlas name may be left out.
#[derive(Debug, Clone, PartialEq)]
pub struct InlineIcon {
    pub source: InlineIconSource,
    /// Height in UI units as written; see [`InlineIcon::size`] for 0 and
    /// missing values.
    pub height: Option<f32>,
    /// Width in UI units as written; see [`InlineIcon::size`].
    pub width: Option<f32>,
    /// Shift to the right, in UI units.
    pub offset_x: f32,
    /// Shift upwards, in UI units.
    pub offset_y: f32,
    /// Texture coordinates `(left, right, top, bottom)` in the 0..1 range,
    /// from the texel rectangle of `|T` and the texture's size.
    pub tex_coords: Option<(f32, f32, f32, f32)>,
    /// Vertex color, each channel 0..1.
    pub vertex_color: Option<[f32; 3]>,
}

impl InlineIcon {
    /// Width and height in UI units next to text whose lines are
    /// `line_height` units high.
    ///
    /// A texture without a height takes no room; a height of 0 is the line
    /// height, and a missing or zero width is the height. An atlas without a
    /// width, or with a width of 0, is a square of the line height whatever
    /// its height; otherwise a missing or zero height is the line height.
    pub fn size(&self, line_height: f32) -> (f32, f32) {
        let positive = |value: Option<f32>| value.filter(|value| *value > 0.0);
        if let InlineIconSource::Atlas(_) = self.source {
            return match positive(self.width) {
                Some(width) => (width, positive(self.height).unwrap_or(line_height)),
                None => (line_height, line_height),
            };
        }
        let Some(height) = self.height else {
            return (0.0, 0.0);
        };
        let height = if height > 0.0 { height } else { line_height };
        (positive(self.width).unwrap_or(height), height)
    }
}

/// A piece of a string: text with every escape removed, or an icon.
#[derive(Debug, Clone, PartialEq)]
pub enum InlinePiece {
    Text(String),
    Icon(InlineIcon),
}

/// Whether `text` holds an inline texture or atlas escape.
pub fn has_inline_icons(text: &str) -> bool {
    text.contains("|T") || text.contains("|A")
}

/// Split `text` into text pieces (colors, hyperlinks and other escapes
/// removed, as by [`strip_wow_markup`]) and icons, in reading order. Empty
/// text pieces are left out. An escape without its closing `|t`/`|a` ends
/// the string, as the client draws nothing after it.
pub fn split_inline_markup(text: &str) -> Vec<InlinePiece> {
    let mut pieces = Vec::new();
    let mut rest = text;
    while let Some((start, close)) = next_icon_escape(rest) {
        push_text(&mut pieces, &rest[..start]);
        let body_start = start + 2;
        let Some(body_len) = rest[body_start..].find(close) else {
            return pieces;
        };
        let body = &rest[body_start..body_start + body_len];
        let icon = if close == "|t" {
            parse_texture_escape(body)
        } else {
            parse_atlas_escape(body)
        };
        if let Some(icon) = icon {
            pieces.push(InlinePiece::Icon(icon));
        }
        rest = &rest[body_start + body_len + 2..];
    }
    push_text(&mut pieces, rest);
    pieces
}

/// Width of the first line of `text` (up to the first line break), with
/// inline icons sized for lines of `line_height` units. `measure` gives the
/// width of a text piece that holds no escapes.
pub fn first_line_width(text: &str, line_height: f32, mut measure: impl FnMut(&str) -> f32) -> f32 {
    let mut width = 0.0;
    for piece in split_inline_markup(text) {
        match piece {
            InlinePiece::Text(text) => {
                let (line, broken) = match text.split_once('\n') {
                    Some((line, _)) => (line, true),
                    None => (text.as_str(), false),
                };
                if !line.is_empty() {
                    width += measure(line);
                }
                if broken {
                    break;
                }
            }
            InlinePiece::Icon(icon) => width += icon.size(line_height).0,
        }
    }
    width
}

/// Height of each line of `text` (split at line breaks, without word wrap):
/// `line_height`, or the tallest icon on the line when it is taller.
pub fn line_heights(text: &str, line_height: f32) -> Vec<f32> {
    let mut heights = vec![line_height];
    for piece in split_inline_markup(text) {
        match piece {
            InlinePiece::Text(text) => {
                for _ in 0..text.matches('\n').count() {
                    heights.push(line_height);
                }
            }
            InlinePiece::Icon(icon) => {
                let last = heights.last_mut().expect("one line at least");
                *last = last.max(icon.size(line_height).1);
            }
        }
    }
    heights
}

fn push_text(pieces: &mut Vec<InlinePiece>, raw: &str) {
    let text = strip_wow_markup(raw);
    if !text.is_empty() {
        pieces.push(InlinePiece::Text(text));
    }
}

/// Byte offset of the next `|T` or `|A` and the escape that closes it. A
/// doubled `||` is a literal bar, not the start of an escape.
fn next_icon_escape(text: &str) -> Option<(usize, &'static str)> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'|' {
            match bytes[i + 1] {
                b'T' => return Some((i, "|t")),
                b'A' => return Some((i, "|a")),
                b'|' => i += 1,
                _ => {}
            }
        }
        i += 1;
    }
    None
}

fn parse_texture_escape(body: &str) -> Option<InlineIcon> {
    let fields: Vec<&str> = body.split(':').collect();
    let path = fields[0].trim();
    if path.is_empty() {
        return None;
    }
    let source = match path.parse::<u32>() {
        Ok(id) => InlineIconSource::FileId(id),
        Err(_) => InlineIconSource::File(path.to_string()),
    };
    let number = |index: usize| {
        fields
            .get(index)
            .and_then(|field| field.trim().parse::<f32>().ok())
    };
    let tex_coords = match (number(5), number(6)) {
        (Some(tex_width), Some(tex_height)) if tex_width > 0.0 && tex_height > 0.0 => {
            match (number(7), number(8), number(9), number(10)) {
                (Some(left), Some(right), Some(top), Some(bottom)) => Some((
                    left / tex_width,
                    right / tex_width,
                    top / tex_height,
                    bottom / tex_height,
                )),
                _ => None,
            }
        }
        _ => None,
    };
    Some(InlineIcon {
        source,
        height: number(1),
        width: number(2),
        offset_x: number(3).unwrap_or(0.0),
        offset_y: number(4).unwrap_or(0.0),
        tex_coords,
        vertex_color: vertex_color(number(11), number(12), number(13)),
    })
}

fn parse_atlas_escape(body: &str) -> Option<InlineIcon> {
    let body = body.strip_prefix(':').unwrap_or(body);
    let fields: Vec<&str> = body.split(':').collect();
    let name = fields[0].trim();
    if name.is_empty() {
        return None;
    }
    let number = |index: usize| {
        fields
            .get(index)
            .and_then(|field| field.trim().parse::<f32>().ok())
    };
    Some(InlineIcon {
        source: InlineIconSource::Atlas(name.to_string()),
        height: number(1),
        width: number(2),
        offset_x: number(3).unwrap_or(0.0),
        offset_y: number(4).unwrap_or(0.0),
        tex_coords: None,
        vertex_color: vertex_color(number(5), number(6), number(7)),
    })
}

/// Escape colors are 0..255 per channel; all three must be given.
fn vertex_color(r: Option<f32>, g: Option<f32>, b: Option<f32>) -> Option<[f32; 3]> {
    let channel = |value: f32| (value / 255.0).clamp(0.0, 1.0);
    Some([channel(r?), channel(g?), channel(b?)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icon(source: InlineIconSource) -> InlineIcon {
        InlineIcon {
            source,
            height: None,
            width: None,
            offset_x: 0.0,
            offset_y: 0.0,
            tex_coords: None,
            vertex_color: None,
        }
    }

    #[test]
    fn splits_text_around_a_texture() {
        let pieces = split_inline_markup("Icon|TInterface\\Icons\\Spell:16|tEnd");
        assert_eq!(
            pieces,
            vec![
                InlinePiece::Text("Icon".into()),
                InlinePiece::Icon(InlineIcon {
                    height: Some(16.0),
                    ..icon(InlineIconSource::File("Interface\\Icons\\Spell".into()))
                }),
                InlinePiece::Text("End".into()),
            ]
        );
    }

    #[test]
    fn reads_every_texture_field() {
        let pieces = split_inline_markup("|T133784:12:20:1:-2:64:32:16:48:0:32:255:0:51|t");
        let [InlinePiece::Icon(parsed)] = pieces.as_slice() else {
            panic!("one icon expected: {pieces:?}");
        };
        assert_eq!(parsed.source, InlineIconSource::FileId(133784));
        assert_eq!((parsed.height, parsed.width), (Some(12.0), Some(20.0)));
        assert_eq!((parsed.offset_x, parsed.offset_y), (1.0, -2.0));
        assert_eq!(parsed.tex_coords, Some((0.25, 0.75, 0.0, 1.0)));
        assert_eq!(parsed.vertex_color, Some([1.0, 0.0, 0.2]));
    }

    #[test]
    fn reads_atlas_with_required_colon() {
        assert_eq!(
            split_inline_markup("|A:hud-backpack:14:16:2:3|a"),
            vec![InlinePiece::Icon(InlineIcon {
                height: Some(14.0),
                width: Some(16.0),
                offset_x: 2.0,
                offset_y: 3.0,
                ..icon(InlineIconSource::Atlas("hud-backpack".into()))
            })]
        );
    }

    #[test]
    fn atlas_without_colon_is_not_an_icon() {
        let pieces = split_inline_markup("|Ahud-backpack:14:16:2:3|a");
        assert!(
            !pieces
                .iter()
                .any(|piece| matches!(piece, InlinePiece::Icon(_)))
        );
    }

    #[test]
    fn wrong_closing_marker_does_not_form_an_icon() {
        for text in ["|Tspell:12|a later |t", "|A:atlas:12:12|t later |a"] {
            let pieces = split_inline_markup(text);
            assert!(
                pieces
                    .iter()
                    .all(|piece| !matches!(piece, InlinePiece::Icon(_))),
                "{text}: {pieces:?}"
            );
        }
    }

    #[test]
    fn strips_other_markup_from_text_pieces() {
        let pieces = split_inline_markup("|cFFFFD100Bags|r |A:hud-backpack:0:0|a |cnIQ3:3|r");
        assert_eq!(
            pieces,
            vec![
                InlinePiece::Text("Bags ".into()),
                InlinePiece::Icon(InlineIcon {
                    height: Some(0.0),
                    width: Some(0.0),
                    ..icon(InlineIconSource::Atlas("hud-backpack".into()))
                }),
                InlinePiece::Text(" 3".into()),
            ]
        );
    }

    #[test]
    fn unterminated_escape_returns_its_opening_pipe_to_text() {
        let pieces = split_inline_markup("Before|TInterface\\Icons\\Spell:16");
        assert!(
            pieces
                .iter()
                .all(|piece| !matches!(piece, InlinePiece::Icon(_)))
        );
        let text = pieces
            .into_iter()
            .filter_map(|piece| match piece {
                InlinePiece::Text(text) => Some(text),
                InlinePiece::Icon(_) => None,
            })
            .collect::<String>();
        assert_eq!(text, "Before|TInterface\\Icons\\Spell:16");
    }

    #[test]
    fn doubled_bar_is_not_an_escape() {
        assert!(matches!(
            split_inline_markup("a||Tb").as_slice(),
            [InlinePiece::Text(_)]
        ));
    }

    #[test]
    fn texture_size_zero_is_the_line_height_and_no_size_takes_no_room() {
        let texture = |height, width| InlineIcon {
            height,
            width,
            ..icon(InlineIconSource::File("x".into()))
        };
        assert_eq!(texture(None, None).size(15.0), (0.0, 0.0));
        assert_eq!(texture(Some(0.0), None).size(15.0), (15.0, 15.0));
        assert_eq!(texture(Some(0.0), Some(0.0)).size(15.0), (15.0, 15.0));
        assert_eq!(texture(Some(20.0), None).size(15.0), (20.0, 20.0));
        assert_eq!(texture(Some(1.0), Some(110.0)).size(15.0), (110.0, 1.0));
    }

    #[test]
    fn atlas_without_width_is_a_line_high_square() {
        let atlas = |height, width| InlineIcon {
            height,
            width,
            ..icon(InlineIconSource::Atlas("x".into()))
        };
        assert_eq!(atlas(Some(24.0), Some(0.0)).size(15.0), (15.0, 15.0));
        assert_eq!(atlas(Some(24.0), None).size(15.0), (15.0, 15.0));
        assert_eq!(atlas(Some(0.0), Some(0.0)).size(15.0), (15.0, 15.0));
        assert_eq!(atlas(Some(12.0), Some(20.0)).size(15.0), (20.0, 12.0));
        assert_eq!(atlas(Some(0.0), Some(20.0)).size(15.0), (20.0, 15.0));
    }

    #[test]
    fn tall_icons_raise_their_line() {
        assert_eq!(
            line_heights("a|A:x:32:32|a b|nc|A:x:8:8|a|nd", 15.0),
            vec![32.0, 15.0, 15.0]
        );
    }

    #[test]
    fn first_line_width_adds_icon_widths() {
        let width = first_line_width("|A:a:14:14|a 3|nsecond |A:b:30:30|a", 12.0, |text| {
            text.chars().count() as f32 * 5.0
        });
        assert_eq!(width, 14.0 + 2.0 * 5.0);
    }
}
