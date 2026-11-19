#![cfg(any(feature = "client-wowforever", feature = "client-mists"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

#[cfg(feature = "client-wowforever")]
#[test]
fn tooltip_processor_post_call_runs_after_original_item_setter() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.exec(
        r#"
        assert(TooltipDataProcessor.AllTypes == "ALL", "original processor must load")
        local source = debug.getinfo(AddTooltipDataAccessor, "S").source
        assert(string.find(source, "TooltipDataHandler.lua", 1, true), "original handler must load")
        local calls, observed_id, observed_lines = 0, nil, 0
        TooltipDataProcessor.AddTooltipPostCall(Enum.TooltipDataType.Item, function(tooltip, data)
            calls = calls + 1
            observed_id = data and data.id
            observed_lines = data and #data.lines or 0
        end)
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        GameTooltip:SetItemByID(6948)
        assert(calls == 1, "the item post-call must run once")
        assert(observed_id == 6948 and observed_lines > 0, "the callback must observe populated item data")
        "#,
    )
    .expect("original tooltip post-call must observe item data");
}

#[cfg(feature = "client-mists")]
#[test]
fn mists_loads_tooltip_processor_from_original_handler() {
    let env = tooltip_full_env_helpers::setup_full_env();
    let result: (String, String) = env
        .eval(
            "return TooltipDataProcessor.AllTypes, \
             debug.getinfo(AddTooltipDataAccessor, 'S').source",
        )
        .expect("original Mists tooltip handler should load");
    assert_eq!(result.0, "ALL");
    assert!(result.1.contains("TooltipDataHandler.lua"), "{}", result.1);
}
