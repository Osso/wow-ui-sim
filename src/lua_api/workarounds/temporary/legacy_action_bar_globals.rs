//! Temporary legacy action-bar global wrappers.
//!
//! `C_ActionBar` is registered by the Rust action-bar API. These globals remain
//! for older Blizzard/addon callers until deprecated global action-bar access is
//! no longer required.

const LEGACY_ACTION_BAR_GLOBALS_LUA: &str = r#"
if ActionButtonUtil == nil then
  ActionButtonUtil = {}
end

BaseActionButtonMixin = BaseActionButtonMixin or {}

ActionButtonUtil.ActionBarActionStatus = ActionButtonUtil.ActionBarActionStatus or {
  NotMissing = 1,
  MissingFromAllBars = 2,
  OnInactiveBonusBar = 3,
  OnDisabledActionBar = 4,
}

ActionButtonUtil.ActionBarButtonNames = ActionButtonUtil.ActionBarButtonNames or {}

if ActionButtonUtil.GetActionBarStatusForSpell == nil then
  function ActionButtonUtil.GetActionBarStatusForSpell(_spellID, _excludeNonPlayerBars, _excludeSpecialPlayerBars)
    return ActionButtonUtil.ActionBarActionStatus.NotMissing
  end
end

if ActionButtonUtil.GetActionBarStatusForPetAction == nil then
  function ActionButtonUtil.GetActionBarStatusForPetAction(_petActionID)
    return ActionButtonUtil.ActionBarActionStatus.NotMissing
  end
end

if ActionButtonUtil.GetActionBarStatusForFlyout == nil then
  function ActionButtonUtil.GetActionBarStatusForFlyout(_flyoutActionID)
    return ActionButtonUtil.ActionBarActionStatus.NotMissing
  end
end

ActionButtonSpellAlertManager = ActionButtonSpellAlertManager or {
  _defaultAlertType = 1,
  activeAlerts = {},
}

local function __wow_legacy_action_button_alert_fields(button)
  local env = debug.getfenv and debug.getfenv(button)
  if type(env) ~= "table" then
    return nil
  end
  local fields = env[1]
  if type(fields) ~= "table" then
    fields = {}
    env[1] = fields
  end
  return fields
end

if rawget(ActionButtonSpellAlertManager, "HasAlert") == nil then
  function ActionButtonSpellAlertManager:HasAlert(button)
    local alertType = self.activeAlerts and self.activeAlerts[button]
    if alertType ~= nil then
      return true, alertType
    end
    return false
  end
end

if rawget(ActionButtonSpellAlertManager, "ShowAlert") == nil then
  function ActionButtonSpellAlertManager:ShowAlert(button, alertType)
    if button == nil then
      return
    end
    alertType = alertType or self._defaultAlertType or 1
    self.activeAlerts[button] = alertType
    local fields = __wow_legacy_action_button_alert_fields(button)
    local alert = fields and rawget(fields, "SpellActivationAlert")
    if alert == nil then
      alert = CreateFrame("Frame", nil, UIParent or button)
      if fields then
        rawset(fields, "SpellActivationAlert", alert)
      end
      button.SpellActivationAlert = alert
    end
    button:Show()
    alert:Show()
  end
end

if rawget(ActionButtonSpellAlertManager, "HideAlert") == nil then
  function ActionButtonSpellAlertManager:HideAlert(button)
    if button == nil then
      return
    end
    self.activeAlerts[button] = nil
    local fields = __wow_legacy_action_button_alert_fields(button)
    local alert = fields and rawget(fields, "SpellActivationAlert")
    if alert ~= nil then
      alert:Hide()
    end
  end
end

local function __wow_legacy_action_bar_forward(globalName, methodName)
    if _G[globalName] == nil and C_ActionBar ~= nil and type(C_ActionBar[methodName]) == "function" then
        _G[globalName] = function(...)
            return C_ActionBar[methodName](...)
        end
    end
end

__wow_legacy_action_bar_forward("GetBonusBarIndex", "GetBonusBarIndex")
__wow_legacy_action_bar_forward("GetBonusBarOffset", "GetBonusBarOffset")
__wow_legacy_action_bar_forward("GetExtraBarIndex", "GetExtraBarIndex")
__wow_legacy_action_bar_forward("GetMultiCastBarIndex", "GetMultiCastBarIndex")
__wow_legacy_action_bar_forward("GetOverrideBarIndex", "GetOverrideBarIndex")
__wow_legacy_action_bar_forward("GetOverrideBarSkin", "GetOverrideBarSkin")
__wow_legacy_action_bar_forward("GetTempShapeshiftBarIndex", "GetTempShapeshiftBarIndex")
__wow_legacy_action_bar_forward("GetVehicleBarIndex", "GetVehicleBarIndex")
__wow_legacy_action_bar_forward("GetActionBarPage", "GetActionBarPage")
__wow_legacy_action_bar_forward("ChangeActionBarPage", "SetActionBarPage")
"#;

#[cfg(feature = "client-mists")]
const MISTS_LEGACY_ACTION_BAR_GLOBALS_LUA: &str = r#"
local function __wow_legacy_action_bar_forward(globalName, methodName)
    if _G[globalName] == nil and C_ActionBar ~= nil and type(C_ActionBar[methodName]) == "function" then
        _G[globalName] = function(...)
            return C_ActionBar[methodName](...)
        end
    end
end

__wow_legacy_action_bar_forward("HasVehicleActionBar", "HasVehicleActionBar")
__wow_legacy_action_bar_forward("HasOverrideActionBar", "HasOverrideActionBar")
__wow_legacy_action_bar_forward("HasBonusActionBar", "HasBonusActionBar")
__wow_legacy_action_bar_forward("HasTempShapeshiftActionBar", "HasTempShapeshiftActionBar")
__wow_legacy_action_bar_forward("HasExtraActionBar", "HasExtraActionBar")
__wow_legacy_action_bar_forward("IsPossessBarVisible", "IsPossessBarVisible")
"#;

#[cfg(not(feature = "client-retail"))]
const ACTION_BUTTON_QUICK_KEYBIND_DEFAULTS_LUA: &str = r#"
local function __wow_legacy_action_bar_noop()
end

if ActionButtonUtil.ShowAllActionButtonGrids == nil then
  ActionButtonUtil.ShowAllActionButtonGrids = __wow_legacy_action_bar_noop
end

if ActionButtonUtil.HideAllActionButtonGrids == nil then
  ActionButtonUtil.HideAllActionButtonGrids = __wow_legacy_action_bar_noop
end

if ActionButtonUtil.SetAllQuickKeybindButtonHighlights == nil then
  ActionButtonUtil.SetAllQuickKeybindButtonHighlights = __wow_legacy_action_bar_noop
end

if ActionButtonUtil.ShowAllQuickKeybindButtonHighlights == nil then
  ActionButtonUtil.ShowAllQuickKeybindButtonHighlights = __wow_legacy_action_bar_noop
end

if ActionButtonUtil.HideAllQuickKeybindButtonHighlights == nil then
  ActionButtonUtil.HideAllQuickKeybindButtonHighlights = __wow_legacy_action_bar_noop
end

"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(LEGACY_ACTION_BAR_GLOBALS_LUA)?;
    // Normal Retail QuickKeybind starts hidden, then uses the eager ActionBar owner.
    #[cfg(not(feature = "client-retail"))]
    lua.exec(ACTION_BUTTON_QUICK_KEYBIND_DEFAULTS_LUA)?;
    #[cfg(feature = "client-mists")]
    lua.exec(MISTS_LEGACY_ACTION_BAR_GLOBALS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn keeps_legacy_bonus_bar_offset_global_backed_by_c_action_bar() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local original = C_ActionBar.GetBonusBarIndex
                C_ActionBar.GetBonusBarIndex = function() return 11 end
                local namespaceOffset = C_ActionBar.GetBonusBarOffset()
                local globalOffset = GetBonusBarOffset()
                C_ActionBar.GetBonusBarIndex = original
                if namespaceOffset ~= 5 or globalOffset ~= 5 then return "offset" end
                if GetBonusBarIndex() ~= C_ActionBar.GetBonusBarIndex() then return "bonus" end
                if GetExtraBarIndex() ~= C_ActionBar.GetExtraBarIndex() then return "extra" end
                if GetMultiCastBarIndex() ~= C_ActionBar.GetMultiCastBarIndex() then return "multicast" end
                if GetOverrideBarIndex() ~= C_ActionBar.GetOverrideBarIndex() then return "override" end
                if GetOverrideBarSkin() ~= C_ActionBar.GetOverrideBarSkin() then return "skin" end
                if GetTempShapeshiftBarIndex() ~= C_ActionBar.GetTempShapeshiftBarIndex() then return "temp" end
                if GetVehicleBarIndex() ~= C_ActionBar.GetVehicleBarIndex() then return "vehicle" end
                if GetActionBarPage() ~= C_ActionBar.GetActionBarPage() then return "page" end
                ChangeActionBarPage(4)
                if C_ActionBar.GetActionBarPage() ~= 4 then return "change_page" end
                return "ok"
                "#,
            )
            .expect("legacy action-bar global should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_legacy_bonus_bar_offset_global() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("GetBonusBarOffset = function() return 42 end")
            .expect("fixture should install existing legacy global");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("legacy action-bar globals should apply");
        }

        let offset: i32 = env
            .eval("return GetBonusBarOffset()")
            .expect("preserved legacy action-bar global should run");

        assert_eq!(offset, 42);
    }

    #[test]
    fn installs_action_button_util_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local enum = ActionButtonUtil and ActionButtonUtil.ActionBarActionStatus
                if not enum then return "missing_enum" end
                if type(BaseActionButtonMixin) ~= "table" then return "missing_base_mixin" end
                if enum.NotMissing ~= 1 or enum.MissingFromAllBars ~= 2 then return "bad_enum" end
                if type(ActionButtonUtil.ActionBarButtonNames) ~= "table" then return "missing_names" end
                if ActionButtonUtil.GetActionBarStatusForSpell(1) ~= enum.NotMissing then return "spell" end
                if ActionButtonUtil.GetActionBarStatusForPetAction(1) ~= enum.NotMissing then return "pet" end
                if ActionButtonUtil.GetActionBarStatusForFlyout(1) ~= enum.NotMissing then return "flyout" end
                if type(ActionButtonSpellAlertManager) ~= "table" then return "missing_alert_manager" end
                if type(ActionButtonSpellAlertManager.HasAlert) ~= "function" then return "missing_has_alert" end
                if type(ActionButtonSpellAlertManager.ShowAlert) ~= "function" then return "missing_show_alert" end
                if type(ActionButtonSpellAlertManager.HideAlert) ~= "function" then return "missing_hide_alert" end
                return "ok"
                "#,
            )
            .expect("ActionButtonUtil defaults should run");

        assert_eq!(result, "ok");
        #[cfg(not(feature = "client-retail"))]
        env.exec(
            r#"
                ActionButtonUtil.ShowAllActionButtonGrids()
                ActionButtonUtil.HideAllActionButtonGrids()
                ActionButtonUtil.SetAllQuickKeybindButtonHighlights()
                ActionButtonUtil.ShowAllQuickKeybindButtonHighlights()
                ActionButtonUtil.HideAllQuickKeybindButtonHighlights()
        "#,
        )
        .expect("non-Retail quick-keybind defaults remain callable");
    }

    #[test]
    fn spell_alert_manager_tracks_and_toggles_alert_frame() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local button = CreateFrame("Button", "TemporarySpellAlertButton", UIParent)
                local before = ActionButtonSpellAlertManager:HasAlert(button)

                ActionButtonSpellAlertManager:ShowAlert(button, 7)
                local during, alertType = ActionButtonSpellAlertManager:HasAlert(button)
                local alert = button.SpellActivationAlert
                local shownDuring = alert ~= nil and alert:IsShown()

                ActionButtonSpellAlertManager:HideAlert(button)
                local after = ActionButtonSpellAlertManager:HasAlert(button)
                local shownAfter = alert ~= nil and alert:IsShown()

                if before then return "bad_before" end
                if not during or alertType ~= 7 then return "bad_during" end
                if not shownDuring then return "bad_shown" end
                if after then return "bad_after" end
                if shownAfter then return "bad_hidden" end
                return "ok"
                "#,
            )
            .expect("spell-alert manager defaults should run");

        assert_eq!(result, "ok");
    }
}
