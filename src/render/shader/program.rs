//! Shader program for WoW UI rendering.
//!
//! This implements `shader::Program` which is used by the `Shader` widget.
//! The program's `draw()` method returns a `WowUiPrimitive` for GPU rendering.

use super::{QuadBatch, WowUiPrimitive};
use iced::mouse;
use iced::widget::shader::{self, Action};
use iced::{Event, Rectangle};
use std::sync::Arc;

/// Shader program for rendering WoW UI frames.
///
/// This struct holds the quad batch data and implements `shader::Program`
/// to integrate with iced's `Shader` widget.
pub struct WowUiProgram {
    /// The quad batch to render.
    quads: Arc<QuadBatch>,
}

impl WowUiProgram {
    /// Create a new program with the given quad batch.
    pub fn new(quads: Arc<QuadBatch>) -> Self {
        Self { quads }
    }
}

impl<Message> shader::Program<Message> for WowUiProgram {
    /// No internal state needed - quads are pre-computed.
    type State = ();

    /// The primitive type we render.
    type Primitive = WowUiPrimitive;

    /// Handle events (mouse, etc).
    fn update(
        &self,
        _state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        // Events are handled at the App level, not here
        None
    }

    /// Create the primitive for rendering.
    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        WowUiPrimitive::new_merged(Arc::clone(&self.quads))
    }

    /// Return default mouse interaction.
    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl std::fmt::Debug for WowUiProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WowUiProgram")
            .field("quad_count", &self.quads.quad_count())
            .finish()
    }
}
