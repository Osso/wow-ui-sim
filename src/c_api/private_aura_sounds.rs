//! INFERRED host-declared sound registrations; native acquisition and playback unknown.

use std::collections::HashSet;

#[derive(Default)]
pub struct PrivateAuraSoundRegistrations {
    /// Explicit live IDs only. The ID domain and removal policies are simulator inferences.
    pub live_ids: HashSet<u32>,
}
