#![cfg(all(
    feature = "optional-tests",
    any(
        feature = "client-retail",
        feature = "client-ptr",
        feature = "client-wowforever",
        feature = "client-mists"
    )
))]

#[test]
fn selected_chat_message_consumer_runs_original_filter_owner() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape(
        &["Blizzard_ChatFrame"],
        &[],
        |env, loaded| {
            let owner_position = loaded
                .iter()
                .position(|name| name == "Blizzard_ChatFrameBase")
                .expect("ChatFrameBase must load");
            let consumer_position = loaded
                .iter()
                .position(|name| name == "Blizzard_ChatFrame")
                .expect("ChatFrame must load");
            assert!(
                owner_position < consumer_position,
                "ChatFrameBase must load before its ChatFrame consumer: {loaded:?}"
            );
            env.exec(
                r#"
                local ownerSource = debug.getinfo(ChatFrameUtil.ProcessMessageEventFilters, "S").source:gsub("\\", "/")
                assert(ownerSource:find("Blizzard_ChatFrameBase/Shared/ChatFrameFilters.lua", 1, true), ownerSource)
                local consumerSource = debug.getinfo(ChatFrameMixin.MessageEventHandler, "S").source:gsub("\\", "/")
                assert(consumerSource:find("ChatFrameOverrides.lua", 1, true), consumerSource)

                local calls = 0
                local callback = function()
                    calls = calls + 1
                    return true
                end
                ChatFrameUtil.AddMessageEventFilter("CHAT_MSG_SAY", callback)
                local ok, discarded = pcall(ChatFrameMixin.MessageEventHandler, {}, "CHAT_MSG_SAY",
                    "probe", "sender", nil, nil, nil, nil, 0, 0,
                    nil, nil, nil, nil, nil, nil, false, nil, nil, { userID = 0 })
                ChatFrameUtil.RemoveMessageEventFilter("CHAT_MSG_SAY", callback)

                assert(ok, tostring(discarded))
                assert(discarded == true, "original chat consumer must discard the filtered message")
                assert(calls == 1, "original filter registry must invoke the registered callback once")
                "#,
            )
            .expect("selected ChatFrame consumer should use the original filter owner");
        },
    );
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn selected_communities_consumer_uses_original_channel_color_and_local_id() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape(
        &["Blizzard_Communities"],
        &[],
        |env, loaded| {
            let owner_position = loaded
                .iter()
                .position(|name| name == "Blizzard_ChatFrameBase")
                .expect("ChatFrameBase must load");
            let consumer_position = loaded
                .iter()
                .position(|name| name == "Blizzard_Communities")
                .expect("Communities must load");
            assert!(
                owner_position < consumer_position,
                "ChatFrameBase must load before Communities: {loaded:?}"
            );
            env.exec(
                r#"
                local colorSource = debug.getinfo(ChatFrameUtil.GetCommunitiesChannelColor, "S").source:gsub("\\", "/")
                assert(colorSource:find("Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua", 1, true), colorSource)
                local localIDSource = debug.getinfo(ChatFrameUtil.GetCommunitiesChannelLocalID, "S").source:gsub("\\", "/")
                assert(localIDSource:find("Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua", 1, true), localIDSource)
                local consumerSource = debug.getinfo(CommunitiesStreamDropdownMixin.SetupMenu, "S").source:gsub("\\", "/")
                assert(consumerSource:find("Blizzard_Communities/CommunitiesStreams.lua", 1, true), consumerSource)

                local stream = { streamId = 77, name = "E2 probe", streamType = Enum.ClubStreamType.Other }
                local communitiesFrame = {
                    GetSelectedClubId = function() return 41 end,
                    GetSelectedStreamForClub = function() return stream end,
                    GetPrivilegesForClub = function() return { canCreateStream = false } end,
                }
                local dropdown = { GetCommunitiesFrame = function() return communitiesFrame end }
                local displayedText

                ChatTypeInfo.OFFICER = { r = 0.12, g = 0.34, b = 0.56 }
                ChatFrameUtil.GetCommunitiesChannel = function() return nil end
                ChatFrameUtil.GetCommunitiesChannelName = function() return "E2 probe" end
                GetChannelName = function() return 73 end
                C_Club.GetClubInfo = function() return { clubType = Enum.ClubType.Guild } end
                C_Club.GetStreamInfo = function() return { streamType = Enum.ClubStreamType.Officer } end
                C_Club.GetStreams = function() return { stream } end
                C_Club.GetClubLimits = function() return { maximumNumberOfStreams = 0 } end
                CommunitiesUtil.SortStreams = function() end
                CommunitiesUtil.GetStreamNotificationSettingsLookup = function() return {} end
                DropdownButtonMixin.SetupMenu = function(self, createMenu)
                    local root = {
                        SetTag = function() end,
                        CreateRadio = function(_, text)
                            displayedText = text
                            return { AddInitializer = function() end }
                        end,
                        CreateButton = function()
                            return { AddInitializer = function() end }
                        end,
                    }
                    createMenu(self, root)
                end

                CommunitiesStreamDropdownMixin.SetupMenu(dropdown)
                local expected = CreateColor(0.12, 0.34, 0.56):WrapTextInColorCode("E2 probe")
                    .. " " .. GRAY_FONT_COLOR:WrapTextInColorCode(COMMUNITIES_STREAM0_CHAT_SHORTCUT_FORMAT:format(73))
                assert(displayedText == expected,
                    "original Communities consumer should show the owner-provided color and local ID")
                "#,
            )
            .expect("selected Communities consumer should use original ChatFrameUtil owners");
        },
    );
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn selected_combat_log_preview_uses_original_inbound_owner() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape(
        &["Blizzard_CombatLogProcessor", "Blizzard_ChatFrame"],
        &[],
        |env, _loaded| {
            env.exec(
                r#"
                local inboundSource = debug.getinfo(CombatLogInbound.GenerateMessage, "S").source:gsub("\\", "/")
                assert(inboundSource:find("Blizzard_CombatLogProcessor/Blizzard_CombatLogProcessorInbound.lua", 1, true), inboundSource)
                local consumerSource = debug.getinfo(CombatConfig_Colorize_Update, "S").source:gsub("\\", "/")
                assert(consumerSource:find("Blizzard_ChatFrame/Mainline/ChatConfigFrame.lua", 1, true), consumerSource)

                local function widget()
                    return {
                        SetChecked = function() end,
                        SetEnabled = function() end,
                        SetVertexColor = function() end,
                    }
                end
                for _, name in ipairs({
                    "CombatConfigColorsColorizeUnitNameCheck",
                    "CombatConfigColorsColorizeSpellNamesCheck",
                    "CombatConfigColorsColorizeSpellNamesSchoolColoring",
                    "CombatConfigColorsColorizeSpellNamesColorSwatchNormalTexture",
                    "CombatConfigColorsColorizeDamageNumberCheck",
                    "CombatConfigColorsColorizeDamageNumberSchoolColoring",
                    "CombatConfigColorsColorizeDamageNumberColorSwatchNormalTexture",
                    "CombatConfigColorsColorizeDamageSchoolCheck",
                    "CombatConfigColorsColorizeEntireLineCheck",
                    "CombatConfigColorsColorizeEntireLineBySource",
                    "CombatConfigColorsColorizeEntireLineByTarget",
                    "CombatConfigColorsHighlightingLine",
                    "CombatConfigColorsHighlightingAbility",
                    "CombatConfigColorsHighlightingDamage",
                    "CombatConfigColorsHighlightingSchool",
                }) do
                    _G[name] = widget()
                end

                local previewTexts = {}
                for index = 1, 2 do
                    _G["CombatConfigColorsExampleString" .. index] = {
                        SetVertexColor = function() end,
                        SetText = function(_, text) previewTexts[index] = text end,
                    }
                end

                CHATCONFIG_SELECTED_FILTER = {
                    settings = {
                        timestamp = false,
                        hideBuffs = false,
                        hideDebuffs = false,
                        fullText = false,
                        unitColoring = false,
                        abilityColoring = false,
                        abilitySchoolColoring = false,
                        amountColoring = false,
                        amountSchoolColoring = false,
                        schoolNameColoring = false,
                        lineColoring = false,
                        lineColorPriority = 1,
                        lineHighlighting = false,
                        abilityHighlighting = false,
                        amountHighlighting = false,
                        schoolNameHighlighting = false,
                    },
                    colors = {
                        defaults = {
                            spell = { r = 1, g = 1, b = 0 },
                            damage = { r = 1, g = 0, b = 0 },
                        },
                        highlightedEvents = {},
                    },
                }

                local ok, errorText = xpcall(CombatConfig_Colorize_Update, debug.traceback)
                assert(ok, errorText)
                assert(type(previewTexts[1]) == "string" and previewTexts[1] ~= "",
                    "original combat-log preview should generate player-event text")
                assert(type(previewTexts[2]) == "string" and previewTexts[2] ~= "",
                    "original combat-log preview should generate creature-event text")
                "#,
            )
            .expect("selected chat-config consumer should use the original inbound formatter");
        },
    );
}
