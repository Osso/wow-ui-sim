//! R84 Retail currency payloads through the original GameTooltip consumers.
#![cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use std::collections::HashMap;

use wow_ui_sim::c_api::c_tooltip_info_currency::{CurrencyTooltipInputs, CurrencyTooltipTextRange};
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

fn retail_env() -> Option<WowLuaEnv> {
    (ACTIVE == ClientProfile::Retail).then(tooltip_full_env_helpers::setup_full_env)
}

fn xp_inputs(local_currency_record: bool) -> CurrencyTooltipInputs {
    CurrencyTooltipInputs {
        flags_0: Some(1 << 3),
        flags_1: Some(1 << 1),
        field_11_xp_quest_difficulty: Some(1),
        local_currency_record: Some(local_currency_record),
        formatted_numbers: HashMap::from([
            (-9, "-9".to_owned()),
            (0, "0".to_owned()),
            (9, "9".to_owned()),
            (99, "99".to_owned()),
            (999, "999".to_owned()),
        ]),
        ..CurrencyTooltipInputs::default()
    }
}

fn seed_currency_tooltip_inputs(env: &WowLuaEnv, inputs: CurrencyTooltipInputs) {
    env.state()
        .borrow_mut()
        .currency_tooltip_inputs
        .insert(2245, inputs);
}

/// `UIPanelTemplates.lua:176` calls this method; `TooltipDataHandler.lua:556` selects its getter.
#[test]
fn set_currency_by_id_preserves_amount_and_defaults_absent_to_zero() {
    let Some(env) = retail_env() else {
        return;
    };
    seed_currency_tooltip_inputs(&env, xp_inputs(true));
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
        assert(data.lines[1].leftText == currencyName and currencyName ~= '', 'currency title must use the resolved CurrencyTypes name')
        assert(data.lines[1].leftColor ~= nil and data.lines[1].wrapText == nil, 'currency title shape')
        assert(#data.lines == 3, 'XP row must end the producer')
        assert(data.lines[3].type == 14, 'currency XP row type')
        assert(data.lines[3].leftText == XP .. ': 0', 'absent currency amount must reach the producer as zero')
        assert(data.lines[3].wrapText == nil, 'XP row must not wrap')
        assert(data.quantity == nil, 'R84 §Q4: no top-level quantity')
        for _, line in ipairs(data.lines) do
            assert(line.rightText == nil and line.rightColor == nil, 'currency rows are left-only')
        end

        GameTooltip:SetCurrencyByID(2245, 99)
        data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[3].leftText == XP .. ': 99', 'GetCurrencyByID must pass amount unchanged')
        assert(#data.lines == 3, 'XP row must prevent spacer rows')
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
    let mut inputs = xp_inputs(false);
    inputs.text_ranges.push(CurrencyTooltipTextRange {
        min_quantity: 99,
        max_quantity: 0,
        name: "Link range title".to_owned(),
        description: "Link range description".to_owned(),
    });
    seed_currency_tooltip_inputs(&env, inputs);
    env.exec(
        r#"
        GameTooltip:SetHyperlink('currency:2245:99')
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetHyperlink', 'SetHyperlink must select GetHyperlink')
        assert(data and data.type == Enum.TooltipDataType.Currency, 'currency hyperlink data type')
        assert(data and data.id == 2245, 'currency hyperlink id')
        assert(data and data.dataInstanceID ~= nil, 'currency hyperlink data instance id')
        assert(data and data.quantity == nil, 'R84 §Q4: no top-level quantity')
        assert(data.lines[1].leftText == 'Link range title', 'currency hyperlink must use its optional amount')
        assert(data.lines[2].leftText == 'Link range description' and data.lines[2].wrapText, 'currency description row')
        assert(#data.lines == 3, 'currency hyperlink XP row')
        assert(data.lines[3].leftText == XP .. ': 99', 'currency hyperlink must not divide its optional amount')
        "#,
    )
    .unwrap();
}

/// AlertFrameSystems.lua:125 calls the original consumer. Host input supplies source rows and does
/// not derive them from unrelated quest reward state.
#[test]
fn set_quest_log_currency_uses_selected_quest_and_normalizes_supplied_amount() {
    let Some(env) = retail_env() else {
        return;
    };
    let quest_id = 81_234;
    {
        let mut state = env.state().borrow_mut();
        state.selected_quest_log_id = Some(quest_id as u32);
        state.currency_tooltip_inputs.insert(2245, xp_inputs(true));
        state
            .quest_currency_tooltip_results
            .insert((quest_id, "reward".to_owned(), 0), (2245, 999));
        state
            .quest_currency_tooltip_results
            .insert((quest_id, "reward".to_owned(), 1), (2245, 99));
        state
            .quest_currency_tooltip_results
            .insert((quest_id + 1, "reward".to_owned(), 0), (2245, 900));
    }
    env.exec(
        r#"
        GameTooltip:SetQuestLogCurrency('ReWaRd', 1.9)
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetQuestLogCurrency', 'quest consumer must select GetQuestLogCurrency')
        assert(data and data.type == Enum.TooltipDataType.Currency, 'quest currency data type')
        assert(data and data.id == 2245, 'quest currency id')
        assert(#data.lines == 3, 'XP row must end the producer')
        assert(data.lines[3].type == 14, 'quest currency XP row type')
        assert(data.lines[3].leftText == XP .. ': 9', 'ASCII case fold and one-based index conversion must select source row zero')
        assert(data.quantity == nil, 'R84 §Q4: no top-level quantity')
        GameTooltip:SetQuestLogCurrency('reward', 2)
        data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[3].leftText == XP .. ': 0', 'currency index two must select source row one')
        GameTooltip:SetQuestLogCurrency('reward', 1, 81235)
        data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[3].leftText == XP .. ': 9', 'explicit quest ID must select its row')
        "#,
    )
    .unwrap();
    {
        let mut state = env.state().borrow_mut();
        state
            .quest_currency_tooltip_results
            .insert((quest_id, "reward".to_owned(), 0), (2245, -999));
    }
    env.exec(
        r#"
        GameTooltip:SetQuestLogCurrency('reward', 1)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[3].leftText == XP .. ': -9', 'signed division must truncate toward zero')
        "#,
    )
    .unwrap();
    {
        let mut state = env.state().borrow_mut();
        state
            .currency_tooltip_inputs
            .get_mut(&2245)
            .unwrap()
            .flags_0 = Some(0);
        state
            .quest_currency_tooltip_results
            .insert((quest_id, "reward".to_owned(), 0), (2245, 999));
    }
    env.exec(
        r#"
        GameTooltip:SetQuestLogCurrency('reward', 1)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[3].leftText == XP .. ': 999', 'quest amount must stay unchanged when bit 3 is clear')
        "#,
    )
    .unwrap();
}

#[test]
fn currency_by_id_selects_the_first_matching_text_range() {
    let Some(env) = retail_env() else {
        return;
    };
    let mut inputs = CurrencyTooltipInputs::default();
    inputs.text_ranges = vec![
        CurrencyTooltipTextRange {
            min_quantity: 99,
            max_quantity: 100,
            name: "First matching title".to_owned(),
            description: "First matching description".to_owned(),
        },
        CurrencyTooltipTextRange {
            min_quantity: 100,
            max_quantity: 0,
            name: "Unbounded title".to_owned(),
            description: "Unbounded description".to_owned(),
        },
    ];
    seed_currency_tooltip_inputs(&env, inputs);
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[1].leftText == C_CurrencyInfo.GetCurrencyInfo(2245).name, 'no match must use ordinary currency text')
        GameTooltip:SetCurrencyByID(2245, 99)
        data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[1].leftText == 'First matching title', 'lower bound must be inclusive')
        assert(data.lines[2].leftText == 'First matching description' and data.lines[2].wrapText, 'range description')
        GameTooltip:SetCurrencyByID(2245, 100)
        data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[1].leftText == 'First matching title', 'upper bound must be inclusive and first row wins')
        GameTooltip:SetCurrencyByID(2245, 101)
        data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[1].leftText == 'Unbounded title', 'zero upper bound must be unbounded')
        "#,
    )
    .unwrap();
}

fn currency_cap_inputs() -> CurrencyTooltipInputs {
    CurrencyTooltipInputs {
        flags_0: Some(0),
        flags_1: Some(0),
        field_7_weekly_maximum: Some(0),
        field_13_world_state_maximum: Some(0),
        field_6_progression_maximum: Some(0),
        local_currency_record: Some(true),
        local_quantity: Some(0),
        seasonal_quantity: Some(50),
        balance_helper_result: Some(25),
        total_row_color_prefix: Some("|cFF112233".to_owned()),
        formatted_numbers: HashMap::from([
            (0, "0".to_owned()),
            (2, "2".to_owned()),
            (9, "9".to_owned()),
            (25, "25".to_owned()),
            (50, "50".to_owned()),
            (70, "70".to_owned()),
            (90, "90".to_owned()),
            (99, "99".to_owned()),
            (100, "100".to_owned()),
            (700, "700".to_owned()),
            (900, "900".to_owned()),
            (999, "999".to_owned()),
        ]),
        ..CurrencyTooltipInputs::default()
    }
}

#[test]
fn quest_currency_index_uses_the_verified_usage_and_range_errors() {
    let Some(env) = retail_env() else {
        return;
    };
    let range_error: String = env
        .eval(
            r#"
            local ok, err = pcall(C_TooltipInfo.GetQuestLogCurrency, 'reward', -1)
            assert(not ok, 'negative currencyIndex must fail argument conversion')
            return tostring(err)
            "#,
        )
        .unwrap();
    assert!(
        range_error.contains("outside of expected range 0 to 4294967295"),
        "unexpected currencyIndex range error: {range_error}"
    );
    let usage_error: String = env
        .eval(
            r#"
            local ok, err = pcall(C_TooltipInfo.GetQuestLogCurrency, 'reward', '1')
            assert(not ok, 'string currencyIndex must fail argument conversion')
            return tostring(err)
            "#,
        )
        .unwrap();
    assert!(
        usage_error.contains(
            "Usage: local data = C_TooltipInfo.GetQuestLogCurrency(type, currencyIndex [, questID])"
        ),
        "unexpected currencyIndex usage error: {usage_error}"
    );
}

#[test]
fn currency_weekly_cap_uses_verified_override_and_color_gates() {
    let Some(env) = retail_env() else {
        return;
    };
    env.exec(
        "CURRENCY_WEEKLY_CAP = 'weekly:%s:%s/%s'; HIGHLIGHT_FONT_COLOR = CreateColor(0.25, 0.5, 0.75); ERROR_COLOR = CreateColor(1, 0, 0)",
    )
    .unwrap();
    let mut inputs = currency_cap_inputs();
    inputs.flags_0 = Some((1 << 2) | (1 << 3));
    inputs.local_quantity = Some(900);
    inputs.weekly_maximum_override = Some(900);
    inputs.field_7_weekly_maximum = Some(200);
    inputs.formatted_numbers.insert(9, "9".to_owned());
    seed_currency_tooltip_inputs(&env, inputs.clone());
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(#data.lines == 4, 'positive weekly maximum must append one row after the spacer')
        assert(data.lines[4].type == 14, 'weekly cap line type')
        assert(data.lines[4].leftText == 'weekly:|cffff0000:9/9', 'override normalization and at-cap error prefix')
        assert(data.lines[4].wrapText == nil, 'weekly cap must not wrap')
        "#,
    )
    .unwrap();

    inputs.flags_0 = Some(1 << 3);
    inputs.local_quantity = Some(99);
    inputs.weekly_maximum_override = Some(900);
    inputs.field_7_weekly_maximum = Some(200);
    inputs.formatted_numbers.insert(0, "0".to_owned());
    inputs.formatted_numbers.insert(2, "2".to_owned());
    seed_currency_tooltip_inputs(&env, inputs.clone());
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[4].leftText == 'weekly:|cff3f7fbf:0/2', 'field 7 maximum and below-cap highlight prefix')
        "#,
    )
    .unwrap();

    inputs.flags_0 = Some((1 << 3) | (1 << 22));
    seed_currency_tooltip_inputs(&env, inputs);
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(#data.lines == 2, 'Flags_0 bit 22 must suppress the spacer and cap section')
        "#,
    )
    .unwrap();
}

#[test]
fn currency_total_rows_follow_local_overrides_and_seasonal_exclusivity() {
    let Some(env) = retail_env() else {
        return;
    };
    env.exec(
        r#"
        CURRENCY_WEEKLY_CAP = 'weekly:%s:%s/%s'
        CURRENCY_TOTAL = 'total:%s:%s'
        CURRENCY_TOTAL_CAP = 'totalcap:%s:%s:%s'
        CURRENCY_SEASON_TOTAL_MAXIMUM = 'seasonmax:%s:%s:%s'
        CURRENCY_SEASON_TOTAL = 'season:%s:%s'
        "#,
    )
    .unwrap();

    for (flags_0, flags_1) in [
        (1 << 3, (1 << 0) | (1 << 6)),
        (1 << 3, (1 << 0) | (1 << 7)),
        ((1 << 3) | (1 << 18), 1 << 0),
    ] {
        let mut inputs = currency_cap_inputs();
        inputs.flags_0 = Some(flags_0);
        inputs.flags_1 = Some(flags_1);
        inputs.local_total_maximum_override = Some(900);
        inputs.field_13_world_state_maximum = Some(4_000);
        inputs.field_6_progression_maximum = Some(5_000);
        inputs.seasonal_quantity = Some(900);
        seed_currency_tooltip_inputs(&env, inputs);
        env.exec(
            r#"
            GameTooltip:SetCurrencyByID(2245, 0)
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(data.lines[4] and data.lines[4].leftText == 'total:|cFF112233:25', 'total row must use the supplied balance result without caller scaling')
            assert(data.lines[5] and data.lines[5].leftText == 'seasonmax:|cFF112233:9:900', 'local maximum is unchanged while seasonal quantity is scaled')
            for _, line in ipairs(data.lines) do
                assert(not line.leftText:find('season:', 1, true), 'season total and season maximum are mutually exclusive')
            end
            "#,
        )
        .unwrap();
    }

    let mut inputs = currency_cap_inputs();
    inputs.flags_0 = Some(1 << 3);
    inputs.flags_1 = Some(1 << 7);
    inputs.local_total_maximum_override = Some(900);
    inputs.seasonal_quantity = Some(900);
    seed_currency_tooltip_inputs(&env, inputs);
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[4].leftText == 'totalcap:|cFF112233:25:900', 'positive maximum without seasonal flag selects total cap')
        "#,
    )
    .unwrap();

    let mut inputs = currency_cap_inputs();
    inputs.flags_0 = Some(1 << 3);
    inputs.flags_1 = Some(1);
    inputs.local_total_maximum_override = Some(700);
    inputs.field_13_world_state_maximum = Some(900);
    inputs.field_6_progression_maximum = Some(5_000);
    inputs.seasonal_quantity = Some(900);
    seed_currency_tooltip_inputs(&env, inputs);
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[5].leftText == 'seasonmax:|cFF112233:9:9', 'nonzero field 13 maximum and seasonal quantity divide by 100')
        "#,
    )
    .unwrap();

    let mut inputs = currency_cap_inputs();
    inputs.flags_0 = Some(1 << 3);
    inputs.flags_1 = Some(0);
    inputs.field_13_world_state_maximum = Some(0);
    inputs.field_6_progression_maximum = Some(900);
    inputs.seasonal_quantity = Some(900);
    seed_currency_tooltip_inputs(&env, inputs);
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[4].leftText == 'totalcap:|cFF112233:25:9', 'field 6 fallback maximum divides by 100')
        "#,
    )
    .unwrap();

    let mut inputs = currency_cap_inputs();
    inputs.flags_1 = Some(1);
    inputs.seasonal_quantity = Some(50);
    seed_currency_tooltip_inputs(&env, inputs);
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(#data.lines == 4 and data.lines[4].leftText == 'season:|cFF112233:50', 'nonpositive maximum selects only season total')
        "#,
    )
    .unwrap();

    let inputs = currency_cap_inputs();
    seed_currency_tooltip_inputs(&env, inputs);
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(#data.lines == 4 and data.lines[4].leftText == 'total:|cFF112233:25', 'zero weekly and total maxima select fallback total')
        "#,
    )
    .unwrap();
}

#[test]
fn currency_account_labels_and_bit_22_control_the_spacer() {
    let Some(env) = retail_env() else {
        return;
    };
    seed_currency_tooltip_inputs(
        &env,
        CurrencyTooltipInputs {
            flags_0: Some(0),
            flags_1: Some(1 << 3),
            field_6_low_byte: Some(1 << 2),
            field_16_transfer_percentage: Some(1.0),
            local_currency_record: Some(true),
            ..CurrencyTooltipInputs::default()
        },
    );
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data and #data.lines >= 4, 'account labels are missing')
        assert(data.lines[2].leftText == ACCOUNT_LEVEL_CURRENCY, 'account-level row')
        assert(data.lines[3].leftText == ACCOUNT_TRANSFERRABLE_CURRENCY, 'transferable row')
        assert(data.lines[4].leftText == REPUTATION_TOOLTIP_ACCOUNT_WIDE_LABEL, 'reputation account-wide row')
        for index = 2, 4 do
            assert(data.lines[index].type == 0 and data.lines[index].leftColor ~= nil, 'account label row shape')
        end
        local hasSpacer = false
        for _, line in ipairs(data.lines) do
            if line.type == 1 then hasSpacer = true end
        end
        assert(hasSpacer, 'supplied local currency record should emit the spacer when bit 22 is clear')
        "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        let inputs = state
            .currency_tooltip_inputs
            .get_mut(&2245)
            .expect("currency tooltip input");
        inputs.flags_0 = Some(1 << 24);
        inputs.flags_1 = Some(0);
    }
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[2].leftText == ACCOUNT_LEVEL_CURRENCY, 'Flags_0 bit 24 should emit the account-level row')
        "#,
    )
    .unwrap();

    env.state()
        .borrow_mut()
        .currency_tooltip_inputs
        .get_mut(&2245)
        .expect("currency tooltip input")
        .flags_0 = Some(1 << 22);
    env.state()
        .borrow_mut()
        .currency_tooltip_inputs
        .get_mut(&2245)
        .expect("currency tooltip input")
        .flags_1 = Some(1 << 3);
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        for _, line in ipairs(data.lines) do
            assert(line.type ~= 1, 'Flags_0 bit 22 must suppress the spacer')
        end
        "#,
    )
    .unwrap();
}
