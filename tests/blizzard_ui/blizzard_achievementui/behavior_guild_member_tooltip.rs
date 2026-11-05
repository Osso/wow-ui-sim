//! Behavior pin: the guild-members tooltip path on a guild achievement
//! reads `GetGuildAchievementNumMembers(id)` then iterates
//! `GetGuildAchievementMemberInfo(id, i)` to populate `GameTooltip`
//! lines, BUT only when `InGuildView()` is true AND the achievement is
//! completed AND the achievement's flags include
//! `ACHIEVEMENT_FLAGS_SHOW_GUILD_MEMBERS = 0x00008000` AND `numMembers
//! > 0`. The PLAN-named entry point `AchievementButton_OnEnter` does
//! NOT exist; the actual entry chain is `AchievementShield_OnEnter`
//! (lua:3124-3154) which calls
//! `AchievementFrameAchievements_CheckGuildMembersTooltip(self)` at
//! lua:3152, and the function under test is the latter at lua:3167-3218.
//!
//! Two distinct edges are pinned across two tests so each body stays
//! comfortably under the readability budget.
//!
//! 1. **Gating edge** — see
//!    `check_guild_members_tooltip_no_ops_outside_guild_view_and_when_num_members_is_zero`.
//!    Pins THREE gates: (a) the `InGuildView()` outer gate at lua:3168
//!    short-circuits the entire function when
//!    `achievementFunctions ~= GUILD_ACHIEVEMENT_FUNCTIONS`; (b) the
//!    `achievementCompleted AND
//!    bit.band(flags, ACHIEVEMENT_FLAGS_SHOW_GUILD_MEMBERS)` gate at
//!    lua:3172 (without flag, the function falls through to the
//!    elseif-criteria-members branch at lua:3199 — not the guild-members
//!    branch); (c) the `numMembers == 0` gate at lua:3174-3177 records
//!    the requesting frame and calls `GetGuildAchievementMembers(id)`
//!    to ask the server, but does NOT iterate or write tooltip lines.
//!
//! 2. **Iteration edge** — see
//!    `check_guild_members_tooltip_pairs_odd_index_left_with_even_index_right_via_add_double_line`.
//!    Pins the AddLine/AddDoubleLine pairing at lua:3183-3196: the
//!    leading `AddLine(GUILD_ACHIEVEMENT_EARNED_BY, 1, 1, 1)` at
//!    lua:3183 (preceded by an `AddLine(" ")` separator at lua:3181 if
//!    the tooltip already has lines), then the iteration at lua:3185-3192
//!    accumulates odd-index names into `leftMemberName` and pairs them
//!    with the next even-index name via `AddDoubleLine`; lua:3194-3196
//!    flushes a leftover odd name with a single `AddLine`. With three
//!    members the contract is: `AddLine(EARNED_BY)`,
//!    `AddDoubleLine("Member1", "Member2")`, `AddLine("Member3")`.
//!
//! **PLAN-named tripwire.** PLAN refers to `AchievementButton_OnEnter`
//! as if it were a guild-aware tooltip handler, but no such global
//! exists in `Mainline/Blizzard_AchievementUI.lua`. The guild-members
//! tooltip path is entered from `AchievementShield_OnEnter` (lua:3124)
//! which calls `_CheckGuildMembersTooltip` (lua:3167) at lua:3152 — a
//! `_G.AchievementButton_OnEnter` lookup MUST stay nil so a future
//! refactor that introduces such a global is flagged loudly.
//!
//! **`depends-on: guild achievement member trio gap` is stale.** All
//! three guild-member globals (`GetGuildAchievementNumMembers`,
//! `GetGuildAchievementMembers`, `GetGuildAchievementMemberInfo`) are
//! already wired by `register_guild_member_globals` at
//! `src/lua_api/globals/missing_surface/achievement_info.rs:295-315`,
//! with backing impls at `:808-845`. The default
//! `state.guild_achievement_members` map is empty, so an unstubbed
//! call would return `0` for the count and `nil` for member info —
//! safe but unhelpful for pinning the iteration shape, hence the
//! Lua-level spies in test 2.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AchievementUI";
const PLAN_NAMED_BUT_ABSENT: &str = "AchievementButton_OnEnter";
const EXPECTED_GATING: &str = "outside_num_called=0 outside_addline_called=0 \
    inside_zero_num_called=1 inside_zero_addline_called=0 inside_zero_double_called=0";
const EXPECTED_ITERATION: &str = "addline_count=2 first_line=GUILD_EARNED_BY_TOKEN \
    last_line=Member3 double_count=1 double_pair=Member1|Member2 num_called=1 info_calls=3";

type GatingProbe = (String, String, String, String);
type IterationProbe = (String, String, String, String);

timerunning_util_panel_fixture! {
#[test]
fn check_guild_members_tooltip_no_ops_outside_guild_view_and_when_num_members_is_zero() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: GatingProbe = env
            .eval(
                r#"
                assert(_G.AchievementFrameAchievements_CheckGuildMembersTooltip,
                    "AchievementFrameAchievements_CheckGuildMembersTooltip must exist (lua:3167)")
                assert(GameTooltip, "GameTooltip global must exist")
                assert(_G.ACHIEVEMENT_FLAGS_SHOW_GUILD_MEMBERS,
                    "ACHIEVEMENT_FLAGS_SHOW_GUILD_MEMBERS must exist (Constants.lua:121)")

                local num_calls = 0
                local original_num = _G.GetGuildAchievementNumMembers
                _G.GetGuildAchievementNumMembers = function(id)
                    num_calls = num_calls + 1
                    return 0
                end

                local original_get_members = _G.GetGuildAchievementMembers
                _G.GetGuildAchievementMembers = function(id) end

                local original_info = _G.GetAchievementInfo
                _G.GetAchievementInfo = function(id)
                    return id, "stub_name", 10, true, 0, 0, 0, "stub_desc",
                        _G.ACHIEVEMENT_FLAGS_SHOW_GUILD_MEMBERS,
                        "Interface\\Icons\\stub", 0, false, false, false
                end

                local addline_calls = 0
                local double_calls = 0
                local original_addline = GameTooltip.AddLine
                local original_double = GameTooltip.AddDoubleLine
                GameTooltip.AddLine = function(self, ...) addline_calls = addline_calls + 1 end
                GameTooltip.AddDoubleLine = function(self, ...)
                    double_calls = double_calls + 1
                end

                local function find_upvalue_index(fn, name)
                    local i = 1
                    while true do
                        local up_name = debug.getupvalue(fn, i)
                        if not up_name then return nil end
                        if up_name == name then return i end
                        i = i + 1
                    end
                end
                local upvalue_idx = find_upvalue_index(
                    _G.AchievementFrameBaseTab_OnClick, "achievementFunctions")
                local original_funcs = select(2,
                    debug.getupvalue(_G.AchievementFrameBaseTab_OnClick, upvalue_idx))

                local stub_self = {id = 4242}

                AchievementFrameAchievements_CheckGuildMembersTooltip(stub_self)
                local outside_num_called = num_calls
                local outside_addline_called = addline_calls

                debug.setupvalue(_G.AchievementFrameBaseTab_OnClick,
                    upvalue_idx, _G.GUILD_ACHIEVEMENT_FUNCTIONS)
                num_calls, addline_calls, double_calls = 0, 0, 0
                AchievementFrameAchievements_CheckGuildMembersTooltip(stub_self)
                local inside_zero_signature = string.format(
                    "outside_num_called=%d outside_addline_called=%d " ..
                    "inside_zero_num_called=%d inside_zero_addline_called=%d " ..
                    "inside_zero_double_called=%d",
                    outside_num_called, outside_addline_called,
                    num_calls, addline_calls, double_calls)

                _G.GetGuildAchievementNumMembers = original_num
                _G.GetGuildAchievementMembers = original_get_members
                _G.GetAchievementInfo = original_info
                GameTooltip.AddLine = original_addline
                GameTooltip.AddDoubleLine = original_double
                debug.setupvalue(_G.AchievementFrameBaseTab_OnClick,
                    upvalue_idx, original_funcs)

                return type(_G.AchievementFrameAchievements_CheckGuildMembersTooltip),
                       type(_G.AchievementButton_OnEnter),
                       type(_G.AchievementShield_OnEnter),
                       inside_zero_signature
                "#,
            )
            .expect("setup + outside-then-inside-zero drive must run cleanly");

        let (check_function_type, plan_named_type, shield_handler_type, gating_signature) =
            observations;

        assert_eq!(
            check_function_type, "function",
            "Expected `_G.AchievementFrameAchievements_CheckGuildMembersTooltip` to be a function \
             (declared at `Mainline/Blizzard_AchievementUI.lua:3167`). Got \
             `{check_function_type}`. A `nil` reading means `AchievementShield_OnEnter` at \
             lua:3152 would crash with `attempt to call a nil value` for every shield-hover \
             event on guild achievements."
        );

        assert_eq!(
            plan_named_type, "nil",
            "Expected `_G.{PLAN_NAMED_BUT_ABSENT}` to be nil — PLAN refers to it as if it were \
             a guild-aware tooltip handler, but no such global exists in \
             `Mainline/Blizzard_AchievementUI.lua`. The actual guild-members tooltip path is \
             entered from `AchievementShield_OnEnter` (lua:3124) which calls \
             `AchievementFrameAchievements_CheckGuildMembersTooltip` (lua:3167) at lua:3152. \
             Got `{plan_named_type}` — a non-nil reading means a future refactor introduced an \
             alias under the PLAN-named symbol; flag the rename and update PLAN wording rather \
             than silently assuming this assertion can be deleted."
        );

        assert_eq!(
            shield_handler_type, "function",
            "Expected `_G.AchievementShield_OnEnter` to be a function (declared at lua:3124, the \
             actual entry point that PLAN's `AchievementButton_OnEnter` was misnaming). Got \
             `{shield_handler_type}`. A `nil` reading means the shield-icon hover path that \
             surfaces the guild-members tooltip would no-op entirely; users hovering guild \
             achievements with the SHOW_GUILD_MEMBERS flag set would see no member list."
        );

        assert_eq!(
            gating_signature, EXPECTED_GATING,
            "Expected outside-guild-view drive (achievementFunctions = ACHIEVEMENT_FUNCTIONS) to \
             record zero calls to GetGuildAchievementNumMembers and zero AddLine calls (the \
             `InGuildView()` outer gate at lua:3168 short-circuits the entire body), and \
             inside-guild-view + numMembers=0 drive to record exactly one call to \
             GetGuildAchievementNumMembers but zero AddLine/AddDoubleLine calls (the \
             `numMembers == 0` gate at lua:3174 takes the request-from-server branch at \
             lua:3175-3177 instead of the iterate-and-write branch at lua:3179-3196). Expected \
             `{EXPECTED_GATING}`. Got `{gating_signature}`. A non-zero `outside_num_called` \
             means the InGuildView gate at lua:3168 leaks — the tooltip would fire member \
             queries for player-tab achievements. A non-zero `inside_zero_addline_called` \
             means the numMembers==0 short-circuit at lua:3174-3177 leaks — the tooltip would \
             write a header line for an empty list. A `inside_zero_num_called=0` means the \
             completed-AND-flag gate at lua:3172 short-circuited before reaching the count \
             read; the spy on GetAchievementInfo returned completed=true with the GUILD_MEMBERS \
             flag, so this would mean the `bit.band` check is broken or the flag constant \
             changed."
        );
    });
}
}

timerunning_util_panel_fixture! {
#[test]
fn check_guild_members_tooltip_pairs_odd_index_left_with_even_index_right_via_add_double_line() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: IterationProbe = env
            .eval(
                r#"
                assert(_G.AchievementFrameAchievements_CheckGuildMembersTooltip,
                    "AchievementFrameAchievements_CheckGuildMembersTooltip must exist (lua:3167)")

                local original_earned_by = _G.GUILD_ACHIEVEMENT_EARNED_BY
                _G.GUILD_ACHIEVEMENT_EARNED_BY = "GUILD_EARNED_BY_TOKEN"

                local num_calls = 0
                local original_num = _G.GetGuildAchievementNumMembers
                _G.GetGuildAchievementNumMembers = function(id)
                    num_calls = num_calls + 1
                    return 3
                end

                local info_calls = {indices = {}}
                local original_info_member = _G.GetGuildAchievementMemberInfo
                _G.GetGuildAchievementMemberInfo = function(id, i)
                    info_calls.indices[#info_calls.indices + 1] = i
                    return "Member" .. tostring(i)
                end

                local original_info = _G.GetAchievementInfo
                _G.GetAchievementInfo = function(id)
                    return id, "stub_name", 10, true, 0, 0, 0, "stub_desc",
                        _G.ACHIEVEMENT_FLAGS_SHOW_GUILD_MEMBERS,
                        "Interface\\Icons\\stub", 0, false, false, false
                end

                local addline_capture = {lines = {}}
                local original_addline = GameTooltip.AddLine
                GameTooltip.AddLine = function(self, text, ...)
                    addline_capture.lines[#addline_capture.lines + 1] = tostring(text)
                end
                local double_capture = {pairs = {}}
                local original_double = GameTooltip.AddDoubleLine
                GameTooltip.AddDoubleLine = function(self, left, right, ...)
                    double_capture.pairs[#double_capture.pairs + 1] =
                        tostring(left) .. "|" .. tostring(right)
                end
                local original_num_lines = GameTooltip.NumLines
                GameTooltip.NumLines = function(self) return 0 end

                local function find_upvalue_index(fn, name)
                    local i = 1
                    while true do
                        local up_name = debug.getupvalue(fn, i)
                        if not up_name then return nil end
                        if up_name == name then return i end
                        i = i + 1
                    end
                end
                local upvalue_idx = find_upvalue_index(
                    _G.AchievementFrameBaseTab_OnClick, "achievementFunctions")
                local original_funcs = select(2,
                    debug.getupvalue(_G.AchievementFrameBaseTab_OnClick, upvalue_idx))
                debug.setupvalue(_G.AchievementFrameBaseTab_OnClick,
                    upvalue_idx, _G.GUILD_ACHIEVEMENT_FUNCTIONS)

                local stub_self = {id = 4242}
                AchievementFrameAchievements_CheckGuildMembersTooltip(stub_self)

                local first_line = addline_capture.lines[1] or "<missing>"
                local last_line = addline_capture.lines[#addline_capture.lines] or "<missing>"
                local first_pair = double_capture.pairs[1] or "<missing>"
                local iteration_signature = string.format(
                    "addline_count=%d first_line=%s last_line=%s " ..
                    "double_count=%d double_pair=%s num_called=%d info_calls=%d",
                    #addline_capture.lines, first_line, last_line,
                    #double_capture.pairs, first_pair, num_calls, #info_calls.indices)

                _G.GUILD_ACHIEVEMENT_EARNED_BY = original_earned_by
                _G.GetGuildAchievementNumMembers = original_num
                _G.GetGuildAchievementMemberInfo = original_info_member
                _G.GetAchievementInfo = original_info
                GameTooltip.AddLine = original_addline
                GameTooltip.AddDoubleLine = original_double
                GameTooltip.NumLines = original_num_lines
                debug.setupvalue(_G.AchievementFrameBaseTab_OnClick,
                    upvalue_idx, original_funcs)

                return type(_G.GetGuildAchievementNumMembers),
                       type(_G.GetGuildAchievementMemberInfo),
                       type(_G.GetGuildAchievementMembers),
                       iteration_signature
                "#,
            )
            .expect("setup + three-member iteration drive must run cleanly");

        let (num_members_type, member_info_type, request_members_type, iteration_signature) =
            observations;

        assert_eq!(
            num_members_type, "function",
            "Expected `_G.GetGuildAchievementNumMembers` to be a function (registered at \
             `register_guild_member_globals` in `achievement_info.rs:295-315`, impl at \
             `:808-817`). Got `{num_members_type}`. PLAN's `depends-on: guild achievement \
             member trio gap` is stale — a `nil` reading would mean the registration \
             regressed and the tooltip path at lua:3173 would crash on every guild-shield \
             hover."
        );

        assert_eq!(
            member_info_type, "function",
            "Expected `_G.GetGuildAchievementMemberInfo` to be a function (registered at \
             `:295-315`, impl at `:823-845`). Got `{member_info_type}`. A `nil` reading \
             would mean the iteration loop at lua:3185-3192 would crash before the first \
             member line could be written; PLAN's depends-on tag claims this is gapped but \
             the registration is wired."
        );

        assert_eq!(
            request_members_type, "function",
            "Expected `_G.GetGuildAchievementMembers` to be a function (registered at \
             `:295-315`, impl at `:819-821` — currently a no-op stub representing the \
             server-fetch hook). Got `{request_members_type}`. A `nil` reading would mean \
             the numMembers==0 fallback at lua:3177 (`GetGuildAchievementMembers(achievementId)` \
             — the request-from-server entry point) would crash, leaving the tooltip in a \
             broken state for fresh guild achievements that haven't received the member \
             list yet."
        );

        assert_eq!(
            iteration_signature, EXPECTED_ITERATION,
            "Expected three-member iteration drive (numMembers stubbed to 3, member-info \
             stubbed to return `\"Member\"..i`, GUILD_ACHIEVEMENT_EARNED_BY stubbed to \
             `\"GUILD_EARNED_BY_TOKEN\"`, NumLines stubbed to 0 to suppress the separator at \
             lua:3181) to produce signature `{EXPECTED_ITERATION}`. Got \
             `{iteration_signature}`. The expected sequence pins lua:3183-3196: leading \
             `AddLine(GUILD_ACHIEVEMENT_EARNED_BY)` (lua:3183), one `AddDoubleLine(\"Member1\", \
             \"Member2\")` (lua:3187 — the pairing of i=1's accumulated `leftMemberName` with \
             i=2's fresh fetch), and one trailing `AddLine(\"Member3\")` (lua:3194-3196 — the \
             leftover-odd-name flush after the loop). An `addline_count` other than 2 means \
             either the leading earned-by line at lua:3183 was severed, or the leftover-odd \
             flush at lua:3194-3196 leaked. A `double_count` other than 1 means the i=1/i=2 \
             pairing at lua:3187 fanned out or short-circuited. A `double_pair` other than \
             `Member1|Member2` means the accumulator at lua:3190 stored the wrong index, OR \
             the AddDoubleLine call at lua:3187 swapped left/right (PLAN's wording would \
             allow either). An `info_calls` other than 3 means the loop bound at lua:3185 \
             (`for i = 1, numMembers do`) is broken — the spy on numMembers returned 3 so \
             the loop should fetch each member exactly once."
        );
    });
}
}
