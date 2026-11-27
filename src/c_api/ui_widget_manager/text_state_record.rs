//! Literal21-field Retail/PTR/Forever supplied TextWithState input; no query producer.

/// Complete explicit host record. No native catalog inference or automatic filler.
#[derive(Debug, Clone, PartialEq)]
pub struct TextWithStateRecord {
    pub shown_state: u32,
    pub enabled_state: u32,
    pub text: String,
    pub tooltip: String,
    pub text_size_type: u32,
    pub font_type: u32,
    pub bottom_padding: f64,
    pub tooltip_loc: u32,
    pub h_align: u32,
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
