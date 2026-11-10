//! Temporary misc global frame defaults.
//!
//! These are shallow startup placeholders for global frames whose real behavior
//! is owned by Blizzard addons. Keep them outside the generic runtime bootstrap
//! so their workaround status remains visible.

const MISC_GLOBAL_FRAME_DEFAULTS_LUA: &str = r#"
local function ensure_named_frame(widgetType, name, parent)
    local existing = rawget(_G, name)
    if existing ~= nil then
        return existing
    end
    if type(CreateFrame) ~= "function" then
        return nil
    end
    local frame = CreateFrame(widgetType or "Frame", name, parent)
    rawset(_G, name, frame)
    return frame
end

local function ensure_child_frame(parent, key)
    if type(parent) ~= "table" then
        return nil
    end
    local child = rawget(parent, key)
    if child ~= nil then
        return child
    end
    child = CreateFrame("Frame", nil, parent)
    rawset(parent, key, child)
    return child
end

local function ensure_game_menu_button_pool(gameMenu)
    if gameMenu == nil
        or gameMenu.buttonPool ~= nil
        or type(CreateFramePool) ~= "function" then
        return
    end

    local buttonPool = CreateFramePool("Button", gameMenu)
    local function ensure_button_text(text)
        local button = buttonPool:Acquire()
        if type(button.SetText) == "function" then
            button:SetText(text)
        end
        if type(button.Show) == "function" then
            button:Show()
        end
    end

    ensure_button_text(GAMEMENU_OPTIONS or "Options")
    ensure_button_text(LOGOUT or "Logout")
    gameMenu.buttonPool = buttonPool
end

local gameMenu = ensure_named_frame("Frame", "GameMenuFrame", UIParent)
if type(gameMenu) == "table" then
    if type(gameMenu.Hide) == "function" then
        gameMenu:Hide()
    end
    ensure_game_menu_button_pool(gameMenu)
end

local objective = rawget(_G, "ObjectiveTrackerFrame")
if objective ~= nil and rawget(objective, "OnAdded") == nil then
    function objective:OnAdded(backgroundAlpha)
        if not self.init then
            self.init = true
            if type(ObjectiveTrackerContainerMixin) == "table" and type(ObjectiveTrackerContainerMixin.Init) == "function" then
                ObjectiveTrackerContainerMixin.Init(self)
            elseif self.Header and self.Header.Text and type(self.Header.Text.SetText) == "function" then
                self.Header.Text:SetText(self.headerText or "")
            end
        end
        if type(self.SetBackgroundAlpha) == "function" then
            self:SetBackgroundAlpha(backgroundAlpha)
        end
    end
end

local objectiveHeader = ensure_child_frame(objective, "Header")
ensure_child_frame(objectiveHeader, "MinimizeButton")
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(MISC_GLOBAL_FRAME_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_misc_global_frame_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(GameMenuFrame) ~= "table" then return "game_menu" end
                if GameMenuFrame:IsVisible() then return "game_menu_visible" end
                if type(CreateFramePool) == "function" then
                    if GameMenuFrame.buttonPool == nil then return "button_pool" end
                    if type(GameMenuFrame.buttonPool.EnumerateActive) ~= "function" then return "button_pool_iter" end

                    local buttonCount = 0
                    for button in GameMenuFrame.buttonPool:EnumerateActive() do
                        buttonCount = buttonCount + 1
                        if button:GetText() == nil or button:GetText() == "" then
                            return "button_text"
                        end
                    end
                    if buttonCount == 0 then return "buttons" end
                end

                if ObjectiveTrackerFrame == nil then return "objective_absent" end
                if type(ObjectiveTrackerFrame) ~= "table" then return "objective" end
                if type(ObjectiveTrackerFrame.Header) ~= "table" then return "header" end
                if type(ObjectiveTrackerFrame.Header.MinimizeButton) ~= "table" then return "minimize" end
                return "ok"
                "#,
            )
            .expect("misc global frame defaults probe should run");

        let expects_tracker = !matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Era
                | crate::client_profile::ClientProfile::Anniversary
        );
        let expected_result = if expects_tracker {
            "ok"
        } else {
            "objective_absent"
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn installs_objective_tracker_on_added_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            ObjectiveTrackerFrame = ObjectiveTrackerFrame or CreateFrame("Frame", "ObjectiveTrackerFrame", UIParent)
            ObjectiveTrackerFrame.Header = ObjectiveTrackerFrame.Header or CreateFrame("Frame", nil, ObjectiveTrackerFrame)
            ObjectiveTrackerFrame.OnAdded = nil
            ObjectiveTrackerFrame.init = nil
            ObjectiveTrackerFrame.headerText = "Tracked"
            ObjectiveTrackerFrame.backgroundAlpha = nil
            ObjectiveTrackerFrame.SetBackgroundAlpha = function(self, alpha)
                self.backgroundAlpha = alpha
            end
            ObjectiveTrackerFrame.Header.Text = {
                text = nil,
                SetText = function(self, text)
                    self.text = text
                end,
            }
            ObjectiveTrackerContainerMixin = {
                initialized = false,
                Init = function(self)
                    ObjectiveTrackerContainerMixin.initialized = self == ObjectiveTrackerFrame
                end,
            }
            "#,
        )
        .expect("fixture should clear objective tracker OnAdded");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("misc global frame defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                if type(ObjectiveTrackerFrame.OnAdded) ~= "function" then return "missing" end
                ObjectiveTrackerFrame:OnAdded(0.25)
                if ObjectiveTrackerFrame.init ~= true then return "init_flag" end
                if ObjectiveTrackerContainerMixin.initialized ~= true then return "mixin_init" end
                if ObjectiveTrackerFrame.backgroundAlpha ~= 0.25 then return "background_alpha" end
                return "ok"
                "#,
            )
            .expect("objective tracker OnAdded probe should run");

        assert_eq!(result, "ok");
    }
}
