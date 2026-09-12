#![cfg(feature = "retail-12-0-7")]

use wow_ui_sim::lua_api::WowLuaEnv;

const MONEY_LINE_ASSERTIONS: &str = r#"
    local expectedMoney = "1|A:coin-gold:14:14:2:0|a 23|A:coin-silver:14:14:2:0|a 45|A:coin-copper:14:14:2:0|a"
    function AssertMoneyLine(index, expectedText, expectedColor)
        local line = _G["GameTooltipTextLeft" .. index]
        assert(line, "missing tooltip line " .. index)
        assert(line:GetText() == expectedText,
            "unexpected money text: " .. tostring(line:GetText()))
        local r, g, b = line:GetTextColor()
        local er, eg, eb = expectedColor:GetRGB()
        assert(math.abs(r - er) < 0.0001 and math.abs(g - eg) < 0.0001
            and math.abs(b - eb) < 0.0001, "unexpected money line color")
    end
    ExpectedMoneyLine = expectedMoney
    SetCVar("colorblindMode", "0")
"#;

fn with_mail_tooltip(assertions: impl FnOnce(&WowLuaEnv) + Send + 'static) {
    crate::common::with_timeout(90, move || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_GameTooltip", "Blizzard_MailFrame"],
            &[],
            |env, _| {
                env.exec(MONEY_LINE_ASSERTIONS)
                    .expect("money tooltip assertions should initialize");
                assertions(env);
            },
        );
    });
}

#[test]
fn tooltip_money_line_mail_enclosed_money_keeps_label_order_and_highlight() {
    with_mail_tooltip(|env| {
        env.exec(
            r#"
            A_Admin.ClearInbox()
            A_Admin.AddMail("AH", "With Gold", "", 12345)
            local _, _, _, _, money = GetInboxHeaderInfo(1)
            assert(money == 12345, "seeded mail money should be observable")
            local item = CreateFrame("Button", nil, UIParent)
            item.index = 1
            item.money = money
            item.hasItem = false
            GameTooltip:ClearLines()
            InboxFrameItem_OnEnter(item)
            assert(GameTooltip:IsShown(), "mail money tooltip should be shown")
            assert(GameTooltip:NumLines() == 2, "expected label followed by money")
            assert(GameTooltipTextLeft1:GetText() == ENCLOSED_MONEY)
            AssertMoneyLine(2, ExpectedMoneyLine, HIGHLIGHT_FONT_COLOR)
            "#,
        )
        .expect("real inbox consumer should show enclosed money in highlight color");
    });
}

#[test]
fn tooltip_money_line_mail_unaffordable_cod_keeps_label_order_and_red() {
    with_mail_tooltip(|env| {
        env.exec(
            r#"
            A_Admin.ClearInbox()
            A_Admin.AddMail("AH", "Payment Due", "", 0)
            A_Admin.SetMoney(12344)
            local item = CreateFrame("Button", nil, UIParent)
            item.index = 1
            item.cod = 12345
            item.hasItem = false
            assert(item.cod > GetMoney(), "COD fixture must be unaffordable")
            GameTooltip:ClearLines()
            InboxFrameItem_OnEnter(item)
            assert(GameTooltip:IsShown(), "mail COD tooltip should be shown")
            assert(GameTooltip:NumLines() == 2, "expected COD label followed by money")
            assert(GameTooltipTextLeft1:GetText() == COD_AMOUNT)
            AssertMoneyLine(2, ExpectedMoneyLine, RED_FONT_COLOR)
            "#,
        )
        .expect("real inbox consumer should show unaffordable COD in red");
    });
}

#[test]
fn tooltip_money_line_loaded_helper_formats_zero_and_boolean_colors() {
    with_mail_tooltip(|env| {
        env.exec(
            r#"
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:ClearLines()
            GameTooltip_AddMoneyLine(GameTooltip, 0)
            GameTooltip_AddMoneyLine(GameTooltip, 12345, false)
            GameTooltip_AddMoneyLine(GameTooltip, 12345, true)
            assert(GameTooltip:NumLines() == 3, "money helper must append one line per call")
            AssertMoneyLine(1, " ", HIGHLIGHT_FONT_COLOR)
            AssertMoneyLine(2, ExpectedMoneyLine, HIGHLIGHT_FONT_COLOR)
            AssertMoneyLine(3, ExpectedMoneyLine, RED_FONT_COLOR)
            "#,
        )
        .expect("loaded money helper should preserve zero text and boolean colors");
    });
}
