//! Temporary character frame surface refresh workaround.
//!
//! The character panel needs title and paper-doll slot refreshes after the
//! Blizzard display hooks run. Keep this bridge isolated until those surfaces
//! are driven by the normal frame lifecycle. Retail title selection itself is
//! owned by original CharacterFrameMixin:UpdateTitle, never forced to player name.

use crate::lua_api::WowLuaEnv;

const CHARACTER_FRAME_SURFACE_REFRESH_WORKAROUND_LUA: &str = r#"
local function get_character_panel_slot_buttons()
    local slotFrameNames = {
        "CharacterHeadSlot",
        "CharacterNeckSlot",
        "CharacterShoulderSlot",
        "CharacterBackSlot",
        "CharacterChestSlot",
        "CharacterShirtSlot",
        "CharacterTabardSlot",
        "CharacterWristSlot",
        "CharacterHandsSlot",
        "CharacterWaistSlot",
        "CharacterLegsSlot",
        "CharacterFeetSlot",
        "CharacterFinger0Slot",
        "CharacterFinger1Slot",
        "CharacterTrinket0Slot",
        "CharacterTrinket1Slot",
        "CharacterMainHandSlot",
        "CharacterSecondaryHandSlot",
    }
    local buttons = {}
    for _, frameName in ipairs(slotFrameNames) do
        local button = _G[frameName]
        if type(button) == "table" then
            table.insert(buttons, button)
        end
    end
    return buttons
end

if type(CharacterFrame) == "table"
    and type(CharacterFrame.GetScript) == "function"
    and type(CharacterFrame.SetScript) == "function" then
    local existing_wrapper = rawget(_G, "__wow_character_frame_onshow_wrapper")
    if CharacterFrame:GetScript("OnShow") ~= existing_wrapper then
        local original_on_show = CharacterFrame:GetScript("OnShow")
        if type(original_on_show) ~= "function" then
            return
        end
        local wrapper = function(self, ...)
            original_on_show(self, ...)
            if type(self.UpdateTitle) == "function" then
                self:UpdateTitle()
            end
            if type(PaperDollItemSlotButton_Update) == "function" then
                for _, button in ipairs(get_character_panel_slot_buttons()) do
                    PaperDollItemSlotButton_Update(button)
                end
            end
        end
        CharacterFrame:SetScript("OnShow", wrapper)
        rawset(_G, "__wow_character_frame_onshow_wrapper", wrapper)
    end
end

if type(CharacterFrame) == "table" and type(CharacterFrame.RefreshDisplay) == "function" then
    local existing_wrapper = rawget(_G, "__wow_character_frame_refresh_display_wrapper")
    if CharacterFrame.RefreshDisplay ~= existing_wrapper then
        local original_refresh_display = CharacterFrame.RefreshDisplay
        local wrapper = function(self, ...)
            original_refresh_display(self, ...)
            if type(self.UpdateTitle) == "function" then
                self:UpdateTitle()
            end
            if type(PaperDollItemSlotButton_Update) == "function" then
                for _, button in ipairs(get_character_panel_slot_buttons()) do
                    PaperDollItemSlotButton_Update(button)
                end
            end
        end
        CharacterFrame.RefreshDisplay = wrapper
        rawset(_G, "__wow_character_frame_refresh_display_wrapper", wrapper)
    end
end

if type(CharacterFrame) == "table" then
    if type(CharacterFrame.RefreshDisplay) == "function" then
        CharacterFrame:RefreshDisplay()
    elseif type(CharacterFrame.UpdateTitle) == "function" then
        CharacterFrame:UpdateTitle()
    end
end

if type(PaperDollItemSlotButton_Update) == "function" then
    for _, button in ipairs(get_character_panel_slot_buttons()) do
        PaperDollItemSlotButton_Update(button)
    end
end

if not preserveOriginalCharacterTitle
    and type(CharacterFrame) == "table"
    and CharacterFrame.TitleContainer
    and CharacterFrame.TitleContainer.TitleText
    and type(CharacterFrame.TitleContainer.TitleText.SetText) == "function" then
    CharacterFrame.TitleContainer.TitleText:SetText(UnitPVPName("player"))
end

for _, button in ipairs(get_character_panel_slot_buttons()) do
    if type(button.icon) == "table" then
        local textureName = GetInventoryItemTexture("player", button:GetID())
        if textureName ~= nil then
            button.icon:SetTexture(textureName)
        elseif button.backgroundTextureName ~= nil then
            button.icon:SetTexture(button.backgroundTextureName)
        end
    end
end
"#;

pub(crate) fn patch(env: &WowLuaEnv) {
    // A chunk-local policy preserves non-Retail ordering and error behavior;
    // no public global or Blizzard source is changed.
    let preserve_original_title = cfg!(feature = "client-retail");
    let source = format!(
        "local preserveOriginalCharacterTitle = {preserve_original_title}\n{CHARACTER_FRAME_SURFACE_REFRESH_WORKAROUND_LUA}"
    );
    let _ = env.exec(&source);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refreshes_title_and_character_slot_icons() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            local scripts = {
                OnShow = function(self)
                    self.originalOnShowCount = (self.originalOnShowCount or 0) + 1
                end,
            }
            titleText = {
                SetText = function(self, text)
                    self.text = text
                end,
            }
            CharacterFrame = {
                TitleContainer = { TitleText = titleText },
                GetScript = function(self, event)
                    return scripts[event]
                end,
                SetScript = function(self, event, script)
                    scripts[event] = script
                end,
                UpdateTitle = function(self)
                    self.updateTitleCount = (self.updateTitleCount or 0) + 1
                    self.TitleContainer.TitleText:SetText("original-title")
                end,
                RefreshDisplay = function(self)
                    self.refreshCount = (self.refreshCount or 0) + 1
                end,
            }
            CharacterHeadSlot = {
                id = 1,
                backgroundTextureName = "fallback-head",
                GetID = function(self)
                    return self.id
                end,
                icon = {
                    SetTexture = function(self, texture)
                        self.texture = texture
                    end,
                },
            }
            CharacterNeckSlot = {
                id = 2,
                backgroundTextureName = "fallback-neck",
                GetID = function(self)
                    return self.id
                end,
                icon = {
                    SetTexture = function(self, texture)
                        self.texture = texture
                    end,
                },
            }
            slotUpdates = 0
            function PaperDollItemSlotButton_Update()
                slotUpdates = slotUpdates + 1
            end
            function UnitPVPName()
                return "Calia"
            end
            function GetInventoryItemTexture(unit, slot_id)
                if slot_id == 1 then
                    return "equipped-head"
                end
                return nil
            end
            "#,
        )
        .expect("character frame test surface should install");

        patch(&env);

        let (refresh_count, title_count, title_text, slot_updates, head_texture, neck_texture): (
            i64,
            i64,
            String,
            i64,
            String,
            String,
        ) = env
            .eval(
                r#"
                return CharacterFrame.refreshCount,
                    CharacterFrame.updateTitleCount,
                    titleText.text,
                    slotUpdates,
                    CharacterHeadSlot.icon.texture,
                    CharacterNeckSlot.icon.texture
                "#,
            )
            .expect("patched character frame state should be readable");

        assert_eq!(refresh_count, 1);
        assert_eq!(title_count, 1);
        let expected_title = if cfg!(feature = "client-retail") {
            "original-title"
        } else {
            "Calia"
        };
        assert_eq!(title_text, expected_title);
        assert_eq!(slot_updates, 4);
        assert_eq!(head_texture, "equipped-head");
        assert_eq!(neck_texture, "fallback-neck");
    }

    #[test]
    fn on_show_wrapper_runs_original_then_refreshes_surface() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            local scripts = {
                OnShow = function(self)
                    self.originalOnShowCount = (self.originalOnShowCount or 0) + 1
                end,
            }
            CharacterFrame = {
                GetScript = function(self, event)
                    return scripts[event]
                end,
                SetScript = function(self, event, script)
                    scripts[event] = script
                end,
                UpdateTitle = function(self)
                    self.updateTitleCount = (self.updateTitleCount or 0) + 1
                end,
            }
            function PaperDollItemSlotButton_Update() end
            function UnitPVPName()
                return "Calia"
            end
            function GetInventoryItemTexture()
                return nil
            end
            "#,
        )
        .expect("character onshow test surface should install");

        patch(&env);

        let (original_count, title_count): (i64, i64) = env
            .eval(
                r#"
                CharacterFrame:GetScript("OnShow")(CharacterFrame)
                return CharacterFrame.originalOnShowCount,
                    CharacterFrame.updateTitleCount
                "#,
            )
            .expect("patched character onshow state should be readable");

        assert_eq!(original_count, 1);
        assert_eq!(title_count, 2);
    }

    #[test]
    fn patch_does_not_create_character_subframe_list() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("CHARACTERFRAME_SUBFRAMES = nil")
            .expect("character subframes fixture should install");

        patch(&env);

        let exists: bool = env
            .eval("return CHARACTERFRAME_SUBFRAMES ~= nil")
            .expect("character subframes probe should run");

        assert!(
            !exists,
            "Character frame refresh patch must not synthesize CharacterFrame.lua globals"
        );
    }
}
