//! INFERRED host-supplied class trainer rows; native evidence is pending as R88cb.

use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_spell;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Debug)]
pub struct TrainerState {
    pub services: Vec<TrainerService>,
    pub selected_service_index: Option<i32>,
    pub step_service_index: Option<i32>,
    pub tradeskill_rank: Option<(u32, u32, u32)>,
    pub show_available: bool,
    pub show_unavailable: bool,
    pub show_used: bool,
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
            selected_service_index: None,
            step_service_index: None,
            tradeskill_rank: None,
            show_available: true,
            show_unavailable: true,
            show_used: false,
        }
    }
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
