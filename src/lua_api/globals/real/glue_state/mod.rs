//! R88ad.7-10 Glue host state (Retail 12.1.5.70688).
//!
//! Every default here is an empty host input unless the field documents
//! verified native image initialization. See `docs/specs/glue-state-r88ad.md`.

use crate::lua_api::WowLuaEnv;
use std::collections::BTreeSet;

/// Host inputs shared by the R88ad Glue globals.
#[derive(Debug, Default)]
pub struct GlueHostState {
    pub server: GlueServerState,
    /// R88ad.8 shared minimum-expansion state. The value Lua first observes is
    /// UNKNOWN, so the empty host input is zero (the zero-filled image storage).
    pub minimum_expansion_level: u32,
    /// R88ad.9 high-resolution file IDs for which a native background object's
    /// availability helper returns nonzero. Empty: no native background objects.
    pub login_background_file_ids: BTreeSet<i32>,
    pub audio: GlueAudioState,
}

/// R88ad.8 connection snapshot read by `GetServerName` and `IsConnectedToServer`.
#[derive(Clone, Debug, Default)]
pub struct GlueServerState {
    /// Native connection-availability flag; its writers are host transitions.
    pub available: bool,
    pub name: Option<String>,
    /// `Cfg_Configs` row hit with its first and second Booleans; `None` is a miss.
    pub config: Option<(bool, bool)>,
    /// Result of the native connection-value test, published only on a row hit.
    pub additional_connection_flag: bool,
    /// Stored realm number of a present connection object.
    pub realm_number: Option<i32>,
}

/// One stored R88ad.10 audio handle. Backend activity is a host input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlueAudioHandle {
    pub id: u32,
    pub kit: i32,
    pub active: bool,
}

/// Audio operations submitted by the Glue audio globals, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GlueAudioRequest {
    StartMusic { kit: i32, handle: u32 },
    StopMusic { handle: u32 },
    StartAmbience { kit: i32, fade_ms: i32, handle: u32 },
    StopAmbience { handle: u32 },
}

/// R88ad.10 in-process music and ambience state.
#[derive(Debug)]
pub struct GlueAudioState {
    /// The two native control flags; both set suppresses `PlayGlueMusic`.
    pub music_suppressed: [bool; 2],
    pub music: GlueAudioHandle,
    pub other_music: GlueAudioHandle,
    pub requested_music_kit: i32,
    /// Stop markers for `music` and `other_music`.
    pub music_stop_markers: [bool; 2],
    pub ambience: GlueAudioHandle,
    pub requested_ambience_kit: i32,
    pub ambience_stop_marker: bool,
    pub requests: Vec<GlueAudioRequest>,
    /// Last issued handle ID; zero is the empty handle.
    pub next_handle: u32,
}

impl Default for GlueAudioState {
    /// Image initialization: zero kits and handles, stop markers true.
    fn default() -> Self {
        Self {
            music_suppressed: [false; 2],
            music: GlueAudioHandle::default(),
            other_music: GlueAudioHandle::default(),
            requested_music_kit: 0,
            music_stop_markers: [true; 2],
            ambience: GlueAudioHandle::default(),
            requested_ambience_kit: 0,
            ambience_stop_marker: true,
            requests: Vec::new(),
            next_handle: 0,
        }
    }
}

/// R88ad.8 native update routine: store a differing value, then notify.
pub fn update_minimum_expansion_level(env: &WowLuaEnv, level: u32) -> crate::Result<()> {
    let previous = std::mem::replace(
        &mut env.state().borrow_mut().glue_state.minimum_expansion_level,
        level,
    );
    if previous != level {
        env.fire_event("MIN_EXPANSION_LEVEL_UPDATED")?;
    }
    Ok(())
}
