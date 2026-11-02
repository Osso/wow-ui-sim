//! Supplied scenario header timer visualization.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioHeaderTimerRecord {
    pub shown_state: u32,
    pub timer_min: f64,
    pub timer_max: f64,
    pub timer_value: f64,
    pub header_text: String,
    pub timer_tooltip: String,
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
