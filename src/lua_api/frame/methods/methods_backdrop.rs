//! Backdrop methods.
//!
//! In modern WoW (post-9.0), backdrop rendering is handled entirely by
//! BackdropTemplateMixin in Lua, which creates child Texture widgets for
//! nine-slice pieces. No Rust methods are needed — the mixin defines
//! SetBackdrop, ApplyBackdrop, SetBackdropColor, SetBackdropBorderColor, etc.
//!
//! Previously this file had Rust implementations of these methods, but they
//! shadowed the Lua mixin methods (Rust `add_method` takes priority over
//! `__index` lookups), preventing the mixin from creating the nine-slice
//! child textures.

use mlua::Lua;

/// No-op: all backdrop methods are handled by BackdropTemplateMixin in Lua.
pub fn add_backdrop_methods(_lua: &Lua, _methods: &mlua::Table) -> mlua::Result<()> {
    Ok(())
}
