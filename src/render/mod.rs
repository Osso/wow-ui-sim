//! Rendering module for WoW UI frames.
//!
//! Provides both canvas-based (CPU) and shader-based (GPU) rendering.

pub mod font {
    pub use crate::font::*;
}
#[cfg(feature = "gui")]
pub mod glyph;
#[cfg(feature = "gui")]
pub mod headless;
#[cfg(feature = "gui")]
pub mod shader;
#[cfg(feature = "gui")]
pub mod text;
#[cfg(feature = "gui")]
pub mod texture;

pub use crate::BlendMode;
pub use crate::font::WowFontSystem;

/// Strip WoW markup from text: textures (`|T...|t`), atlases (`|A...|a`),
/// colors (`|cXXXXXXXX`/`|r`), and hyperlinks (`|H...|h`/`|h`).
/// Preserves plain text content visible to the player.
pub fn strip_wow_markup(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '|' && consume_escape(&mut chars, &mut result) {
            continue;
        }
        result.push(c);
    }

    result
}

/// Try to consume a WoW escape sequence after `|`. Returns true if consumed.
fn consume_escape(chars: &mut std::iter::Peekable<std::str::Chars>, result: &mut String) -> bool {
    let Some(&next) = chars.peek() else {
        return false;
    };
    match next {
        'T' | 'A' => skip_delimited_span(chars, if next == 'T' { 't' } else { 'a' }),
        'H' => skip_delimited_span(chars, 'h'),
        'h' | 'r' => {
            chars.next();
            true
        }
        'n' => {
            chars.next();
            result.push('\n');
            true
        }
        'c' => {
            chars.next();
            // Two color forms: the classic `|cAARRGGBB` and the named
            // `|cn<colorName>:` the client uses for item quality. Consuming a
            // fixed eight characters from a named code eats past its colon and
            // into whatever escape follows it.
            if chars.peek() == Some(&'n') {
                chars.next();
                // A named code ends at its colon. Stop at a `|` too: a color
                // name cannot contain one, so a code missing its colon costs
                // the code rather than the rest of the string.
                while let Some(&ch) = chars.peek() {
                    if ch == '|' {
                        break;
                    }
                    chars.next();
                    if ch == ':' {
                        break;
                    }
                }
            } else {
                for _ in 0..8 {
                    chars.next();
                }
            }
            true
        }
        _ => false,
    }
}

/// Skip from current position to `|{end_marker}` (e.g. `|T...|t`).
fn skip_delimited_span(chars: &mut std::iter::Peekable<std::str::Chars>, end_marker: char) -> bool {
    chars.next(); // consume the opening letter
    while let Some(ch) = chars.next() {
        if ch == '|' && chars.peek() == Some(&end_marker) {
            chars.next();
            return true;
        }
    }
    true
}

#[cfg(feature = "gui")]
pub use glyph::{GlyphAtlas, emit_text_quads};
#[cfg(feature = "gui")]
pub(crate) use shader::ThreeSlicePathParams;
#[cfg(feature = "gui")]
pub use shader::{
    FrameQuadSnapshot, GpuBcTextureData, GpuTextureAtlas, GpuTextureData, NineSliceTextures,
    QuadBatch, QuadVertex, TextureEntry, TextureRequest, WowUiPipeline, WowUiPrimitive,
    WowUiProgram, load_texture_or_crop,
};
#[cfg(feature = "gui")]
pub use text::TextRenderer;
#[cfg(feature = "gui")]
pub use texture::{
    draw_horizontal_slice_texture, draw_nine_slice_texture, draw_scaled_texture,
    draw_texture_with_texcoords, draw_tiled_texture,
};

#[cfg(test)]
mod tests {
    use super::strip_wow_markup;

    #[test]
    fn strips_spell_link_before_wow_newline_escape() {
        assert_eq!(
            strip_wow_markup("|cFF2959D3|Hspell:1225135|h[Suppression Zones]|h|r|nNext"),
            "[Suppression Zones]\nNext"
        );
    }
}
