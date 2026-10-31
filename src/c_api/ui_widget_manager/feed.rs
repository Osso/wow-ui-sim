//! Host feed operations mutate supplied state before synchronous consumer events.
use super::{
    BulletTextListRecord, ButtonHeaderRecord, CaptureBarRecord, DiscreteProgressStepsRecord,
    DoubleIconAndTextRecord, DoubleStatusBarRecord, HorizontalCurrenciesRecord, IconAndTextRecord,
    IconTextAndBackgroundRecord, IconTextAndCurrenciesRecord, ItemDisplayRecord, ProcessingUnit,
    SpacerRecord, SpellDisplayRecord, StatusBarRecord, TextColumnRowVisualizationInfo,
    TextWithStateRecord, TextWithSubtextRecord, TextureAndTextRecord, TextureAndTextRowRecord,
    WidgetId, WidgetSetEntry, WidgetSetRecord,
};
use crate::lua_api::WowLuaEnv;
use crate::{Error, Result};
use rilua::LuaApiMut;

#[derive(Debug, Clone)]
pub enum WidgetVisualization {
    IconAndText(IconAndTextRecord),
    IconTextAndBackground(IconTextAndBackgroundRecord),
    IconTextAndCurrencies(IconTextAndCurrenciesRecord),
    HorizontalCurrencies(HorizontalCurrenciesRecord),
    ItemDisplay(ItemDisplayRecord),
    CaptureBar(CaptureBarRecord),
    StatusBar(StatusBarRecord),
    DoubleStatusBar(DoubleStatusBarRecord),
    TextureAndText(TextureAndTextRecord),
    TextureAndTextRow(TextureAndTextRowRecord),
    TextWithState(TextWithStateRecord),
    TextWithSubtext(TextWithSubtextRecord),
    DoubleIconAndText(DoubleIconAndTextRecord),
    BulletTextList(BulletTextListRecord),
    TextColumnRow(TextColumnRowVisualizationInfo),
    SpellDisplay(SpellDisplayRecord),
    Spacer(SpacerRecord),
    DiscreteProgressSteps(DiscreteProgressStepsRecord),
    ButtonHeader(ButtonHeaderRecord),
    #[cfg(feature = "retail-12-0-0")]
    PreyHuntProgress(crate::c_api::c_prey::PreyHuntProgressWidget),
}

impl WidgetVisualization {
    fn widget_type(&self) -> u32 {
        match self {
            Self::IconAndText(_) => 0,
            Self::IconTextAndBackground(_) => 4,
            Self::IconTextAndCurrencies(_) => 7,
            Self::HorizontalCurrencies(_) => 9,
            Self::ItemDisplay(_) => 27,
            Self::CaptureBar(_) => 1,
            Self::StatusBar(_) => 2,
            Self::DoubleStatusBar(_) => 3,
            Self::TextureAndText(_) => 12,
            Self::TextureAndTextRow(_) => 15,
            Self::TextWithState(_) => 8,
            Self::TextWithSubtext(_) => 25,
            Self::DoubleIconAndText(_) => 5,
            Self::BulletTextList(_) => 10,
            Self::TextColumnRow(_) => 21,
            Self::SpellDisplay(_) => 13,
            Self::Spacer(_) => 22,
            Self::DiscreteProgressSteps(_) => 19,
            Self::ButtonHeader(_) => 30,
            #[cfg(feature = "retail-12-0-0")]
            Self::PreyHuntProgress(_) => 31,
        }
    }
}

impl WowLuaEnv {
    /// Replace/remove set membership, then reconcile registered containers.
    /// Visualization records survive membership removal for independent lookup.
    pub fn replace_ui_widget_set(
        &self,
        set_id: f64,
        record: Option<WidgetSetRecord>,
    ) -> Result<()> {
        {
            let mut sim = self.state.borrow_mut();
            let key = WidgetId::from_number(set_id);
            if let Some(record) = record {
                sim.ui_widget_inputs.sets.insert(key, record);
            } else {
                sim.ui_widget_inputs.sets.remove(&key);
            }
        }
        self.fire_event("UPDATE_ALL_UI_WIDGETS")
    }

    /// Supply a visualization and activate/update its membership before notification.
    pub fn update_ui_widget(
        &self,
        set_id: f64,
        entry: WidgetSetEntry,
        record: WidgetVisualization,
    ) -> Result<()> {
        self.update_ui_widget_for_unit(set_id, entry, record, false)
    }

    /// Supply a GUID-attached visualization and notify registered GUID consumers.
    pub fn update_ui_widget_for_guid(
        &self,
        set_id: f64,
        entry: WidgetSetEntry,
        record: WidgetVisualization,
    ) -> Result<()> {
        if entry.unit_token.is_none() {
            return Err(Error::Other("GUID widget update requires a GUID".into()));
        }
        self.update_ui_widget_for_unit(set_id, entry, record, true)
    }

    fn update_ui_widget_for_unit(
        &self,
        set_id: f64,
        entry: WidgetSetEntry,
        record: WidgetVisualization,
        is_guid: bool,
    ) -> Result<()> {
        validate_visualization(&entry, &record)?;
        let deliver;
        {
            let mut sim = self.state.borrow_mut();
            let set = sim
                .ui_widget_inputs
                .sets
                .get_mut(&WidgetId::from_number(set_id))
                .ok_or_else(|| Error::Other("UI widget update requires a supplied set".into()))?;
            upsert_membership(&mut set.widgets, &entry);
            if let Some(unit) = &entry.unit_token {
                let context = &mut sim.ui_widget_inputs.unit_context;
                let key = if is_guid {
                    ProcessingUnit::Guid(unit.clone())
                } else {
                    ProcessingUnit::Token(unit.clone())
                };
                context
                    .unit_visualizations
                    .insert((key, WidgetId::from_number(entry.widget_id)), record);
                deliver = context.registrations.contains(&(unit.clone(), is_guid));
            } else {
                supply_visualization(&mut sim, entry.widget_id, record);
                deliver = true;
            }
        }
        if deliver {
            self.fire_ui_widget_update(set_id, &entry)?;
        }
        Ok(())
    }

    pub fn remove_ui_widget_for_unit(
        &self,
        set_id: f64,
        widget_id: f64,
        _unit: ProcessingUnit,
    ) -> Result<bool> {
        self.remove_ui_widget(set_id, widget_id)
    }

    /// Remove membership without deleting a record shared by other containers.
    pub fn remove_ui_widget(&self, set_id: f64, widget_id: f64) -> Result<bool> {
        let removed = {
            let mut sim = self.state.borrow_mut();
            let Some(set) = sim
                .ui_widget_inputs
                .sets
                .get_mut(&WidgetId::from_number(set_id))
            else {
                return Ok(false);
            };
            let previous_count = set.widgets.len();
            let key = WidgetId::from_number(widget_id);
            set.widgets
                .retain(|entry| WidgetId::from_number(entry.widget_id) != key);
            set.widgets.len() != previous_count
        };
        if removed {
            self.fire_event("UPDATE_ALL_UI_WIDGETS")?;
        }
        Ok(removed)
    }

    fn fire_ui_widget_update(&self, set_id: f64, entry: &WidgetSetEntry) -> Result<()> {
        let (payload, saved_top) = {
            let mut lua = self.lua.borrow_mut();
            let state = lua.state_mut();
            let saved_top = state.top;
            let payload = super::queries::build_widget_info(state, set_id, entry);
            state.push(payload);
            // Cached containers filter `unit`, while API documentation names
            // `unitToken`. Model this event adapter until that consumer changes.
            if let Some(unit) = &entry.unit_token {
                let value = crate::lua_api::methods::create_string(state, unit);
                crate::lua_api::methods::table_set(state, payload, "unit", value);
            }
            (payload, saved_top)
        };
        let result = self.fire_event_with_args("UPDATE_UI_WIDGET", &[payload]);
        self.lua.borrow_mut().state_mut().top = saved_top;
        result
    }
}

fn validate_visualization(entry: &WidgetSetEntry, record: &WidgetVisualization) -> Result<()> {
    if entry.widget_type != record.widget_type() {
        return Err(Error::Other(
            "UI widget type does not match its visualization".into(),
        ));
    }
    #[cfg(feature = "retail-12-0-0")]
    if matches!(record, WidgetVisualization::PreyHuntProgress(_)) {
        let id = entry.widget_id;
        if id <= 0.0 || id > u32::MAX as f64 || !id.is_finite() || id.fract() != 0.0 {
            return Err(Error::Other(
                "Prey widgetID must be a positive integer".into(),
            ));
        }
    }
    Ok(())
}

fn upsert_membership(entries: &mut Vec<WidgetSetEntry>, entry: &WidgetSetEntry) {
    let key = WidgetId::from_number(entry.widget_id);
    if let Some(existing) = entries
        .iter_mut()
        .find(|item| WidgetId::from_number(item.widget_id) == key)
    {
        *existing = entry.clone();
    } else {
        entries.push(entry.clone());
    }
}

fn supply_visualization(sim: &mut crate::lua_api::SimState, id: f64, record: WidgetVisualization) {
    let key = WidgetId::from_number(id);
    match record {
        WidgetVisualization::IconAndText(record) => {
            sim.ui_widget_inputs.icon_and_text.insert(key, record);
        }
        WidgetVisualization::IconTextAndBackground(record) => {
            sim.ui_widget_inputs
                .icon_text_and_background
                .insert(key, record);
        }
        WidgetVisualization::IconTextAndCurrencies(record) => {
            sim.ui_widget_inputs
                .icon_text_and_currencies
                .insert(key, record);
        }
        WidgetVisualization::HorizontalCurrencies(record) => {
            sim.ui_widget_inputs
                .horizontal_currencies
                .insert(key, record);
        }
        WidgetVisualization::ItemDisplay(record) => {
            sim.ui_widget_inputs.item_display.insert(key, record);
        }
        WidgetVisualization::CaptureBar(record) => {
            sim.ui_widget_inputs.capture_bars.insert(key, record);
        }
        WidgetVisualization::StatusBar(record) => {
            sim.ui_widget_inputs.status_bars.insert(key, record);
        }
        WidgetVisualization::DoubleStatusBar(record) => {
            sim.ui_widget_inputs.double_status_bars.insert(key, record);
        }
        WidgetVisualization::TextureAndText(record) => {
            sim.ui_widget_inputs.texture_and_text.insert(key, record);
        }
        WidgetVisualization::TextureAndTextRow(record) => {
            sim.ui_widget_inputs
                .texture_and_text_row
                .insert(key, record);
        }
        WidgetVisualization::Spacer(record) => {
            sim.ui_widget_inputs.spacers.insert(key, record);
        }
        WidgetVisualization::SpellDisplay(record) => {
            sim.ui_widget_inputs.spell_display.insert(key, record);
        }
        WidgetVisualization::TextWithState(record) => {
            sim.ui_widget_inputs.text_with_state.insert(key, record);
        }
        WidgetVisualization::TextWithSubtext(record) => {
            sim.ui_widget_inputs.text_with_subtext.insert(key, record);
        }
        WidgetVisualization::DoubleIconAndText(record) => {
            sim.ui_widget_inputs
                .double_icon_and_text
                .insert(key, record);
        }
        WidgetVisualization::DiscreteProgressSteps(record) => {
            sim.ui_widget_inputs
                .discrete_progress_steps
                .insert(key, record);
        }
        WidgetVisualization::BulletTextList(record) => {
            sim.ui_widget_inputs.bullet_text_lists.insert(key, record);
        }
        WidgetVisualization::TextColumnRow(record) => {
            sim.ui_widget_inputs.text_column_rows.insert(key, record);
        }
        WidgetVisualization::ButtonHeader(record) => {
            sim.button_header_inputs.widgets.insert(key, record);
        }
        #[cfg(feature = "retail-12-0-0")]
        WidgetVisualization::PreyHuntProgress(record) => {
            sim.prey_inputs.widgets.insert(id as u32, record);
        }
    }
}
