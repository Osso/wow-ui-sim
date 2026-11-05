//! Event-registration surface for the `Blizzard_ActionBar` lane.
//! Each test pins one frame's post-OnLoad `IsEventRegistered` set plus
//! its `OnEvent` script wiring. Where PLAN under-counts the source's
//! registrations, the events are split into PLAN-named vs
//! source-additional slices so a mismatch surfaces with a clear cause.
//!
//! `RegisterUnitEvent(event, ...)` and `RegisterEvent(event)` both
//! insert into the frame's `registered_events` set
//! (`src/lua_api/frame/methods/text_attribute_event/events.rs:51-70`
//! vs 41-49) — the unit filter is ignored for registration-set
//! membership and only matters at dispatch-filter time. So
//! `IsEventRegistered("UNIT_FLAGS")` returns true after a
//! `RegisterUnitEvent("UNIT_FLAGS", "pet")` call.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_ActionBar";
const FRAME_NAME: &str = "ActionBarButtonEventsFrame";
const FRAME_XML_SITE: &str = "Shared/ActionButtonComponentTemplate.xml:89";
const ONLOAD_LUA_SITE: &str = "Shared/ActionButton.lua:203";
const DERIVED_MIXIN_LUA_SITE: &str = "Shared/ActionButton.lua:1443";

/// Events the PLAN line names verbatim. All seven are registered via
/// `RegisterEvent` at `Shared/ActionButton.lua:205-211`, in this exact
/// order, as the first seven statements after `self.frames = {}`.
const PLAN_NAMED_EVENTS: &[&str] = &[
    "PLAYER_ENTERING_WORLD",
    "ACTIONBAR_SLOT_CHANGED",
    "UPDATE_BINDINGS",
    "GAME_PAD_ACTIVE_CHANGED",
    "UPDATE_SHAPESHIFT_FORM",
    "ACTIONBAR_UPDATE_COOLDOWN",
    "PET_BAR_UPDATE",
];

/// Events the source registers but PLAN omits. `UNIT_FLAGS` and
/// `UNIT_AURA` are registered via `RegisterUnitEvent(event, "pet")` at
/// `Shared/ActionButton.lua:212-213` — the unit filter is ignored for
/// registration-set membership, so `IsEventRegistered` returns true.
/// `PLAYER_MOUNT_DISPLAY_CHANGED` is a plain `RegisterEvent` at line
/// 214, after the unit-event pair. Pinned here so a regression that
/// drops any of them surfaces with a meaningful failure rather than
/// silently breaking the mixin's `OnEvent` fan-out at lua:220-225.
const SOURCE_ADDITIONAL_EVENTS: &[&str] =
    &["UNIT_FLAGS", "UNIT_AURA", "PLAYER_MOUNT_DISPLAY_CHANGED"];

post_wrath_ui_fixture! {
/// Pin `ActionBarButtonEventsFrame`'s post-OnLoad event-registration
/// surface and the OnEvent script wiring. **Spec/source mismatch** —
/// PLAN names 7 events; OnLoad at lua:203-218 registers 10 (3 extras
/// in `SOURCE_ADDITIONAL_EVENTS`). 12 assertions: existence (1) +
/// PLAN-named (7) + source-additional (3) + OnEvent script (1).
#[cfg(feature = "optional-tests")]
#[test]
fn action_bar_button_events_frame_registers_plan_and_source_additional_events_after_onload() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{FRAME_NAME:?}])"))
            .expect("ActionBarButtonEventsFrame global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{FRAME_NAME:?}]` to be a table after `{ROOT}` loads, got \
             `{frame_type}`. XML at `{FRAME_XML_SITE}` declares the frame; nil reading \
             means XML chunk failed, name= changed, or frame removed."
        );

        for event in PLAN_NAMED_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .expect("IsEventRegistered must run cleanly");

            assert!(
                registered,
                "Expected `{FRAME_NAME}:IsEventRegistered({event:?})` to be true after \
                 `{ROOT}` loads. Mixin OnLoad at `{ONLOAD_LUA_SITE}` registers `{event}` \
                 (one of the first seven `RegisterEvent` calls). XML wires \
                 `<OnLoad method=\"OnLoad\"/>`; derived mixin at \
                 `{DERIVED_MIXIN_LUA_SITE}` shallow-copies the base OnLoad. False \
                 reading: OnLoad did not run, RegisterEvent removed, or method-style \
                 OnLoad dispatch regressed. Without `{event}`, the OnEvent fan-out at \
                 lua:220-225 (`for k, frame in pairs(self.frames) do \
                 frame:OnEvent(event, ...) end`) silently drops `{event}`."
            );
        }

        for event in SOURCE_ADDITIONAL_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .expect("IsEventRegistered must run cleanly");

            assert!(
                registered,
                "Expected `{FRAME_NAME}:IsEventRegistered({event:?})` to be true after \
                 `{ROOT}` loads. PLAN omits this; source registers it in the same \
                 OnLoad at `{ONLOAD_LUA_SITE}` (UNIT_FLAGS+UNIT_AURA via \
                 RegisterUnitEvent(\"pet\") lua:212-213, PLAYER_MOUNT_DISPLAY_CHANGED \
                 plain lua:214). False reading: OnLoad regressed, the \
                 `RegisterUnitEvent` registration path stopped marking events \
                 (events.rs:62), or Blizzard removed the call — fan-out at lua:220-225 \
                 drops `{event}`."
            );
        }

        let onevent_script: String = env
            .eval(&format!(
                "return type(_G[{FRAME_NAME:?}]:GetScript(\"OnEvent\"))"
            ))
            .expect("`GetScript(\"OnEvent\")` must run cleanly on ActionBarButtonEventsFrame");

        assert_eq!(
            onevent_script, "function",
            "Expected `{FRAME_NAME}:GetScript(\"OnEvent\")` to be a function after \
             `{ROOT}` loads, got `{onevent_script}`. XML at `{FRAME_XML_SITE}` (line 92) \
             wires `<OnEvent method=\"OnEvent\"/>` against the mixin's OnEvent at \
             lua:220, whose body fans the event out to every per-button frame in \
             `self.frames`. Without the script, every registered event fires but no \
             dispatch runs — breaks ActionButton OnLoad's \
             `ActionBarButtonEventsFrame:RegisterFrame(self)` subscription path \
             (lua:459)."
        );
    });
}
}

const ACTION_EVENTS_FRAME_NAME: &str = "ActionBarActionEventsFrame";
const ACTION_EVENTS_FRAME_XML_SITE: &str = "Shared/ActionButtonComponentTemplate.xml:96";
const ACTION_EVENTS_ONLOAD_LUA_SITE: &str = "Shared/ActionButton.lua:245";

/// Every event the source registers on `ActionBarActionEventsFrame`'s
/// OnLoad at `Shared/ActionButton.lua:245-288`. The list is structured
/// to mirror the source's four ordered blocks so a regression deleting
/// any sub-block produces a localised failure rather than a single
/// noisy panic.
///
/// **Block 1 — pre-spellcast plain events (14, lua:248-261).** Note:
/// `ACTIONBAR_UPDATE_USABLE` (lua:247) is COMMENTED OUT in source with
/// the inline note "replaced with ACTION_USABLE_CHANGED" — it is NOT
/// registered, so it does NOT appear here. (Pinning a commented-out
/// event would prove a parser stripped comments and registered the
/// line by mistake.) `UNIT_SPELLCAST_SENT` (lua:261) is registered as
/// a PLAIN event here, NOT a unit event — the IsSpellcastEvent helper
/// at lua:291-308 still classifies it as a spellcast for OnEvent
/// dispatch routing, but the registration itself is plain.
///
/// **Block 2 — spellcast unit events (11, lua:262-272).** All
/// registered via `RegisterUnitEvent(event, "player")`. The
/// IsSpellcastEvent helper enumerates 12 events as "spellcast" for
/// dispatch — 11 of these unit-registered ones plus the plain
/// `UNIT_SPELLCAST_SENT` from Block 1.
///
/// **Block 3 — post-spellcast plain events (6, lua:274-279).** Plain
/// `RegisterEvent` calls separated from Block 1 by the spellcast
/// block; the source orders them this way to keep the spellcast set
/// visually grouped.
///
/// **Block 4 — loss-of-control unit + spell-icon plain (3, lua:280-282).**
/// Two `RegisterUnitEvent("player")` for `LOSS_OF_CONTROL_*`, then one
/// trailing plain `RegisterEvent("SPELL_UPDATE_ICON")`.
///
/// Total: 14 + 11 + 6 + 3 = 34 events. PLAN says "25+" — a deliberate
/// loose lower bound. The actual count of 34 is the contract; pinning
/// it tightly catches both directions of drift.
const ACTION_EVENTS_REGISTERED: &[&str] = &[
    // Block 1 — pre-spellcast plain events (lua:248-261)
    "SPELL_UPDATE_CHARGES",
    "UPDATE_INVENTORY_ALERTS",
    "TRADE_SKILL_SHOW",
    "TRADE_SKILL_CLOSE",
    "ARCHAEOLOGY_CLOSED",
    "PLAYER_ENTER_COMBAT",
    "PLAYER_LEAVE_COMBAT",
    "START_AUTOREPEAT_SPELL",
    "STOP_AUTOREPEAT_SPELL",
    "UNIT_ENTERED_VEHICLE",
    "UNIT_EXITED_VEHICLE",
    "COMPANION_UPDATE",
    "UNIT_INVENTORY_CHANGED",
    "UNIT_SPELLCAST_SENT",
    // Block 2 — spellcast unit events (lua:262-272)
    "UNIT_SPELLCAST_INTERRUPTED",
    "UNIT_SPELLCAST_SUCCEEDED",
    "UNIT_SPELLCAST_FAILED",
    "UNIT_SPELLCAST_START",
    "UNIT_SPELLCAST_STOP",
    "UNIT_SPELLCAST_CHANNEL_START",
    "UNIT_SPELLCAST_CHANNEL_STOP",
    "UNIT_SPELLCAST_RETICLE_TARGET",
    "UNIT_SPELLCAST_RETICLE_CLEAR",
    "UNIT_SPELLCAST_EMPOWER_START",
    "UNIT_SPELLCAST_EMPOWER_STOP",
    // Block 3 — post-spellcast plain events (lua:274-279)
    "LEARNED_SPELL_IN_SKILL_LINE",
    "PET_STABLE_UPDATE",
    "PET_STABLE_SHOW",
    "SPELL_ACTIVATION_OVERLAY_GLOW_SHOW",
    "SPELL_ACTIVATION_OVERLAY_GLOW_HIDE",
    "UPDATE_SUMMONPETS_ACTION",
    // Block 4 — loss-of-control unit + spell-icon plain (lua:280-282)
    "LOSS_OF_CONTROL_ADDED",
    "LOSS_OF_CONTROL_UPDATE",
    "SPELL_UPDATE_ICON",
];

/// Event explicitly NOT registered (commented out in source). Pinned
/// as a tripwire so a future "uncomment to restore" change is forced
/// to update the spec.
const ACTION_EVENTS_COMMENTED_OUT: &str = "ACTIONBAR_UPDATE_USABLE";

post_wrath_ui_fixture! {
/// Pin `ActionBarActionEventsFrame`'s post-OnLoad event-registration
/// surface, the OnEvent script wiring, and the commented-out
/// `ACTIONBAR_UPDATE_USABLE` tripwire.
///
/// 37 assertions split into:
///
/// **Frame existence (1):** `_G.ActionBarActionEventsFrame` is a
/// table.
///
/// **All 34 registered events (34):** loop `ACTION_EVENTS_REGISTERED`
/// and pin `IsEventRegistered(event) == true`.
///
/// **Commented-out tripwire (1):** pin `IsEventRegistered(
/// "ACTIONBAR_UPDATE_USABLE") == false` — the comment at lua:247 says
/// it was "replaced with ACTION_USABLE_CHANGED", and a true reading
/// would prove a parser regression that stripped Lua comments and
/// registered the line.
///
/// **OnEvent script (1):** `type(:GetScript("OnEvent")) == "function"`
/// — XML at xml:99 wires `<OnEvent method="OnEvent"/>` against the
/// mixin's `OnEvent` body at lua:310.
#[cfg(feature = "optional-tests")]
#[test]
fn action_bar_action_events_frame_registers_thirty_four_spell_action_events_after_onload() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{ACTION_EVENTS_FRAME_NAME:?}])"))
            .expect("ActionBarActionEventsFrame global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{ACTION_EVENTS_FRAME_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{frame_type}`. The XML at `{ACTION_EVENTS_FRAME_XML_SITE}` \
             declares this frame as `<Frame name=\"ActionBarActionEventsFrame\" \
             mixin=\"ActionBarActionEventsFrameMixin\">`. A nil reading means either \
             the XML chunk failed to execute or the frame was removed. Every \
             event-registration assertion below depends on this frame existing."
        );

        for event in ACTION_EVENTS_REGISTERED {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{ACTION_EVENTS_FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .unwrap_or_else(|err| {
                    panic!(
                        "`{ACTION_EVENTS_FRAME_NAME}:IsEventRegistered({event:?})` raised: {err}"
                    )
                });

            assert!(
                registered,
                "Expected `{ACTION_EVENTS_FRAME_NAME}:IsEventRegistered({event:?})` to be \
                 true after `{ROOT}` loads. The mixin OnLoad at \
                 `{ACTION_EVENTS_ONLOAD_LUA_SITE}` registers `{event}` as part of one of \
                 four ordered blocks (pre-spellcast plain @ lua:248-261, spellcast \
                 unit-events @ lua:262-272, post-spellcast plain @ lua:274-279, \
                 loss-of-control unit + spell-icon plain @ lua:280-282 — see \
                 `ACTION_EVENTS_REGISTERED` const docstring for the full block \
                 attribution). The XML at `{ACTION_EVENTS_FRAME_XML_SITE}` wires \
                 `<OnLoad method=\"OnLoad\"/>`, and the simulator's `register_event` and \
                 `register_unit_event` impls in \
                 `src/lua_api/frame/methods/text_attribute_event/events.rs` both insert \
                 the event into the frame's `registered_events` set, so \
                 `IsEventRegistered` returns true regardless of the registration mode. A \
                 false reading means either OnLoad did not run, the registration call \
                 was removed, or the loader's method-style OnLoad dispatch regressed. \
                 Without `{event}` registered, the OnEvent dispatch at lua:310-336 — \
                 which fans non-spellcast events out to every per-button frame in \
                 `self.frames` and routes spellcast events through the \
                 `MatchesActiveButtonSpellID` filter — would silently drop `{event}` \
                 from the per-button update path."
            );
        }

        let commented_registered: bool = env
            .eval(&format!(
                "return _G[{ACTION_EVENTS_FRAME_NAME:?}]:IsEventRegistered({ACTION_EVENTS_COMMENTED_OUT:?})"
            ))
            .unwrap_or_else(|err| {
                panic!(
                    "`{ACTION_EVENTS_FRAME_NAME}:IsEventRegistered({ACTION_EVENTS_COMMENTED_OUT:?})` raised: {err}"
                )
            });

        assert!(
            !commented_registered,
            "Expected `{ACTION_EVENTS_FRAME_NAME}:IsEventRegistered(\
             {ACTION_EVENTS_COMMENTED_OUT:?})` to be false after `{ROOT}` loads. The \
             mixin OnLoad at `{ACTION_EVENTS_ONLOAD_LUA_SITE}` line 247 contains the \
             RegisterEvent call for `{ACTION_EVENTS_COMMENTED_OUT}` as a Lua line \
             comment (`--self:RegisterEvent(\"ACTIONBAR_UPDATE_USABLE\");`) with the \
             trailing note `replaced with ACTION_USABLE_CHANGED`. A true reading would \
             prove a Lua parser regression that strips comments before tokenisation, \
             OR Blizzard restored the registration without removing the replacement \
             note (in which case the spec needs updating to acknowledge the dual-event \
             registration)."
        );

        let onevent_script: String = env
            .eval(&format!(
                "return type(_G[{ACTION_EVENTS_FRAME_NAME:?}]:GetScript(\"OnEvent\"))"
            ))
            .expect("`GetScript(\"OnEvent\")` must run cleanly on ActionBarActionEventsFrame");

        assert_eq!(
            onevent_script, "function",
            "Expected `{ACTION_EVENTS_FRAME_NAME}:GetScript(\"OnEvent\")` to be a \
             function after `{ROOT}` loads, got `{onevent_script}`. The XML at \
             `{ACTION_EVENTS_FRAME_XML_SITE}` line 99 wires \
             `<OnEvent method=\"OnEvent\"/>` against the mixin's `OnEvent` body at \
             `Shared/ActionButton.lua:310`. The body branches on event name: a \
             `UNIT_INVENTORY_CHANGED` arm refreshes the tooltip owner if it's the \
             player, an `IsSpellcastEvent(event)` arm fans spellcast events through a \
             `MatchesActiveButtonSpellID` filter (lua:316-330), and the default arm \
             fans the event out to every per-button frame in `self.frames` \
             (lua:331-335). Without an OnEvent script wired, all 34 registered events \
             above would fire but the dispatch would have no handler — every \
             ActionButton would lose its action-update path."
        );
    });
}
}

const MAIN_ACTION_BAR_FRAME_NAME: &str = "MainActionBar";
const MAIN_ACTION_BAR_XML_SITE: &str = "Mainline/MainActionBar.xml:29";
const MAIN_ACTION_BAR_SCRIPTS_XML_SITE: &str = "Mainline/MainActionBar.xml:170";
const MAIN_ACTION_BAR_ONLOAD_LUA_SITE: &str = "Shared/MainActionBar.lua:5";

/// Events `MainActionBarMixin:OnLoad` registers at
/// `Shared/MainActionBar.lua:5-11`. Both are plain `RegisterEvent`
/// calls (lua:6 and lua:7), matching the PLAN line verbatim and with
/// no commented-out siblings or unit-event variants. After the two
/// registrations, OnLoad does two more things: sets `self.state =
/// "player"` (lua:9) and calls `MainActionBar.ActionBarPageNumber.Text:SetText(
/// C_ActionBar.GetActionBarPage())` (lua:10) — those non-registration
/// side effects are not pinned here.
const MAIN_ACTION_BAR_REGISTERED_EVENTS: &[&str] =
    &["ACTIONBAR_PAGE_CHANGED", "NEUTRAL_FACTION_SELECT_RESULT"];

/// Pin `MainActionBar`'s post-OnLoad event-registration surface.
///
/// **No spec/source mismatch on this one.** PLAN claim and source
/// agree exactly: `MainActionBarMixin:OnLoad` at
/// `Shared/MainActionBar.lua:5-11` registers exactly two events
/// (lua:6, lua:7), and the XML at `Mainline/MainActionBar.xml:172`
/// wires `<OnLoad method="OnLoad" inherit="prepend"/>` plus an OnEvent
/// dispatch at xml:173. The `inherit="prepend"` attribute means the
/// inherited (template) OnLoad runs FIRST and the instance (mixin)
/// OnLoad runs second — per the CLAUDE.md note on the counterintuitive
/// XML attribute. The template chain is `EditModeActionBarTemplate`
/// (xml:29 `inherits=`) → `EditModeSystemTemplate` → so the EditMode
/// system OnLoad runs before the mixin's two `RegisterEvent` calls,
/// but neither inherited OnLoad registers `ACTIONBAR_PAGE_CHANGED` or
/// `NEUTRAL_FACTION_SELECT_RESULT`, so the two PLAN-named events are
/// solely the mixin's contribution.
///
/// Four assertions:
///
/// 1. `type(_G.MainActionBar) == "table"` — frame exists, resolved
///    from `name="MainActionBar"` at `Mainline/MainActionBar.xml:29`.
/// 2-3. `IsEventRegistered("ACTIONBAR_PAGE_CHANGED") == true` and
///    `IsEventRegistered("NEUTRAL_FACTION_SELECT_RESULT") == true` —
///    OnLoad ran and registered both events.
/// 4. `type(:GetScript("OnEvent")) == "function"` — XML wires OnEvent
///    at xml:173, dispatch body at lua:30-36 has explicit arms for
///    each PLAN-named event (page-changed updates the
///    `ActionBarPageNumber.Text` from `C_ActionBar.GetActionBarPage()`,
///    neutral-faction-select calls `self:UpdateEndCaps()`).
#[cfg(feature = "optional-tests")]
#[test]
fn main_action_bar_registers_action_bar_page_changed_and_neutral_faction_select_result_after_onload()
 {
    wrath_action_case!(callbacks::main_events);
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{MAIN_ACTION_BAR_FRAME_NAME:?}])"))
            .expect("MainActionBar global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{MAIN_ACTION_BAR_FRAME_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{frame_type}`. The XML at `{MAIN_ACTION_BAR_XML_SITE}` declares \
             this frame as `<Frame name=\"MainActionBar\" \
             inherits=\"EditModeActionBarTemplate\" enableMouse=\"true\" \
             parent=\"UIParent\" frameLevel=\"50\" mixin=\"MainActionBarMixin\">`. A nil \
             reading means either the XML chunk failed to execute, the name= attribute \
             changed, or the frame was removed. Every event-registration assertion below \
             depends on this frame existing."
        );

        for event in MAIN_ACTION_BAR_REGISTERED_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{MAIN_ACTION_BAR_FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .unwrap_or_else(|err| {
                    panic!(
                        "`{MAIN_ACTION_BAR_FRAME_NAME}:IsEventRegistered({event:?})` raised: {err}"
                    )
                });

            assert!(
                registered,
                "Expected `{MAIN_ACTION_BAR_FRAME_NAME}:IsEventRegistered({event:?})` to \
                 be true after `{ROOT}` loads. The mixin OnLoad at \
                 `{MAIN_ACTION_BAR_ONLOAD_LUA_SITE}` declares \
                 `function MainActionBarMixin:OnLoad()` and registers `{event}` as one \
                 of its first two statements (lua:6 for ACTIONBAR_PAGE_CHANGED, lua:7 \
                 for NEUTRAL_FACTION_SELECT_RESULT). The XML at \
                 `{MAIN_ACTION_BAR_SCRIPTS_XML_SITE}` line 172 wires \
                 `<OnLoad method=\"OnLoad\" inherit=\"prepend\"/>` — `inherit=\"prepend\"` \
                 means the template's inherited OnLoad runs FIRST and the mixin's \
                 OnLoad runs second (the template chain via `EditModeActionBarTemplate` \
                 at xml:29 `inherits=` does not register either event, so this is \
                 solely the mixin's contribution). A false reading means either the \
                 mixin OnLoad did not run (regression in the loader's method-style \
                 OnLoad dispatch against `inherit=\"prepend\"`) or the RegisterEvent \
                 call was removed. Without `{event}` registered, the OnEvent dispatch \
                 arm at lua:31-35 — which updates `ActionBarPageNumber.Text` from \
                 `C_ActionBar.GetActionBarPage()` for ACTIONBAR_PAGE_CHANGED, or calls \
                 `self:UpdateEndCaps()` for NEUTRAL_FACTION_SELECT_RESULT — would never \
                 fire."
            );
        }

        let onevent_script: String = env
            .eval(&format!(
                "return type(_G[{MAIN_ACTION_BAR_FRAME_NAME:?}]:GetScript(\"OnEvent\"))"
            ))
            .expect("`GetScript(\"OnEvent\")` must run cleanly on MainActionBar");

        assert_eq!(
            onevent_script, "function",
            "Expected `{MAIN_ACTION_BAR_FRAME_NAME}:GetScript(\"OnEvent\")` to be a \
             function after `{ROOT}` loads, got `{onevent_script}`. The XML at \
             `{MAIN_ACTION_BAR_SCRIPTS_XML_SITE}` line 173 wires \
             `<OnEvent method=\"OnEvent\" inherit=\"prepend\"/>` against the mixin's \
             `OnEvent` body at `Shared/MainActionBar.lua:30`. The body has explicit \
             arms for each PLAN-named event: ACTIONBAR_PAGE_CHANGED refreshes \
             `MainActionBar.ActionBarPageNumber.Text` from \
             `C_ActionBar.GetActionBarPage()` (lua:31-32), and \
             NEUTRAL_FACTION_SELECT_RESULT triggers `self:UpdateEndCaps()` (lua:33-34) \
             — the latter re-evaluates the gryphon end-cap visibility for the new \
             faction's bar art. Without an OnEvent script wired, both registered \
             events would fire but the dispatch would have no handler, leaving the \
             page number text stuck on the initial `OnLoad`-time value (lua:10) and \
             the end-caps stuck on the old faction's art."
        );
    });
}

const STANCE_BAR_FRAME_NAME: &str = "StanceBar";
const STANCE_BAR_XML_SITE: &str = "Mainline/StanceBar.xml:12";
const STANCE_BAR_SCRIPTS_XML_SITE: &str = "Mainline/StanceBar.xml:32";
const STANCE_BAR_ONLOAD_LUA_SITE: &str = "Shared/StanceBar.lua:6";

/// `StanceBarMixin:OnLoad` (`Shared/StanceBar.lua:6-9`) registers one
/// event at lua:7 — PLAN matches exactly.
const STANCE_BAR_REGISTERED_EVENTS: &[&str] = &["UPDATE_SHAPESHIFT_COOLDOWN"];

/// Pin `StanceBar`'s post-OnLoad event-registration surface. No
/// spec/source mismatch. XML at `Mainline/StanceBar.xml:33` wires
/// `<OnLoad ... inherit="prepend"/>` + OnEvent at xml:34; the
/// `EditModeActionBarTemplate` chain at xml:12 does not register
/// `UPDATE_SHAPESHIFT_COOLDOWN`. Three assertions: frame exists, event
/// registered, OnEvent script wired (single arm at lua:12-14).
#[cfg(feature = "optional-tests")]
#[test]
fn stance_bar_registers_update_shapeshift_cooldown_after_onload() {
    wrath_action_case!(callbacks::stance_events);
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{STANCE_BAR_FRAME_NAME:?}])"))
            .expect("StanceBar global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{STANCE_BAR_FRAME_NAME:?}]` to be a table after `{ROOT}` loads, \
             got `{frame_type}`. XML at `{STANCE_BAR_XML_SITE}` declares \
             `<Frame name=\"StanceBar\" mixin=\"StanceBarMixin\" ...>`. Nil reading: \
             XML chunk failed, name= changed, or frame removed."
        );

        for event in STANCE_BAR_REGISTERED_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{STANCE_BAR_FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .expect("StanceBar:IsEventRegistered must run cleanly");

            assert!(
                registered,
                "Expected `{STANCE_BAR_FRAME_NAME}:IsEventRegistered({event:?})` to be \
                 true after `{ROOT}` loads. Mixin OnLoad at \
                 `{STANCE_BAR_ONLOAD_LUA_SITE}` registers `{event}` at lua:7; XML at \
                 `{STANCE_BAR_SCRIPTS_XML_SITE}:33` wires \
                 `<OnLoad ... inherit=\"prepend\"/>`. False reading: OnLoad did not \
                 run, RegisterEvent was removed, or prepend dispatch regressed. \
                 Without `{event}`, the single OnEvent arm at lua:12-14 \
                 (`self:UpdateState()`) never fires."
            );
        }

        let onevent_script: String = env
            .eval(&format!(
                "return type(_G[{STANCE_BAR_FRAME_NAME:?}]:GetScript(\"OnEvent\"))"
            ))
            .expect("`GetScript(\"OnEvent\")` must run cleanly on StanceBar");

        assert_eq!(
            onevent_script, "function",
            "Expected `{STANCE_BAR_FRAME_NAME}:GetScript(\"OnEvent\")` to be a function \
             after `{ROOT}` loads, got `{onevent_script}`. XML at \
             `{STANCE_BAR_SCRIPTS_XML_SITE}:34` wires \
             `<OnEvent method=\"OnEvent\" inherit=\"prepend\"/>` against the mixin's \
             `OnEvent` body at `Shared/StanceBar.lua:11` — a single \
             UPDATE_SHAPESHIFT_COOLDOWN arm calling `self:UpdateState()` (lua:12-14). \
             Without the script, the event fires but the dispatch has no handler."
        );
    });
}

const PET_ACTION_BAR_FRAME_NAME: &str = "PetActionBar";
const PET_ACTION_BAR_XML_SITE: &str = "Mainline/PetActionBar.xml:33";
const PET_ACTION_BAR_SCRIPTS_XML_SITE: &str = "Mainline/PetActionBar.xml:55";
const PET_ACTION_BAR_ONLOAD_LUA_SITE: &str = "Shared/PetActionBar.lua:49";

/// PLAN-named events — 9 plain `RegisterEvent` calls in
/// `PetActionBarMixin:OnLoad` (lua:50-60), in source order.
const PET_ACTION_BAR_PLAN_NAMED_EVENTS: &[&str] = &[
    "PLAYER_CONTROL_LOST",
    "PLAYER_CONTROL_GAINED",
    "UNIT_PET",
    "PET_BAR_UPDATE",
    "PET_BAR_UPDATE_COOLDOWN",
    "PET_BAR_UPDATE_USABLE",
    "PET_UI_UPDATE",
    "PLAYER_TARGET_CHANGED",
    "UPDATE_VEHICLE_ACTIONBAR",
];

/// Source-additional events PLAN omits. Each has an OnEvent dispatch
/// arm (lua:72-90) — dropping any breaks the corresponding refresh.
const PET_ACTION_BAR_SOURCE_ADDITIONAL_EVENTS: &[&str] = &[
    "PLAYER_FARSIGHT_FOCUS_CHANGED",
    "UNIT_FLAGS",
    "PLAYER_MOUNT_DISPLAY_CHANGED",
    "UNIT_AURA",
];

/// Pin `PetActionBar`'s post-OnLoad event-registration surface.
/// **Spec/source mismatch — PLAN under-counts.** PLAN names 9 events;
/// source `PetActionBarMixin:OnLoad` (`Shared/PetActionBar.lua:49-68`)
/// registers 13. Four source-only extras: PLAYER_FARSIGHT_FOCUS_CHANGED
/// (lua:52), UNIT_FLAGS (lua:54), PLAYER_MOUNT_DISPLAY_CHANGED (lua:61),
/// UNIT_AURA via `RegisterUnitEvent("pet")` (lua:62). Unit filter is
/// ignored for registration-set membership (both modes share
/// `registered_events.insert(event)` in
/// `src/lua_api/frame/methods/text_attribute_event/events.rs:51-70`).
/// XML at xml:56 wires `<OnLoad ... inherit="prepend"/>`; the
/// `EditModeActionBarTemplate` chain at xml:33 does not register any
/// of these 13 events. 15 assertions: existence (1), PLAN-named (9),
/// source-additional (4), OnEvent script (1).
#[cfg(feature = "optional-tests")]
#[test]
fn pet_action_bar_registers_plan_and_source_additional_events_after_onload() {
    wrath_action_case!(callbacks::pet_events);
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{PET_ACTION_BAR_FRAME_NAME:?}])"))
            .expect("PetActionBar global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{PET_ACTION_BAR_FRAME_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{frame_type}`. XML at `{PET_ACTION_BAR_XML_SITE}` declares \
             `<Frame name=\"PetActionBar\" mixin=\"PetActionBarMixin\" ...>`. Nil \
             reading: XML chunk failed, name= changed, or frame removed."
        );

        for event in PET_ACTION_BAR_PLAN_NAMED_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{PET_ACTION_BAR_FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .expect("PetActionBar:IsEventRegistered must run cleanly");

            assert!(
                registered,
                "Expected `{PET_ACTION_BAR_FRAME_NAME}:IsEventRegistered({event:?})` to \
                 be true after `{ROOT}` loads. Mixin OnLoad at \
                 `{PET_ACTION_BAR_ONLOAD_LUA_SITE}` registers `{event}` at lua:50-60; \
                 XML at `{PET_ACTION_BAR_SCRIPTS_XML_SITE}:56` wires \
                 `<OnLoad ... inherit=\"prepend\"/>`. False reading: OnLoad did not \
                 run, RegisterEvent was removed, or prepend dispatch regressed. The \
                 OnEvent dispatch at lua:70-91 silently drops `{event}` from the \
                 pet-bar visibility / state-refresh / cooldown arms."
            );
        }

        for event in PET_ACTION_BAR_SOURCE_ADDITIONAL_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{PET_ACTION_BAR_FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .expect("PetActionBar:IsEventRegistered must run cleanly");

            assert!(
                registered,
                "Expected `{PET_ACTION_BAR_FRAME_NAME}:IsEventRegistered({event:?})` to \
                 be true after `{ROOT}` loads. PLAN omits this event but source \
                 registers it in the same OnLoad at `{PET_ACTION_BAR_ONLOAD_LUA_SITE}` \
                 (PLAYER_FARSIGHT_FOCUS_CHANGED lua:52, UNIT_FLAGS lua:54, \
                 PLAYER_MOUNT_DISPLAY_CHANGED lua:61, UNIT_AURA via \
                 RegisterUnitEvent(\"pet\") lua:62 — unit filter ignored for \
                 registration-set membership). False reading: OnLoad regressed, the \
                 call was removed, or unit-event registration stopped marking. \
                 OnEvent dispatch at lua:72-90 has arms for all four — dropping any \
                 breaks the corresponding refresh path."
            );
        }

        let onevent_script: String = env
            .eval(&format!(
                "return type(_G[{PET_ACTION_BAR_FRAME_NAME:?}]:GetScript(\"OnEvent\"))"
            ))
            .expect("`GetScript(\"OnEvent\")` must run cleanly on PetActionBar");

        assert_eq!(
            onevent_script, "function",
            "Expected `{PET_ACTION_BAR_FRAME_NAME}:GetScript(\"OnEvent\")` to be a \
             function after `{ROOT}` loads, got `{onevent_script}`. XML at \
             `{PET_ACTION_BAR_SCRIPTS_XML_SITE}:57` wires \
             `<OnEvent method=\"OnEvent\" inherit=\"prepend\"/>` against the mixin's \
             `OnEvent` body at `Shared/PetActionBar.lua:70` — explicit arms for every \
             registered event (pet-bar visibility lua:72-81, generic refresh lua:82-83, \
             pet-filter lua:84-87, cooldown lua:88-89). Without the script, all 13 \
             events fire but no handler runs, leaving the pet bar stuck on its \
             OnLoad-time `:Update()` snapshot at lua:63."
        );
    });
}

const VEHICLE_LEAVE_FRAME_NAME: &str = "MainMenuBarVehicleLeaveButton";
const VEHICLE_LEAVE_XML_SITE: &str = "Shared/VehicleLeaveButton.xml:4";
const VEHICLE_LEAVE_SCRIPTS_XML_SITE: &str = "Shared/VehicleLeaveButton.xml:13";
const VEHICLE_LEAVE_ONLOAD_LUA_SITE: &str = "Shared/VehicleLeaveButton.lua:4";
const VEHICLE_LEAVE_REGISTERED_EVENTS: &[&str] = &[
    "UPDATE_BONUS_ACTIONBAR",
    "UPDATE_MULTI_CAST_ACTIONBAR",
    "UNIT_ENTERED_VEHICLE",
    "UNIT_EXITED_VEHICLE",
    "VEHICLE_UPDATE",
];

/// Pin `MainMenuBarVehicleLeaveButton`'s post-OnLoad event-registration
/// surface. PLAN names 5 events and the source registers exactly those 5
/// in `MainMenuBarVehicleLeaveButtonMixin:OnLoad` (lua:5-9) — no
/// mismatch. XML at xml:14 wires `<OnLoad ... inherit="prepend"/>`; the
/// `EditModeVehicleLeaveButtonSystemTemplate` chain at xml:4 contributes
/// no event registrations of its own. OnEvent at lua:25-27 has a single
/// catch-all body (`self:Update();`) — no per-event arms, so dropping
/// any registration silently breaks the show/enable/highlight refresh
/// driven by `:Update` at lua:37-52. 7 assertions: existence (1),
/// 5 registrations, OnEvent script (1).
#[cfg(feature = "optional-tests")]
#[test]
fn main_menu_bar_vehicle_leave_button_registers_five_events_after_onload() {
    wrath_action_case!(callbacks::vehicle_leave_events);
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{VEHICLE_LEAVE_FRAME_NAME:?}])"))
            .expect("MainMenuBarVehicleLeaveButton global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{VEHICLE_LEAVE_FRAME_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{frame_type}`. XML at `{VEHICLE_LEAVE_XML_SITE}` declares \
             `<Button name=\"MainMenuBarVehicleLeaveButton\" \
             mixin=\"MainMenuBarVehicleLeaveButtonMixin\" parent=\"MainActionBar\" \
             parentKey=\"VehicleLeaveButton\" ...>`. Nil reading: XML chunk failed, \
             name= changed, parent MainActionBar absent, or frame removed."
        );

        for event in VEHICLE_LEAVE_REGISTERED_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{VEHICLE_LEAVE_FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .expect("VehicleLeaveButton:IsEventRegistered must run cleanly");

            assert!(
                registered,
                "Expected `{VEHICLE_LEAVE_FRAME_NAME}:IsEventRegistered({event:?})` to \
                 be true after `{ROOT}` loads. Mixin OnLoad at \
                 `{VEHICLE_LEAVE_ONLOAD_LUA_SITE}` registers `{event}` at lua:5-9; XML \
                 at `{VEHICLE_LEAVE_SCRIPTS_XML_SITE}:14` wires \
                 `<OnLoad ... inherit=\"prepend\"/>`. False reading: OnLoad did not \
                 run, RegisterEvent was removed, or prepend dispatch regressed. The \
                 OnEvent body at lua:25-27 is a single catch-all `self:Update();` — \
                 dropping any registration silently breaks the show/enable/highlight \
                 refresh driven by `:Update` at lua:37-52."
            );
        }

        let onevent_script: String = env
            .eval(&format!(
                "return type(_G[{VEHICLE_LEAVE_FRAME_NAME:?}]:GetScript(\"OnEvent\"))"
            ))
            .expect("`GetScript(\"OnEvent\")` must run cleanly on VehicleLeaveButton");

        assert_eq!(
            onevent_script, "function",
            "Expected `{VEHICLE_LEAVE_FRAME_NAME}:GetScript(\"OnEvent\")` to be a \
             function after `{ROOT}` loads, got `{onevent_script}`. XML at \
             `{VEHICLE_LEAVE_SCRIPTS_XML_SITE}:15` wires `<OnEvent method=\"OnEvent\"/>` \
             (no inherit attribute — instance handler replaces any inherited one) \
             against the mixin's `OnEvent` body at `Shared/VehicleLeaveButton.lua:25` — \
             a catch-all `self:Update()` call at lua:26. Without the script, all 5 \
             events fire but no handler runs, leaving the button stuck on its \
             OnLoad-time `hidden=\"true\"` XML default at xml:4."
        );
    });
}
