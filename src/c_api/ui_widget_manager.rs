//! Supplied widget sets, visualizations, and synchronous host update feeds.
use std::collections::HashMap;

mod bar_records;
mod bullet;
mod bullet_record;
pub use bullet_record::BulletTextListRecord;
mod discrete;
mod discrete_record;
pub use discrete_record::DiscreteProgressStepsRecord;
mod double_icon_and_text;
mod double_icon_and_text_record;
pub use double_icon_and_text_record::DoubleIconAndTextRecord;
mod double_status_bar;
mod double_status_bar_record;
mod texture_and_text;
mod texture_and_text_record;
mod texture_and_text_row;
mod texture_and_text_row_record;
pub use double_status_bar_record::DoubleStatusBarRecord;
pub use texture_and_text_record::TextureAndTextRecord;
pub use texture_and_text_row_record::{TextureAndTextEntryRecord, TextureAndTextRowRecord};
mod bars;
mod horizontal_currencies;
mod horizontal_currencies_record;
mod hud;
pub use bar_records::{CaptureBarRecord, StatusBarRecord};
pub use horizontal_currencies_record::HorizontalCurrenciesRecord;
mod feed;
mod fixtures;
mod icon_text_and_background;
mod icon_text_and_background_record;
mod icon_text_and_currencies;
mod icon_text_and_currencies_record;
mod inputs;
mod item_display;
mod item_display_record;
mod unit_context;
pub use icon_text_and_background_record::IconTextAndBackgroundRecord;
pub use icon_text_and_currencies_record::{IconTextAndCurrenciesRecord, UiWidgetCurrencyRecord};
pub use item_display_record::{ItemDisplayRecord, UiWidgetItemRecord};
mod spacer;
mod spacer_record;
mod spell_display;
mod spell_display_record;
pub use spacer_record::SpacerRecord;
mod text_column_row;
mod text_column_row_record;
mod text_state;
mod text_state_record;
mod text_with_subtext;
mod text_with_subtext_record;
pub use feed::WidgetVisualization;
pub use spell_display_record::{SpellDisplayRecord, WidgetSpellInfoRecord};
pub use text_column_row_record::{TextColumnRowEntryInfo, TextColumnRowVisualizationInfo};
pub use text_state_record::TextWithStateRecord;
pub use text_with_subtext_record::TextWithSubtextRecord;
pub(crate) mod queries;
pub(crate) use fixtures::seed_demo_world_quest_widgets;
pub use inputs::{IconAndTextRecord, UiWidgetInputs, WidgetSetEntry, WidgetSetRecord};
pub use unit_context::{ProcessingUnit, WidgetUnitContext};

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
pub(crate) mod button_header;

/// Modeled numeric key, not a native widget-ID range or availability claim.
/// Preserve f64 bits except that equal numeric signed zeroes share one key.
/// No integer cast, positivity rule, or invented native rejection policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    pub fn from_number(number: f64) -> Self {
        let bits = if number == 0.0 {
            0.0_f64.to_bits()
        } else {
            number.to_bits()
        };
        Self(bits)
    }
}

/// Whole record must be supplied. No Default DTO or automatic field filler.
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonHeaderRecord {
    pub shown_state: u32,
    pub header_text: String,
    pub tooltip: String,
    pub tooltip_loc: u32,
    pub buttons: Vec<WidgetSpellButtonRecord>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct WidgetSpellButtonRecord {
    pub spell_id: f64,
    pub tooltip: String,
    pub icon: u32,
    pub enabled_state: u32,
    pub cooldown_type: u32,
}

/// Default is an empty supplied-record map, never a default record.
#[derive(Debug, Default)]
pub struct ButtonHeaderInputs {
    pub widgets: HashMap<WidgetId, ButtonHeaderRecord>,
}
