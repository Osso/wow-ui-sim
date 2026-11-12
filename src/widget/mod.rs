//! Widget system implementing WoW's UI object hierarchy.

use sim_widget::widget::anchor;
pub(crate) mod editbox;
mod editbox_history;
mod frame;
use sim_widget::widget::frame_enums;
mod frame_size;
use sim_widget::widget::frame_types;
mod registry;

pub use crate::atlas::NineSliceAtlasInfo;
pub use anchor::{Anchor, AnchorPoint};
pub use editbox::EditBoxPresentation;
pub use frame::{
    AlphaGradient, AttributeValue, Backdrop, Color, Frame, Gradient, LineAnchor,
    MinimapBlobLayerStyle, MinimapBlobRingStyle, TextJustify, TextOutline,
};
pub use frame_enums::{DrawLayer, FrameStrata};
pub use frame_types::TextSegment;
#[cfg(feature = "client-wrath")]
pub(crate) use registry::has_queryable_rect;
pub use registry::{AnchorCyclePath, RenderDirtyBatch, RenderDirtySource, WidgetRegistry};

pub use sim_widget::widget::{WidgetType, next_region_order, next_widget_id};
