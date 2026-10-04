//! INFERRED registration inputs and observed aura state for sound playback.

use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct AuraSoundRegistration {
    pub trigger: u32,
    pub unit_token: String,
    pub spell_id: u32,
    pub sound_file_name: Option<String>,
    pub sound_file_id: Option<u32>,
    pub output_channel: Option<String>,
}

pub struct PrivateAuraSoundRegistrations {
    /// Existing removal contract also permits host-seeded IDs without payloads.
    pub live_ids: HashSet<u32>,
    pub registrations: HashMap<u32, AuraSoundRegistration>,
    /// INFERRED allocator: monotonic nonzero IDs; None means exhausted.
    pub next_id: Option<u32>,
    /// Last aura snapshot per registered unit, diffed each tick.
    pub observed: HashMap<String, super::playback::UnitAuraSnapshot>,
    /// Every aura sound played, in order.
    pub playbacks: Vec<super::playback::AuraSoundPlayback>,
}

impl Default for PrivateAuraSoundRegistrations {
    fn default() -> Self {
        Self {
            live_ids: HashSet::new(),
            registrations: HashMap::new(),
            next_id: Some(1),
            observed: HashMap::new(),
            playbacks: Vec::new(),
        }
    }
}
