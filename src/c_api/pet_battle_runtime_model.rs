//! Explicit host inputs for the Retail/Forever pet-battle UI projection.
//! INFERRED model boundary; empty data is not a generated battle or server reply.
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PetBattleRuntime {
    pub is_wild: bool,
    pub qualities: HashMap<(i32, i32), u8>,
    pub abilities: HashMap<i32, PetBattleAbility>,
    pub ability_states: HashMap<(i32, i32, i32), PetBattleAbilityState>,
    pub auras: HashMap<(i32, i32), Vec<PetBattleAura>>,
    pub parser_states: BTreeMap<String, i32>,
    pub matchmaking: PetBattleMatchmaking,
    pub selected_action: Option<(i32, Option<i32>)>,
    pub requests: Vec<PetBattleRequest>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PetBattleAbility {
    pub name: String,
    pub icon: i32,
    pub max_cooldown: i32,
    pub description: String,
    pub num_turns: i32,
    pub pet_type: i32,
    pub no_strong_weak_hints: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PetBattleAbilityState {
    pub usable: bool,
    pub cooldown: i32,
    pub lockdown: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PetBattleAura {
    pub ability_id: i32,
    pub instance_id: i32,
    pub turns_remaining: i32,
    pub is_buff: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PetBattleMatchmaking {
    /// Native state encoding from R88f.D.63, not Enum.PetBattleQueueStatus.
    pub status: u8,
    /// INFERRED nominal empty input: zero. Native default integer is UNKNOWN.
    pub estimated_time: i32,
    /// D.63 publishes the supplied integer multiplied by 0.001.
    pub queued_time_ms: i32,
    pub can_accept: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PetBattleRequest {
    StartMatchmaking,
    AcceptMatch,
    StartDuel { unit: String, exact_match: bool },
    AcceptDuel,
    ReportPet(i32),
    ReportUnit(String),
}

impl PetBattleRuntime {
    /// Drain requests for the embedding host; recording is not a server acknowledgement.
    pub fn take_requests(&mut self) -> Vec<PetBattleRequest> {
        std::mem::take(&mut self.requests)
    }
}
