//! Global WoW API functions.
//!
//! This module contains the split WoW API implementations:
//! - `addon_api` - C_AddOns namespace and legacy addon functions
//! - `locale_api` - Locale, region, and build info functions
//! - `create_frame` - CreateFrame function implementation
//! - `unit_api` - Unit information functions (UnitName, UnitClass, etc.)
//! - `timer_api` - C_Timer namespace for timer management
//! - `enum_api` - Enum table with game enumerations
//! - `c_map_api` - C_Map and map/location related namespaces
//! - `c_quest_api` - C_QuestLog, C_TaskQuest, and quest related namespaces
//! - `c_collection_api` - C_MountJournal, C_PetJournal, C_ToyBox, C_Transmog, etc.
//! - `c_misc_api` - Miscellaneous C_* namespaces (C_ScenarioInfo, C_TooltipInfo, etc.)
//! - `c_system_api` - System C_* namespaces (C_XMLUtil, C_Console, C_VoiceChat, C_TTSSettings, etc.)
//! - `dropdown_api` - UIDropDownMenu system
//! - `strings` - UI string constants (ERR_*, localization, font codes, etc.)
//! - `utility_api` - Table manipulation (wipe, tinsert, tContains), string utilities, secure functions
//! - `font_api` - Font object creation (CreateFont, CreateFontFamily, standard fonts)
//! - `settings_api` - Settings namespace for addon configuration UI
//! - `mixin_api` - UI mixins (POIButtonMixin, MapCanvasPinMixin, Menu, MenuUtil)
//! - `player_api` - Player related functions (BattleNet, specialization, action bars)
//! - `pool_api` - Pool creation (CreateTexturePool, CreateFramePool, CreateObjectPool)
//! - `cvar_api` - CVar and key binding functions
//! - `global_frames` - Global frame objects (UIParent, WorldFrame, PlayerFrame, etc.)
//!
//! The main `register_globals` function is still in `globals_legacy.rs`
//! but calls into these split modules.

pub mod addon_api;
pub mod c_collection_api;
pub mod c_item_api;
pub mod c_map_api;
pub mod c_misc_api;
pub mod c_quest_api;
pub mod c_system_api;
pub mod create_frame;
pub mod cvar_api;
pub mod dropdown_api;
pub mod enum_api;
pub mod enum_data;
pub mod font_api;
pub mod global_frames;
pub mod item_api;
pub mod locale_api;
pub mod mixin_api;
pub mod player_api;
pub mod pool_api;
pub mod spell_api;
pub mod quest_frames;
pub mod settings_api;
pub mod system_api;
pub mod strings;
pub mod timer_api;
pub mod tooltip_api;
pub mod ui_frames;
pub mod unit_api;
pub mod utility_api;

// Re-export for backwards compatibility
pub use strings::register_all_ui_strings;

pub use super::globals_legacy::register_globals;
