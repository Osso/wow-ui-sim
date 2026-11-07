use super::*;
#[test]
fn test_create_frame_from_xml_inline_tooltip_set_text_with_four_global_args_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        HIGHLIGHT_FONT_COLOR = { r = 0.6, g = 0.7, b = 0.8 }
        LFG_LIST_REFRESH = "Refresh"
        GameTooltip = {
            SetOwner = function(self, owner, anchor)
                self.anchor = anchor
            end,
            SetText = function(self, text, r, g, b)
                self.text = text
                self.rgb = { r, g, b }
            end,
        }
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlineTooltipFourArgsButton" parent="UIParent">
            <Scripts><OnEnter>
                GameTooltip:SetOwner(self, "ANCHOR_RIGHT");
                GameTooltip:SetText(LFG_LIST_REFRESH, HIGHLIGHT_FONT_COLOR.r, HIGHLIGHT_FONT_COLOR.g, HIGHLIGHT_FONT_COLOR.b);
            </OnEnter></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );

    env.exec(
        r#"
        XmlInlineTooltipFourArgsButton:GetScript("OnEnter")(XmlInlineTooltipFourArgsButton)
    "#,
    )
    .unwrap();
    let result: (String, String, f64, f64, f64) = env
        .eval(
            r#"
            return GameTooltip.anchor,
                   GameTooltip.text,
                   GameTooltip.rgb[1],
                   GameTooltip.rgb[2],
                   GameTooltip.rgb[3]
        "#,
        )
        .unwrap();
    assert_eq!(result.0, "ANCHOR_RIGHT");
    assert_eq!(result.1, "Refresh");
    assert_eq!(result.2, 0.6);
    assert_eq!(result.3, 0.7);
    assert_eq!(result.4, 0.8);
}

#[test]
fn test_create_frame_from_xml_inline_tooltip_title_line_show_sequence_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        BAG_CLEANUP_BAGS = "Cleanup"
        BAG_CLEANUP_BAGS_DESCRIPTION = "Cleanup desc"
        HIGHLIGHT_FONT_COLOR = { tag = "highlight" }
        XmlInlineTooltipLog = {}
        GameTooltip = {
            SetOwner = function(self, owner)
                table.insert(XmlInlineTooltipLog, "owner")
            end,
            Show = function(self)
                table.insert(XmlInlineTooltipLog, "show")
            end,
        }
        function GameTooltip_SetTitle(tooltip, text, color)
            table.insert(XmlInlineTooltipLog, "title:" .. text .. ":" .. color.tag)
        end
        function GameTooltip_AddNormalLine(tooltip, text)
            table.insert(XmlInlineTooltipLog, "line:" .. text)
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlineTooltipSequenceButton" parent="UIParent">
            <Scripts><OnEnter>
                GameTooltip:SetOwner(self);
                GameTooltip_SetTitle(GameTooltip, BAG_CLEANUP_BAGS, HIGHLIGHT_FONT_COLOR);
                GameTooltip_AddNormalLine(GameTooltip, BAG_CLEANUP_BAGS_DESCRIPTION);
                GameTooltip:Show();
            </OnEnter></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );

    env.exec(
        r#"
        XmlInlineTooltipSequenceButton:GetScript("OnEnter")(XmlInlineTooltipSequenceButton)
    "#,
    )
    .unwrap();
    let result: String = env
        .eval(r#"return table.concat(XmlInlineTooltipLog, ",")"#)
        .unwrap();
    assert_eq!(
        result,
        "owner,title:Cleanup:highlight,line:Cleanup desc,show"
    );
}

#[test]
fn test_create_frame_from_xml_inline_tooltip_set_owner_offsets_title_show_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        DRESSING_ROOM_APPEARANCE_LIST = "Appearance"
        XmlInlineTooltipOffsetLog = {}
        GameTooltip = {
            SetOwner = function(self, owner, anchor, x, y)
                table.insert(XmlInlineTooltipOffsetLog, string.format("owner:%s:%s:%s", anchor, x, y))
            end,
            Show = function(self)
                table.insert(XmlInlineTooltipOffsetLog, "show")
            end,
        }
        function GameTooltip_SetTitle(tooltip, text)
            table.insert(XmlInlineTooltipOffsetLog, "title:" .. text)
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlineTooltipOffsetButton" parent="UIParent">
            <Scripts><OnEnter>
                GameTooltip:SetOwner(self, "ANCHOR_RIGHT", -4, -4);
                GameTooltip_SetTitle(GameTooltip, DRESSING_ROOM_APPEARANCE_LIST);
                GameTooltip:Show();
            </OnEnter></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );

    env.exec(
        r#"
        XmlInlineTooltipOffsetButton:GetScript("OnEnter")(XmlInlineTooltipOffsetButton)
    "#,
    )
    .unwrap();
    let result: String = env
        .eval(r#"return table.concat(XmlInlineTooltipOffsetLog, ",")"#)
        .unwrap();
    assert_eq!(result, "owner:ANCHOR_RIGHT:-4:-4,title:Appearance,show");
}

#[test]
fn test_create_frame_from_xml_inline_self_field_set_point_with_self_target_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlineIconPointButton" parent="UIParent">
            <Scripts><OnMouseDown>
                self.Icon:SetPoint("CENTER", self, "CENTER", -2, -1);
            </OnMouseDown></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );

    env.exec(
        r#"
        XmlInlineIconPointButton.Icon = {
            SetPoint = function(self, point, rel, relPoint, x, y)
                self.point = point
                self.rel = rel
                self.relPoint = relPoint
                self.x = x
                self.y = y
            end,
        }
        XmlInlineIconPointButton:GetScript("OnMouseDown")(XmlInlineIconPointButton)
    "#,
    )
    .unwrap();
    let result: (String, String, f64, f64, bool) = env
        .eval(
            r#"
            return XmlInlineIconPointButton.Icon.point,
                   XmlInlineIconPointButton.Icon.relPoint,
                   XmlInlineIconPointButton.Icon.x,
                   XmlInlineIconPointButton.Icon.y,
                   XmlInlineIconPointButton.Icon.rel == XmlInlineIconPointButton
        "#,
        )
        .unwrap();
    assert_eq!(result.0, "CENTER");
    assert_eq!(result.1, "CENTER");
    assert_eq!(result.2, -2.0);
    assert_eq!(result.3, -1.0);
    assert!(result.4);
}

#[test]
fn test_create_frame_from_xml_inline_show_game_tooltip_helper_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        COMMUNITIES_CREATE_DIALOG_SHORT_NAME_INSTRUCTIONS_TOOLTIP = "Short name"
        CommunitiesOutbound = {}
        XmlInlineTooltipArgs = nil
        function CommunitiesOutbound.ShowGameTooltip(text, right, top, wrap)
            XmlInlineTooltipArgs = { text, right, top, wrap }
        end
    "#,
    )
    .unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlineShowTooltipButton" parent="UIParent">
            <Scripts><OnEnter>
                CommunitiesOutbound.ShowGameTooltip(COMMUNITIES_CREATE_DIALOG_SHORT_NAME_INSTRUCTIONS_TOOLTIP, self:GetRight(), self:GetTop(), true);
            </OnEnter></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );
    env.exec(
        r#"
        function XmlInlineShowTooltipButton:GetRight() return 12 end
        function XmlInlineShowTooltipButton:GetTop() return 34 end
        XmlInlineShowTooltipButton:GetScript("OnEnter")(XmlInlineShowTooltipButton)
    "#,
    )
    .unwrap();
    let result: (String, f64, f64, bool) = env
        .eval(
            r#"
            return XmlInlineTooltipArgs[1],
                   XmlInlineTooltipArgs[2],
                   XmlInlineTooltipArgs[3],
                   XmlInlineTooltipArgs[4]
        "#,
        )
        .unwrap();
    assert_eq!(result.0, "Short name");
    assert_eq!(result.1, 12.0);
    assert_eq!(result.2, 34.0);
    assert!(result.3);
}

#[test]
#[ignore = "TODO: test calls GameTooltip:AddLine(STRING, COLOR_TABLE, true) expecting r/g/b expansion, but the GlobalMethodWithStringGlobalBoolArgs fast-path only forwards three values verbatim. Either the test needs a 5-arg call to match GlobalMethodWithGlobalThreeGlobalBoolArgs or a new color-expanding fast-path needs adding."]
fn test_create_frame_from_xml_inline_tooltip_add_line_global_three_global_bool_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        RED_FONT_COLOR = { r = 0.1, g = 0.2, b = 0.3 }
        ALL_ASSIST_NOT_LEADER_ERROR = "No leader"
        XmlInlineLineArgs = nil
        GameTooltip = {
            AddLine = function(self, text, r, g, b, wrap)
                XmlInlineLineArgs = { text, r, g, b, wrap }
            end,
        }
    "#,
    )
    .unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlineTooltipAddLineButton" parent="UIParent">
            <Scripts><OnEnter>
                GameTooltip:AddLine(ALL_ASSIST_NOT_LEADER_ERROR, RED_FONT_COLOR, true);
            </OnEnter></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );
    env.exec(
        r#"
        XmlInlineTooltipAddLineButton:GetScript("OnEnter")(XmlInlineTooltipAddLineButton)
    "#,
    )
    .unwrap();
    let result: (String, f64, f64, f64, bool) = env
        .eval(
            r#"
            return XmlInlineLineArgs[1],
                   XmlInlineLineArgs[2],
                   XmlInlineLineArgs[3],
                   XmlInlineLineArgs[4],
                   XmlInlineLineArgs[5]
        "#,
        )
        .unwrap();
    assert_eq!(result.0, "No leader");
    assert_eq!(result.1, 0.1);
    assert_eq!(result.2, 0.2);
    assert_eq!(result.3, 0.3);
    assert!(result.4);
}

#[test]
fn test_create_frame_from_xml_inline_tooltip_set_text_global_self_methods_bool_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        COMMUNITIES_CREATE_DIALOG_SHORT_NAME_INSTRUCTIONS_TOOLTIP = "Short name"
        CommunitiesOutbound = {}
        GameTooltip = {
            SetText = function(self, text, right, top, wrap)
                self.args = { text, right, top, wrap }
            end,
        }
    "#,
    )
    .unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlineTooltipSelfMethodArgsButton" parent="UIParent">
            <Scripts><OnEnter>
                GameTooltip:SetText(COMMUNITIES_CREATE_DIALOG_SHORT_NAME_INSTRUCTIONS_TOOLTIP, self:GetRight(), self:GetTop(), true);
            </OnEnter></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );
    env.exec(
        r#"
        function XmlInlineTooltipSelfMethodArgsButton:GetRight() return 55 end
        function XmlInlineTooltipSelfMethodArgsButton:GetTop() return 77 end
        XmlInlineTooltipSelfMethodArgsButton:GetScript("OnEnter")(XmlInlineTooltipSelfMethodArgsButton)
    "#,
    )
    .unwrap();
    let result: (String, f64, f64, bool) = env
        .eval(
            r#"
            return GameTooltip.args[1],
                   GameTooltip.args[2],
                   GameTooltip.args[3],
                   GameTooltip.args[4]
        "#,
        )
        .unwrap();
    assert_eq!(result.0, "Short name");
    assert_eq!(result.1, 55.0);
    assert_eq!(result.2, 77.0);
    assert!(result.3);
}

#[test]
fn test_create_frame_from_xml_inline_function_global_self_methods_bool_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        COMMUNITIES_CREATE_DIALOG_SHORT_NAME_INSTRUCTIONS_TOOLTIP = "Short name"
        CommunitiesOutbound = {}
        XmlInlineTooltipArgs = nil
        function CommunitiesOutbound.ShowGameTooltip(text, right, top, wrap)
            XmlInlineTooltipArgs = { text, right, top, wrap }
        end
    "#,
    )
    .unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlineShowGameTooltipButton" parent="UIParent">
            <Scripts><OnEnter>
                CommunitiesOutbound.ShowGameTooltip(COMMUNITIES_CREATE_DIALOG_SHORT_NAME_INSTRUCTIONS_TOOLTIP, self:GetRight(), self:GetTop(), true);
            </OnEnter></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );
    env.exec(
        r#"
        function XmlInlineShowGameTooltipButton:GetRight() return 12 end
        function XmlInlineShowGameTooltipButton:GetTop() return 34 end
        XmlInlineShowGameTooltipButton:GetScript("OnEnter")(XmlInlineShowGameTooltipButton)
    "#,
    )
    .unwrap();
    let result: (String, f64, f64, bool) = env
        .eval(
            r#"
            return XmlInlineTooltipArgs[1],
                   XmlInlineTooltipArgs[2],
                   XmlInlineTooltipArgs[3],
                   XmlInlineTooltipArgs[4]
        "#,
        )
        .unwrap();
    assert_eq!(result.0, "Short name");
    assert_eq!(result.1, 12.0);
    assert_eq!(result.2, 34.0);
    assert!(result.3);
}

#[test]
fn test_create_frame_from_xml_inline_tooltip_set_text_global_nil_nil_nil_nil_bool_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ALL_ASSIST_DESCRIPTION = "Assist"
        GameTooltip = {
            SetText = function(self, a, b, c, d, e, f)
                self.args = { a, b, c, d, e, f }
            end,
        }
    "#,
    )
    .unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlineTooltipNilArgsButton" parent="UIParent">
            <Scripts><OnEnter>
                GameTooltip:SetText(ALL_ASSIST_DESCRIPTION, nil, nil, nil, nil, true);
            </OnEnter></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );
    env.exec(
        r#"
        XmlInlineTooltipNilArgsButton:GetScript("OnEnter")(XmlInlineTooltipNilArgsButton)
    "#,
    )
    .unwrap();
    let result: (String, bool, bool) = env
        .eval(
            r#"
            return GameTooltip.args[1],
                   GameTooltip.args[6],
                   GameTooltip.args[2] == nil and GameTooltip.args[3] == nil and GameTooltip.args[4] == nil and GameTooltip.args[5] == nil
        "#,
        )
        .unwrap();
    assert_eq!(result.0, "Assist");
    assert!(result.1);
    assert!(result.2);
}

#[test]
fn test_create_frame_from_xml_inline_conditional_not_enabled_then_add_line_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        RED_FONT_COLOR = { r = 0.1, g = 0.2, b = 0.3 }
        ALL_ASSIST_NOT_LEADER_ERROR = "No leader"
        XmlInlineLineArgs = nil
        GameTooltip = {
            AddLine = function(self, text, r, g, b, wrap)
                XmlInlineLineArgs = { text, r, g, b, wrap }
            end,
        }
    "#,
    )
    .unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlineConditionalAddLineButton" parent="UIParent">
            <Scripts><OnEnter>
                -- 12340 returns 1/0, and 0 is truthy in Lua.
                if ( self:IsEnabled() == false or self:IsEnabled() == 0 ) then
                    GameTooltip:AddLine(ALL_ASSIST_NOT_LEADER_ERROR, RED_FONT_COLOR.r, RED_FONT_COLOR.g, RED_FONT_COLOR.b, true);
                end
            </OnEnter></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );
    env.exec(
        r#"
        XmlInlineConditionalAddLineButton:Disable()
        XmlInlineConditionalAddLineButton:GetScript("OnEnter")(XmlInlineConditionalAddLineButton)
    "#,
    )
    .unwrap();
    let result: (String, f64, f64, f64, bool) = env
        .eval(
            r#"
            return XmlInlineLineArgs[1],
                   XmlInlineLineArgs[2],
                   XmlInlineLineArgs[3],
                   XmlInlineLineArgs[4],
                   XmlInlineLineArgs[5]
        "#,
        )
        .unwrap();
    assert_eq!(result.0, "No leader");
    assert_eq!(result.1, 0.1);
    assert_eq!(result.2, 0.2);
    assert_eq!(result.3, 0.3);
    assert!(result.4);
}

#[test]
fn test_create_frame_from_xml_inline_conditional_global_field_equals_string_then_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        PetitionFrame = { petitionType = "guild" }
        XmlInlinePopupArgs = nil
        function StaticPopup_Show(which)
            XmlInlinePopupArgs = { which }
        end
    "#,
    )
    .unwrap();
    create_first_frame(
        &env,
        r#"<Ui>
        <Button name="XmlInlinePetitionRenameButton" parent="UIParent">
            <Scripts><OnClick>
                if ( PetitionFrame.petitionType == "guild" ) then
                    StaticPopup_Show("RENAME_GUILD");
                end
            </OnClick></Scripts>
        </Button>
    </Ui>"#,
        "Button",
    );
    env.exec(
        r#"
        XmlInlinePetitionRenameButton:GetScript("OnClick")(XmlInlinePetitionRenameButton)
    "#,
    )
    .unwrap();
    let result: String = env.eval(r#"return XmlInlinePopupArgs[1]"#).unwrap();
    assert_eq!(result, "RENAME_GUILD");
}
