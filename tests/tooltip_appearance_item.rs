//! Original dress-up hover and tooltip accessor over supplied appearance sources.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::c_api::c_transmog_collection::AppearanceSourceInfo;

#[test]
fn tooltip_appearance_item_original_hover_reads_current_source_item() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    for (source_id, item_id) in [(90001, 1728), (90002, 127830)] {
        env.state().borrow_mut().transmog_appearance_sources.insert(
            source_id,
            AppearanceSourceInfo {
                category: 1,
                item_appearance_id: 0,
                can_have_illusion: false,
                icon: 0,
                is_collected: false,
                item_link: format!("item:{item_id}"),
                transmoglink: String::new(),
                source_type: None,
                item_subclass: 0,
                ignore_model_attachment_checks_for_illusion: false,
            },
        );
    }
    env.exec(
        r#"
        assert(DressUpFrameTransmogSetButtonMixin, 'original dress-up mixin must load')
        local button = CreateFrame('Button', nil, UIParent)
        button.HighlightTexture = button:CreateTexture()
        for _, case in ipairs({{90001, 1728}, {90002, 127830}}) do
            button.elementData = {itemModifiedAppearanceID = case[1]}
            DressUpFrameTransmogSetButtonMixin.OnEnter(button)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetItemByItemModifiedAppearanceID', 'appearance accessor missing')
            assert(data and data.id == case[2], 'must use selected appearance source item')
            local expected = C_TooltipInfo.GetItemByID(case[2])
            assert(#data.lines == #expected.lines and #data.lines > 0)
            for index, line in ipairs(expected.lines) do
                assert(data.lines[index].leftText == line.leftText, 'must preserve generated catalog lines')
            end
            assert(GameTooltip:GetLeftLine(1):GetText() == expected.lines[1].leftText)
            assert(GameTooltip:IsShown())
            DressUpFrameTransmogSetButtonMixin.OnLeave(button)
            assert(not GameTooltip:IsShown())
        end
        assert(C_TooltipInfo.GetItemByItemModifiedAppearanceID(999999999) == nil)
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .transmog_appearance_sources
        .get_mut(&90001)
        .unwrap()
        .item_link = "item:127830".into();
    env.exec(
        "GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE'); \
         GameTooltip:SetItemByItemModifiedAppearanceID(90001); \
         assert(GameTooltip:GetPrimaryTooltipData().id == 127830)",
    )
    .unwrap();
}
