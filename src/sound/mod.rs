//! Audio playback for WoW UI sounds via rodio.
//!
//! When the `sound` feature is disabled (e.g. during tests), all methods are
//! no-ops and `SoundManager::new()` always returns `None`.

#[cfg(feature = "sound")]
mod real;

#[cfg(feature = "sound")]
pub use real::SoundManager;

#[cfg(not(feature = "sound"))]
mod stub;

#[cfg(not(feature = "sound"))]
pub use stub::SoundManager;
