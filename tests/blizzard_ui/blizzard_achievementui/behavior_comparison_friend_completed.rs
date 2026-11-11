//! Behavior pin: PLAN-named "friend-completed checkmark" is misleading
//! — `AchievementComparisonTemplateMixin:Init(elementData)` at
//! `Mainline/Blizzard_AchievementUI.lua:2688-2762` does NOT toggle a
//! checkmark widget. The friend-side visual cue is split across THREE
//! orthogonal writes: (a) `friend.Status:SetText(FormatShortDate(...))`
//! at lua:2752 vs `friend.Status:SetText(INCOMPLETE)` at lua:2758;
//! (b) `friend:Saturate()` at lua:2754 (gated on `saturatedStyle` change
//! at lua:2753) vs `friend:Desaturate()` at lua:2759; (c) `friend.completed`
//! property write at lua:2751 (`true`) vs lua:2757 (`nil`). NO checkmark
//! texture exists in the XML chain — the comparison row is two cards
//! (`Player` left, `Friend` right) and the "completion" indicator is
//! the date string + the saturation state, not a check icon.
//!
//! Source map of the contract:
//!
//! ```lua
//! -- lua:2682 AchievementComparisonTemplateMixin = {};
//! -- lua:2688-2762
//! function AchievementComparisonTemplateMixin:Init(elementData)
//!     local category = elementData.category
//!     local index = elementData.index
//!     local id, name, points, completed, month, day, year, description,
//!         flags, icon, rewardText, isGuild, wasEarnedByMe, earnedBy =
//!             GetAchievementInfo(category, index)         -- lua:2691
//!     assertsafe(id ~= nil, ...)                            -- lua:2693
//!
//!     if ( GetPreviousAchievement(id) ) then               -- lua:2695
//!         points = AchievementButton_GetProgressivePoints(id)
//!     end
//!
//!     if ( self.id ~= id ) then                            -- lua:2700 (cache gate)
//!         self.id = id                                     -- lua:2701
//!         ...
//!         local friendCompleted, friendMonth, friendDay, friendYear =
//!             GetAchievementComparisonInfo(id)             -- lua:2716
//!         ...
//!         if ( friendCompleted ) then                      -- lua:2750
//!             friend.completed = true                      -- lua:2751
//!             friend.Status:SetText(FormatShortDate(
//!                 friendDay, friendMonth, friendYear))     -- lua:2752
//!             if ( friend.saturatedStyle ~= saturatedStyle ) then
//!                 friend:Saturate()                        -- lua:2754
//!             end
//!         else
//!             friend.completed = nil                       -- lua:2757
//!             friend.Status:SetText(INCOMPLETE)            -- lua:2758
//!             friend:Desaturate()                          -- lua:2759
//!         end
//!     end
//! end
//! ```
//!
//! `GetAchievementComparisonInfo(id)` is registered at
//! `register_comparison_getters` in
//! `src/lua_api/globals/missing_surface/achievement_info.rs:339-364`,
//! and `SetAchievementComparisonUnit(unit)` at
//! `register_comparison_unit_mutators` at `:323-336` — the depends-on
//! tag `SetAchievementComparisonUnit gap` is stale on both halves.
//!
//! **Spec/source mismatch on FOUR axes:**
//!
//! 1. **No checkmark widget.** PLAN says "friend-completed checkmark".
//!    No such texture exists in `Blizzard_AchievementUI.xml` near the
//!    comparison template — `grep -n "[Cc]heck" Blizzard_AchievementUI.xml`
//!    returns nothing matching a friend-side check icon. The visual
//!    contract is the date text (when completed) vs `INCOMPLETE` token
//!    (when not), plus the Saturate/Desaturate filter on the whole
//!    Friend card. PLAN's wording would lead a reader to look for a
//!    `friend.Checkmark:Show()/:Hide()` line that doesn't exist.
//! 2. **Three orthogonal writes per branch.** Even granting "checkmark"
//!    as shorthand for "completed marker", PLAN collapses the three
//!    writes (`friend.completed` boolean, `friend.Status:SetText`, and
//!    `friend:Saturate()`/`Desaturate()`) into one. A test that pins
//!    only one of the three would miss regressions on the others.
//! 3. **Cache gate at lua:2700.** PLAN's wording (`Init ... sets the
//!    checkmark from GetAchievementComparisonInfo(id)`) implies a fresh
//!    fetch on every Init call. In reality, lua:2700 short-circuits the
//!    entire body when `self.id == id` — so `GetAchievementComparisonInfo`
//!    fires once per (frame, achievement-id) pair, not once per Init
//!    call. This matters for scroll-list reuse: when the ScrollBox
//!    rebinds the same frame to a different elementData with the same
//!    id, the second Init is a no-op.
//! 4. **Depends-on stale.** `(depends-on: SetAchievementComparisonUnit gap)`
//!    — both `SetAchievementComparisonUnit` (registered at
//!    `achievement_info.rs:323-336`) and `GetAchievementComparisonInfo`
//!    (registered at `:339-364`) are wired. The bare default state has
//!    no comparison data so an unstubbed call would return `nil` for
//!    every field (effectively "incomplete with no date"); the test
//!    installs Lua-level spies to drive both branches deterministically.
//!
//! Two tests split the completed and incomplete branches. Each
//! drives `Mixin.Init(fake_frame, {category=N, index=N})` with a
//! `fake_frame` that exposes the full `Player`/`Friend` sub-table
//! tree the body touches. Spies are installed for `GetAchievementInfo`,
//! `GetAchievementComparisonInfo`, `GetPreviousAchievement`,
//! `AchievementShield_SetPoints`, `FormatShortDate`, and `INCOMPLETE`
//! (the global string constant), then restored after each drive.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AchievementUI";
const PLAN_NAMED_BUT_ABSENT: &str = "AchievementComparison_SetCheckmark";
const FORMAT_SHORT_DATE_TOKEN: &str = "DATE_TOKEN_15_6_24";
const INCOMPLETE_TOKEN: &str = "INCOMPLETE_TOKEN";
const EXPECTED_COMPLETED: &str = "comparison_calls=1 status_calls=1 \
    status_text=DATE_TOKEN_15_6_24 friend_saturate=1 friend_desaturate=0 friend_completed_flag=true";
const EXPECTED_INCOMPLETE: &str = "comparison_calls=1 status_calls=1 \
    status_text=INCOMPLETE_TOKEN friend_saturate=0 friend_desaturate=1 friend_completed_flag=nil \
    cache_short_circuit_comparison_calls=1";

type CompletedProbe = (String, String, String, String, String);
type IncompleteProbe = (String, String);

const FAKE_FRAME_BUILDER: &str = r#"
    local function noop_text() end
    local function noop_show() end
    local function build_card(saturate_counter_key, desaturate_counter_key, captures)
        local card = {
            Label = {SetText = noop_text},
            Description = {SetText = noop_text},
            Icon = {texture = {SetTexture = noop_text}},
            Shield = {
                Icon = {SetTexture = noop_text},
                Points = {SetText = noop_text, SetFontObject = noop_text},
            },
            DateCompleted = {
                SetText = noop_text,
                SetShown = noop_show,
                Show = noop_show,
                Hide = noop_show,
            },
            Status = {
                SetText = function(self, text)
                    captures.status_calls = captures.status_calls + 1
                    captures.status_text = tostring(text)
                end,
            },
            Saturate = function(self)
                captures[saturate_counter_key] = captures[saturate_counter_key] + 1
            end,
            Desaturate = function(self)
                captures[desaturate_counter_key] = captures[desaturate_counter_key] + 1
            end,
        }
        return card
    end
"#;

#[test]
fn comparison_template_init_completed_writes_format_short_date_and_saturates_friend_card() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        -- Explicit C-query inputs drive the original native row, never a fake card.
        GetAchievementInfo = function() return 4242, "fixture achievement", 10, false, nil, nil, nil, "fixture description", 0, "Interface\\Icons\\INV_Misc_QuestionMark", "" end
        GetPreviousAchievement = function() return nil end
        local row = assert(AchievementFrameComparisonContainer.buttons[1])
        row.id = nil
        GetAchievementComparisonInfo = function() return true, 6, 15, 24 end
        local background = _G[row.friend:GetName().."Background"]
        local incompleteTexture = background:GetTexture()
        AchievementFrameComparison_DisplayAchievement(row, 92, 1)
        assert(background:GetTexture() ~= incompleteTexture)
        assert(row.id == 4242 and row.friend.completed)
        assert(row.friend.status:GetText() == string.format(SHORTDATE, 15, 6, 24))
        assert(row.friend.shield.points:GetText() == "10")
        local expected = AchievementFrame:CreateTexture()
        expected:SetTexture("Interface\\AchievementFrame\\UI-Achievement-Parchment-Horizontal")
        assert(expected:GetTexture() ~= nil)
        assert(_G[row.friend:GetName().."Background"]:GetTexture() == expected:GetTexture())
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: CompletedProbe = env
            .eval(&format!(
                r#"
                assert(_G.AchievementComparisonTemplateMixin,
                    "AchievementComparisonTemplateMixin must exist (lua:2682)")
                assert(_G.AchievementComparisonTemplateMixin.Init,
                    "AchievementComparisonTemplateMixin:Init must exist (lua:2688)")

                {fake_frame_builder}

                local captures = {{
                    comparison_calls = 0,
                    status_calls = 0,
                    status_text = "<unset>",
                    friend_saturate = 0,
                    friend_desaturate = 0,
                    friend_completed_flag = "<unset>",
                }}

                local original_get_info = _G.GetAchievementInfo
                _G.GetAchievementInfo = function(category, index)
                    return 4242, "stub_name", 10, true, 6, 15, 24, "stub_desc",
                        0, "Interface\\Icons\\stub", "", false, true, ""
                end
                local original_get_prev = _G.GetPreviousAchievement
                _G.GetPreviousAchievement = function(id) return nil end
                local original_get_comparison = _G.GetAchievementComparisonInfo
                _G.GetAchievementComparisonInfo = function(id)
                    captures.comparison_calls = captures.comparison_calls + 1
                    return true, 6, 15, 24
                end
                local original_set_points = _G.AchievementShield_SetPoints
                _G.AchievementShield_SetPoints = function() end
                local original_format = _G.FormatShortDate
                _G.FormatShortDate = function(d, m, y) return "{format_token}" end

                local fake_frame = {{id = nil}}
                fake_frame.Player = build_card("player_saturate", "player_desaturate", captures)
                fake_frame.Friend = build_card("friend_saturate", "friend_desaturate", captures)
                captures.player_saturate = 0
                captures.player_desaturate = 0

                AchievementComparisonTemplateMixin.Init(fake_frame, {{category = 1, index = 1}})

                captures.friend_completed_flag = tostring(fake_frame.Friend.completed)

                _G.GetAchievementInfo = original_get_info
                _G.GetPreviousAchievement = original_get_prev
                _G.GetAchievementComparisonInfo = original_get_comparison
                _G.AchievementShield_SetPoints = original_set_points
                _G.FormatShortDate = original_format

                local completed_signature = string.format(
                    "comparison_calls=%d status_calls=%d status_text=%s " ..
                    "friend_saturate=%d friend_desaturate=%d friend_completed_flag=%s",
                    captures.comparison_calls, captures.status_calls,
                    captures.status_text, captures.friend_saturate,
                    captures.friend_desaturate, captures.friend_completed_flag)

                return type(_G.AchievementComparisonTemplateMixin),
                       type(_G.AchievementComparisonTemplateMixin.Init),
                       type(_G.GetAchievementComparisonInfo),
                       type(_G.SetAchievementComparisonUnit),
                       completed_signature
                "#,
                fake_frame_builder = FAKE_FRAME_BUILDER,
                format_token = FORMAT_SHORT_DATE_TOKEN,
            ))
            .expect("comparison-template completed-branch drive must run cleanly");

        let (mixin_type, init_type, comparison_getter_type, comparison_unit_setter_type, signature) =
            observations;

        assert_eq!(
            mixin_type, "table",
            "Expected `_G.AchievementComparisonTemplateMixin` to be a table (declared at \
             `Mainline/Blizzard_AchievementUI.lua:2682`). Got `{mixin_type}`. A `nil` reading \
             means the template-mixin module assignment regressed — every comparison row \
             would crash in the ScrollBox initializer at lua:2767-2769 \
             (`view:SetElementInitializer(\"AchievementComparisonTemplate\", function(frame, \
             elementData) frame:Init(elementData) end)`)."
        );

        assert_eq!(
            init_type, "function",
            "Expected `AchievementComparisonTemplateMixin.Init` to be a function (declared at \
             lua:2688). Got `{init_type}`. A `nil` reading means the per-row initializer \
             would crash with `attempt to call nil` on every comparison-frame entry, and the \
             friend-side date/INCOMPLETE write at lua:2752/2758 would never fire."
        );

        assert_eq!(
            comparison_getter_type, "function",
            "Expected `_G.GetAchievementComparisonInfo` to be a function (registered at \
             `register_comparison_getters` in `achievement_info.rs:339-364`). Got \
             `{comparison_getter_type}`. PLAN's `depends-on: SetAchievementComparisonUnit \
             gap` tag is stale on the getter half too — a `nil` reading would mean the call \
             at lua:2716 crashes before either branch (completed/incomplete) gets a chance \
             to write friend.Status."
        );

        assert_eq!(
            comparison_unit_setter_type, "function",
            "Expected `_G.SetAchievementComparisonUnit` to be a function (registered at \
             `register_comparison_unit_mutators` in `achievement_info.rs:323-336`). Got \
             `{comparison_unit_setter_type}`. The depends-on tag PLAN names is stale — both \
             the unit setter (called from `_SetUnit` at lua:2836) and the per-id getter \
             (called from `Init` at lua:2716) are wired."
        );

        assert_eq!(
            signature, EXPECTED_COMPLETED,
            "Expected completed-branch signature to match. The drive stubs \
             `GetAchievementInfo` to return id=4242 with completed=true, \
             `GetAchievementComparisonInfo` to return `(true, 6, 15, 24)` (friendCompleted, \
             month, day, year), `FormatShortDate(d, m, y)` to return \
             `{FORMAT_SHORT_DATE_TOKEN:?}`, and zeros all card-saturation counters before \
             calling `AchievementComparisonTemplateMixin.Init(fake_frame, {{category=1, \
             index=1}})`. Expected `{EXPECTED_COMPLETED}`. Got `{signature}`. \
             A `comparison_calls=0` means the cache gate at lua:2700 (`if self.id ~= id`) \
             leaked — `fake_frame.id` was nil so the gate should have routed into the body. \
             A `status_text` other than the format-short-date token means the friend.Status \
             write at lua:2752 routed to the wrong branch (likely the elseif at lua:2756-2760 \
             fired instead — meaning `friendCompleted` came back falsy from the spy). A \
             `friend_saturate=0` means the saturatedStyle gate at lua:2753 short-circuited \
             — but the fake card has no `saturatedStyle` field initially, so it should \
             differ from the local `saturatedStyle = \"normal\"` at lua:2706 and Saturate \
             must fire. A `friend_desaturate=1` would mean the else branch at lua:2759 \
             fired instead — the friend card got desaturated despite friendCompleted=true. \
             A `friend_completed_flag=nil` would mean lua:2751 (`friend.completed = true`) \
             didn't fire."
        );
    });
}

#[test]
fn comparison_template_init_incomplete_writes_incomplete_token_and_short_circuits_on_repeat_id() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        -- Explicit C-query inputs drive the original native row, never a fake card.
        GetAchievementInfo = function() return 4242, "fixture achievement", 10, false, nil, nil, nil, "fixture description", 0, "Interface\\Icons\\INV_Misc_QuestionMark", "" end
        GetPreviousAchievement = function() return nil end
        local row = assert(AchievementFrameComparisonContainer.buttons[1])
        row.id = nil
        local calls = 0
        GetAchievementComparisonInfo = function() calls = calls + 1; return false end
        AchievementFrameComparison_DisplayAchievement(row, 92, 1)
        assert(not row.friend.completed and row.friend.status:GetText() == INCOMPLETE)
        assert(calls == 1)
        AchievementFrameComparison_DisplayAchievement(row, 92, 1)
        assert(calls == 1 and row.friend.status:GetText() == INCOMPLETE)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: IncompleteProbe = env
            .eval(&format!(
                r#"
                {fake_frame_builder}

                local captures = {{
                    comparison_calls = 0,
                    status_calls = 0,
                    status_text = "<unset>",
                    friend_saturate = 0,
                    friend_desaturate = 0,
                    friend_completed_flag = "<unset>",
                }}

                local original_get_info = _G.GetAchievementInfo
                _G.GetAchievementInfo = function(category, index)
                    return 7777, "stub_name", 10, false, 0, 0, 0, "stub_desc",
                        0, "Interface\\Icons\\stub", "", false, false, ""
                end
                local original_get_prev = _G.GetPreviousAchievement
                _G.GetPreviousAchievement = function(id) return nil end
                local original_get_comparison = _G.GetAchievementComparisonInfo
                _G.GetAchievementComparisonInfo = function(id)
                    captures.comparison_calls = captures.comparison_calls + 1
                    return false, nil, nil, nil
                end
                local original_set_points = _G.AchievementShield_SetPoints
                _G.AchievementShield_SetPoints = function() end
                local original_format = _G.FormatShortDate
                _G.FormatShortDate = function(d, m, y) return "should_not_be_called" end
                local original_incomplete = _G.INCOMPLETE
                _G.INCOMPLETE = "{incomplete_token}"

                local fake_frame = {{id = nil}}
                fake_frame.Player = build_card("player_saturate", "player_desaturate", captures)
                fake_frame.Friend = build_card("friend_saturate", "friend_desaturate", captures)
                captures.player_saturate = 0
                captures.player_desaturate = 0

                AchievementComparisonTemplateMixin.Init(fake_frame, {{category = 1, index = 1}})

                captures.friend_completed_flag = tostring(fake_frame.Friend.completed)
                local first_drive_signature = string.format(
                    "comparison_calls=%d status_calls=%d status_text=%s " ..
                    "friend_saturate=%d friend_desaturate=%d friend_completed_flag=%s",
                    captures.comparison_calls, captures.status_calls,
                    captures.status_text, captures.friend_saturate,
                    captures.friend_desaturate, captures.friend_completed_flag)

                AchievementComparisonTemplateMixin.Init(fake_frame, {{category = 9, index = 9}})
                local cache_signature = string.format(
                    "cache_short_circuit_comparison_calls=%d", captures.comparison_calls)

                _G.GetAchievementInfo = original_get_info
                _G.GetPreviousAchievement = original_get_prev
                _G.GetAchievementComparisonInfo = original_get_comparison
                _G.AchievementShield_SetPoints = original_set_points
                _G.FormatShortDate = original_format
                _G.INCOMPLETE = original_incomplete

                local incomplete_signature = first_drive_signature .. " " .. cache_signature

                return type(_G[ "{plan_named_but_absent}" ]),
                       incomplete_signature
                "#,
                fake_frame_builder = FAKE_FRAME_BUILDER,
                incomplete_token = INCOMPLETE_TOKEN,
                plan_named_but_absent = PLAN_NAMED_BUT_ABSENT,
            ))
            .expect("comparison-template incomplete-branch drive must run cleanly");

        let (plan_named_function_type, signature) = observations;

        assert_eq!(
            plan_named_function_type, "nil",
            "Expected `_G.{PLAN_NAMED_BUT_ABSENT}` to be nil — PLAN says `Init ... sets the \
             friend-completed checkmark`, but no `*_SetCheckmark` global exists in \
             `Mainline/Blizzard_AchievementUI.lua`. The actual friend-side completion cue \
             is split across THREE writes inside `Init`: `friend.completed` boolean at \
             lua:2751/2757, `friend.Status:SetText(FormatShortDate(...))` vs `:SetText(INCOMPLETE)` \
             at lua:2752/2758, and `friend:Saturate()`/`:Desaturate()` at lua:2754/2759. \
             Got `{plan_named_function_type}`. A non-nil reading means a future refactor \
             introduced an alias under the PLAN-named symbol; flag the rename and update \
             PLAN's wording rather than silently dropping this tripwire."
        );

        assert_eq!(
            signature, EXPECTED_INCOMPLETE,
            "Expected incomplete-branch signature plus cache short-circuit to match. The \
             first drive stubs `GetAchievementInfo` with completed=false, \
             `GetAchievementComparisonInfo` to return `(false, nil, nil, nil)`, and \
             `_G.INCOMPLETE = {INCOMPLETE_TOKEN:?}`. The second drive uses different \
             `category=9, index=9` but the SAME stubbed id=7777 — the cache gate at \
             lua:2700 (`if self.id ~= id`) should short-circuit the entire body, leaving \
             `comparison_calls` at 1 (NOT 2). Expected `{EXPECTED_INCOMPLETE}`. Got \
             `{signature}`. A `status_text` of the format-short-date stub means the \
             completed branch at lua:2752 fired instead of the else at lua:2758 — meaning \
             `friendCompleted` came back truthy from the spy despite returning false. A \
             `friend_desaturate=0` means the unconditional `:Desaturate()` at lua:2759 \
             didn't fire (note: unlike Saturate at lua:2754 which is gated on \
             saturatedStyle change, Desaturate is unconditional). A \
             `friend_completed_flag` other than `nil` means the explicit `friend.completed \
             = nil` write at lua:2757 leaked. A `cache_short_circuit_comparison_calls=2` \
             means the cache gate at lua:2700 leaked — the second Init re-fetched \
             `GetAchievementComparisonInfo` despite `self.id == id`, which would cause \
             ScrollBox row recycling to do redundant fetches on every rebind."
        );
    });
}
