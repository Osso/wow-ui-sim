//! UserData implementation for FrameHandle with all frame methods.
//!
//! This module imports method implementations from split submodules:
//! - methods_meta: Metamethods (__index, __newindex, __len, __eq)
//! - methods_core: Core frame methods (GetName, SetSize, Show/Hide, etc.)
//! - methods_anchor: Anchor/point methods (SetPoint, ClearAllPoints, etc.)
//! - methods_event: Event registration (RegisterEvent, UnregisterEvent, etc.)
//! - methods_script: Script handlers (SetScript, GetScript, HookScript, etc.)
//! - methods_attribute: Attribute methods (GetAttribute, SetAttribute, etc.)
//! - methods_backdrop: Backdrop methods (SetBackdrop, SetBackdropColor, etc.)
//! - methods_create: Child creation (CreateTexture, CreateFontString, etc.)
//! - methods_texture: Texture methods (SetTexture, SetAtlas, SetTexCoord, etc.)
//! - methods_text: Text/FontString methods (SetText, SetFont, SetJustifyH, etc.)
//! - methods_button: Button-specific methods (SetNormalTexture, etc.)
//! - methods_widget: Widget-specific methods (EditBox, Slider, StatusBar, etc.)
//! - methods_helpers: Shared helper functions

use super::FrameHandle;
use mlua::{UserData, UserDataMethods};

mod methods_anchor;
mod methods_attribute;
mod methods_backdrop;
mod methods_button;
mod methods_core;
mod methods_create;
mod methods_event;
mod methods_helpers;
mod methods_hierarchy;
mod methods_meta;
mod methods_misc;
mod methods_script;
mod methods_text;
mod methods_texture;
mod methods_widget;

// Re-export helpers for use by other modules if needed
#[allow(unused_imports)]
pub use methods_helpers::{calculate_frame_height, calculate_frame_width};
pub(crate) use methods_core::fire_on_show_recursive;

impl UserData for FrameHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Add metamethods (__index, __newindex, __len, __eq)
        methods_meta::add_metamethods(methods);

        // Add core frame methods (GetName, SetSize, Show/Hide, strata/level, etc.)
        methods_core::add_core_methods(methods);

        // Add hierarchy methods (GetParent, SetParent, GetChildren, GetRegions, etc.)
        methods_hierarchy::add_hierarchy_methods(methods);

        // Add miscellaneous stubs (Minimap, ScrollingMessage, Alerts, etc.)
        methods_misc::add_misc_methods(methods);

        // Add anchor/point methods (SetPoint, ClearAllPoints, SetAllPoints, etc.)
        methods_anchor::add_anchor_methods(methods);

        // Add event registration methods (RegisterEvent, UnregisterEvent, etc.)
        methods_event::add_event_methods(methods);

        // Add script handler methods (SetScript, GetScript, HookScript, etc.)
        methods_script::add_script_methods(methods);

        // Add attribute methods (GetAttribute, SetAttribute, etc.)
        methods_attribute::add_attribute_methods(methods);

        // Add backdrop methods (SetBackdrop, SetBackdropColor, etc.)
        methods_backdrop::add_backdrop_methods(methods);

        // Add child creation methods (CreateTexture, CreateFontString, etc.)
        methods_create::add_create_methods(methods);

        // Add texture-related methods (SetTexture, SetAtlas, SetTexCoord, etc.)
        methods_texture::add_texture_methods(methods);

        // Add text/FontString methods (SetText, SetFont, SetJustifyH, etc.)
        methods_text::add_text_methods(methods);

        // Add button-specific methods (SetNormalTexture, GetFontString, etc.)
        methods_button::add_button_methods(methods);

        // Add widget-specific methods (EditBox, Slider, StatusBar, Cooldown, etc.)
        methods_widget::add_widget_methods(methods);
    }
}
