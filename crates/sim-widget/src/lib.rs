//! Profile-neutral widget types, identities and geometry.
pub mod atlas;
pub mod widget;

/// Blend mode for quad rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum BlendMode {
    /// Standard alpha blending: src * alpha + dst * (1 - alpha)
    #[default]
    Alpha = 0,
    /// Additive blending: src + dst (for highlight textures)
    Additive = 1,
    /// `MOD`: multiplies what is already drawn by the texture colour
    /// (dst * src); the texture's alpha does not take part.
    Modulate = 2,
    /// `DISABLE`: no blending; the texture colour replaces what is drawn,
    /// opaque, whatever its alpha.
    Opaque = 3,
    /// `ALPHAKEY`: alpha test; texels under the key are not drawn, the rest
    /// are drawn opaque.
    AlphaKey = 4,
}

impl BlendMode {
    /// The mode for a `SetBlendMode` / XML `alphaMode` token (upper case).
    /// Unknown tokens blend like `BLEND`.
    pub fn from_token(token: &str) -> Self {
        match token {
            "ADD" => Self::Additive,
            "MOD" => Self::Modulate,
            "DISABLE" => Self::Opaque,
            "ALPHAKEY" => Self::AlphaKey,
            _ => Self::Alpha,
        }
    }
}

/// Computed layout position for a frame.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[cfg(test)]
mod blend_mode_tests {
    use super::BlendMode;

    #[test]
    fn blend_mode_tokens_map_to_their_modes() {
        assert_eq!(BlendMode::from_token("BLEND"), BlendMode::Alpha);
        assert_eq!(BlendMode::from_token("ADD"), BlendMode::Additive);
        assert_eq!(BlendMode::from_token("MOD"), BlendMode::Modulate);
        assert_eq!(BlendMode::from_token("DISABLE"), BlendMode::Opaque);
        assert_eq!(BlendMode::from_token("ALPHAKEY"), BlendMode::AlphaKey);
        assert_eq!(BlendMode::from_token("UNKNOWN"), BlendMode::Alpha);
    }
}
