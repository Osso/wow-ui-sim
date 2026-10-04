//! Explicit host chat ingress. INFERRED queue/default/classification policy.

use crate::event::EventArg;
use std::collections::VecDeque;

#[derive(Default)]
pub struct HostChatInputs {
    pub pending: VecDeque<HostChatMessage>,
}

pub struct HostChatMessage {
    pub kind: HostChatKind,
    pub arguments: Vec<HostChatArgument>,
}

pub struct HostChatArgument {
    pub value: EventArg,
    /// Independent source classification; never cleared by a lockdown exemption.
    pub secret: bool,
}

#[derive(Clone, Copy)]
pub enum HostChatKind {
    CombatFactionChange,
    CombatHonorGain,
    CombatMiscInfo,
    CombatXpGain,
    Currency,
    Filtered,
    Loot,
    Money,
    Restricted,
    Say,
}

impl HostChatKind {
    pub fn event_name(self) -> &'static str {
        match self {
            Self::CombatFactionChange => "CHAT_MSG_COMBAT_FACTION_CHANGE",
            Self::CombatHonorGain => "CHAT_MSG_COMBAT_HONOR_GAIN",
            Self::CombatMiscInfo => "CHAT_MSG_COMBAT_MISC_INFO",
            Self::CombatXpGain => "CHAT_MSG_COMBAT_XP_GAIN",
            Self::Currency => "CHAT_MSG_CURRENCY",
            Self::Filtered => "CHAT_MSG_FILTERED",
            Self::Loot => "CHAT_MSG_LOOT",
            Self::Money => "CHAT_MSG_MONEY",
            Self::Restricted => "CHAT_MSG_RESTRICTED",
            Self::Say => "CHAT_MSG_SAY",
        }
    }

    pub fn secret_during_lockdown(self) -> bool {
        matches!(self, Self::Say)
    }
}
