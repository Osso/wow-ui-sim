//! Environment-owned widget sets and visualization inputs. No native feed inference.
use std::collections::HashMap;

use super::{
    BulletTextListRecord, CaptureBarRecord, DiscreteProgressStepsRecord, DoubleIconAndTextRecord,
    DoubleStatusBarRecord, HorizontalCurrenciesRecord, IconTextAndBackgroundRecord,
    IconTextAndCurrenciesRecord, ItemDisplayRecord, SpacerRecord, SpellDisplayRecord,
    StatusBarRecord, TextColumnRowVisualizationInfo, TextWithStateRecord, TextWithSubtextRecord,
    TextureAndTextRecord, TextureAndTextRowRecord, WidgetId, WidgetVisualization,
};

#[derive(Debug, Clone, PartialEq)]
pub struct WidgetSetEntry {
    pub widget_id: f64,
    pub widget_type: u32,
    pub unit_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WidgetSetRecord {
    pub layout_direction: u32,
    pub vertical_padding: f64,
    pub widgets: Vec<WidgetSetEntry>,
}

impl UiWidgetInputs {
    pub(crate) fn unit_visualization(&self, id: WidgetId) -> Option<Option<&WidgetVisualization>> {
        let records = &self.unit_context.unit_visualizations;
        if self.unit_context.unit_widget_ids.contains(&id) {
            Some(
                self.unit_context
                    .processing_unit
                    .as_ref()
                    .and_then(|unit| records.get(&(unit.clone(), id))),
            )
        } else {
            None
        }
    }

    pub(super) fn resolve_visualization<T: Clone>(
        &self,
        id: WidgetId,
        global: &HashMap<WidgetId, T>,
        extract: impl Fn(&WidgetVisualization) -> Option<&T>,
    ) -> Option<T> {
        match self.unit_visualization(id) {
            Some(record) => record.and_then(extract).cloned(),
            None => global.get(&id).cloned(),
        }
    }
}

/// Complete supplied record; field values are never inferred from widget IDs.
#[derive(Debug, Clone, PartialEq)]
pub struct IconAndTextRecord {
    pub state: u32,
    pub text: String,
    pub tooltip: String,
    pub dynamic_tooltip: String,
    pub tooltip_loc: u32,
    pub shift_text_type: u32,
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

#[derive(Debug, Default)]
pub struct UiWidgetInputs {
    pub unit_context: super::WidgetUnitContext,
    pub sets: HashMap<WidgetId, WidgetSetRecord>,
    pub icon_and_text: HashMap<WidgetId, IconAndTextRecord>,
    pub icon_text_and_background: HashMap<WidgetId, IconTextAndBackgroundRecord>,
    pub icon_text_and_currencies: HashMap<WidgetId, IconTextAndCurrenciesRecord>,
    pub horizontal_currencies: HashMap<WidgetId, HorizontalCurrenciesRecord>,
    pub item_display: HashMap<WidgetId, ItemDisplayRecord>,
    pub quest_sets: HashMap<(i32, i32), f64>,
    pub status_bars: HashMap<WidgetId, StatusBarRecord>,
    pub capture_bars: HashMap<WidgetId, CaptureBarRecord>,
    pub double_status_bars: HashMap<WidgetId, DoubleStatusBarRecord>,
    pub texture_and_text: HashMap<WidgetId, TextureAndTextRecord>,
    pub texture_and_text_row: HashMap<WidgetId, TextureAndTextRowRecord>,
    pub discrete_progress_steps: HashMap<WidgetId, DiscreteProgressStepsRecord>,
    pub spell_display: HashMap<WidgetId, SpellDisplayRecord>,
    pub spacers: HashMap<WidgetId, SpacerRecord>,
    pub text_with_state: HashMap<WidgetId, TextWithStateRecord>,
    pub text_with_subtext: HashMap<WidgetId, TextWithSubtextRecord>,
    pub double_icon_and_text: HashMap<WidgetId, DoubleIconAndTextRecord>,
    pub bullet_text_lists: HashMap<WidgetId, BulletTextListRecord>,
    pub text_column_rows: HashMap<WidgetId, TextColumnRowVisualizationInfo>,
    /// Zero means no selected HUD set in this simulator fixture, not a native default.
    pub top_center_widget_set_id: f64,
    pub below_minimap_widget_set_id: f64,
    pub power_bar_widget_set_id: f64,
    pub objective_tracker_widget_set_id: f64,
}
