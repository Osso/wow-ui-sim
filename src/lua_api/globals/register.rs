//! Main register_globals function and core utilities.
//!
//! This module contains the main registration function that orchestrates
//! registering all WoW API globals, plus core Lua utilities like print,
//! type, ipairs, pairs, getmetatable, and setmetatable.

// NOTE: This file is a placeholder for future refactoring.
// The actual register_globals function is still in globals_legacy.rs
// and calls into the split modules (addon_api, locale_api) for some functionality.
//
// Future work:
// 1. Move the core registration logic here
// 2. Create misc_api.rs for all the namespace registrations (C_*, Enum, etc.)
// 3. Have register_globals just orchestrate calls to the split modules
