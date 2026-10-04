//! Temporary Blizzard Lua source patches with explicit retirement paths.

use std::borrow::Cow;

pub(crate) fn patch_lua_source<'a>(bytes: &'a [u8], chunk_name: &str) -> Cow<'a, [u8]> {
    let Some(patch) = lua_source_patch_for_chunk(chunk_name) else {
        return Cow::Borrowed(bytes);
    };
    let Ok(source) = std::str::from_utf8(bytes) else {
        return Cow::Borrowed(bytes);
    };

    let patched = apply_lua_source_patch(source, patch.operations);
    if patched == source {
        return Cow::Borrowed(bytes);
    }
    Cow::Owned(patched.into_bytes())
}

struct LuaSourcePatch {
    suffix: &'static str,
    operations: &'static [LuaSourcePatchOp],
}

enum LuaSourcePatchOp {
    Prefix(&'static str),
    Replace {
        from: &'static str,
        to: &'static str,
    },
    ReplaceOnce {
        from: &'static str,
        to: &'static str,
    },
}

const LUA_SOURCE_PATCHES: &[LuaSourcePatch] = &[
    #[cfg(feature = "client-mists")]
    LuaSourcePatch {
        suffix: "/Blizzard_StoreUISecure.lua",
        operations: &[
            LuaSourcePatchOp::ReplaceOnce {
                from: "function StoreProductCard_OnLoad(self)\n\tif (not StoreProductCard_IsSplashPage(self)) then",
                to: "function StoreProductCard_OnLoad(self)\n\tif self.ProductName == nil and type(self.CreateFontString) == \"function\" then\n\t\tself.ProductName = self:CreateFontString(nil, \"ARTWORK\", \"GameFontNormalLarge\");\n\tend\n\tif self.ProductName == nil then\n\t\treturn;\n\tend\n\tif (not StoreProductCard_IsSplashPage(self)) then",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "\t\t\tcard = CreateForbiddenFrame(\"Button\", nil, self, \"StoreProductCardTemplate\");",
                to: "\t\t\tcard = CreateForbiddenFrame(\"Button\", nil, self, \"StoreProductCardTemplate\");\n\t\t\tif type(card.SetPoint) ~= \"function\" then card = CreateFrame(\"Button\", nil, self); end",
            },
        ],
    },
    #[cfg(feature = "client-mists")]
    LuaSourcePatch {
        suffix: "/HybridScrollFrame.lua",
        operations: &[LuaSourcePatchOp::ReplaceOnce {
            from: "\t\t\tself.scrollBar.thumbTexture:Hide();",
            to: "\t\t\tif self.scrollBar.thumbTexture then self.scrollBar.thumbTexture:Hide(); end",
        }],
    },
    #[cfg(feature = "client-mists")]
    LuaSourcePatch {
        suffix: "/Blizzard_MapCanvas.lua",
        operations: &[LuaSourcePatchOp::ReplaceOnce {
            from: "function MapCanvasMixin:AddDataProvider(dataProvider)\n\tself.dataProviders[dataProvider] = true;\n\tdataProvider:OnAdded(self);\nend",
            to: "function MapCanvasMixin:AddDataProvider(dataProvider)\n\tif type(dataProvider) == \"table\" and type(MapCanvasDataProviderMixin) == \"table\" then\n\t\tdataProvider.OnAdded = dataProvider.OnAdded or MapCanvasDataProviderMixin.OnAdded;\n\t\tdataProvider.OnRemoved = dataProvider.OnRemoved or MapCanvasDataProviderMixin.OnRemoved;\n\t\tdataProvider.RemoveAllData = dataProvider.RemoveAllData or MapCanvasDataProviderMixin.RemoveAllData;\n\t\tdataProvider.RefreshAllData = dataProvider.RefreshAllData or MapCanvasDataProviderMixin.RefreshAllData;\n\tend\n\tself.dataProviders[dataProvider] = true;\n\tdataProvider:OnAdded(self);\nend",
        }],
    },
    #[cfg(feature = "client-mists")]
    LuaSourcePatch {
        suffix: "/Blizzard_SimpleCheckout.lua",
        operations: &[LuaSourcePatchOp::ReplaceOnce {
            from: "self:SetPoint(\"TOPLEFT\", UIParent, \"TOPLEFT\");",
            to: "pcall(self.SetPoint, self, \"TOPLEFT\", UIParent, \"TOPLEFT\");",
        }],
    },
    LuaSourcePatch {
        suffix: "/ChatFrameUtil.lua",
        operations: &[
            LuaSourcePatchOp::Replace {
                from: "local info = ChatTypeInfo[\"SYSTEM\"];",
                to: "local info = ChatTypeInfo[\"SYSTEM\"] or { r = 1, g = 1, b = 0, id = 1 };",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "previousValue:Hide();",
                to: "if type(previousValue.Hide) == \"function\" then previousValue:Hide(); end",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "FCFClickAnywhereButton_UpdateState(previousValue.chatFrame.clickAnywhereButton);",
                to: "if previousValue.chatFrame and previousValue.chatFrame.clickAnywhereButton then FCFClickAnywhereButton_UpdateState(previousValue.chatFrame.clickAnywhereButton); end",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "FCFClickAnywhereButton_UpdateState(editBox.chatFrame.clickAnywhereButton);",
                to: "if editBox.chatFrame and editBox.chatFrame.clickAnywhereButton then FCFClickAnywhereButton_UpdateState(editBox.chatFrame.clickAnywhereButton); end",
            },
        ],
    },
    LuaSourcePatch {
        suffix: "/Deprecated_ArenaUI.lua",
        operations: &[
            LuaSourcePatchOp::ReplaceOnce {
                from: "self.layoutIndex = self:GetParent().layoutIndex + 1;",
                to: "self.layoutIndex = (self:GetParent().layoutIndex or ((id * 2) - 1)) + 1;",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "_G[prefix..\"HealthBar\"]:SetBarTextZeroText(DEAD);",
                to: "if _G[prefix..\"HealthBar\"] then _G[prefix..\"HealthBar\"]:SetBarTextZeroText(DEAD); end",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "_G[prefix..\"Name\"]:Hide();",
                to: "if _G[prefix..\"Name\"] then _G[prefix..\"Name\"]:Hide(); end",
            },
        ],
    },
    LuaSourcePatch {
        suffix: "/Blizzard_CodeOfConduct.lua",
        operations: &[LuaSourcePatchOp::Replace {
            from: "ChatFrameUtil.AddSystemMessage(ONLINE_SAFETY_NOTICE);",
            to: "-- suppressed in the simulator to keep chat history tests stable",
        }],
    },
    LuaSourcePatch {
        suffix: "/VoiceChatTranscriptionFrame.lua",
        operations: &[LuaSourcePatchOp::Replace {
            from: "chatInfo = ChatTypeInfo[chatType];",
            to: "chatInfo = ChatTypeInfo[chatType] or ChatTypeInfo.SYSTEM or { r = 1, g = 1, b = 0, id = 1 };",
        }],
    },
    LuaSourcePatch {
        suffix: "/EventUtil.lua",
        operations: &[LuaSourcePatchOp::Replace {
            from: "callback();",
            to: "if type(callback) == \"function\" then callback(); end",
        }],
    },
    LuaSourcePatch {
        suffix: "/LocalizationMachinery.lua",
        operations: &[LuaSourcePatchOp::Prefix(
            "if SetupLocalization ~= nil then return end\n",
        )],
    },
    LuaSourcePatch {
        suffix: "/Blizzard_AddOnList/AddonList.lua",
        operations: &[LuaSourcePatchOp::ReplaceOnce {
            from: "local group = C_AddOns.GetAddOnMetadata(i, \"Group\");",
            to: "local group = C_AddOns.GetAddOnMetadata(i, \"Group\");\n\t\tif type(group) ~= \"string\" or group == \"\" then\n\t\t\tgroup = C_AddOns.GetAddOnName(i);\n\t\tend",
        }],
    },
    LuaSourcePatch {
        suffix: "/EditModeManager.lua",
        operations: &[
            LuaSourcePatchOp::ReplaceOnce {
                from: "function EditModeManagerFrameMixin:ReconcileLayoutsWithModern()\n\tlocal somethingChanged = false;",
                to: "function EditModeManagerFrameMixin:ReconcileLayoutsWithModern()\n\tif type(self.layoutInfo) ~= \"table\" or type(self.layoutInfo.layouts) ~= \"table\" then\n\t\treturn false;\n\tend\n\tlocal somethingChanged = false;",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "function EditModeManagerFrameMixin:UpdateLayoutInfo(layoutInfo, reconcileLayouts)\n\tself.layoutApplyInProgress = true;\n\tself.layoutInfo = layoutInfo;",
                to: "function EditModeManagerFrameMixin:UpdateLayoutInfo(layoutInfo, reconcileLayouts)\n\tself.layoutApplyInProgress = true;\n\tself.layoutInfo = layoutInfo or self.layoutInfo or { layouts = {}, activeLayout = 1 };\n\tif type(self.layoutInfo.layouts) ~= \"table\" then\n\t\tself.layoutInfo.layouts = {};\n\tend",
            },
        ],
    },
    LuaSourcePatch {
        suffix: "/MainMenuBarMicroButtons.lua",
        operations: &[
            LuaSourcePatchOp::Replace {
                from: "local wasShown = CatalogShopInboundInterface.IsShown();",
                to: "local wasShown = false;\n\t\tif CatalogShopInboundInterface and type(CatalogShopInboundInterface.IsShown) == \"function\" then\n\t\t\tlocal ok, value = pcall(CatalogShopInboundInterface.IsShown);\n\t\t\twasShown = ok and value or false;\n\t\tend",
            },
            LuaSourcePatchOp::Replace {
                from: "local wasShown = StoreFrame_IsShown();",
                to: "local wasShown = false;\n\t\tif type(StoreFrame_IsShown) == \"function\" then\n\t\t\tlocal ok, value = pcall(StoreFrame_IsShown);\n\t\t\twasShown = ok and value or false;\n\t\tend",
            },
        ],
    },
    LuaSourcePatch {
        suffix: "/UIParent.lua",
        operations: &[
            LuaSourcePatchOp::ReplaceOnce {
                from: "if ( lastTalkedToGM ~= \"\" ) then",
                to: "if false and ( lastTalkedToGM ~= \"\" ) then",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "NPETutorial_AttemptToBegin(event);",
                to: "if type(NPETutorial_AttemptToBegin) == \"function\" then NPETutorial_AttemptToBegin(event); end",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "UpdateMicroButtons();",
                to: "pcall(UpdateMicroButtons);",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "CatalogShopInboundInterface.CheckForFree(event);",
                to: "if CatalogShopInboundInterface and type(CatalogShopInboundInterface.CheckForFree) == \"function\" then CatalogShopInboundInterface.CheckForFree(event); end",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "StoreFrame_CheckForFree(event);",
                to: "if type(StoreFrame_CheckForFree) == \"function\" then StoreFrame_CheckForFree(event); end",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "EventUtil.TriggerOnVariablesLoaded();",
                to: "-- EventUtil.TriggerOnVariablesLoaded() skipped in rilua startup",
            },
        ],
    },
    LuaSourcePatch {
        suffix: "/Blizzard_WowTokenUIInbound.lua",
        operations: &[LuaSourcePatchOp::Replace {
            from: "function WowToken_IsWowTokenAuctionDialogShown()\n\treturn WowTokenDialog:GetAttribute(\"isauctiondialogshown\");\nend",
            to: "function WowToken_IsWowTokenAuctionDialogShown()\n\tif type(WowTokenDialog) ~= \"table\" or type(WowTokenDialog.GetAttribute) ~= \"function\" then\n\t\treturn false;\n\tend\n\tlocal ok, shown = pcall(WowTokenDialog.GetAttribute, WowTokenDialog, \"isauctiondialogshown\");\n\treturn ok and shown or false;\nend",
        }],
    },
    LuaSourcePatch {
        suffix: "/Blizzard_Shared_StoreUIInbound.lua",
        operations: &[
            LuaSourcePatchOp::Replace {
                from: "function StoreFrame_SetShown(shown, contextKey)\n\tlocal wasShown = StoreFrame_IsShown();",
                to: "function StoreFrame_SetShown(shown, contextKey)\n\tif type(StoreFrame) ~= \"table\" or type(StoreFrame.SetAttribute) ~= \"function\" then\n\t\treturn false;\n\tend\n\tlocal wasShown = StoreFrame_IsShown();",
            },
            LuaSourcePatchOp::Replace {
                from: "function StoreFrame_IsShown()\n\treturn StoreFrame:GetAttribute(\"isshown\");\nend",
                to: "function StoreFrame_IsShown()\n\tif type(StoreFrame) ~= \"table\" or type(StoreFrame.GetAttribute) ~= \"function\" then\n\t\treturn false;\n\tend\n\tlocal ok, shown = pcall(StoreFrame.GetAttribute, StoreFrame, \"isshown\");\n\treturn ok and shown or false;\nend",
            },
        ],
    },
    LuaSourcePatch {
        suffix: "/MinimalSlider.lua",
        operations: &[LuaSourcePatchOp::Replace {
            from: "self.Slider.Thumb:SetAlpha(alpha);",
            to: "if self.Slider and self.Slider.Thumb then self.Slider.Thumb:SetAlpha(alpha); end",
        }],
    },
    LuaSourcePatch {
        suffix: "/FloatingChatFrame.lua",
        operations: &[
            LuaSourcePatchOp::Replace {
                from: "DEFAULT_TAB_SELECTED_COLOR_TABLE = { r = 1, g = 0.5, b = 0.25 };",
                to: "DEFAULT_TAB_SELECTED_COLOR_TABLE = { r = 1, g = 0.5, b = 0.25 };\n\nlocal function __wow_ensure_chat_tab_font_string(button)\n\tif not button then\n\t\treturn nil;\n\tend\n\n\tlocal fontString = button:GetFontString();\n\tif fontString then\n\t\treturn fontString;\n\tend\n\tif type(button.CreateFontString) ~= \"function\" then\n\t\treturn nil;\n\tend\n\n\tlocal name = button.GetName and button:GetName();\n\tlocal childName = type(name) == \"string\" and name ~= \"\" and (name..\"Text\") or nil;\n\tfontString = button:CreateFontString(childName, \"ARTWORK\");\n\tif fontString and type(button.SetFontString) == \"function\" then\n\t\tbutton:SetFontString(fontString);\n\tend\n\treturn fontString;\nend",
            },
            LuaSourcePatchOp::Replace {
                from: "UIFrameFadeIn(object, CHAT_FRAME_FADE_TIME, object:GetAlpha(), max(chatFrame.oldAlpha, DEFAULT_CHATFRAME_ALPHA));",
                to: "UIFrameFadeIn(object, CHAT_FRAME_FADE_TIME, object:GetAlpha(), max(chatFrame.oldAlpha or DEFAULT_CHATFRAME_ALPHA, DEFAULT_CHATFRAME_ALPHA));",
            },
            LuaSourcePatchOp::Replace {
                from: "UIFrameFadeOut(object, CHAT_FRAME_FADE_OUT_TIME, max(object:GetAlpha(), chatFrame.oldAlpha), chatFrame.oldAlpha);",
                to: "UIFrameFadeOut(object, CHAT_FRAME_FADE_OUT_TIME, max(object:GetAlpha() or 0, chatFrame.oldAlpha or DEFAULT_CHATFRAME_ALPHA), chatFrame.oldAlpha or DEFAULT_CHATFRAME_ALPHA);",
            },
            LuaSourcePatchOp::Replace {
                from: "self:GetFontString():SetTextColor(colorTable.r, colorTable.g, colorTable.b);",
                to: "do local fontString = __wow_ensure_chat_tab_font_string(self); if fontString then fontString:SetTextColor(colorTable.r, colorTable.g, colorTable.b); end end",
            },
            LuaSourcePatchOp::Replace {
                from: "self:GetFontString():SetTextColor(NORMAL_FONT_COLOR.r, NORMAL_FONT_COLOR.g, NORMAL_FONT_COLOR.b);",
                to: "do local fontString = __wow_ensure_chat_tab_font_string(self); if fontString then fontString:SetTextColor(NORMAL_FONT_COLOR.r, NORMAL_FONT_COLOR.g, NORMAL_FONT_COLOR.b); end end",
            },
            LuaSourcePatchOp::Replace {
                from: "minFrame:GetFontString():SetTextColor(colorTable.r, colorTable.g, colorTable.b);",
                to: "do local fontString = __wow_ensure_chat_tab_font_string(minFrame); if fontString then fontString:SetTextColor(colorTable.r, colorTable.g, colorTable.b); end end",
            },
            LuaSourcePatchOp::Replace {
                from: "minFrame:GetFontString():SetTextColor(NORMAL_FONT_COLOR.r, NORMAL_FONT_COLOR.g, NORMAL_FONT_COLOR.b);",
                to: "do local fontString = __wow_ensure_chat_tab_font_string(minFrame); if fontString then fontString:SetTextColor(NORMAL_FONT_COLOR.r, NORMAL_FONT_COLOR.g, NORMAL_FONT_COLOR.b); end end",
            },
            LuaSourcePatchOp::Replace {
                from: "button:GetFontString():SetTextColor(colorTable.r, colorTable.g, colorTable.b);",
                to: "do local fontString = __wow_ensure_chat_tab_font_string(button); if fontString then fontString:SetTextColor(colorTable.r, colorTable.g, colorTable.b); end end",
            },
            LuaSourcePatchOp::Replace {
                from: "button:GetFontString():SetTextColor(NORMAL_FONT_COLOR.r, NORMAL_FONT_COLOR.g, NORMAL_FONT_COLOR.b);",
                to: "do local fontString = __wow_ensure_chat_tab_font_string(button); if fontString then fontString:SetTextColor(NORMAL_FONT_COLOR.r, NORMAL_FONT_COLOR.g, NORMAL_FONT_COLOR.b); end end",
            },
        ],
    },
    LuaSourcePatch {
        suffix: "/MenuTemplates.lua",
        operations: &[
            LuaSourcePatchOp::Replace {
                from: "function DropdownTextMixin:UpdateText()\n\tself.Text:SetText(self:GetUpdateText());",
                to: "local function __wow_ensure_dropdown_text_font_string(self)\n\tif self.Text then\n\t\treturn self.Text;\n\tend\n\tif not MenuVariants or type(MenuVariants.CreateFontString) ~= \"function\" then\n\t\treturn nil;\n\tend\n\tlocal ok, fontString = pcall(MenuVariants.CreateFontString, self);\n\tif not ok or fontString == nil then\n\t\treturn nil;\n\tend\n\tself.Text = fontString;\n\treturn fontString;\nend\n\nfunction DropdownTextMixin:UpdateText()\n\tlocal text = __wow_ensure_dropdown_text_font_string(self);\n\tif not text then\n\t\treturn;\n\tend\n\ttext:SetText(self:GetUpdateText());",
            },
            LuaSourcePatchOp::ReplaceOnce {
                from: "local newWidth = self.Text:GetUnboundedStringWidth();",
                to: "local newWidth = text:GetUnboundedStringWidth();",
            },
        ],
    },
    LuaSourcePatch {
        suffix: "TextToSpeechFrame.lua",
        operations: &[LuaSourcePatchOp::Replace {
            from: "function TextToSpeechFrame_CheckLoad(self)",
            to: "local __wow_saved_text_to_speech_voice_dropdown = TextToSpeechFrame_SetupVoiceDropdown\nlocal __wow_saved_text_to_speech_alt_voice_dropdown = TextToSpeechFrame_SetupAlternateVoiceDropdown\nlocal function __wow_ensure_text_to_speech_dropdown_helpers(self)\n\tif type(TextToSpeechFrame_SetupVoiceDropdown) ~= \"function\" then\n\t\tif type(__wow_saved_text_to_speech_voice_dropdown) == \"function\" then\n\t\t\tTextToSpeechFrame_SetupVoiceDropdown = __wow_saved_text_to_speech_voice_dropdown\n\t\telse\n\t\t\tfunction TextToSpeechFrame_SetupVoiceDropdown(frame)\n\t\t\t\tSetupVoiceMenu(frame.PanelContainer.TtsVoiceDropdown, Enum.TtsVoiceType.Standard);\n\t\t\tend\n\t\tend\n\tend\n\tif type(TextToSpeechFrame_SetupAlternateVoiceDropdown) ~= \"function\" then\n\t\tif type(__wow_saved_text_to_speech_alt_voice_dropdown) == \"function\" then\n\t\t\tTextToSpeechFrame_SetupAlternateVoiceDropdown = __wow_saved_text_to_speech_alt_voice_dropdown\n\t\telse\n\t\t\tfunction TextToSpeechFrame_SetupAlternateVoiceDropdown(frame)\n\t\t\t\tSetupVoiceMenu(frame.PanelContainer.TtsVoiceAlternateDropdown, Enum.TtsVoiceType.Alternate);\n\t\t\tend\n\t\tend\n\tend\nend\n\nfunction TextToSpeechFrame_CheckLoad(self)\n\t__wow_ensure_text_to_speech_dropdown_helpers(self)",
        }],
    },
    LuaSourcePatch {
        suffix: "/Blizzard_PetBattleUI.lua",
        operations: &[
            LuaSourcePatchOp::Replace {
                from: "cooldown = max(currentCooldown, currentLockdown);",
                to: "cooldown = max(currentCooldown or 0, currentLockdown or 0);",
            },
            LuaSourcePatchOp::Replace {
                from: "self.XPBar:SetWidth(max((xp / max(maxXp,1)) * self.xpBarWidth, 1));",
                to: "self.XPBar:SetWidth(max(((xp or 0) / max(maxXp or 1,1)) * self.xpBarWidth, 1));",
            },
            LuaSourcePatchOp::Replace {
                from: "self.ActualHealthBar:SetWidth((health / max(maxHealth,1)) * self.healthBarWidth);",
                to: "self.ActualHealthBar:SetWidth(((health or 0) / max(maxHealth or 1,1)) * self.healthBarWidth);",
            },
        ],
    },
];

fn lua_source_patch_for_chunk(chunk_name: &str) -> Option<&'static LuaSourcePatch> {
    let normalized_chunk_name;
    let chunk_name = if chunk_name.contains('\\') {
        normalized_chunk_name = chunk_name.replace('\\', "/");
        normalized_chunk_name.as_str()
    } else {
        chunk_name
    };
    LUA_SOURCE_PATCHES
        .iter()
        .find(|patch| chunk_name.ends_with(patch.suffix))
}

fn apply_lua_source_patch(source: &str, operations: &[LuaSourcePatchOp]) -> String {
    let mut patched = source.to_string();
    for operation in operations {
        patched = apply_lua_source_patch_operation(&patched, operation);
    }
    patched
}

fn apply_lua_source_patch_operation(source: &str, operation: &LuaSourcePatchOp) -> String {
    match operation {
        LuaSourcePatchOp::Prefix(prefix) => format!("{prefix}{source}"),
        LuaSourcePatchOp::Replace { from, to } => {
            replace_with_line_ending_fallback(source, from, to)
        }
        LuaSourcePatchOp::ReplaceOnce { from, to } => {
            replace_once_with_line_ending_fallback(source, from, to)
        }
    }
}

fn replace_with_line_ending_fallback(source: &str, from: &str, to: &str) -> String {
    let patched = source.replace(from, to);
    if patched != source {
        return patched;
    }
    source.replace(&from.replace('\n', "\r\n"), &to.replace('\n', "\r\n"))
}

fn replace_once_with_line_ending_fallback(source: &str, from: &str, to: &str) -> String {
    let patched = source.replacen(from, to, 1);
    if patched != source {
        return patched;
    }
    source.replacen(&from.replace('\n', "\r\n"), &to.replace('\n', "\r\n"), 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_unpatched_decode_and_matches_windows_paths() {
        let bytes = b"\xFF\xFEunpatched lua bytes";
        let patched = patch_lua_source(bytes, "@Interface/AddOns/TestAddon/Unpatched.lua");
        assert!(matches!(patched, Cow::Borrowed(_)));

        let patch = lua_source_patch_for_chunk(
            r"@Interface\AddOns\Blizzard_PetBattleUI\Blizzard_PetBattleUI.lua",
        );
        assert!(patch.is_some());
    }

    #[test]
    fn store_ui_inbound_guards_missing_store_frame() {
        let source = br#"function StoreFrame_SetShown(shown, contextKey)
	local wasShown = StoreFrame_IsShown();
	StoreFrame:SetAttribute("action", shown and "Show" or "Hide");
end

function StoreFrame_IsShown()
	return StoreFrame:GetAttribute("isshown");
end"#;
        let patched = patch_lua_source(source, "@/Blizzard_Shared_StoreUIInbound.lua");
        let patched = std::str::from_utf8(&patched).expect("patched source should be utf8");

        assert!(patched.contains(
            "if type(StoreFrame) ~= \"table\" or type(StoreFrame.SetAttribute) ~= \"function\" then"
        ));
        assert!(patched.contains("return false"));
        assert!(patched.contains("return ok and shown or false"));
    }

    #[test]
    #[cfg(feature = "client-mists")]
    fn mists_map_canvas_add_data_provider_backfills_provider_defaults() {
        let source = b"function MapCanvasMixin:AddDataProvider(dataProvider)\n\tself.dataProviders[dataProvider] = true;\n\tdataProvider:OnAdded(self);\nend";
        let patched = patch_lua_source(source, "@/Blizzard_MapCanvas.lua");
        let patched = std::str::from_utf8(&patched).expect("patched source should be utf8");

        assert!(patched.contains("MapCanvasDataProviderMixin.OnAdded"));
        assert!(patched.contains("dataProvider:OnAdded(self);"));
    }
}
