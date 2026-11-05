//! WM107: the original `SetPvpBrawl` accessor composes the brawl rows with the
//! native line types, gate order and active/special-event differences.

#![cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::state::{PvpBrawlInfo, SpecialEventBrawlInfo};

tooltip_selection_getters_fixture! {
    fn active_brawl(
        name: &str,
        map: &str,
        description: &str,
        can_queue: bool,
        remaining: Option<f64>,
    ) -> PvpBrawlInfo {
        PvpBrawlInfo {
            brawl_id: 701,
            name: name.to_string(),
            short_description: "never shown".to_string(),
            long_description: description.to_string(),
            can_queue,
            min_level: 10,
            max_level: 80,
            groups_allowed: true,
            cross_faction_allowed: false,
            time_left_until_next_change: remaining,
            brawl_type: 2,
            map_names: vec![map.to_string()],
            includes_all_arenas: false,
            min_item_level: 300.0,
            should_hide_reward_icon: true,
        }
    }

    fn special_brawl(name: &str, description: &str) -> SpecialEventBrawlInfo {
        SpecialEventBrawlInfo {
            brawl_id: 702,
            name: name.to_string(),
            short_description: "never shown".to_string(),
            long_description: description.to_string(),
            can_queue: false,
            min_level: 10,
            max_level: 80,
            groups_allowed: false,
            cross_faction_allowed: true,
            time_left_until_next_change: None,
            brawl_type: 3,
            map_names: vec!["Arathi Basin".to_string(), "Deepwind Gorge".to_string()],
            includes_all_arenas: true,
            min_item_level: 300,
            should_hide_reward_icon: true,
        }
    }

    #[test]
    fn tooltip_pvp_brawl_active_record_composes_the_verified_rows() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.state().borrow_mut().active_brawl = Some(active_brawl(
            "Southshore vs. Tarren Mill",
            "Tarren Mill",
            "Team Deathmatch rules.",
            true,
            Some(7200.0),
        ));
        env.exec(
            r#"
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetPvpBrawl()
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetPvpBrawl', 'pvp-brawl accessor missing')
            assert(data and data.type == Enum.TooltipDataType.PvPBrawl, 'brawl tooltip type')
            assert(data.id == nil, 'brawl tooltip has no identity id')
            local lines = data.lines
            local function dump()
                local parts = {}
                for index, line in ipairs(lines) do
                    parts[index] = ('%d:%s|%s|%s,%s,%s'):format(index, tostring(line.type),
                        tostring(line.leftText), tostring(line.leftColor and line.leftColor.r),
                        tostring(line.leftColor and line.leftColor.g),
                        tostring(line.leftColor and line.leftColor.b))
                end
                return table.concat(parts, ' ')
            end
            assert(#lines == 8, 'active brawl row count: ' .. #lines)
            assert(lines[1].type == 0 and lines[1].leftText == 'Southshore vs. Tarren Mill',
                'raw record name is a plain line: ' .. dump())
            assert(lines[1].leftColor.b > 0.9,
                ('name uses the highlight colour: %s,%s,%s'):format(
                    tostring(lines[1].leftColor.r), tostring(lines[1].leftColor.g),
                    tostring(lines[1].leftColor.b)))
            assert(lines[2].type == 42, 'ends-in must serialize as DisabledLine: ' .. tostring(lines[2].type))
            assert(lines[2].leftText:find('Ends in', 1, true), 'ends-in text: ' .. tostring(lines[2].leftText))
            assert(math.abs(lines[2].leftColor.r - 0.5019608) < 1e-3
                and math.abs(lines[2].leftColor.g - 0.5019608) < 1e-3
                and math.abs(lines[2].leftColor.b - 0.5019608) < 1e-3,
                'ends-in uses the trivial/disabled colour')
            assert(lines[3].type == 1 and lines[3].leftText == ' '
                and lines[3].leftColor.b < 0.1,
                'blank spacer with the normal colour: ' .. dump())
            assert(lines[4].type == 0 and lines[4].leftText == BRAWL_TOOLTIP_MAP,
                'single-map heading')
            assert(lines[5].type == 0
                and lines[5].leftText == string.format(BRAWL_TOOLTIP_MAP_ENTRY, 'Tarren Mill'),
                'explicit map row')
            assert(lines[6].type == 1 and lines[6].leftText == ' ', 'rules spacer')
            assert(lines[7].type == 0 and lines[7].leftText == BRAWL_TOOLTIP_RULES,
                'rules heading')
            assert(lines[8].type == 0 and lines[8].leftText == 'Team Deathmatch rules.',
                'long description row')
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_pvp_brawl_special_record_omits_ends_in_and_forces_queue_true() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.state().borrow_mut().special_event_brawl =
            Some(special_brawl("Arathi Basin Brawl", "Control the bases."));
        env.exec(
            r#"
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetPvpBrawl(true)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetPvpBrawl', 'pvp-brawl accessor missing')
            assert(data and data.type == Enum.TooltipDataType.PvPBrawl)
            local lines = data.lines
            local function dump()
                local parts = {}
                for index, line in ipairs(lines) do
                    parts[index] = ('%d:%s|%s|%s,%s,%s'):format(index, tostring(line.type),
                        tostring(line.leftText), tostring(line.leftColor and line.leftColor.r),
                        tostring(line.leftColor and line.leftColor.g),
                        tostring(line.leftColor and line.leftColor.b))
                end
                return table.concat(parts, ' ')
            end
            assert(#lines == 7, 'special brawl row count: ' .. #lines)
            assert(lines[1].leftText == 'Arathi Basin Brawl', 'special brawl name')
            for _, line in ipairs(lines) do
                assert(line.type ~= 42, 'special brawl never emits the ends-in row')
            end
            assert(lines[2].type == 1 and lines[2].leftText == ' ')
            assert(lines[3].type == 0 and lines[3].leftText == BRAWL_TOOLTIP_MAPS,
                'plural map heading: ' .. dump())
            assert(lines[4].type == 0 and lines[4].leftText == PVP_BRAWL_TOOLTIP_ALL_ARENA_MAPS,
                'all-arena arm emits one row instead of per-map rows')
            assert(lines[5].type == 1 and lines[5].leftText == ' ')
            assert(lines[6].type == 0 and lines[6].leftText == BRAWL_TOOLTIP_RULES)
            -- The record has canQueue false; the special-event path forces it true.
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_pvp_brawl_nonqueueable_active_record_is_a_miss() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.state().borrow_mut().active_brawl = Some(active_brawl(
            "Closed Brawl",
            "Closed map",
            "Closed rules.",
            false,
            Some(0.0),
        ));
        env.exec(
            r#"
            assert(select('#', C_TooltipInfo.GetPvpBrawl()) == 0,
                'a nonqueueable active record must take the missing route')
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetPvpBrawl()
            assert(GameTooltip:GetPrimaryTooltipData() == nil, 'closed brawl must not show a tooltip')
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_pvp_brawl_missing_record_is_a_miss() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            assert(select('#', C_TooltipInfo.GetPvpBrawl()) == 0, 'no active record')
            assert(select('#', C_TooltipInfo.GetPvpBrawl(true)) == 0, 'no special record')
            "#,
        )
        .unwrap();
    }
}
