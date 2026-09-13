//! Frame storage size estimation helpers.

use std::collections::{BTreeSet, HashMap, HashSet};

use super::frame::Frame;
use super::frame_types::{AttributeValue, ModelWidgetState, PlayerModelState};

impl Frame {
    pub fn storage_estimate_bytes(&self) -> usize {
        self.dynamic_string_bytes()
            + vec_bytes(&self.children)
            + vec_bytes(&self.anchors)
            + hash_set_string_bytes(&self.registered_events)
            + hash_map_string_string_bytes(&self.registered_unit_events)
            + hash_set_string_bytes(&self.pass_through_buttons)
            + hash_map_string_attribute_value_bytes(&self.attributes)
            + hash_map_string_u64_bytes(&self.children_keys)
            + hash_set_string_bytes(&self.registered_mouse_buttons)
            + hash_set_string_bytes(&self.registered_click_buttons)
            + hash_set_string_bytes(&self.registered_drag_buttons)
            + btree_set_bytes(&self.disabled_draw_layers)
            + vec_bytes(&self.mask_textures)
            + model_state_bytes(&self.model_state)
            + vec_string_bytes(&self.editbox_history)
    }

    fn dynamic_string_bytes(&self) -> usize {
        self.core_string_bytes()
            + self.texture_string_bytes()
            + self.minimap_string_bytes()
            + option_string_bytes(&self.statusbar_texture_path)
            + self.slider_orientation.capacity()
            + self.statusbar_orientation.capacity()
            + self.editbox_input_language.capacity()
    }

    fn core_string_bytes(&self) -> usize {
        option_string_bytes(&self.object_type_name)
            + option_string_bytes(&self.name)
            + option_string_bytes(&self.texture)
            + option_string_bytes(&self.text)
            + option_string_bytes(&self.font)
            + option_string_bytes(&self.parent_key)
            + option_string_bytes(&self.atlas)
            + option_string_bytes(&self.nine_slice_layout)
            + option_string_bytes(&self.alpha_mode)
    }

    fn texture_string_bytes(&self) -> usize {
        option_string_bytes(&self.normal_texture)
            + option_string_bytes(&self.pushed_texture)
            + option_string_bytes(&self.highlight_texture)
            + option_string_bytes(&self.disabled_texture)
            + option_string_bytes(&self.checked_texture)
            + option_string_bytes(&self.disabled_checked_texture)
            + option_string_bytes(&self.left_texture)
            + option_string_bytes(&self.middle_texture)
            + option_string_bytes(&self.right_texture)
    }

    fn minimap_string_bytes(&self) -> usize {
        option_string_bytes(&self.minimap_blip_texture)
            + option_string_bytes(&self.fog_of_war_background_atlas)
            + option_string_bytes(&self.fog_of_war_mask_atlas)
            + std::mem::size_of_val(&self.fog_of_war_ui_map_id)
            + option_string_bytes(&self.minimap_mask_texture)
            + option_string_bytes(&self.minimap_icon_texture)
            + option_string_bytes(&self.minimap_player_texture)
            + option_string_bytes(&self.minimap_poi_arrow_texture)
            + option_string_bytes(&self.minimap_corpse_poi_arrow_texture)
            + option_string_bytes(&self.minimap_static_poi_arrow_texture)
    }
}

fn option_string_bytes(value: &Option<String>) -> usize {
    value.as_ref().map_or(0, String::capacity)
}

fn model_state_bytes(value: &Option<Box<ModelWidgetState>>) -> usize {
    let Some(state) = value.as_deref() else {
        return 0;
    };
    std::mem::size_of::<ModelWidgetState>()
        + option_string_bytes(&state.model_path)
        + state.model_scene_actor_ids.capacity() * std::mem::size_of::<u64>()
        + model_scene_actor_tag_bytes(state)
        + player_model_string_bytes(&state.player_model_state)
}

fn model_scene_actor_tag_bytes(state: &ModelWidgetState) -> usize {
    state.model_scene_actor_tags.capacity() * std::mem::size_of::<(String, u64)>()
        + state
            .model_scene_actor_tags
            .iter()
            .map(|(tag, _)| tag.capacity())
            .sum::<usize>()
}

fn player_model_string_bytes(state: &PlayerModelState) -> usize {
    option_string_bytes(&state.last_unit)
        + option_string_bytes(&state.last_item)
        + option_string_bytes(&state.last_item_appearance)
}

fn vec_bytes<T>(values: &[T]) -> usize {
    std::mem::size_of_val(values)
}

fn vec_string_bytes(values: &[String]) -> usize {
    vec_bytes(values) + values.iter().map(String::capacity).sum::<usize>()
}

fn hash_set_string_bytes(values: &HashSet<String>) -> usize {
    values.capacity() * std::mem::size_of::<String>()
        + values.iter().map(String::capacity).sum::<usize>()
}

fn hash_map_string_u64_bytes(values: &HashMap<String, u64>) -> usize {
    values.capacity() * std::mem::size_of::<(String, u64)>()
        + values.keys().map(String::capacity).sum::<usize>()
}

fn hash_map_string_string_bytes(values: &HashMap<String, String>) -> usize {
    values.capacity() * std::mem::size_of::<(String, String)>()
        + values
            .iter()
            .map(|(key, value)| key.capacity() + value.capacity())
            .sum::<usize>()
}

fn hash_map_string_attribute_value_bytes(values: &HashMap<String, AttributeValue>) -> usize {
    values.capacity() * std::mem::size_of::<(String, AttributeValue)>()
        + values
            .iter()
            .map(|(key, value)| key.capacity() + attribute_value_bytes(value))
            .sum::<usize>()
}

fn attribute_value_bytes(value: &AttributeValue) -> usize {
    match value {
        AttributeValue::String(text) | AttributeValue::LuaRef(text) => text.capacity(),
        AttributeValue::Number(_) | AttributeValue::Boolean(_) | AttributeValue::Nil => 0,
    }
}

fn btree_set_bytes<T>(values: &BTreeSet<T>) -> usize {
    values.len() * std::mem::size_of::<T>()
}
