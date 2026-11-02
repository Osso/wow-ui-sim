//! Fallible shutdown shared by GUI close, Lua Quit and UI reload.

use super::env::WowLuaEnv;
use super::state::SimState;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::saved_variables::SavedVariablesManager;
use crate::{Error, Result};
use rilua::{LuaApiMut, Val};

pub(crate) enum SessionLifecycle {
    Active,
    Incomplete(String),
    Finished,
}

pub(super) struct ShutdownState {
    in_world: bool,
    #[cfg(feature = "client-wrath")]
    protected_actions: bool,
    #[cfg(feature = "client-wrath")]
    mirror_timers: Vec<super::state::MirrorTimer>,
}

impl SimState {
    pub(crate) fn mark_addon_initialization_failed(
        &mut self,
        addon: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.addon_initialization_failures
            .insert(addon.into(), message.into());
    }

    pub(crate) fn mark_addon_initialized(&mut self, addon: &str) {
        self.addon_initialization_failures.remove(addon);
    }

    pub(crate) fn record_addon_initialization_errors_since(&mut self, first_record: usize) {
        let errors: Vec<_> = self
            .lua_error_records
            .iter()
            .skip(first_record)
            .cloned()
            .collect();
        for error in errors {
            match error.addon_name {
                Some(addon) if addon != "__BuiltIn" => {
                    self.mark_addon_initialization_failed(addon, error.message);
                }
                _ => {
                    self.session_lifecycle = SessionLifecycle::Incomplete(format!(
                        "UI initialization callback failed: {}",
                        error.message
                    ));
                }
            }
        }
    }
}

impl WowLuaEnv {
    /// Protect an owner's uninitialized globals until it is disabled or initialized again.
    pub fn mark_addon_initialization_failed(
        &self,
        addon: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.state
            .borrow_mut()
            .mark_addon_initialization_failed(addon, message);
    }

    pub fn mark_addon_initialized(&self, addon: &str) {
        self.state.borrow_mut().mark_addon_initialized(addon);
    }

    /// Attribute current initialization errors without treating historical diagnostics as failures.
    pub fn record_addon_initialization_errors_since(&self, first_record: usize) {
        self.state
            .borrow_mut()
            .record_addon_initialization_errors_since(first_record);
    }

    /// A partially reconstructed environment may only be repaired, reloaded or closed.
    pub fn is_ui_environment_usable(&self) -> bool {
        matches!(
            self.state.borrow().session_lifecycle,
            SessionLifecycle::Active
        )
    }

    /// Finish callbacks and save once; an intact environment can retry a failed attempt.
    pub fn finalize_session(
        &mut self,
        saved_vars: &mut Option<SavedVariablesManager>,
    ) -> Result<()> {
        {
            let state = self.state.borrow();
            match &state.session_lifecycle {
                SessionLifecycle::Finished => return Ok(()),
                SessionLifecycle::Incomplete(error) => return Err(Error::Other(error.clone())),
                SessionLifecycle::Active => {}
            }
        }
        let shutdown_state = self.capture_shutdown_state();
        if let Err(error) = self.save_final_session(saved_vars) {
            self.restore_shutdown_state(shutdown_state);
            return Err(error);
        }
        self.state.borrow_mut().session_lifecycle = SessionLifecycle::Finished;
        Ok(())
    }

    fn save_final_session(&mut self, saved_vars: &mut Option<SavedVariablesManager>) -> Result<()> {
        self.ensure_initialized_addons(saved_vars.as_ref())?;
        if let Some(manager) = saved_vars {
            self.retry_saved_inputs(manager)?;
        }
        self.fire_shutdown_events(true)?;
        self.ensure_initialized_addons(saved_vars.as_ref())?;
        self.ensure_ui_environment_usable()?;
        if let Some(manager) = saved_vars {
            let exclusions = self.addon_save_exclusions();
            self.save_initialized_addons(manager, &exclusions)?;
            #[cfg(feature = "client-wrath")]
            if exclusions.is_empty() {
                super::placed_frames::save(self, manager)?;
            }
        }
        Ok(())
    }

    fn ensure_initialized_addons(&self, manager: Option<&SavedVariablesManager>) -> Result<()> {
        let Some(manager) = manager else {
            return Ok(());
        };
        let registered = manager.registered_addons();
        let state = self.state.borrow();
        let mut failures: Vec<_> = state
            .addon_initialization_failures
            .iter()
            .filter(|(name, _)| {
                registered.contains(name)
                    && state
                        .addons
                        .iter()
                        .find(|addon| &addon.folder_name == *name)
                        .is_none_or(|addon| addon.enabled)
            })
            .map(|(name, error)| format!("{name}: {error}"))
            .collect();
        if failures.is_empty() {
            Ok(())
        } else {
            failures.sort();
            Err(Error::Other(format!(
                "addon initialization incomplete: {}",
                failures.join("; ")
            )))
        }
    }

    pub(super) fn ensure_ui_environment_usable(&self) -> Result<()> {
        match &self.state.borrow().session_lifecycle {
            SessionLifecycle::Incomplete(error) => Err(Error::Other(error.clone())),
            SessionLifecycle::Finished => Err(Error::Other("session is already finalized".into())),
            SessionLifecycle::Active => Ok(()),
        }
    }

    pub(super) fn addon_save_exclusions(&self) -> std::collections::HashSet<String> {
        self.state
            .borrow()
            .addon_initialization_failures
            .keys()
            .cloned()
            .collect()
    }

    pub(super) fn retry_saved_inputs(&self, manager: &mut SavedVariablesManager) -> Result<()> {
        let disabled: std::collections::HashSet<_> = self
            .state
            .borrow()
            .addons
            .iter()
            .filter(|addon| !addon.enabled)
            .map(|addon| addon.folder_name.clone())
            .collect();
        manager.retry_failed_inputs_excluding(self.rilua_mut().state_mut(), &disabled)
    }

    pub(super) fn capture_shutdown_state(&self) -> ShutdownState {
        let state = self.state.borrow();
        ShutdownState {
            in_world: state.in_world,
            #[cfg(feature = "client-wrath")]
            protected_actions: state.reload_allows_protected_actions,
            #[cfg(feature = "client-wrath")]
            mirror_timers: state.world.mirror_timers.clone(),
        }
    }

    pub(super) fn restore_shutdown_state(&self, saved: ShutdownState) {
        let mut state = self.state.borrow_mut();
        state.in_world = saved.in_world;
        #[cfg(feature = "client-wrath")]
        {
            state.reload_allows_protected_actions = saved.protected_actions;
            state.world.mirror_timers = saved.mirror_timers;
        }
    }

    pub(super) fn save_initialized_addons(
        &self,
        manager: &SavedVariablesManager,
        exclusions: &std::collections::HashSet<String>,
    ) -> Result<()> {
        if exclusions.is_empty() {
            return manager.save_all(self.rilua_mut().state_mut());
        }
        for addon in manager.registered_addons() {
            if !exclusions.contains(addon) {
                manager.save_addon(self.rilua_mut().state_mut(), addon)?;
            }
        }
        Ok(())
    }

    pub(super) fn fire_shutdown_events(&self, closing_client: bool) -> Result<()> {
        let previous_errors = self.state.borrow().lua_errors.len();
        match ACTIVE {
            ClientProfile::Wrath => self.fire_wrath_reload_shutdown()?,
            ClientProfile::WowForever => self.fire_forever_shutdown(closing_client)?,
            _ => return Ok(()),
        }
        // Normal event delivery reports callback errors and continues other listeners.
        // Keep that behavior while preventing a failed shutdown from producing a save.
        let state = self.state.borrow();
        if let Some(error) = state.lua_errors.get(previous_errors) {
            return Err(Error::Other(format!(
                "session shutdown callback failed: {error}"
            )));
        }
        Ok(())
    }

    fn fire_forever_shutdown(&self, closing_client: bool) -> Result<()> {
        // Researched best guess, not measured native chronology. See
        // docs/specs/saved-variables-native-evidence.md#shutdown-evidence.
        let leaving_world = self.state.borrow().in_world;
        if leaving_world {
            self.state.borrow_mut().in_world = false;
            self.fire_event("PLAYER_LEAVING_WORLD")?;
        }
        self.fire_event("PLAYER_LOGOUT")?;
        self.fire_event_with_args("ADDONS_UNLOADING", &[Val::Bool(closing_client)])
    }
}

#[cfg(test)]
#[path = "env_lifecycle_tests.rs"]
mod tests;
