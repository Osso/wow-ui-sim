//! Supplied scenario header delves visualization.
use super::{UiWidgetCurrencyRecord, WidgetSpellInfoRecord};

#[derive(Debug, Clone, PartialEq)]
pub struct WidgetRewardRecord {
    pub shown_state: u32,
    pub earned_tooltip: String,
    pub unearned_tooltip: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioHeaderDelvesRecord {
    pub shown_state: u32,
    pub header_text: String,
    pub tooltip: String,
    pub tooltip_loc: u32,
    pub tier_text: String,
    pub tier_tooltip_spell_id: Option<f64>,
    pub currencies: Vec<UiWidgetCurrencyRecord>,
    pub spells: Vec<WidgetSpellInfoRecord>,
    pub reward_info: WidgetRewardRecord,
    pub widget_size_setting: f64,
    pub texture_kit: String,
    pub frame_texture_kit: String,
    pub has_timer: bool,
    pub order_index: f64,
    pub widget_tag: String,
    pub in_anim_type: u32,
    pub out_anim_type: u32,
    pub widget_scale: u32,
    pub layout_direction: u32,
    pub model_scene_layer: u32,
    pub scripted_animation_effect_id: f64,
}
