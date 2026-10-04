#![cfg(feature = "retail-12-0-7")]

#[test]
fn loaded_money_helper_reflects_mail_amount_changes_and_append_order() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_GameTooltip", "Blizzard_MailFrame"],
            &[],
            |env, _| {
                env.exec(
                    r#"
                    SetCVar('colorblindMode', '0')
                    A_Admin.ClearInbox()
                    A_Admin.AddMail('Fixture', 'Copper', '', 123)
                    A_Admin.AddMail('Fixture', 'Gold', '', 20000)
                    GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                    GameTooltip:ClearLines()
                    GameTooltip:AddLine('fixture label')
                    local _, _, _, _, first = GetInboxHeaderInfo(1)
                    local _, _, _, _, second = GetInboxHeaderInfo(2)
                    assert(first == 123 and second == 20000)
                    GameTooltip_AddMoneyLine(GameTooltip, first, false)
                    GameTooltip_AddMoneyLine(GameTooltip, second, true)
                    GameTooltip_AddMoneyLine(GameTooltip, 0)
                    assert(GameTooltip:NumLines() == 4)
                    assert(GameTooltipTextLeft1:GetText() == 'fixture label')
                    assert(GameTooltipTextLeft2:GetText() ==
                        '1|A:coin-silver:14:14:2:0|a 23|A:coin-copper:14:14:2:0|a')
                    assert(GameTooltipTextLeft3:GetText() == '2|A:coin-gold:14:14:2:0|a')
                    assert(GameTooltipTextLeft4:GetText() == ' ')
                    for index, color in pairs({[2]=HIGHLIGHT_FONT_COLOR, [3]=RED_FONT_COLOR,
                        [4]=HIGHLIGHT_FONT_COLOR}) do
                        local r, g, b = _G['GameTooltipTextLeft' .. index]:GetTextColor()
                        local er, eg, eb = color:GetRGB()
                        assert(math.abs(r-er) < 0.0001 and math.abs(g-eg) < 0.0001
                            and math.abs(b-eb) < 0.0001)
                    end
                    GameTooltip:ClearLines()
                    GameTooltip_AddMoneyLine(GameTooltip, second, false)
                    assert(GameTooltip:NumLines() == 1)
                    assert(GameTooltipTextLeft1:GetText() == '2|A:coin-gold:14:14:2:0|a')
                    "#,
                )
                .expect("loaded helper formats concrete live mailbox amounts, not a fixed prefix");
            },
        );
    });
}
