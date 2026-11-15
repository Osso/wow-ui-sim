//! Temporary PlayerSpells OnLoad backfill.
//!
//! PlayerSpells child frames can be used before their Blizzard OnLoad handlers
//! have initialized per-tab state. This keeps those frames usable until the
//! simulator models the PlayerSpells lifecycle ordering closely enough to drop
//! the backfill.

use crate::lua_api::LoaderEnv;
#[cfg(test)]
use crate::lua_api::WowLuaEnv;

const PLAYER_SPELLS_UTIL_BOOTSTRAP_LUA: &str = r#"
if type(PlayerSpellsUtil) ~= "table" then
    PlayerSpellsUtil = {}
end
if type(PlayerSpellsUtil.FrameTabs) ~= "table" then
    PlayerSpellsUtil.FrameTabs = {}
end
local frameTabs = {
    ClassSpecializations = 1,
    ClassTalents = 2,
    SpellBook = 3,
}
for key, value in pairs(frameTabs) do
    if PlayerSpellsUtil.FrameTabs[key] == nil then
        PlayerSpellsUtil.FrameTabs[key] = value
    end
end
if type(PlayerSpellsUtil.SpellBookCategories) ~= "table" then
    PlayerSpellsUtil.SpellBookCategories = {}
end
local spellBookCategories = {
    Class = 1,
    General = 2,
    Pet = 3,
}
for key, value in pairs(spellBookCategories) do
    if PlayerSpellsUtil.SpellBookCategories[key] == nil then
        PlayerSpellsUtil.SpellBookCategories[key] = value
    end
end

if type(PlayerSpellsFrame_LoadUI) ~= "function" then
    function PlayerSpellsFrame_LoadUI()
        if type(C_AddOns) == "table" and type(C_AddOns.LoadAddOn) == "function" then
            return C_AddOns.LoadAddOn("Blizzard_PlayerSpells")
        end
        if type(UIParentLoadAddOn) == "function" then
            return UIParentLoadAddOn("Blizzard_PlayerSpells")
        end
        return nil
    end
end

local function load_player_spells_frame()
    if not PlayerSpellsFrame and type(PlayerSpellsFrame_LoadUI) == "function" then
        PlayerSpellsFrame_LoadUI()
    end
    return PlayerSpellsFrame
end

local function call_playerspells_util(methodName, bootstrapMethod, ...)
    load_player_spells_frame()
    if type(PlayerSpellsUtil) ~= "table" then
        return nil
    end
    local method = rawget(PlayerSpellsUtil, methodName)
    if type(method) ~= "function" or method == bootstrapMethod then
        return nil
    end
    return method(...)
end

if rawget(PlayerSpellsUtil, "GetCurrentTabID") == nil then
    function PlayerSpellsUtil.GetCurrentTabID()
        local frame = load_player_spells_frame()
        if not frame or type(frame.GetCurrentTabID) ~= "function" then
            return nil
        end
        return frame:GetCurrentTabID()
    end
end
local bootstrapTogglePlayerSpellsFrame
if rawget(PlayerSpellsUtil, "TogglePlayerSpellsFrame") == nil then
    bootstrapTogglePlayerSpellsFrame = function(suggestedTab, inspectUnit)
        return call_playerspells_util(
            "TogglePlayerSpellsFrame",
            bootstrapTogglePlayerSpellsFrame,
            suggestedTab,
            inspectUnit
        )
    end
    PlayerSpellsUtil.TogglePlayerSpellsFrame = bootstrapTogglePlayerSpellsFrame
end
local bootstrapOpenToSpellBookTabAtSpell
if rawget(PlayerSpellsUtil, "OpenToSpellBookTabAtSpell") == nil then
    bootstrapOpenToSpellBookTabAtSpell = function(spellID, knownSpellsOnly, toggleFlyout, flyoutReason)
        return call_playerspells_util(
            "OpenToSpellBookTabAtSpell",
            bootstrapOpenToSpellBookTabAtSpell,
            spellID,
            knownSpellsOnly,
            toggleFlyout,
            flyoutReason
        )
    end
    PlayerSpellsUtil.OpenToSpellBookTabAtSpell = bootstrapOpenToSpellBookTabAtSpell
end
if rawget(PlayerSpellsUtil, "ToggleClassTalentFrame") == nil then
    function PlayerSpellsUtil.ToggleClassTalentFrame(inspectUnit)
        return PlayerSpellsUtil.TogglePlayerSpellsFrame(PlayerSpellsUtil.FrameTabs.ClassTalents, inspectUnit)
    end
end
if rawget(PlayerSpellsUtil, "ToggleSpellBookFrame") == nil then
    function PlayerSpellsUtil.ToggleSpellBookFrame(spellBookCategory)
        return PlayerSpellsUtil.TogglePlayerSpellsFrame(
            PlayerSpellsUtil.FrameTabs.SpellBook,
            spellBookCategory
        )
    end
end
local bootstrapOpenToSpellBookTab
if rawget(PlayerSpellsUtil, "OpenToSpellBookTab") == nil then
    bootstrapOpenToSpellBookTab = function()
        return call_playerspells_util("OpenToSpellBookTab", bootstrapOpenToSpellBookTab)
    end
    PlayerSpellsUtil.OpenToSpellBookTab = bootstrapOpenToSpellBookTab
end
if TogglePlayerSpellsFrame == nil then
    function TogglePlayerSpellsFrame(suggestedTab, inspectUnit)
        return PlayerSpellsUtil.TogglePlayerSpellsFrame(suggestedTab, inspectUnit)
    end
end
"#;

const PLAYER_SPELLS_UTIL_OPEN_TAB_FALLBACK_LUA: &str = r#"
if rawget(PlayerSpellsUtil, "OpenToClassTalentsTab") == nil then
    function PlayerSpellsUtil.OpenToClassTalentsTab(inspectUnit)
        return PlayerSpellsUtil.TogglePlayerSpellsFrame(PlayerSpellsUtil.FrameTabs.ClassTalents, inspectUnit)
    end
end
if rawget(PlayerSpellsUtil, "OpenToClassSpecializationsTab") == nil then
    function PlayerSpellsUtil.OpenToClassSpecializationsTab()
        return PlayerSpellsUtil.TogglePlayerSpellsFrame(PlayerSpellsUtil.FrameTabs.ClassSpecializations)
    end
end
"#;

const PLAYERSPELLS_ONLOAD_BACKFILL_LUA: &str = r#"
local function backfill_onload(frame, needs_init)
    if not frame or not needs_init then
        return
    end
    if type(frame.OnLoad) == "function" then
        frame:OnLoad()
        return
    end
    local handler = nil
    if type(frame.GetScript) == "function" then
        handler = frame:GetScript("OnLoad")
    end
    if type(handler) == "function" then
        handler(frame)
    end
end

local function backfill_pvp_talent_slot_setup(frame)
    if not frame then
        return
    end

    if type(frame.ReleaseAllCameras) == "function" and type(frame.cameras) ~= "table" then
        if type(frame.OnLoad) == "function" then
            frame:OnLoad()
        else
            frame.cameras = {}
            frame.tagToCamera = {}
            frame.tagToActor = {}
        end
    end

    if type(frame.Slots) == "table" then
        for slotIndex, slot in ipairs(frame.Slots) do
            if slot and slot.slotIndex == nil and type(slot.SetUp) == "function" then
                slot:SetUp(slotIndex)
            end
        end
    end

    if type(frame.GetChildren) == "function" then
        local children = { frame:GetChildren() }
        for _, child in ipairs(children) do
            backfill_pvp_talent_slot_setup(child)
        end
    end
end

local function backfill_playerspells_tab(frame_tab)
    if not PlayerSpellsFrame or not PlayerSpellsUtil or not PlayerSpellsUtil.FrameTabs then
        return
    end

    if frame_tab == PlayerSpellsUtil.FrameTabs.ClassSpecializations then
        backfill_onload(
            PlayerSpellsFrame.SpecFrame,
            PlayerSpellsFrame.SpecFrame and PlayerSpellsFrame.SpecFrame.SpecContentFramePool == nil
        )
    elseif frame_tab == PlayerSpellsUtil.FrameTabs.ClassTalents then
        backfill_onload(
            PlayerSpellsFrame.TalentsFrame,
            PlayerSpellsFrame.TalentsFrame and PlayerSpellsFrame.TalentsFrame.initialBasePanOffsetX == nil
        )
        backfill_pvp_talent_slot_setup(PlayerSpellsFrame.TalentsFrame)
    elseif frame_tab == PlayerSpellsUtil.FrameTabs.SpellBook then
        backfill_onload(
            PlayerSpellsFrame.SpellBookFrame,
            PlayerSpellsFrame.SpellBookFrame and PlayerSpellsFrame.SpellBookFrame.internalTabTracker == nil
        )
    end
end

if PlayerSpellsFrame then
    if type(PlayerSpellsFrame.IsInspecting) ~= "function"
        and type(PlayerSpellsFrameMixin) == "table"
        and type(Mixin) == "function"
    then
        Mixin(PlayerSpellsFrame, PlayerSpellsFrameMixin)
    end

    backfill_onload(PlayerSpellsFrame, PlayerSpellsFrame.internalTabTracker == nil)
    if not __wow_uisim_playerspells_backfill_wrapped then
        __wow_uisim_playerspells_backfill_wrapped = true

        if type(PlayerSpellsFrame.TrySetTab) == "function" then
            local original_try_set_tab = PlayerSpellsFrame.TrySetTab
            PlayerSpellsFrame.TrySetTab = function(self, frame_tab)
                backfill_playerspells_tab(frame_tab)
                return original_try_set_tab(self, frame_tab)
            end
        end

        if type(PlayerSpellsFrame.SetInspecting) == "function" then
            local original_set_inspecting = PlayerSpellsFrame.SetInspecting
            PlayerSpellsFrame.SetInspecting = function(self, inspect_unit, inspect_string, inspect_string_level)
                if inspect_unit or inspect_string then
                    backfill_playerspells_tab(PlayerSpellsUtil.FrameTabs.ClassTalents)
                end
                return original_set_inspecting(self, inspect_unit, inspect_string, inspect_string_level)
            end
        end
    end
end
"#;

fn profile_has_original_player_spells_util_owner() -> bool {
    matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    )
}

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PLAYER_SPELLS_UTIL_BOOTSTRAP_LUA)?;
    if !profile_has_original_player_spells_util_owner() {
        lua.exec(PLAYER_SPELLS_UTIL_OPEN_TAB_FALLBACK_LUA)?;
    }
    Ok(())
}

pub(crate) fn patch(env: &LoaderEnv<'_>) -> Result<(), crate::Error> {
    env.exec(PLAYERSPELLS_ONLOAD_BACKFILL_LUA)
}

#[cfg(test)]
fn patch_env(env: &WowLuaEnv) {
    env.exec(PLAYERSPELLS_ONLOAD_BACKFILL_LUA)
        .expect("PlayerSpells OnLoad backfill should install");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_playerspells_util_bootstrap_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("PlayerSpellsUtil = nil; TogglePlayerSpellsFrame = nil")
            .expect("fixture should clear PlayerSpellsUtil");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("PlayerSpellsUtil bootstrap should apply");
        }

        let has_open_talents_fallback: bool = env
            .eval("return type(PlayerSpellsUtil.OpenToClassTalentsTab) == 'function'")
            .expect("class-talent open fallback availability probe should run");
        assert_eq!(
            has_open_talents_fallback,
            !super::profile_has_original_player_spells_util_owner()
        );
        let has_open_specializations_fallback: bool = env
            .eval("return type(PlayerSpellsUtil.OpenToClassSpecializationsTab) == 'function'")
            .expect("specialization open fallback availability probe should run");
        assert_eq!(
            has_open_specializations_fallback,
            !super::profile_has_original_player_spells_util_owner()
        );

        let result: String = env
            .eval(
                r#"
                if PlayerSpellsUtil.FrameTabs.ClassSpecializations ~= 1 then return "spec_tab" end
                if PlayerSpellsUtil.FrameTabs.ClassTalents ~= 2 then return "talents_tab" end
                if PlayerSpellsUtil.FrameTabs.SpellBook ~= 3 then return "spellbook_tab" end
                if PlayerSpellsUtil.SpellBookCategories.Class ~= 1 then return "class_category" end
                if PlayerSpellsUtil.SpellBookCategories.General ~= 2 then return "general_category" end
                if PlayerSpellsUtil.SpellBookCategories.Pet ~= 3 then return "pet_category" end
                if type(PlayerSpellsUtil.GetCurrentTabID) ~= "function" then return "current_tab" end
                if type(PlayerSpellsUtil.ToggleClassTalentFrame) ~= "function" then return "talent_toggle" end
                if type(PlayerSpellsUtil.OpenToSpellBookTabAtSpell) ~= "function" then return "spell_at" end
                if type(PlayerSpellsFrame_LoadUI) ~= "function" then return "load_ui" end
                if type(TogglePlayerSpellsFrame) ~= "function" then return "global_toggle" end
                return "ok"
                "#,
            )
            .expect("PlayerSpellsUtil bootstrap shape probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn playerspells_util_bootstrap_preserves_existing_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            PlayerSpellsUtil = {
                FrameTabs = { ClassTalents = "talents" },
                TogglePlayerSpellsFrame = function(tab, inspectUnit)
                    return tab, inspectUnit
                end,
            }
            "#,
        )
        .expect("fixture should install existing PlayerSpellsUtil members");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("PlayerSpellsUtil bootstrap should apply");
        }

        let result: String = env
            .eval(
                r#"
                if PlayerSpellsUtil.FrameTabs.ClassTalents ~= "talents" then return "existing_tab" end
                if PlayerSpellsUtil.FrameTabs.SpellBook ~= 3 then return "filled_tab" end
                if PlayerSpellsUtil.SpellBookCategories.Pet ~= 3 then return "category" end
                local tab, unit = PlayerSpellsUtil.ToggleClassTalentFrame("target")
                if tab ~= "talents" or unit ~= "target" then return "toggle" end
                return "ok"
                "#,
            )
            .expect("PlayerSpellsUtil preservation probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn backfills_player_spells_frame_on_install() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_playerspells_fixture(&env);

        patch_env(&env);

        let internal_tab_tracker: String = env
            .eval("return PlayerSpellsFrame.internalTabTracker")
            .expect("PlayerSpellsFrame OnLoad should run");

        assert_eq!(internal_tab_tracker, "loaded");
    }

    #[test]
    fn wraps_try_set_tab_to_backfill_spellbook_tab() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_playerspells_fixture(&env);

        patch_env(&env);

        let (spellbook_tracker, selected_tab): (String, String) = env
            .eval(
                r#"
                PlayerSpellsFrame:TrySetTab(PlayerSpellsUtil.FrameTabs.SpellBook)
                return PlayerSpellsFrame.SpellBookFrame.internalTabTracker,
                    PlayerSpellsFrame.selectedTab
                "#,
            )
            .expect("wrapped TrySetTab should run");

        assert_eq!(spellbook_tracker, "loaded");
        assert_eq!(selected_tab, "SpellBook");
    }

    #[test]
    fn wraps_set_inspecting_to_backfill_talents_tab() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_playerspells_fixture(&env);

        patch_env(&env);

        let (talents_pan_offset, inspecting_unit): (i64, String) = env
            .eval(
                r#"
                PlayerSpellsFrame:SetInspecting("target")
                return PlayerSpellsFrame.TalentsFrame.initialBasePanOffsetX,
                    PlayerSpellsFrame.inspectingUnit
                "#,
            )
            .expect("wrapped SetInspecting should run");

        assert_eq!(talents_pan_offset, 10);
        assert_eq!(inspecting_unit, "target");
    }

    #[test]
    fn ignores_missing_player_spells_frame() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        patch_env(&env);

        let player_spells_frame_type: String = env
            .eval("return type(PlayerSpellsFrame)")
            .expect("missing PlayerSpellsFrame should be untouched");

        assert_eq!(player_spells_frame_type, "nil");
    }

    fn install_playerspells_fixture(env: &WowLuaEnv) {
        install_playerspells_util_fixture(env);
        install_playerspells_frame_fixture(env);
    }

    fn install_playerspells_util_fixture(env: &WowLuaEnv) {
        env.exec(
            r#"
            PlayerSpellsUtil = {
                FrameTabs = {
                    ClassSpecializations = "ClassSpecializations",
                    ClassTalents = "ClassTalents",
                    SpellBook = "SpellBook",
                },
            }
            "#,
        )
        .expect("PlayerSpellsUtil fixture should install");
    }

    fn install_playerspells_frame_fixture(env: &WowLuaEnv) {
        env.exec(
            r#"
            PlayerSpellsFrame = {
                SpecFrame = {
                    OnLoad = function(self)
                        self.SpecContentFramePool = "loaded"
                    end,
                },
                TalentsFrame = {
                    OnLoad = function(self)
                        self.initialBasePanOffsetX = 10
                    end,
                },
                SpellBookFrame = {
                    OnLoad = function(self)
                        self.internalTabTracker = "loaded"
                    end,
                },
                OnLoad = function(self)
                    self.internalTabTracker = "loaded"
                end,
                TrySetTab = function(self, frameTab)
                    self.selectedTab = frameTab
                end,
                SetInspecting = function(self, inspectUnit)
                    self.inspectingUnit = inspectUnit
                end,
            }
            "#,
        )
        .expect("PlayerSpellsFrame fixture should install");
    }
}
