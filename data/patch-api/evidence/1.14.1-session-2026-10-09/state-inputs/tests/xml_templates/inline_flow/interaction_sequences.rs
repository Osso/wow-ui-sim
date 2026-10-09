use super::*;
#[test]
fn test_create_frame_from_xml_inline_method_then_unchecked_parent_field_clear_and_show_text_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        XmlInlineMethodThenUncheckedCalls = 0
        function XmlInlineWrappedOnClick(self)
            XmlInlineMethodThenUncheckedCalls = XmlInlineMethodThenUncheckedCalls + 1
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui>
        <Frame name="XmlInlineUncheckedRoot" parent="UIParent">
            <CheckButton name="XmlInlineUncheckedButton" parent="XmlInlineUncheckedRoot">
                <Scripts><OnClick>
                    self:OnClick();
                    if (not self:GetChecked()) then
                        self:GetParent().EditBox:SetText("");
                        self:GetParent().EditBox.Text:Show();
                    end
                </OnClick></Scripts>
            </CheckButton>
        </Frame>
    </Ui>"#,
        "Frame",
    );

    env.exec(
        r#"
        function XmlInlineUncheckedButton:OnClick()
            XmlInlineWrappedOnClick(self)
        end
        XmlInlineUncheckedRoot.EditBox = {
            value = "123",
            Text = {
                shown = false,
                Show = function(self)
                    self.shown = true
                end,
            },
            SetText = function(self, value)
                self.value = value
            end,
        }
        XmlInlineUncheckedButton:SetChecked(false)
        XmlInlineUncheckedButton:GetScript("OnClick")(XmlInlineUncheckedButton)
    "#,
    )
    .unwrap();
    let unchecked_result: (i32, String, bool) = env
        .eval(
            r#"
            return XmlInlineMethodThenUncheckedCalls,
                   XmlInlineUncheckedRoot.EditBox.value,
                   XmlInlineUncheckedRoot.EditBox.Text.shown
        "#,
        )
        .unwrap();
    assert_eq!(unchecked_result.0, 1);
    assert_eq!(unchecked_result.1, "");
    assert!(unchecked_result.2);

    env.exec(
        r#"
        XmlInlineUncheckedRoot.EditBox.value = "456"
        XmlInlineUncheckedRoot.EditBox.Text.shown = false
        XmlInlineUncheckedButton:SetChecked(true)
        XmlInlineUncheckedButton:GetScript("OnClick")(XmlInlineUncheckedButton)
    "#,
    )
    .unwrap();
    let checked_result: (i32, String, bool) = env
        .eval(
            r#"
            return XmlInlineMethodThenUncheckedCalls,
                   XmlInlineUncheckedRoot.EditBox.value,
                   XmlInlineUncheckedRoot.EditBox.Text.shown
        "#,
        )
        .unwrap();
    assert_eq!(checked_result.0, 2);
    assert_eq!(checked_result.1, "456");
    assert!(!checked_result.2);
}

#[test]
fn test_create_frame_from_xml_inline_local_global_path_conditional_method_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        BrowserSettingsTooltip = {}
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlineBrowserButton" parent="UIParent">
            <Scripts><OnClick>
                local browser = BrowserSettingsTooltip.browser
                if (browser) then
                    browser:DeleteCookies()
                end
            </OnClick></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );

    env.exec(
        r#"
        BrowserSettingsTooltip.browser = {
            calls = 0,
            DeleteCookies = function(self)
                self.calls = self.calls + 1
            end,
        }
        XmlInlineBrowserButton:GetScript("OnClick")(XmlInlineBrowserButton)
    "#,
    )
    .unwrap();
    let first: i32 = env
        .eval("return BrowserSettingsTooltip.browser.calls")
        .unwrap();
    assert_eq!(first, 1);

    env.exec(
        r#"
        BrowserSettingsTooltip.browser = nil
        XmlInlineBrowserButton:GetScript("OnClick")(XmlInlineBrowserButton)
        return true
    "#,
    )
    .unwrap();
}

#[test]
fn test_create_frame_from_xml_inline_conditional_self_text_empty_show_text_child_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Frame name="XmlInlineEmptyTextRoot" parent="UIParent">
            <EditBox name="XmlInlineEmptyTextEditBox" parent="XmlInlineEmptyTextRoot">
                <Scripts><OnEditFocusLost>
                    if ( self:GetText() == "" ) then
                        self.Text:Show();
                    end
                </OnEditFocusLost></Scripts>
            </EditBox>
        </Frame>
    </Ui>"#,
        "Frame",
    );

    env.exec(
        r#"
        XmlInlineEmptyTextEditBox.Text = {
            shown = false,
            Show = function(self) self.shown = true end,
        }
        XmlInlineEmptyTextEditBox:SetText("")
        XmlInlineEmptyTextEditBox:GetScript("OnEditFocusLost")(XmlInlineEmptyTextEditBox)
    "#,
    )
    .unwrap();
    let shown: bool = env
        .eval("return XmlInlineEmptyTextEditBox.Text.shown")
        .unwrap();
    assert!(shown);
}

#[test]
fn test_create_frame_from_xml_inline_grandparent_method_with_not_self_checked_arg_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Frame name="XmlInlineGrandparentRoot" parent="UIParent">
            <Frame name="XmlInlineGrandparentParent" parent="XmlInlineGrandparentRoot">
                <CheckButton name="XmlInlineGrandparentChild" parent="XmlInlineGrandparentParent">
                    <Scripts><OnClick>
                        self:GetParent():GetParent():SetDisabledStateOnCommunityFinderOptions(not self:GetChecked())
                    </OnClick></Scripts>
                </CheckButton>
            </Frame>
        </Frame>
    </Ui>"#,
        "Frame",
    );

    env.exec(
        r#"
        function XmlInlineGrandparentRoot:SetDisabledStateOnCommunityFinderOptions(value)
            self.disabledState = value
        end
        XmlInlineGrandparentChild:SetChecked(false)
        XmlInlineGrandparentChild:GetScript("OnClick")(XmlInlineGrandparentChild)
    "#,
    )
    .unwrap();
    let first: bool = env
        .eval("return XmlInlineGrandparentRoot.disabledState")
        .unwrap();
    assert!(first);

    env.exec(
        r#"
        XmlInlineGrandparentChild:SetChecked(true)
        XmlInlineGrandparentChild:GetScript("OnClick")(XmlInlineGrandparentChild)
    "#,
    )
    .unwrap();
    let second: bool = env
        .eval("return XmlInlineGrandparentRoot.disabledState")
        .unwrap();
    assert!(!second);
}

#[test]
fn test_create_frame_from_xml_inline_function_with_self_gettext_result_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        XmlInlineFilterName = nil
        function CombatConfig_SetFilterName(value)
            XmlInlineFilterName = value
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui>
        <Frame name="XmlInlineGetTextRoot" parent="UIParent">
            <EditBox name="XmlInlineGetTextEditBox" parent="XmlInlineGetTextRoot">
                <Scripts><OnEnterPressed>
                    CombatConfig_SetFilterName(self:GetText());
                    self:HighlightText(0, -1);
                </OnEnterPressed></Scripts>
            </EditBox>
        </Frame>
    </Ui>"#,
        "Frame",
    );

    env.exec(
        r#"
        XmlInlineGetTextEditBox:SetText("my-filter")
        function XmlInlineGetTextEditBox:HighlightText(startIndex, endIndex)
            self.highlightStart = startIndex
            self.highlightEnd = endIndex
        end
        XmlInlineGetTextEditBox:GetScript("OnEnterPressed")(XmlInlineGetTextEditBox)
    "#,
    )
    .unwrap();
    let result: (String, i32, i32) = env
        .eval(
            r#"
            return XmlInlineFilterName,
                   XmlInlineGetTextEditBox.highlightStart,
                   XmlInlineGetTextEditBox.highlightEnd
        "#,
        )
        .unwrap();
    assert_eq!(result.0, "my-filter");
    assert_eq!(result.1, 0);
    assert_eq!(result.2, -1);
}

#[test]
fn test_create_frame_from_xml_inline_copy_club_ticket_to_clipboard_from_parent_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        C_Club = {}
        ClubTicketUtil = {}
        XmlInlineClipboardValue = nil
        function C_Club.GetClubInfo(clubId)
            return { id = clubId }
        end
        function ClubTicketUtil.FormatTicket(clubInfo, linkText)
            return string.format("club:%d:%s", clubInfo.id, linkText)
        end
        function CopyToClipboard(value)
            XmlInlineClipboardValue = value
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui>
        <Frame name="XmlInlineClubTicketRoot" parent="UIParent">
            <Button name="XmlInlineClubTicketButton" parent="XmlInlineClubTicketRoot">
                <Scripts><OnClick>
                    local clubId = self:GetParent():GetClubId();
                    local clubInfo = clubId and C_Club.GetClubInfo(clubId);
                    if clubInfo then
                        CopyToClipboard(ClubTicketUtil.FormatTicket(clubInfo, self:GetParent().LinkIDText:GetText()));
                    end
                </OnClick></Scripts>
            </Button>
        </Frame>
    </Ui>"#,
        "Frame",
    );

    env.exec(
        r#"
        function XmlInlineClubTicketRoot:GetClubId()
            return 17
        end
        XmlInlineClubTicketRoot.LinkIDText = {
            GetText = function()
                return "abc123"
            end,
        }
        XmlInlineClubTicketButton:GetScript("OnClick")(XmlInlineClubTicketButton)
    "#,
    )
    .unwrap();
    let result: String = env.eval("return XmlInlineClipboardValue").unwrap();
    assert_eq!(result, "club:17:abc123");
}

#[test]
fn test_create_frame_from_xml_inline_play_sound_then_copy_club_ticket_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        C_Club = {}
        ClubTicketUtil = {}
        SOUNDKIT = { IG_MAINMENU_OPTION_CHECKBOX_ON = 42 }
        XmlInlineClipboardValue = nil
        XmlInlineLastSound = nil
        function C_Club.GetClubInfo(clubId)
            return { id = clubId }
        end
        function ClubTicketUtil.FormatTicket(clubInfo, linkText)
            return string.format("club:%d:%s", clubInfo.id, linkText)
        end
        function CopyToClipboard(value)
            XmlInlineClipboardValue = value
        end
        function PlaySound(sound)
            XmlInlineLastSound = sound
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui>
        <Frame name="XmlInlineClubTicketSoundRoot" parent="UIParent">
            <Button name="XmlInlineClubTicketSoundButton" parent="XmlInlineClubTicketSoundRoot">
                <Scripts><OnClick>
                    PlaySound(SOUNDKIT.IG_MAINMENU_OPTION_CHECKBOX_ON);
                    local clubId = self:GetParent():GetClubId();
                    local clubInfo = clubId and C_Club.GetClubInfo(clubId);
                    if clubInfo then
                        CopyToClipboard(ClubTicketUtil.FormatTicket(clubInfo, self:GetParent().LinkIDText:GetText()));
                    end
                </OnClick></Scripts>
            </Button>
        </Frame>
    </Ui>"#,
        "Frame",
    );

    env.exec(
        r#"
        function XmlInlineClubTicketSoundRoot:GetClubId()
            return 23
        end
        XmlInlineClubTicketSoundRoot.LinkIDText = {
            GetText = function()
                return "ticket"
            end,
        }
        XmlInlineClubTicketSoundButton:GetScript("OnClick")(XmlInlineClubTicketSoundButton)
    "#,
    )
    .unwrap();
    let result: (i32, String) = env
        .eval("return XmlInlineLastSound, XmlInlineClipboardValue")
        .unwrap();
    assert_eq!(result.0, 42);
    assert_eq!(result.1, "club:23:ticket");
}

#[test]
fn test_create_frame_from_xml_inline_parent_field_local_click_if_enabled_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Frame name="XmlInlineCreateButtonRoot" parent="UIParent">
            <EditBox name="XmlInlineCreateButtonEditBox" parent="XmlInlineCreateButtonRoot">
                <Scripts><OnEnterPressed>
                    local createButton = self:GetParent().CreateButton;
                    if createButton:IsEnabled() then
                        createButton:GetScript("OnClick")(createButton);
                    end
                </OnEnterPressed></Scripts>
            </EditBox>
        </Frame>
    </Ui>"#,
        "Frame",
    );

    env.exec(
        r#"
        XmlInlineCreateButtonRoot.CreateButton = CreateFrame("Button", nil, XmlInlineCreateButtonRoot)
        XmlInlineCreateButtonRoot.calls = 0
        XmlInlineCreateButtonRoot.CreateButton:SetScript("OnClick", function(self)
            XmlInlineCreateButtonRoot.calls = XmlInlineCreateButtonRoot.calls + 1
        end)
        XmlInlineCreateButtonRoot.CreateButton:Enable()
        XmlInlineCreateButtonEditBox:GetScript("OnEnterPressed")(XmlInlineCreateButtonEditBox)
        XmlInlineCreateButtonRoot.CreateButton:Disable()
        XmlInlineCreateButtonEditBox:GetScript("OnEnterPressed")(XmlInlineCreateButtonEditBox)
    "#,
    )
    .unwrap();
    let calls: i32 = env.eval("return XmlInlineCreateButtonRoot.calls").unwrap();
    assert_eq!(calls, 1);
}

#[test]
fn test_create_frame_from_xml_inline_grandparent_field_method_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Frame name="XmlInlineGrandparentFieldRoot" parent="UIParent">
            <Frame name="XmlInlineGrandparentFieldParent" parent="XmlInlineGrandparentFieldRoot">
                <EditBox name="XmlInlineGrandparentFieldChild" parent="XmlInlineGrandparentFieldParent">
                    <Scripts><OnEnterPressed>
                        self:GetParent():GetParent().EditBox:SetFocus();
                    </OnEnterPressed></Scripts>
                </EditBox>
            </Frame>
        </Frame>
    </Ui>"#,
        "Frame",
    );

    env.exec(
        r#"
        XmlInlineGrandparentFieldRoot.EditBox = {
            focused = false,
            SetFocus = function(self)
                self.focused = true
            end,
        }
        XmlInlineGrandparentFieldChild:GetScript("OnEnterPressed")(XmlInlineGrandparentFieldChild)
    "#,
    )
    .unwrap();
    let focused: bool = env
        .eval("return XmlInlineGrandparentFieldRoot.EditBox.focused")
        .unwrap();
    assert!(focused);
}

#[test]
fn test_create_frame_from_xml_inline_tooltip_then_parent_assign_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        HIGHLIGHT_FONT_COLOR = { r = 0.1, g = 0.2, b = 0.3 }
        BROWSER_DELETE_COOKIES_TOOLTIP = "Cookies"
        GameTooltip = {
            SetOwner = function(self, owner, anchor)
                self.owner = owner
                self.anchor = anchor
            end,
            SetText = function(self, text, r, g, b, maybe_nil, wrap)
                self.text = text
                self.color = { r, g, b }
                self.maybe_nil = maybe_nil
                self.wrap = wrap
            end,
        }
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui>
        <Frame name="XmlInlineTooltipAssignRoot" parent="UIParent">
            <Button name="XmlInlineTooltipAssignButton" parent="XmlInlineTooltipAssignRoot">
                <Scripts><OnEnter>
                    GameTooltip:SetOwner(self, "ANCHOR_CURSOR_RIGHT");
                    GameTooltip:SetText(BROWSER_DELETE_COOKIES_TOOLTIP, HIGHLIGHT_FONT_COLOR.r, HIGHLIGHT_FONT_COLOR.g, HIGHLIGHT_FONT_COLOR.b, nil, true);
                    self:GetParent().isCounting = nil;
                </OnEnter></Scripts>
            </Button>
        </Frame>
    </Ui>"#,
        "Frame",
    );

    env.exec(
        r#"
        XmlInlineTooltipAssignRoot.isCounting = true
        XmlInlineTooltipAssignButton:GetScript("OnEnter")(XmlInlineTooltipAssignButton)
    "#,
    )
    .unwrap();
    let result: (String, String, bool, bool) = env
        .eval(
            r#"
            return GameTooltip.anchor,
                   GameTooltip.text,
                   GameTooltip.wrap,
                   XmlInlineTooltipAssignRoot.isCounting == nil
        "#,
        )
        .unwrap();
    assert_eq!(result.0, "ANCHOR_CURSOR_RIGHT");
    assert_eq!(result.1, "Cookies");
    assert!(result.2);
    assert!(result.3);
}
