//! INFERRED host-supplied class trainer rows; native evidence is pending as R88cb.

use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_spell;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

#[derive(Clone, Debug)]
pub struct TrainerState {
    pub services: Vec<TrainerService>,
    pub trainer_type: TrainerType,
    pub categorize_ui: bool,
    pub selected_service_index: Option<i32>,
    pub step_service_index: Option<i32>,
    pub tradeskill_rank: Option<(u32, u32, u32)>,
    pub show_available: bool,
    pub show_unavailable: bool,
    pub show_used: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrainerType {
    General,
    TalentsObsolete,
    Tradeskills,
    Pet,
}

impl TrainerType {
    fn enum_value(self) -> f64 {
        match self {
            Self::General => 0.0,
            Self::TalentsObsolete => 1.0,
            Self::Tradeskills => 2.0,
            Self::Pet => 3.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TrainerService {
    pub spell_id: u32,
    pub name: String,
    pub sub_text: String,
    pub service_type: TrainerServiceType,
    pub icon: u32,
    pub cost: u64,
    pub required_level: Option<u32>,
    pub required_skill: Option<TrainerSkillRequirement>,
    pub ability_requirements: Vec<TrainerAbilityRequirement>,
    pub is_profession: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrainerServiceType {
    Available,
    Unavailable,
    Used,
}

impl Default for TrainerState {
    fn default() -> Self {
        // INFERRED from the existing Mists trainer UI defaults; R88cb is pending.
        Self {
            services: Vec::new(),
            trainer_type: TrainerType::Tradeskills,
            categorize_ui: false,
            selected_service_index: None,
            step_service_index: None,
            tradeskill_rank: None,
            show_available: true,
            show_unavailable: true,
            show_used: false,
        }
    }
}

pub(crate) fn register_forever_trainer_namespace(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCategorizeTrainerUI",
        get_categorize_trainer_ui,
    )?;
    table_set_rust_fn_static(state, namespace, "GetTrainerType", get_trainer_type)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetCategorizeTrainerUI",
        set_categorize_trainer_ui,
    )
}

fn get_categorize_trainer_ui(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.trainer.categorize_ui;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn get_trainer_type(state: &mut LuaState) -> LuaResult<u32> {
    let trainer_type = borrow_state(state)?.trainer.trainer_type.enum_value();
    state.push(Val::Num(trainer_type));
    Ok(1)
}

fn set_categorize_trainer_ui(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.trainer.categorize_ui = enabled;
    Ok(0)
}

pub(crate) fn get_trainer_service_tooltip(state: &mut LuaState) -> LuaResult<u32> {
    let service_index = Option::<i32>::from_stack(state, 1)?;
    let spell_id = match (crate::client_profile::ACTIVE, service_index) {
        (
            crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever,
            Some(index),
        ) => borrow_state(state)?
            .trainer
            .service_at(index)
            .map(|service| service.spell_id),
        _ => None,
    };
    let tooltip = spell_id
        .filter(|spell_id| *spell_id > 0)
        .map(|spell_id| tooltip_for_spell(state, spell_id))
        .unwrap_or(Val::Nil);
    state.push(tooltip);
    Ok(1)
}

impl TrainerState {
    /// Service indexes are one-based over the currently enabled type filters.
    pub fn service_at(&self, index: i32) -> Option<&TrainerService> {
        let visible_index = usize::try_from(index.checked_sub(1)?).ok()?;
        self.services
            .iter()
            .filter(|service| self.service_is_visible(service))
            .nth(visible_index)
    }

    pub fn visible_service_count(&self) -> usize {
        self.services
            .iter()
            .filter(|service| self.service_is_visible(service))
            .count()
    }

    fn service_is_visible(&self, service: &TrainerService) -> bool {
        match service.service_type {
            TrainerServiceType::Available => self.show_available,
            TrainerServiceType::Unavailable => self.show_unavailable,
            TrainerServiceType::Used => self.show_used,
        }
    }
}

impl TrainerServiceType {
    pub fn as_lua_value(&self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unavailable => "unavailable",
            Self::Used => "used",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TrainerSkillRequirement {
    pub name: String,
    pub rank: u32,
    pub met: bool,
}

#[derive(Clone, Debug)]
pub struct TrainerAbilityRequirement {
    pub name: String,
    pub met: bool,
}
