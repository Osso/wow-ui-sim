//! Profile-independent generated catalogs, shared by all client builds.
//!
//! Item catalogs are committed exactly as their generators write them, so
//! `rustfmt::skip` keeps cargo fmt from rewriting them; rustfmt also aborts
//! on `item_spec_inputs.rs` with a terabyte allocation.

#[path = "../data/currencies.rs"]
pub mod currencies;
#[path = "../data/currency_containers.rs"]
pub mod currency_containers;
#[path = "../data/encounter_journal.rs"]
pub mod encounter_journal_data;
#[path = "../data/garrison_currency.rs"]
pub mod garrison_currency;
#[path = "../data/garrison_follower_xp.rs"]
pub mod garrison_follower_xp;
#[rustfmt::skip]
#[path = "../data/gem_items.rs"]
pub mod gem_items;
#[rustfmt::skip]
#[path = "../data/item_categories.rs"]
pub mod item_categories;
#[rustfmt::skip]
#[path = "../data/item_classifiers.rs"]
pub mod item_classifiers;
#[rustfmt::skip]
#[path = "../data/item_children.rs"]
pub mod item_children;
#[rustfmt::skip]
#[path = "../data/item_cursor_grid.rs"]
pub mod item_cursor_grid;
#[rustfmt::skip]
#[path = "../data/item_enchants.rs"]
pub mod item_enchants;
#[rustfmt::skip]
#[path = "../data/item_spell_metadata.rs"]
pub mod item_spell_metadata;
#[rustfmt::skip]
#[path = "../data/item_spell_ranges.rs"]
pub mod item_spell_ranges;
#[rustfmt::skip]
#[path = "../data/item_sets.rs"]
pub mod item_sets;
#[rustfmt::skip]
#[path = "../data/item_spec_inputs.rs"]
pub mod item_spec_inputs;
#[rustfmt::skip]
#[path = "../data/item_uniqueness.rs"]
pub mod item_uniqueness;
#[rustfmt::skip]
#[path = "../data/items.rs"]
pub mod items;
#[path = "../data/manifest_interface_data.rs"]
pub mod manifest_interface_data;
#[path = "../data/map_art.rs"]
pub mod map_art;
#[path = "../data/map_challenge_modes.rs"]
pub mod map_challenge_modes;
#[path = "../data/model_scenes.rs"]
pub mod model_scenes;
#[path = "../data/perks_categories.rs"]
pub mod perks_categories;
#[path = "../data/pet_talent_trees.rs"]
pub mod pet_talent_trees;
pub mod profession_item_overrides;
#[path = "../data/profession_spec_reminders.rs"]
pub mod profession_spec_reminders;
#[path = "../data/quest_poi_blobs.rs"]
pub mod quest_poi_blobs;
#[path = "../data/quest_ui_map.rs"]
pub mod quest_ui_map;
#[path = "../data/spec_display_spells.rs"]
pub mod spec_display_spells;
#[path = "../data/specializations.rs"]
pub mod specializations;
#[path = "../data/spell_descriptions.rs"]
pub mod spell_descriptions;
#[path = "../data/spell_power.rs"]
pub mod spell_power;
#[path = "../data/spells.rs"]
pub mod spells;
#[path = "../data/traits.rs"]
pub mod traits;
#[path = "../data/ui_maps.rs"]
pub mod ui_maps;
#[path = "../data/zones.rs"]
pub mod zones;
