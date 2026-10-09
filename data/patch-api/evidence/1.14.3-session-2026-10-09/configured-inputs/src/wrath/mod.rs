//! Wrath (3.3.5a) client-profile compatibility layer.
//!
//! This module is compiled for non-retail profiles that need pre-mainline
//! compatibility helpers. Retail builds never see it; Mists, Era, and
//! Anniversary reuse the shared frame-method stubs and permissive event
//! validation while keeping Wrath-only Lua bootstraps gated separately.
//!
//! Contents:
//! - `frame_methods` — Rust-side frame method stubs (`SetBackdropColor`,
//!   `SetBackdropBorderColor`, `IgnoreDepth`, `SetPlayerTextureHeight`) that
//!   legacy addons call directly on Frame; retail moved these to
//!   `BackdropTemplateMixin`.
//! - `compat_bootstrap` — loads the bundled Lua snippet that supplies global
//!   stubs and Lua-5.0-era string/math aliases Wrath relies on.

pub mod compat_bootstrap;
pub mod frame_methods;
pub mod post_load;

/// Wrath/mists/era/anniversary profiles accept any non-empty event name;
/// the mainline `events.yaml` strict-list doesn't cover pre-Cataclysm,
/// MoP-Classic, or Vanilla.
pub fn is_registerable_event(name: &str) -> bool {
    !name.is_empty()
}
