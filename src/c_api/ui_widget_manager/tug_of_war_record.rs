//! Supplied tug-of-war widget visualization.
#[derive(Debug, Clone, PartialEq)]
pub struct WidgetIconRecord {
    pub source_type: u32,
    pub source_id: f64,
    pub size_type: u32,
    pub tooltip: String,
    pub tooltip_loc: u32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct TugOfWarRecord {
    pub shown_state: u32,
    pub min_value: f64,
    pub max_value: f64,
    pub current_value: f64,
    pub neutral_zone_center: f64,
    pub neutral_zone_size: f64,
    pub left_icon_info: WidgetIconRecord,
    pub right_icon_info: WidgetIconRecord,
    pub glow_anim_type: u32,
    pub tooltip: String,
    pub tooltip_loc: u32,
    pub neutral_fill_style: u32,
    pub marker_arrow_shown_state: u32,
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
