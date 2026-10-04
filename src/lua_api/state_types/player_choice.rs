//! Local state backing the `C_PlayerChoice` compatibility surface.

#[derive(Debug, Clone, Default)]
pub struct PlayerChoiceState {
    pub current: Option<PlayerChoiceInfo>,
    pub num_rerolls: i32,
    pub remaining_time: Option<f64>,
    pub waiting_for_response: bool,
    pub last_response_id: Option<i32>,
    pub reroll_requested: bool,
    pub ui_closed: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerChoiceInfo {
    pub object_guid: String,
    pub choice_id: i32,
    pub question_text: String,
    pub pending_choice_text: String,
    pub ui_texture_kit: String,
    pub hide_warboard_header: bool,
    pub keep_open_after_choice: bool,
    pub show_choices_as_list: bool,
    pub requires_selection: bool,
    pub hide_answer_art: bool,
    pub show_choices_as_grid: bool,
    pub options: Vec<PlayerChoiceOptionInfo>,
    pub sound_kit_id: Option<i32>,
    pub close_ui_sound_kit_id: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerChoiceOptionInfo {
    pub id: i32,
    pub description: String,
    pub header: String,
    pub choice_art_id: i32,
    pub desaturated_art: bool,
    pub disabled_option: bool,
    pub has_rewards: bool,
    pub reward_info: PlayerChoiceOptionRewardInfo,
    pub ui_texture_kit: String,
    pub max_stacks: i32,
    pub buttons: Vec<PlayerChoiceOptionButtonInfo>,
    pub widget_set_id: Option<i32>,
    pub spell_id: Option<i32>,
    pub rarity: Option<i32>,
    pub type_art_id: Option<i32>,
    pub header_icon_atlas_element: Option<String>,
    pub sub_header: Option<String>,
    pub consolidate_widgets: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerChoiceOptionButtonInfo {
    pub id: i32,
    pub text: String,
    pub disabled: bool,
    pub show_checkmark: bool,
    pub hide_button_show_text: bool,
    pub selected: bool,
    pub confirmation: Option<String>,
    pub tooltip: Option<String>,
    pub reward_quest_id: Option<i32>,
    pub sound_kit_id: Option<i32>,
    pub list_text: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerChoiceOptionRewardInfo {
    pub currency_rewards: Vec<PlayerChoiceRewardCurrencyInfo>,
    pub item_rewards: Vec<PlayerChoiceRewardItemInfo>,
    pub reputation_rewards: Vec<PlayerChoiceRewardReputationInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerChoiceRewardCurrencyInfo {
    pub currency_id: i32,
    pub name: String,
    pub currency_texture: i32,
    pub quantity: i32,
    pub is_currency_container: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerChoiceRewardItemInfo {
    pub item_id: i32,
    pub name: String,
    pub quantity: i32,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerChoiceRewardReputationInfo {
    pub faction_id: i32,
    pub quantity: i32,
}
