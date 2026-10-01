//! Explicit Recent Allies inputs only; query registration/publication is pending.

/// Disabled and empty by default: inferred simulator policy, not native evidence.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecentAlliesInput {
    pub enabled: bool,
    pub entries: Vec<RecentAllyData>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyData {
    pub state_data: RecentAllyStateData,
    pub character_data: RecentAllyCharacterData,
    pub interaction_data: RecentAllyInteractionData,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyStateData {
    pub is_online: bool,
    pub is_dnd: bool,
    pub is_afk: bool,
    pub is_converted_legacy_friend: bool,
    pub pin_expiration_date: Option<i64>,
    pub friend_request_sent_this_session: bool,
    pub current_location: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyCharacterData {
    pub guid: String,
    pub name: String,
    pub full_name: String,
    pub realm_name: String,
    pub level: i32,
    pub class_id: i32,
    pub race_id: i32,
    pub sex: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyInteractionData {
    pub interactions: Vec<RecentAllyInteraction>,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyInteraction {
    /// Numeric value of the documented RolodexType enum; no inferred variants.
    pub interaction_type: i32,
    pub description: String,
    pub timestamp: i64,
    pub context_data: RecentAllyInteractionContextData,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyInteractionContextData {
    pub item_id: Option<i64>,
    pub location_name: Option<String>,
    pub activity_difficulty_id: Option<i32>,
    pub activity_difficulty_level: Option<i32>,
}
