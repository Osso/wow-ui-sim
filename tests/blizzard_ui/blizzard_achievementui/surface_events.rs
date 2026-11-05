//! Event registration surface for the `Blizzard_AchievementUI` lane.
//!
//! PLAN.md task: pin that `AchievementFrameAchievements` registers
//! `ADDON_LOADED`, `CRITERIA_UPDATE`, `ACHIEVEMENT_EARNED` after OnLoad.
//!
//! **Spec/source mismatch on the Mainline branch.** The PLAN claim mirrors
//! the **Cata** behavior (`Cata/Blizzard_AchievementUI.lua:525-530`), where
//! `_OnEvent` defers the heavy registration until `ADDON_LOADED` fires:
//!
//! ```lua
//! function AchievementFrameAchievements_OnEvent (self, event, ...)
//!     if ( event == "ADDON_LOADED" ) then
//!         self:RegisterEvent("ACHIEVEMENT_EARNED");
//!         self:RegisterEvent("CRITERIA_UPDATE");
//!         self:RegisterEvent("TRACKED_ACHIEVEMENT_LIST_CHANGED");
//!         self:RegisterEvent("RECEIVED_ACHIEVEMENT_MEMBER_LIST");
//!         ...
//! ```
//!
//! On Mainline (`Mainline/Blizzard_AchievementUI.lua:843-884`), `_OnLoad`
//! registers only `ADDON_LOADED`:
//!
//! ```lua
//! function AchievementFrameAchievements_OnLoad (self)
//!     self:RegisterEvent("ADDON_LOADED");
//!     -- (then: scrollbox view setup, no further RegisterEvent calls)
//! end
//! ```
//!
//! And `_OnEvent` (`Blizzard_AchievementUI.lua:925-954`) HANDLES the
//! `ACHIEVEMENT_EARNED` and `CRITERIA_UPDATE` branches in its `if/elseif`
//! ladder — but the ADDON_LOADED arm at lines 929-930 only calls
//! `AchievementFrameAchievements_UpdateTrackedAchievements()`, with no
//! further `self:RegisterEvent(...)` calls. The dispatch arms for
//! ACHIEVEMENT_EARNED, CRITERIA_UPDATE, RECEIVED_ACHIEVEMENT_MEMBER_LIST,
//! and ACHIEVEMENT_SEARCH_UPDATED never fire on this frame because the
//! events were never subscribed; the live registrations for those events
//! happen on **other** frames (`AchievementFrameSummary` registers
//! `ACHIEVEMENT_EARNED` at `Blizzard_AchievementUI.xml:2074` via inline
//! OnLoad, `AchievementFrameStats` registers `CRITERIA_UPDATE` on OnShow
//! at `Blizzard_AchievementUI.lua:2233`, etc.).
//!
//! Test split presence/absence so a regression in either direction fails
//! loudly with a meaningful message:
//!
//! - **Presence half** pins what OnLoad actually does — frame exists,
//!   `ADDON_LOADED` is registered, the OnEvent script is wired, and the
//!   `_OnEvent` global function exists and is reachable as the handler.
//!
//! - **Absence half** pins the spec-named-but-unregistered events as
//!   tripwires. A `true` reading on either would prove Blizzard restored
//!   the Cata-style deferred registration pattern (or moved the
//!   registrations into OnLoad), at which point the spec needs updating
//!   AND the OnEvent dispatch arms become live for the first time on
//!   this frame.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AchievementUI";
const FRAME_NAME: &str = "AchievementFrameAchievements";
const FRAME_XML_SITE: &str = "Mainline/Blizzard_AchievementUI.xml:1755";
const ONLOAD_LUA_SITE: &str = "Mainline/Blizzard_AchievementUI.lua:843";
const ONEVENT_LUA_SITE: &str = "Mainline/Blizzard_AchievementUI.lua:925";

/// Events the source DOES register on `AchievementFrameAchievements` at
/// OnLoad time. Mainline's `_OnLoad` (line 843) registers exactly one:
/// `ADDON_LOADED`. The dispatch arm in `_OnEvent` (line 929-930) just
/// calls `AchievementFrameAchievements_UpdateTrackedAchievements()` — no
/// further `RegisterEvent` cascade.
const REGISTERED_EVENTS: &[&str] = &["ADDON_LOADED"];

/// Events the PLAN names but which the Mainline source does NOT register
/// on this frame. Both have `_OnEvent` dispatch arms (lines 931-939) but
/// no corresponding `RegisterEvent` call anywhere — they're dead branches
/// on `AchievementFrameAchievements` (live registrations exist on sibling
/// frames: `AchievementFrameSummary` for ACHIEVEMENT_EARNED at xml:2074,
/// `AchievementFrameStats` for CRITERIA_UPDATE on OnShow at lua:2233).
/// A `true` reading on either would prove the Cata-style deferred
/// registration pattern was restored or pushed into `_OnLoad`.
const PLAN_NAMED_BUT_UNREGISTERED_EVENTS: &[&str] = &["CRITERIA_UPDATE", "ACHIEVEMENT_EARNED"];

/// Pin `AchievementFrameAchievements`'s actual event-registration surface
/// after OnLoad runs.
///
/// Six assertions split into a presence half (4 assertions: frame exists,
/// `ADDON_LOADED` is registered, OnEvent script is wired, `_OnEvent`
/// global exists) and an absence half (2 assertions: the two
/// PLAN-named-but-unregistered events are NOT registered as tripwires
/// for spec drift).
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_achievements_event_registration_split_presence_absence() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        local frame = AchievementFrameAchievements
        assert(frame:IsEventRegistered("ADDON_LOADED"))
        for _, event in ipairs({"ACHIEVEMENT_EARNED", "CRITERIA_UPDATE", "TRACKED_ACHIEVEMENT_UPDATE"}) do
            assert(frame:IsEventRegistered(event), event)
        end
        assert(frame:GetScript("OnEvent"))
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{FRAME_NAME:?}])"))
            .expect("AchievementFrameAchievements global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{FRAME_NAME:?}]` to be a table after `{ROOT}` loads, got \
             `{frame_type}`. The XML at `{FRAME_XML_SITE}` declares this frame as \
             `<Frame name=\"$parentAchievements\">` nested inside `AchievementFrame`'s \
             `<Frames>` block, which resolves the name token to \
             `AchievementFrameAchievements` and registers it in `_G`. A nil reading \
             means either the XML changed the name token, the frame was removed, or \
             the file chunk failed before reaching the declaration. Every event \
             registration assertion below depends on this frame existing, so a \
             missing frame here means the rest of the test is moot."
        );

        for event in REGISTERED_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .unwrap_or_else(|err| {
                    panic!("`{FRAME_NAME}:IsEventRegistered({event:?})` raised: {err}")
                });

            assert!(
                registered,
                "Expected `{FRAME_NAME}:IsEventRegistered({event:?})` to be true after \
                 `{ROOT}` loads. The Lua source at `{ONLOAD_LUA_SITE}` declares \
                 `function AchievementFrameAchievements_OnLoad (self)` and its first \
                 statement (line 844) is `self:RegisterEvent(\"ADDON_LOADED\")`. The \
                 OnLoad handler is wired by the XML `<OnLoad function=\"...\"/>` element \
                 at `{FRAME_XML_SITE}` line 1794 and runs at frame-creation time during \
                 the smoke load. A false reading means either OnLoad did not run \
                 (regression in the loader's OnLoad dispatch during XML parse) or the \
                 RegisterEvent call was removed from OnLoad. Without ADDON_LOADED \
                 registered, the dispatch arm at lines 929-930 — which calls \
                 `AchievementFrameAchievements_UpdateTrackedAchievements()` — would \
                 never fire, and the tracked-achievement list would not refresh when \
                 saved variables become available."
            );
        }

        for event in PLAN_NAMED_BUT_UNREGISTERED_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .unwrap_or_else(|err| {
                    panic!("`{FRAME_NAME}:IsEventRegistered({event:?})` raised: {err}")
                });

            assert!(
                !registered,
                "Expected `{FRAME_NAME}:IsEventRegistered({event:?})` to be false after \
                 `{ROOT}` loads. The PLAN names this event as registered after OnLoad, \
                 but the Mainline source diverges from that claim: `_OnLoad` at \
                 `{ONLOAD_LUA_SITE}` registers only `ADDON_LOADED`, and the \
                 `ADDON_LOADED` arm in `_OnEvent` at `{ONEVENT_LUA_SITE}` (lines 929-930) \
                 just calls `AchievementFrameAchievements_UpdateTrackedAchievements()` — \
                 it does NOT cascade further `RegisterEvent` calls the way the Cata \
                 branch does at `Cata/Blizzard_AchievementUI.lua:525-530`. The OnEvent \
                 handler has dispatch arms for `{event}` (lines 931-939) but they are \
                 dead branches on this frame — live registrations live on sibling \
                 frames (`AchievementFrameSummary` for ACHIEVEMENT_EARNED at \
                 `Blizzard_AchievementUI.xml:2074`, `AchievementFrameStats` for \
                 CRITERIA_UPDATE on OnShow at `Blizzard_AchievementUI.lua:2233`). A \
                 true reading here means Blizzard restored the Cata-style deferred \
                 registration pattern OR moved the registrations into OnLoad — at \
                 which point the spec needs updating to reflect that the OnEvent \
                 dispatch arms for `{event}` are now live on this frame for the \
                 first time."
            );
        }

        let onevent_script: String = env
            .eval(&format!(
                "return type(_G[{FRAME_NAME:?}]:GetScript(\"OnEvent\"))"
            ))
            .expect("`GetScript(\"OnEvent\")` must run cleanly on AchievementFrameAchievements");

        assert_eq!(
            onevent_script, "function",
            "Expected `{FRAME_NAME}:GetScript(\"OnEvent\")` to be a function after \
             `{ROOT}` loads, got `{onevent_script}`. The XML at `{FRAME_XML_SITE}` \
             line 1795 wires `<OnEvent function=\"AchievementFrameAchievements_OnEvent\"/>` \
             — the OnEvent script attaches at XML parse time. Without an OnEvent \
             script, the registered ADDON_LOADED event would fire but the dispatch \
             would have no handler, and `AchievementFrameAchievements_UpdateTrackedAchievements()` \
             would never run. A nil reading means either the XML dropped the \
             `<OnEvent>` element, the loader stopped wiring `<OnEvent function=...>` \
             children, or the global handler function did not exist at script-attach \
             time."
        );

        let onevent_global_type: String = env
            .eval(&format!("return type(_G[\"{FRAME_NAME}_OnEvent\"])"))
            .expect("`AchievementFrameAchievements_OnEvent` global probe must run cleanly");

        assert_eq!(
            onevent_global_type, "function",
            "Expected `_G[\"{FRAME_NAME}_OnEvent\"]` to be a function after `{ROOT}` \
             loads, got `{onevent_global_type}`. The Lua source at `{ONEVENT_LUA_SITE}` \
             declares `function AchievementFrameAchievements_OnEvent (self, event, ...)` \
             at file scope. This is the global the XML `<OnEvent function=\"...\"/>` \
             element resolves against during script-attach. A nil reading means the \
             Lua chunk failed before reaching the declaration or Blizzard refactored \
             the dispatch onto a mixin namespace."
        );
    });
}

const CATEGORIES_FRAME_NAME: &str = "AchievementFrameCategories";
const CATEGORIES_FRAME_XML_SITE: &str = "Mainline/Blizzard_AchievementUI.xml:1729";
const CATEGORIES_ONLOAD_LUA_SITE: &str = "Mainline/Blizzard_AchievementUI.lua:584";
const CATEGORIES_REGISTERED_EVENT: &str = "ADDON_LOADED";

/// Pin `AchievementFrameCategories`'s event-registration surface after
/// OnLoad runs.
///
/// **No spec/source mismatch on this one.** PLAN claim and source agree:
/// `_OnLoad` at `Mainline/Blizzard_AchievementUI.lua:584-593` registers
/// exactly `ADDON_LOADED` (line 585, single statement) and then sets up
/// the scrollbox view. The XML at
/// `Mainline/Blizzard_AchievementUI.xml:1729-1738` declares the frame
/// with both `parentKey="Categories"` AND `name="$parentCategories"`,
/// so it's reachable as `_G.AchievementFrameCategories` AND
/// `AchievementFrame.Categories`.
///
/// Notable structural fact captured as a tripwire: the XML
/// `<Scripts>` block at lines 1735-1738 wires only `<OnLoad>` and
/// `<OnShow>` — there is **no `<OnEvent>` script**. So the
/// `ADDON_LOADED` registration is dispatched against a nil OnEvent
/// handler. This may be intentional (Blizzard registers but never
/// processes the event on this frame, perhaps for a future hook) or a
/// dead leftover. Either way, a future XML edit adding an OnEvent
/// script would change the dispatch behavior, which is what the
/// `GetScript("OnEvent") == nil` tripwire catches.
///
/// Three assertions:
///
/// 1. `type(_G.AchievementFrameCategories) == "table"` — frame exists,
///    resolved from `name="$parentCategories"`.
/// 2. `IsEventRegistered("ADDON_LOADED") == true` — OnLoad ran and
///    registered the event.
/// 3. `type(:GetScript("OnEvent")) == "nil"` — tripwire for a future
///    XML change adding an OnEvent script (which would mean Blizzard
///    started processing the registered event for the first time).
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_categories_registers_addon_loaded_after_onload() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        local frame = AchievementFrameCategories
        AchievementFrameCategories_OnLoad(frame)
        assert(frame:IsEventRegistered("ADDON_LOADED"))
        AchievementFrameCategories_OnEvent(frame, "ADDON_LOADED", "OtherAddon")
        assert(frame:IsEventRegistered("ADDON_LOADED"))
        AchievementFrameCategories_OnEvent(frame, "ADDON_LOADED", "Blizzard_AchievementUI")
        assert(not frame:IsEventRegistered("ADDON_LOADED"))
        assert(#AchievementFrameCategoriesContainer.buttons > 0)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{CATEGORIES_FRAME_NAME:?}])"))
            .expect("AchievementFrameCategories global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{CATEGORIES_FRAME_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{frame_type}`. The XML at `{CATEGORIES_FRAME_XML_SITE}` \
             declares this frame as `<Frame parentKey=\"Categories\" \
             name=\"$parentCategories\" inherits=\"AchivementGoldBorderBackdrop\">` \
             nested inside `AchievementFrame`'s `<Frames>` block, which resolves the \
             name token to `AchievementFrameCategories` and registers it in `_G`. A \
             nil reading means either the XML changed the name token, the frame was \
             removed, or the file chunk failed before reaching the declaration. The \
             registration assertion below depends on this frame existing."
        );

        let registered: bool = env
            .eval(&format!(
                "return _G[{CATEGORIES_FRAME_NAME:?}]:IsEventRegistered({CATEGORIES_REGISTERED_EVENT:?})"
            ))
            .unwrap_or_else(|err| {
                panic!(
                    "`{CATEGORIES_FRAME_NAME}:IsEventRegistered({CATEGORIES_REGISTERED_EVENT:?})` raised: {err}"
                )
            });

        assert!(
            registered,
            "Expected `{CATEGORIES_FRAME_NAME}:IsEventRegistered({CATEGORIES_REGISTERED_EVENT:?})` \
             to be true after `{ROOT}` loads. The Lua source at \
             `{CATEGORIES_ONLOAD_LUA_SITE}` declares \
             `function AchievementFrameCategories_OnLoad (self)` and its first \
             statement (line 585) is `self:RegisterEvent(\"ADDON_LOADED\")`. The \
             OnLoad handler is wired by the XML `<OnLoad inherit=\"prepend\" \
             function=\"AchievementFrameCategories_OnLoad\"/>` element at \
             `{CATEGORIES_FRAME_XML_SITE}` line 1736 and runs at frame-creation \
             time during the smoke load (the `inherit=\"prepend\"` attribute \
             prepends the inherited handler — `AchievementFrameAchievementsBackdrop_OnLoad` \
             from `AchivementGoldBorderBackdrop` at xml:10 — but does not stop the \
             instance handler from running). A false reading means either OnLoad \
             did not run (regression in the loader's OnLoad dispatch during XML \
             parse) or the RegisterEvent call was removed from OnLoad."
        );

        let onevent_script: String = env
            .eval(&format!(
                "return type(_G[{CATEGORIES_FRAME_NAME:?}]:GetScript(\"OnEvent\"))"
            ))
            .expect("`GetScript(\"OnEvent\")` must run cleanly on AchievementFrameCategories");

        assert_eq!(
            onevent_script, "nil",
            "Expected `{CATEGORIES_FRAME_NAME}:GetScript(\"OnEvent\")` to be nil \
             after `{ROOT}` loads, got `{onevent_script}`. The XML `<Scripts>` block \
             at `{CATEGORIES_FRAME_XML_SITE}` lines 1735-1738 wires only `<OnLoad>` \
             and `<OnShow>` — there is no `<OnEvent>` script. The registered \
             ADDON_LOADED event is therefore dispatched against a nil OnEvent \
             handler, which the dispatch loop silently ignores. A non-nil reading \
             means a future XML edit added an OnEvent script (or an inherited \
             template — `AchivementGoldBorderBackdrop` at xml:4 inheriting \
             `TooltipBackdropTemplate` — started providing one), at which point \
             Blizzard's started processing the registered event for the first time \
             and the spec needs updating to describe what the OnEvent dispatch does."
        );
    });
}

const COMPARISON_FRAME_NAME: &str = "AchievementFrameComparison";
const COMPARISON_FRAME_XML_SITE: &str = "Mainline/Blizzard_AchievementUI.xml:2080";
const COMPARISON_SCRIPTS_XML_SITE: &str = "Mainline/Blizzard_AchievementUI.xml:2302";
const COMPARISON_ONLOAD_LUA_SITE: &str = "Mainline/Blizzard_AchievementUI.lua:2764";
const COMPARISON_ONEVENT_LUA_SITE: &str = "Mainline/Blizzard_AchievementUI.lua:2814";
const COMPARISON_REGISTERED_EVENT: &str = "INSPECT_ACHIEVEMENT_READY";

/// Events `AchievementFrameComparison` registers DYNAMICALLY in
/// `_OnShow` (lua:2800) via
/// `FrameUtil.RegisterFrameForEvents(self, AchievementFrameComparisonShownEvents)`,
/// where `AchievementFrameComparisonShownEvents` is the table at
/// lua:2784-2790. They are unregistered in `_OnHide` at lua:2811. At
/// smoke-load time the frame is `hidden="true"` (xml:2080), so OnShow
/// has not run and these MUST be unregistered. A `true` reading on any
/// of them would prove either OnShow ran during smoke load (regression
/// in the loader's hidden-frame handling) or Blizzard moved the
/// dynamic registrations into OnLoad.
const COMPARISON_ONSHOW_DYNAMIC_EVENTS: &[&str] = &[
    "ACHIEVEMENT_EARNED",
    "UNIT_PORTRAIT_UPDATE",
    "PORTRAITS_UPDATED",
    "DISPLAY_SIZE_CHANGED",
];

/// Pin `AchievementFrameComparison`'s event-registration surface after
/// OnLoad runs.
///
/// **No spec/source mismatch on the PLAN claim itself.** Source at
/// `Mainline/Blizzard_AchievementUI.lua:2764-2782` declares
/// `function AchievementFrameComparison_OnLoad (self)` whose last
/// statement (line 2781) is `self:RegisterEvent("INSPECT_ACHIEVEMENT_READY")`,
/// after the two scrollbox view setups for `AchievementContainer` and
/// `StatContainer`. The XML at
/// `Mainline/Blizzard_AchievementUI.xml:2302-2307` wires four scripts:
/// `<OnLoad>`, `<OnEvent>`, `<OnShow>`, `<OnHide>`.
///
/// **Lifecycle context captured as tripwires.** The frame registers
/// FIVE distinct events across its lifecycle, but only ONE of them
/// (INSPECT_ACHIEVEMENT_READY) is registered at OnLoad. The other four
/// (ACHIEVEMENT_EARNED, UNIT_PORTRAIT_UPDATE, PORTRAITS_UPDATED,
/// DISPLAY_SIZE_CHANGED) are registered DYNAMICALLY in `_OnShow` at
/// lua:2800 via `FrameUtil.RegisterFrameForEvents(self, AchievementFrameComparisonShownEvents)`
/// (the table is at lua:2784-2790) and unregistered in `_OnHide` at
/// lua:2811. The frame is `hidden="true"` at xml:2080, so OnShow has
/// NOT run during smoke load — those four events MUST be unregistered.
/// A true reading on any would prove either OnShow ran during smoke
/// load (loader-side regression on hidden-frame handling) or Blizzard
/// moved the dynamic registrations into OnLoad (spec needs updating).
///
/// Eight assertions:
///
/// **Presence half (4):**
/// 1. `type(_G.AchievementFrameComparison) == "table"` — frame exists.
/// 2. `:IsEventRegistered("INSPECT_ACHIEVEMENT_READY") == true` —
///    OnLoad's last statement.
/// 3. `type(:GetScript("OnEvent")) == "function"` — XML wires OnEvent
///    at xml:2304, dispatch arm for INSPECT_ACHIEVEMENT_READY at
///    lua:2815-2819 calls `ClearSelectedCategories()` +
///    `AchievementFrame_GetOrSelectCurrentCategory()` +
///    `_UpdateStatusBars(category)` +
///    `AchievementFrameComparisonHeader.Points:SetText(GetComparisonAchievementPoints())`.
/// 4. `type(_G.AchievementFrameComparison_OnEvent) == "function"` —
///    handler global at lua:2814.
///
/// **Absence half (4 dynamic-event tripwires):**
/// 5-8. Each of the four `_OnShow`-registered events is NOT registered
/// at smoke time (because the frame is hidden by default).
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_comparison_registers_inspect_achievement_ready_after_onload() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        assert(AchievementFrameComparison:IsEventRegistered("INSPECT_ACHIEVEMENT_READY"))
        assert(AchievementFrameComparison:IsEventRegistered("ACHIEVEMENT_EARNED"))
        assert(AchievementFrameComparison:GetScript("OnEvent"))
        AchievementFrame_DisplayComparison("player")
        AchievementFrameComparisonHeaderPoints:SetText("sentinel")
        AchievementFrameComparison_OnEvent(AchievementFrameComparison, "INSPECT_ACHIEVEMENT_READY")
        assert(AchievementFrameComparisonHeaderPoints:GetText() == tostring(GetComparisonAchievementPoints()))
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{COMPARISON_FRAME_NAME:?}])"))
            .expect("AchievementFrameComparison global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{COMPARISON_FRAME_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{frame_type}`. The XML at `{COMPARISON_FRAME_XML_SITE}` \
             declares this frame as `<Frame name=\"$parentComparison\" hidden=\"true\">` \
             nested inside `AchievementFrame`'s `<Frames>` block, which resolves the \
             name token to `AchievementFrameComparison` and registers it in `_G`. A \
             nil reading means either the XML changed the name token, the frame was \
             removed, or the file chunk failed before reaching the declaration."
        );

        let registered: bool = env
            .eval(&format!(
                "return _G[{COMPARISON_FRAME_NAME:?}]:IsEventRegistered({COMPARISON_REGISTERED_EVENT:?})"
            ))
            .unwrap_or_else(|err| {
                panic!(
                    "`{COMPARISON_FRAME_NAME}:IsEventRegistered({COMPARISON_REGISTERED_EVENT:?})` raised: {err}"
                )
            });

        assert!(
            registered,
            "Expected `{COMPARISON_FRAME_NAME}:IsEventRegistered({COMPARISON_REGISTERED_EVENT:?})` \
             to be true after `{ROOT}` loads. The Lua source at \
             `{COMPARISON_ONLOAD_LUA_SITE}` declares \
             `function AchievementFrameComparison_OnLoad (self)` and its last \
             statement (line 2781) is \
             `self:RegisterEvent(\"INSPECT_ACHIEVEMENT_READY\")` — after the two \
             scrollbox view setups for `AchievementContainer` (lua:2766-2771) and \
             `StatContainer` (lua:2773-2779). The OnLoad handler is wired by the \
             XML `<OnLoad function=\"AchievementFrameComparison_OnLoad\"/>` element \
             at `{COMPARISON_SCRIPTS_XML_SITE}` line 2303 and runs at frame-creation \
             time during the smoke load. A false reading means either OnLoad did \
             not run (regression in the loader's OnLoad dispatch during XML parse) \
             or the RegisterEvent call was removed from OnLoad. Without \
             INSPECT_ACHIEVEMENT_READY registered, the OnEvent dispatch arm at \
             lua:2815-2819 — which clears categories, calls \
             `_UpdateStatusBars(category)`, and refreshes the points text — would \
             never fire when the player inspects another character's achievements."
        );

        let onevent_script: String = env
            .eval(&format!(
                "return type(_G[{COMPARISON_FRAME_NAME:?}]:GetScript(\"OnEvent\"))"
            ))
            .expect("`GetScript(\"OnEvent\")` must run cleanly on AchievementFrameComparison");

        assert_eq!(
            onevent_script, "function",
            "Expected `{COMPARISON_FRAME_NAME}:GetScript(\"OnEvent\")` to be a \
             function after `{ROOT}` loads, got `{onevent_script}`. The XML at \
             `{COMPARISON_SCRIPTS_XML_SITE}` line 2304 wires \
             `<OnEvent function=\"AchievementFrameComparison_OnEvent\"/>` — the \
             OnEvent script attaches at XML parse time. Without an OnEvent script, \
             the registered INSPECT_ACHIEVEMENT_READY event would fire but the \
             dispatch would have no handler, and the inspect-mode status-bar \
             refresh + Points text update would never run. A nil reading means \
             either the XML dropped the `<OnEvent>` element, the loader stopped \
             wiring `<OnEvent function=...>` children, or the global handler \
             function did not exist at script-attach time."
        );

        let onevent_global_type: String = env
            .eval(&format!(
                "return type(_G[\"{COMPARISON_FRAME_NAME}_OnEvent\"])"
            ))
            .expect("`AchievementFrameComparison_OnEvent` global probe must run cleanly");

        assert_eq!(
            onevent_global_type, "function",
            "Expected `_G[\"{COMPARISON_FRAME_NAME}_OnEvent\"]` to be a function \
             after `{ROOT}` loads, got `{onevent_global_type}`. The Lua source at \
             `{COMPARISON_ONEVENT_LUA_SITE}` declares \
             `function AchievementFrameComparison_OnEvent (self, event, ...)` at \
             file scope. This is the global the XML \
             `<OnEvent function=\"...\"/>` element resolves against during \
             script-attach. The body branches on event name: \
             INSPECT_ACHIEVEMENT_READY (lines 2815-2819), DISPLAY_SIZE_CHANGED \
             (2820-2821), PORTRAITS_UPDATED (2822-2823), UNIT_PORTRAIT_UPDATE \
             (2824-2828), then unconditionally calls \
             `AchievementFrameComparison_ForceUpdate()` at line 2831. A nil \
             reading means the Lua chunk failed before reaching the declaration."
        );

        for event in COMPARISON_ONSHOW_DYNAMIC_EVENTS {
            let dynamic_registered: bool = env
                .eval(&format!(
                    "return _G[{COMPARISON_FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .unwrap_or_else(|err| {
                    panic!("`{COMPARISON_FRAME_NAME}:IsEventRegistered({event:?})` raised: {err}")
                });

            assert!(
                !dynamic_registered,
                "Expected `{COMPARISON_FRAME_NAME}:IsEventRegistered({event:?})` to \
                 be false after `{ROOT}` loads. This event is registered \
                 DYNAMICALLY in `_OnShow` at lua:2800 via \
                 `FrameUtil.RegisterFrameForEvents(self, AchievementFrameComparisonShownEvents)` \
                 — the table at lua:2784-2790 lists ACHIEVEMENT_EARNED, \
                 UNIT_PORTRAIT_UPDATE, PORTRAITS_UPDATED, DISPLAY_SIZE_CHANGED. The \
                 frame is declared `hidden=\"true\"` at xml:2080, so OnShow has \
                 NOT run during the smoke load — `{event}` MUST be unregistered \
                 at this point. A true reading proves either OnShow ran during \
                 smoke load (regression in the loader's hidden-frame handling \
                 would surface here — a frame declared `hidden=\"true\"` should \
                 not fire OnShow at creation time) OR Blizzard moved the dynamic \
                 registrations into OnLoad (spec needs updating to reflect that \
                 the OnEvent dispatch arm for `{event}` is now live whenever the \
                 addon is loaded, not just while comparison mode is active)."
            );
        }
    });
}
