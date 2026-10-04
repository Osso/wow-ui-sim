//! Explicit host chat ingress. INFERRED queue/default/classification policy.

use crate::event::EventArg;
use std::collections::VecDeque;

#[derive(Default)]
pub struct HostChatInputs {
    pub pending: VecDeque<HostChatMessage>,
}

pub struct HostChatMessage {
    pub kind: HostChatKind,
    /// Leading `ChatMessageEventParams` payloads (text .. suppressRaidIcons).
    pub arguments: Vec<HostChatArgument>,
    /// Trailing `discordInfo` payload; the default describes a non-Discord line.
    pub discord_info: DiscordChatInfo,
}

/// `DiscordChatInfo` (DiscordConstantsDocumentation.lua). `user_id == 0`
/// marks a message that did not originate from Discord.
#[derive(Clone, Debug, Default)]
pub struct DiscordChatInfo {
    pub user_id: f64,
    pub global_name: String,
    /// `Enum.DiscordDisplayNameType`.
    pub display_name_type: i32,
    pub last_online_guid: String,
    pub last_online_name: String,
    pub has_attachment: bool,
    pub has_poll: bool,
    pub has_embed: bool,
    pub has_sticker: bool,
    pub has_emoji: bool,
    pub has_error: bool,
    pub has_forwarded_message: bool,
    pub forwarded_message: String,
    pub from_discord: bool,
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
