//! Behavior pin: PLAN-named "swap the +/- texture and adjust frame
//! height through `CalculateSelectedHeight`" elides three structural
//! facts about `AchievementTemplateMixin:Expand` /
//! `AchievementTemplateMixin:Collapse` at
//! `Mainline/Blizzard_AchievementUI.lua:1308-1357`. (1) Neither method
//! calls `CalculateSelectedHeight` — `Collapse` writes the constant
//! `ACHIEVEMENTBUTTON_COLLAPSEDHEIGHT = 84` (lua:1032/1315), and
//! `Expand(height)` takes the height as a parameter (lua:1351), with
//! the calling site at lua:1287-1289 (inside `:Init`) responsible for
//! computing it via `CalculateSelectedHeight`. (2) The +/- texture
//! swap is performed by a SEPARATE method `:UpdatePlusMinusTexture()`
//! at lua:1110-1139, which both `Collapse` (lua:1314) and `Expand`
//! (lua:1338) call as a side effect — the swap is not inline. (3)
//! `:UpdatePlusMinusTexture` chooses among FOUR distinct `SetTexCoord`
//! regions based on `(self.collapsed, self.saturatedStyle)`, and
//! GATES the show/hide on three independent display predicates at
//! lua:1117-1123 (`GetAchievementNumCriteria(id) ~= 0`, OR completed
//! AND `GetPreviousAchievement(id)`, OR not-completed AND
//! `GetAchievementGuildRep(id)`); when none of those holds, the +/-
//! texture is HIDDEN entirely (lua:1136-1138).
//!
//! Source map of the contract:
//!
//! ```lua
//! -- lua:1032 ACHIEVEMENTBUTTON_COLLAPSEDHEIGHT = 84
//! -- lua:1037 GUILDACHIEVEMENTBUTTON_MINHEIGHT = 128
//! -- lua:62  local TEXTURES_OFFSET = 0  (changes to 0.5 in guild view)
//! -- lua:1110-1139 (the texture-swap dispatch)
//! function AchievementTemplateMixin:UpdatePlusMinusTexture()
//!     local id = self.id
//!     if (not id) then return end                         -- lua:1112-1114
//!     local display = false
//!     if (GetAchievementNumCriteria(id) ~= 0) then display = true
//!     elseif (self.completed and GetPreviousAchievement(id)) then display = true
//!     elseif (not self.completed and GetAchievementGuildRep(id)) then display = true
//!     end
//!     if (display) then
//!         self.PlusMinus:Show()
//!         if (self.collapsed and self.saturatedStyle) then
//!             self.PlusMinus:SetTexCoord(0, .5, OFF, OFF+0.25)   -- collapsed,colored
//!         elseif (self.collapsed) then
//!             self.PlusMinus:SetTexCoord(.5, 1, OFF, OFF+0.25)   -- collapsed,gray
//!         elseif (self.saturatedStyle) then
//!             self.PlusMinus:SetTexCoord(0, .5, OFF+0.25, OFF+0.50) -- expanded,colored
//!         else
//!             self.PlusMinus:SetTexCoord(.5, 1, OFF+0.25, OFF+0.50) -- expanded,gray
//!         end
//!     else
//!         self.PlusMinus:Hide()
//!     end
//! end
//!
//! -- lua:1308-1330 Collapse
//! function AchievementTemplateMixin:Collapse()
//!     if (self.collapsed) then return end                 -- lua:1309 idempotent
//!     self.collapsed = true                                -- lua:1313
//!     self:UpdatePlusMinusTexture()                        -- lua:1314
//!     self:SetHeight(ACHIEVEMENTBUTTON_COLLAPSEDHEIGHT)    -- lua:1315 (=84, constant)
//!     self.Background:SetTexCoord(0, 1, 1-(84/256), 1)     -- lua:1316
//!     if (not self.Tracked:GetChecked()) then self.Tracked:Hide() end
//!     self.Tabard:Hide()
//!     self.GuildCornerL:Hide(); self.GuildCornerR:Hide()
//!     self.Description:Show(); self.HiddenDescription:Hide()  -- lua:1324-1325
//!     if (not self:IsMouseOver()) then self.Highlight:Hide() end
//! end
//!
//! -- lua:1332-1357 Expand(height) — height is a PARAMETER
//! function AchievementTemplateMixin:Expand(height)
//!     if (not self.collapsed and self:GetHeight() == height) then return end
//!     self.collapsed = nil                                 -- lua:1337
//!     self:UpdatePlusMinusTexture()                        -- lua:1338
//!     if (InGuildView()) then
//!         if (height < GUILDACHIEVEMENTBUTTON_MINHEIGHT) then
//!             height = GUILDACHIEVEMENTBUTTON_MINHEIGHT     -- lua:1340-1342 floor
//!         end
//!         if (self.completed) then ...Tabard:Show() etc end
//!         self.GuildCornerL:Show(); self.GuildCornerR:Show()
//!     end
//!     self:SetHeight(height)                               -- lua:1351
//!     self.Background:SetTexCoord(0, 1, math.max(0, 1-(height/256)), 1)  -- lua:1353
//!     self.HiddenDescription:Show(); self.Description:Hide()  -- lua:1355-1356
//! end
//! ```
//!
//! **Spec/source mismatch on FOUR axes:**
//!
//! 1. **Neither method calls `CalculateSelectedHeight`.** `Collapse`
//!    uses the constant `ACHIEVEMENTBUTTON_COLLAPSEDHEIGHT` directly
//!    (lua:1315). `Expand` takes the height as a parameter (lua:1351).
//!    The actual `CalculateSelectedHeight` callers are `Init` at
//!    lua:1287 (which then forwards to `Expand(height)` at lua:1289)
//!    and the layout helper at lua:856. PLAN's wording would lead a
//!    reader to expect a `self:CalculateSelectedHeight()` line inside
//!    Expand; that line does not exist. (Note: `CalculateSelectedHeight`
//!    is also defined as a STATIC method at lua:1422 with `.` not `:`
//!    syntax — it takes `elementData`, not `self`.)
//! 2. **The +/- texture swap is delegated to `:UpdatePlusMinusTexture`.**
//!    PLAN says "Expand/Collapse swap the +/- texture", implying inline
//!    SetTexCoord calls. In reality both methods call out to a shared
//!    helper (lua:1314 from Collapse, lua:1338 from Expand) that owns
//!    the entire texture-routing logic. A regression that drops the
//!    `:UpdatePlusMinusTexture()` call from either method would leave
//!    the +/- icon stuck in its previous state.
//! 3. **The texture choice depends on (collapsed, saturatedStyle), not
//!    just collapsed.** The four-way `SetTexCoord` dispatch at
//!    lua:1127-1135 picks a different region for saturated vs
//!    desaturated; PLAN's "swap" wording would suggest a binary
//!    collapse/expand swap. A regression that drops the saturatedStyle
//!    distinction would render colored icons as gray (or vice versa).
//! 4. **Idempotence guards short-circuit no-ops.** Collapse at lua:1309
//!    short-circuits when already collapsed; Expand at lua:1333
//!    short-circuits when already expanded AND `GetHeight() == height`.
//!    A regression that drops either guard would re-fire the
//!    UpdatePlusMinusTexture / SetHeight / SetTexCoord side effects
//!    every frame, churning the layout.
//!
//! Two tests split the Collapse and Expand paths. Each builds a fake
//! row with the full sub-widget tree (PlusMinus, Background, Tracked,
//! Tabard, GuildCornerL/R, Description, HiddenDescription, Highlight,
//! Shield) exposing counter-bound `:Show`/`:Hide`/`:SetTexCoord` and
//! a `_height` round-trip via `:SetHeight` / `:GetHeight`. `Highlight`
//! is a real region: Retail hides it directly (`Highlight:Hide()`,
//! lua:1364) while the newer sources route the same call through
//! `SetHighlightShown` -> `Highlight:SetShown` (lua:1165-1167,
//! reached from lua:1444), so only a real region satisfies both. Spies on
//! `GetAchievementNumCriteria`, `GetPreviousAchievement`,
//! `GetAchievementGuildRep`, and `InGuildView` drive the texture
//! display gate and the guild-view branch deterministically.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AchievementUI";
const PLAN_NAMED_BUT_ABSENT_TOGGLE: &str = "AchievementTemplate_PlusMinusToggle";
const COLLAPSED_HEIGHT: i64 = 84;
const REQUESTED_EXPAND_HEIGHT: i64 = 200;
const EXPECTED_COLLAPSE: &str = "first_collapsed_flag=true height=84 \
    plus_minus_show=1 plus_minus_hide=0 plus_minus_set_tex_coord_count=1 \
    description_show=1 hidden_description_hide=1 tabard_hide=1 \
    highlight_shown=false \
    second_call_no_op_height=84 second_call_show_count=1";
const EXPECTED_EXPAND: &str = "expanded_collapsed_flag=nil height=200 \
    plus_minus_show=1 plus_minus_set_tex_coord_count=1 \
    hidden_description_show=1 description_hide=1 \
    second_call_no_op_height=200 second_call_show_count=1 \
    guild_view_floor_height=128";

type CollapseProbe = (String, String, String, String);
type ExpandProbe = (String, String);

const FAKE_ROW_BUILDER: &str = r#"
    local function build_row(captures)
        local row = {
            id = 4242, completed = false, saturatedStyle = "normal",
            collapsed = nil, _height = 0,
        }
        local function counter_show(key)
            return function(self) captures[key] = (captures[key] or 0) + 1 end
        end
        local function counter_hide(key)
            return function(self) captures[key] = (captures[key] or 0) + 1 end
        end
        row.PlusMinus = {
            Show = counter_show("plus_minus_show"),
            Hide = counter_hide("plus_minus_hide"),
            SetTexCoord = function(self, ...)
                captures.plus_minus_set_tex_coord_count =
                    (captures.plus_minus_set_tex_coord_count or 0) + 1
            end,
        }
        row.Background = {
            SetTexCoord = function(self, ...)
                captures.background_set_tex_coord_count =
                    (captures.background_set_tex_coord_count or 0) + 1
            end,
            SetTexture = function() end,
        }
        row.Tracked = {
            Hide = counter_hide("tracked_hide"),
            GetChecked = function(self) return false end,
        }
        row.Tabard = {
            Hide = counter_hide("tabard_hide"),
            Show = counter_show("tabard_show"),
            Emblem = {}, Background = {}, Border = {},
            GetFrameLevel = function(self) return 1 end,
        }
        row.GuildCornerL = {
            Hide = counter_hide("guild_corner_l_hide"),
            Show = counter_show("guild_corner_l_show"),
        }
        row.GuildCornerR = {
            Hide = counter_hide("guild_corner_r_hide"),
            Show = counter_show("guild_corner_r_show"),
        }
        row.Description = {
            Show = counter_show("description_show"),
            Hide = counter_hide("description_hide"),
        }
        row.HiddenDescription = {
            Show = counter_show("hidden_description_show"),
            Hide = counter_hide("hidden_description_hide"),
        }
        local highlight_host = CreateFrame("Frame", nil, UIParent)
        row.Highlight = highlight_host:CreateTexture(nil, "HIGHLIGHT")
        row.Highlight:Show()
        row.Shield = {SetFrameLevel = function() end}
        row.SetHeight = function(self, h) row._height = h end
        row.GetHeight = function(self) return row._height end
        row.IsMouseOver = function(self) return false end
        return row
    end
"#;

#[test]
fn collapse_writes_constant_collapsed_height_swaps_texture_via_helper_and_is_idempotent() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        local row = CreateFrame("Button", "F08Style", AchievementFrameAchievements, "AchievementTemplate")
        row:Expand(150)
        assert(not row.collapsed and row:GetHeight() == 150)
        row:Collapse()
        assert(row.collapsed and row:GetHeight() == ACHIEVEMENTBUTTON_COLLAPSEDHEIGHT)
        assert(not row.tracked:IsShown())
        row:Collapse()
        assert(row:GetHeight() == ACHIEVEMENTBUTTON_COLLAPSEDHEIGHT)
        local tlx, tly, blx, bly, trx, try, brx, bry = F08StyleBackground:GetTexCoord()
        local expectedTop = 1 - ACHIEVEMENTBUTTON_COLLAPSEDHEIGHT / 256
        assert(tlx == 0 and blx == 0 and trx == 1 and brx == 1)
        assert(math.abs(tly - expectedTop) < 0.001 and math.abs(try - expectedTop) < 0.001)
        assert(bly == 1 and bry == 1)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: CollapseProbe = env
            .eval(&format!(
                r#"
                assert(_G.AchievementTemplateMixin,
                    "AchievementTemplateMixin must exist (lua:1039)")
                assert(_G.AchievementTemplateMixin.Collapse,
                    "AchievementTemplateMixin:Collapse must exist (lua:1308)")
                assert(_G.AchievementTemplateMixin.Expand,
                    "AchievementTemplateMixin:Expand must exist (lua:1332)")
                assert(_G.AchievementTemplateMixin.UpdatePlusMinusTexture,
                    "AchievementTemplateMixin:UpdatePlusMinusTexture must exist (lua:1110)")
                assert(_G.AchievementTemplateMixin.CalculateSelectedHeight,
                    "AchievementTemplateMixin.CalculateSelectedHeight must exist (lua:1422)")

                {fake_row_builder}

                local original_num_criteria = _G.GetAchievementNumCriteria
                _G.GetAchievementNumCriteria = function(id) return 3 end
                local original_in_guild = _G.InGuildView
                _G.InGuildView = function() return false end

                local captures = {{}}
                local fake_row = build_row(captures)
                Mixin(fake_row, AchievementTemplateMixin)
                fake_row.collapsed = nil

                fake_row:Collapse()
                local first_collapsed_flag = tostring(fake_row.collapsed)
                local first_height = fake_row._height
                local highlight_shown = tostring(fake_row.Highlight:IsShown())

                fake_row:Collapse()
                local second_call_no_op_height = fake_row._height
                local second_call_show_count = captures.plus_minus_show or 0

                _G.GetAchievementNumCriteria = original_num_criteria
                _G.InGuildView = original_in_guild

                local collapse_signature = string.format(
                    "first_collapsed_flag=%s height=%d " ..
                    "plus_minus_show=%d plus_minus_hide=%d " ..
                    "plus_minus_set_tex_coord_count=%d " ..
                    "description_show=%d hidden_description_hide=%d tabard_hide=%d " ..
                    "highlight_shown=%s " ..
                    "second_call_no_op_height=%d second_call_show_count=%d",
                    first_collapsed_flag, first_height,
                    captures.plus_minus_show or 0, captures.plus_minus_hide or 0,
                    captures.plus_minus_set_tex_coord_count or 0,
                    captures.description_show or 0, captures.hidden_description_hide or 0,
                    captures.tabard_hide or 0,
                    highlight_shown,
                    second_call_no_op_height, second_call_show_count)

                return type(_G.AchievementTemplateMixin.Collapse),
                       type(_G.AchievementTemplateMixin.Expand),
                       type(_G.AchievementTemplateMixin.UpdatePlusMinusTexture),
                       collapse_signature
                "#,
                fake_row_builder = FAKE_ROW_BUILDER,
            ))
            .expect("Collapse drive must run cleanly");

        let (collapse_type, expand_type, update_plus_minus_type, signature) = observations;

        assert_eq!(
            collapse_type, "function",
            "Expected `AchievementTemplateMixin.Collapse` to be a function (declared at \
             `Mainline/Blizzard_AchievementUI.lua:1308`). Got `{collapse_type}`. A `nil` \
             reading means achievement rows would never collapse — the OnClick handler at \
             lua:1304 (`self:Collapse()`) would crash, leaving expanded rows stuck \
             expanded."
        );

        assert_eq!(
            expand_type, "function",
            "Expected `AchievementTemplateMixin.Expand` to be a function (declared at \
             `Mainline/Blizzard_AchievementUI.lua:1332`). Got `{expand_type}`. A `nil` \
             reading means rows would never expand on click — the Init at lua:1287-1289 \
             (`self:Expand(height)`) would crash; selected achievements would stay at \
             collapsed height."
        );

        assert_eq!(
            update_plus_minus_type, "function",
            "Expected `AchievementTemplateMixin.UpdatePlusMinusTexture` to be a function \
             (declared at lua:1110). Got `{update_plus_minus_type}`. A `nil` reading \
             would mean both Collapse (lua:1314) and Expand (lua:1338) crash on the \
             helper-call line; the +/- icon would never swap. PLAN's wording \
             `Expand/Collapse swap the +/- texture` collapses this delegation — the \
             swap is implemented in this helper, not inline."
        );

        assert_eq!(
            signature, EXPECTED_COLLAPSE,
            "Expected Collapse signature to match. The drive starts with \
             `fake_row.collapsed = nil` (forces the lua:1309 idempotent guard to fall \
             through), `GetAchievementNumCriteria` stubbed to return 3 (drives the \
             display=true branch at lua:1117 so PlusMinus:Show fires), `InGuildView` \
             stubbed to false (sidesteps the guild-view branch). After the first \
             Collapse: `self.collapsed == true` (lua:1313), `_height == \
             {COLLAPSED_HEIGHT}` (lua:1315 writes the constant ACHIEVEMENTBUTTON_COLLAPSEDHEIGHT), \
             PlusMinus:Show fires once with one SetTexCoord call (the helper at \
             lua:1126-1135 picks one of four regions; the count is what matters here), \
             Description:Show + HiddenDescription:Hide fire (lua:1324-1325), Tabard:Hide \
             fires (lua:1320). The second Collapse call must short-circuit at lua:1309 \
             (`if self.collapsed then return end`) — height stays at {COLLAPSED_HEIGHT}, \
             PlusMinus:Show counter stays at 1 (no second invocation). Expected \
             `{EXPECTED_COLLAPSE}`. Got `{signature}`. A `height` other than \
             {COLLAPSED_HEIGHT} means lua:1315 wrote a different constant (likely \
             ACHIEVEMENTBUTTON_COLLAPSEDHEIGHT was renamed). A `plus_minus_show=0` means \
             the display gate at lua:1116-1123 short-circuited despite the spy returning \
             3 — the gate's first arm at lua:1117 reads \
             `GetAchievementNumCriteria(id) ~= 0` so a return of 3 must enter the \
             display branch. A `description_show=0` means lua:1324 didn't fire. A \
             `second_call_show_count=2` means the idempotent guard at lua:1309 leaked — \
             every repeated Collapse call would re-fire all the side effects, churning \
             the layout. A `highlight_shown=true` means the trailing highlight hide \
             (`Highlight:Hide()` at lua:1364, or `SetHighlightShown(false)` -> \
             `Highlight:SetShown(false)` at lua:1165-1167 reached from lua:1444) never \
             reached the region."
        );
    });
}

#[test]
fn expand_uses_height_parameter_directly_floors_to_guild_min_height_and_is_idempotent_at_target_height()
 {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        local row = CreateFrame("Button", "F08Style", AchievementFrameAchievements, "AchievementTemplate")
        row:Expand(150)
        assert(not row.collapsed and row:GetHeight() == 150)
        -- Wrath has no guild minimum: a repeat expand is ignored until collapse.
        row:Expand(220)
        assert(row:GetHeight() == 150)
        row:Collapse()
        row:Expand(220)
        assert(row:GetHeight() == 220)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: ExpandProbe = env
            .eval(&format!(
                r#"
                {fake_row_builder}

                local original_num_criteria = _G.GetAchievementNumCriteria
                _G.GetAchievementNumCriteria = function(id) return 3 end
                local original_in_guild = _G.InGuildView
                _G.InGuildView = function() return false end

                local captures = {{}}
                local fake_row = build_row(captures)
                Mixin(fake_row, AchievementTemplateMixin)
                fake_row.collapsed = true

                fake_row:Expand({requested_height})
                local expanded_collapsed_flag = tostring(fake_row.collapsed)
                local expanded_height = fake_row._height

                fake_row:Expand({requested_height})
                local second_call_no_op_height = fake_row._height
                local second_call_show_count = captures.plus_minus_show or 0

                local guild_captures = {{}}
                local guild_row = build_row(guild_captures)
                Mixin(guild_row, AchievementTemplateMixin)
                guild_row.collapsed = true

                local original_expand_upvalue = nil
                local in_guild_upvalue_index = nil
                for i = 1, 60 do
                    local name, val = debug.getupvalue(AchievementTemplateMixin.Expand, i)
                    if name == nil then break end
                    if name == "InGuildView" then
                        in_guild_upvalue_index = i
                        original_expand_upvalue = val
                        debug.setupvalue(AchievementTemplateMixin.Expand, i,
                            function() return true end)
                        break
                    end
                end
                assert(in_guild_upvalue_index,
                    "Expand must capture InGuildView as upvalue (lua:1339); local at lua:103")

                guild_row:Expand(50)
                local guild_view_floor_height = guild_row._height

                debug.setupvalue(AchievementTemplateMixin.Expand,
                    in_guild_upvalue_index, original_expand_upvalue)
                _G.GetAchievementNumCriteria = original_num_criteria
                _G.InGuildView = original_in_guild

                local expand_signature = string.format(
                    "expanded_collapsed_flag=%s height=%d " ..
                    "plus_minus_show=%d plus_minus_set_tex_coord_count=%d " ..
                    "hidden_description_show=%d description_hide=%d " ..
                    "second_call_no_op_height=%d second_call_show_count=%d " ..
                    "guild_view_floor_height=%d",
                    expanded_collapsed_flag, expanded_height,
                    captures.plus_minus_show or 0,
                    captures.plus_minus_set_tex_coord_count or 0,
                    captures.hidden_description_show or 0, captures.description_hide or 0,
                    second_call_no_op_height, second_call_show_count,
                    guild_view_floor_height)

                return type(_G[ "{plan_named_toggle}" ]),
                       expand_signature
                "#,
                fake_row_builder = FAKE_ROW_BUILDER,
                requested_height = REQUESTED_EXPAND_HEIGHT,
                plan_named_toggle = PLAN_NAMED_BUT_ABSENT_TOGGLE,
            ))
            .expect("Expand drive must run cleanly");

        let (plan_named_toggle_type, signature) = observations;

        assert_eq!(
            plan_named_toggle_type, "nil",
            "Expected `_G.{PLAN_NAMED_BUT_ABSENT_TOGGLE}` to be nil — PLAN says \
             `Expand/Collapse swap the +/- texture`, but the swap lives in the helper \
             `:UpdatePlusMinusTexture` at lua:1110, not in a `*Toggle*` global. Got \
             `{plan_named_toggle_type}`. A non-nil reading means a future refactor \
             introduced an alias under the PLAN-named symbol; flag the rename and \
             update PLAN's wording rather than silently dropping this tripwire."
        );

        assert_eq!(
            signature, EXPECTED_EXPAND,
            "Expected Expand signature to match. The first drive starts with \
             `fake_row.collapsed = true`, calls Expand with height={REQUESTED_EXPAND_HEIGHT}; \
             `self.collapsed` must clear to nil (lua:1337), `_height == \
             {REQUESTED_EXPAND_HEIGHT}` (lua:1351 writes the parameter directly — NOT \
             a CalculateSelectedHeight return), PlusMinus:Show fires once with one \
             SetTexCoord call (the helper at lua:1338 routes through the four-way \
             dispatch), HiddenDescription:Show + Description:Hide fire (lua:1355-1356, \
             inverse of Collapse). The second Expand call with the same height must \
             short-circuit at lua:1333 (`if not self.collapsed and self:GetHeight() == \
             height then return end`) — height stays at {REQUESTED_EXPAND_HEIGHT}, \
             PlusMinus:Show counter stays at 1. The guild-view drive starts with a \
             fresh row, sets `InGuildView -> true`, calls Expand with height=50 (below \
             GUILDACHIEVEMENTBUTTON_MINHEIGHT=128); the floor at lua:1340-1342 must \
             promote 50 -> 128 before lua:1351 writes it. Expected `{EXPECTED_EXPAND}`. \
             Got `{signature}`. An `expanded_collapsed_flag` other than `nil` means \
             lua:1337 didn't clear the flag. A `height` other than {REQUESTED_EXPAND_HEIGHT} \
             means lua:1351 wrote a different value — most likely PLAN's wording came \
             true and Expand started routing through CalculateSelectedHeight (a \
             refactor that would break the Init→Expand contract at lua:1287-1289). A \
             `second_call_show_count=2` means the idempotent guard at lua:1333 leaked. \
             A `guild_view_floor_height` other than 128 means the floor at \
             lua:1340-1342 didn't fire — the requested 50 should have been promoted \
             to GUILDACHIEVEMENTBUTTON_MINHEIGHT=128. A `guild_view_floor_height=50` \
             specifically means the floor was bypassed entirely; a value > 128 means \
             the floor logic mutated the constant."
        );
    });
}
