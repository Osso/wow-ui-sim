//! R84 Retail currency payloads through the original GameTooltip consumers.
#![cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

fn retail_env() -> Option<WowLuaEnv> {
    (ACTIVE == ClientProfile::Retail).then(tooltip_full_env_helpers::setup_full_env)
}

/// `UIPanelTemplates.lua:176` calls this method; `TooltipDataHandler.lua:556` selects its getter.
#[test]
fn set_currency_by_id_publishes_r84_currency_rows() {
    let Some(env) = retail_env() else {
        return;
    };
    env.exec(
        r#"
        local button = CreateFrame('Frame', nil, UIParent)
        button.currencyID = 2245
        CurrencyLayoutFrameIconMixin.OnEnter(button)

        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        local currencyName = C_CurrencyInfo.GetCurrencyInfo(2245).name
        assert(info and info.getterName == 'GetCurrencyByID', 'currency consumer must select GetCurrencyByID')
        assert(data and data.type == Enum.TooltipDataType.Currency, 'currency data type')
        assert(data and data.id == 2245, 'currency id')
        assert(data and data.dataInstanceID ~= nil, 'currency data instance id')
        assert(data and data.quantity == nil, 'R84 §Q4: no top-level quantity')
        assert(data.lines[1].leftText == currencyName and currencyName ~= '', 'currency title must use the resolved CurrencyTypes name')
        assert(data.lines[1].leftColor ~= nil and data.lines[1].wrapText == nil, 'currency title shape')
        for _, line in ipairs(data.lines) do
            assert(line.rightText == nil and line.rightColor == nil, 'currency rows are left-only')
        end
        "#,
    )
    .unwrap();
}

/// `TooltipDataHandler.lua:590` maps SetHyperlink to GetHyperlink. No native currency-link caller was found.
#[test]
fn set_hyperlink_resolves_currency_links() {
    let Some(env) = retail_env() else {
        return;
    };
    env.exec(
        r#"
        GameTooltip:SetHyperlink('currency:2245')
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetHyperlink', 'SetHyperlink must select GetHyperlink')
        assert(data and data.type == Enum.TooltipDataType.Currency, 'currency hyperlink data type')
        assert(data and data.id == 2245, 'currency hyperlink id')
        assert(data and data.dataInstanceID ~= nil, 'currency hyperlink data instance id')
        assert(data and data.quantity == nil, 'R84 §Q4: no top-level quantity')
        assert(data.lines[1] and data.lines[1].leftText, 'currency hyperlink title')
        "#,
    )
    .unwrap();
}
