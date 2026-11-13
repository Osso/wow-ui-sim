use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

// PRESERVATION: FrameXMLUtil's selected owner supplies the open helpers before
// UIPanels_Game registers the item-reference consumers that call them.
#[test]
fn selected_itemref_open_helpers_do_not_toggle_visible_tabs() {
    with_blizzard_addon_smoke_shape(
        &["Blizzard_UIPanels_Game", "Blizzard_PlayerSpells"],
        &[],
        |env, loaded| {
            assert!(
                loaded.iter().any(|name| name == "Blizzard_PlayerSpells"),
                "the LoadOnDemand PlayerSpells owner must load"
            );

            let (talents_open, specializations_open): (bool, bool) = env
                .eval(
                    r#"
                    assert(C_AddOns.IsAddOnLoaded("Blizzard_FrameXMLUtil"),
                        "FrameXMLUtil must load before its item-reference consumers")
                    assert(C_AddOns.IsAddOnLoaded("Blizzard_UIPanels_Game"),
                        "the original item-reference consumers must load")
                    assert(C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells"),
                        "PlayerSpells must be runtime-loaded")

                    local ownerSuffix = "Blizzard_FrameXMLUtil/Mainline/PlayerSpellsUtil.lua"
                    local consumerSuffix = "Blizzard_UIPanels_Game/Mainline/ItemRefHandlers.lua"
                    local function source_of(value)
                        return debug.getinfo(value, "S").source:gsub("\\", "/")
                    end
                    local function assert_original_owner(helper)
                        local source = source_of(helper)
                        assert(source:sub(-#ownerSuffix) == ownerSuffix,
                            "expected the original FrameXMLUtil owner, got " .. source)
                    end
                    local function original_link_handler(linkType)
                        for index = 1, 16 do
                            local name, value = debug.getupvalue(LinkUtil.ProcessLink, index)
                            if not name then
                                break
                            end
                            if type(value) == "table" and type(value[linkType]) == "function" then
                                return value[linkType]
                            end
                        end
                        error("missing registered item-reference handler for " .. linkType)
                    end
                    local function open_link_twice(linkType, helper, tab)
                        assert_original_owner(helper)
                        assert(LinkUtil.IsLinkHandlerRegistered(linkType),
                            "the selected item-reference handler must be registered")
                        local handler = original_link_handler(linkType)
                        local consumerSource = source_of(handler)
                        assert(consumerSource:sub(-#consumerSuffix) == consumerSuffix,
                            "expected the original ItemRefHandlers consumer, got " .. consumerSource)

                        local frame = PlayerSpellsFrame
                        assert(frame, "PlayerSpells must create its panel")
                        if frame:IsShown() then
                            HideUIPanel(frame)
                        end

                        LinkUtil.ProcessLink(linkType .. ":", "", nil)
                        local opened = frame:IsShown() and frame:IsFrameTabActive(tab)
                        LinkUtil.ProcessLink(linkType .. ":", "", nil)
                        local remainedOpen = frame:IsShown() and frame:IsFrameTabActive(tab)
                        return opened and remainedOpen
                    end

                    local talentsOpen = open_link_twice(
                        LinkTypes.TalentsUI,
                        PlayerSpellsUtil.OpenToClassTalentsTab,
                        PlayerSpellsUtil.FrameTabs.ClassTalents
                    )
                    local specializationsOpen = open_link_twice(
                        LinkTypes.SpecializationsUI,
                        PlayerSpellsUtil.OpenToClassSpecializationsTab,
                        PlayerSpellsUtil.FrameTabs.ClassSpecializations
                    )
                    return talentsOpen, specializationsOpen
                    "#,
                )
                .expect("the selected ItemRef consumers must open PlayerSpells tabs");

            assert!(
                talents_open,
                "the original talent link must leave the already-open talent tab visible"
            );
            assert!(
                specializations_open,
                "the original specialization link must leave the already-open specialization tab visible"
            );
        },
    );
}
