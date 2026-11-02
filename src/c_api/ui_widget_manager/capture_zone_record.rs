//! Supplied capture zone visualization.
use super::ZoneEntryRecord;

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureZoneRecord {
    pub shown_state: u32,
    pub mode: u32,
    pub leading_edge_type: u32,
    pub danger_flash_type: u32,
    pub zone_info: ZoneEntryRecord,
    pub tooltip_loc: u32,
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
