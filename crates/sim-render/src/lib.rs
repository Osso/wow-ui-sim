//! Reusable markup and GPU/canvas primitives, below runtime composition.
pub use sim_texture::{self as texture_data, logging};
pub mod font;
pub mod inline_markup;
pub use crate as render;
#[cfg(feature = "gui")]
pub use shader::{QuadBatch, QuadVertex, TextureRequest};
pub use sim_widget::{BlendMode, widget};
#[cfg(feature = "gui")]
pub mod shader;
#[cfg(feature = "gui")]
pub mod texture;

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
