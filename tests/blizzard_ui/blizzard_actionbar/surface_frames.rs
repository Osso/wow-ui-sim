//! Frame-shape surface pins for the `Blizzard_ActionBar` lane —
//! `MainActionBar` identity and its 12 action-button surface.
//!
//! PLAN.md task: `MainActionBar` exists, parent UIParent, has 12 button
//! children (`MainActionBarButton1`..`12`).
//!
//! **Spec/source mismatch — the PLAN line is wrong on TWO axes.** The
//! MainActionBar XML at `Mainline/MainActionBar.xml:29` declares the
//! frame correctly:
//!
//! ```xml
//! <Frame name="MainActionBar" inherits="EditModeActionBarTemplate"
//!        enableMouse="true" parent="UIParent" frameLevel="50"
//!        mixin="MainActionBarMixin">
//!     <Size x="454" y="35"/>
//!     ...
//!     <KeyValues>
//!         <KeyValue key="numButtons" value="12" type="number"/>
//!         <KeyValue key="buttonTemplate"
//!                   value="MainBarActionBarButtonTemplate" type="string"/>
//!         ...
//!     </KeyValues>
//!     ...
//! </Frame>
//! ```
//!
//! The 12 buttons are NOT direct children of `MainActionBar`. Each
//! button is wrapped in an intermediate container Frame, and the buttons
//! themselves use a special-case name. Both behaviors come from the
//! shared `ActionBarMixin:ActionBar_OnLoad` at
//! `Shared/ActionBar.lua:3-55`:
//!
//! ```lua
//! for i=1, self.numButtons do
//!     local buttonContainer = CreateFrame("Frame",
//!         actionBarName.."ButtonContainer"..i, self,
//!         "ActionBarButtonContainerTemplate", i);
//!
//!     local buttonName;
//!     if self == MainActionBar then
//!         buttonName = "ActionButton"..i;        -- <-- special case!
//!     elseif self == StanceBar then
//!         buttonName = "StanceButton"..i;
//!     elseif self == PetActionBar then
//!         buttonName = "PetActionButton"..i;
//!     elseif self == PossessActionBar then
//!         buttonName = "PossessButton"..i;
//!     else
//!         buttonName = actionBarName.."Button"..i;
//!     end
//!
//!     local actionButton = CreateFrame("CheckButton", buttonName,
//!         buttonContainer, self.buttonTemplate, i);
//!     ...
//!     table.insert(self.actionButtons, actionButton);
//! end
//! ```
//!
//! So the actual 12-deep surface is:
//!
//! - `MainActionBar` (parent: `UIParent`)
//!   - `MainActionBarButtonContainer<N>` (parent: `MainActionBar`,
//!     N = 1..12) — the layer the PLAN line would have meant by "12
//!     button children" if it were checking direct children of the bar.
//!     - `ActionButton<N>` (parent: `MainActionBarButtonContainer<N>`)
//!       — the CheckButton the player actually clicks. Named per the
//!       comment at `Shared/ActionBar.lua:16-18`: *"Different naming
//!       for these bars is to avoid errors with legacy code"* —
//!       Bindings.xml entries that reference `ActionButton1` directly,
//!       plus the legacy `ActionButton<N>:Click()` call sites in
//!       Blizzard's own `SecureActionButton_OnClick` plumbing, all
//!       pre-date the multi-bar refactor and would break under any
//!       other naming scheme.
//!
//! Conclusion: `_G.MainActionBarButton1`..`MainActionBarButton12` do
//! NOT exist. They cannot exist at the same time as `_G.ActionButton1`
//! ..`ActionButton12` (the special-case branch is mutually exclusive
//! with the fall-through that uses `actionBarName.."Button"..i`).
//!
//! Test split along the spec/source boundary, modeled on the same
//! pattern as
//! `tests/blizzard_ui/blizzard_achievementui/surface_frames.rs`'s
//! `achievement_frame_search_progress_bar_split_presence_absence`:
//!
//! - **Presence half** (`main_action_bar_publishes_expected_panel_identity`):
//!   pin the three PLAN facts that ARE true — `MainActionBar` exists
//!   as a table global, has `parent == UIParent`, and
//!   `MainActionBar.actionButtons` is a 12-entry array (the internal
//!   contract every consumer of the bar reads, set at
//!   `Shared/ActionBar.lua:7` and populated at `:41`). The frameLevel
//!   declared in XML (50) is intentionally NOT pinned: the
//!   `EditModeSystemMixin:OnSystemLoad` chain triggered by
//!   `EditModeActionBar_OnLoad` at `Shared/ActionBar.lua:256-264`
//!   bumps the runtime level by +2 (the bar reports 52 post-load),
//!   so a strict `frameLevel == 50` would fail despite the XML being
//!   correct. The PLAN line doesn't ask for frameLevel — pinning a
//!   derived runtime value would conflate XML correctness with
//!   EditMode plumbing.
//!
//! - **Actual button publishing half**
//!   (`main_action_bar_publishes_action_button_globals_and_button_containers`):
//!   pin that the 12 PLAN-meant slots ARE published, just not under
//!   the PLAN names. Each slot N has `_G.ActionButton<N>` as the
//!   CheckButton (special-case name from
//!   `Shared/ActionBar.lua:19-20`), `_G.MainActionBarButtonContainer<N>`
//!   as the wrapping Frame (line 14 — every multi-bar uses the same
//!   `ButtonContainer` suffix on the bar's own name), and the parent
//!   chain `ActionButton<N> → MainActionBarButtonContainer<N> →
//!   MainActionBar` resolves cleanly via `:GetParent()` on each step.
//!
//! - **Absence half**
//!   (`main_action_bar_does_not_publish_plan_named_button_globals`):
//!   pin that `_G.MainActionBarButton<N>` is nil for N = 1..12. A
//!   non-nil reading would prove either Blizzard removed the
//!   special-case branch at `Shared/ActionBar.lua:19-20` (so
//!   MainActionBar's buttons fell through to
//!   `actionBarName.."Button"..i` and got the PLAN-named global) or
//!   the simulator double-published the buttons under both names
//!   (which would shadow `ActionButton<N>` consumers via stale
//!   references). Either way the spec needs revisiting before the
//!   test is "fixed" by mutating the assertion.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_ActionBar";
const FRAME_NAME: &str = "MainActionBar";
const XML_SITE: &str = "Mainline/MainActionBar.xml:29";
const ACTION_BAR_LUA_SITE: &str = "Shared/ActionBar.lua:3-55";
const NUM_BUTTONS: usize = 12;

#[cfg(feature = "optional-tests")]
#[test]
fn main_action_bar_publishes_expected_panel_identity() {
    wrath_action_case!(callbacks::main_identity);
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{FRAME_NAME:?}])"))
            .expect("MainActionBar global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{FRAME_NAME:?}]` to be a table after `{ROOT}` loads, got \
             `{frame_type}`. The XML at `{XML_SITE}` declares this frame with \
             `name=\"MainActionBar\"` and `parent=\"UIParent\"`, so the named-frame \
             registration runs at XML-load time. A nil reading means either the XML did not \
             execute (a regression in the load pipeline) or the frame failed to register its \
             name. Every downstream consumer that names `MainActionBar` directly would \
             surface a nil-table-method error — including `IsNormalActionBarState()` at \
             `Shared/MultiActionBars.lua:54` (`return MainActionBar:IsShown()`), \
             `MainActionBarMixin:OnLoad`/`OnEvent`/`OnShow` at `Shared/MainActionBar.lua`, \
             every page-cycle handler that calls `MainActionBar.ActionBarPageNumber.Text:SetText` \
             at `Shared/MainActionBar.lua:10`, and the assist-combat highlight hook at \
             `Mainline/AssistedCombatManager.lua:223` (`MainActionBar:GetEndCapsFrameLevel`)."
        );

        let parent_name: String = env
            .eval(&format!("return _G[{FRAME_NAME:?}]:GetParent():GetName()"))
            .expect("`GetParent():GetName()` must run cleanly on MainActionBar");

        assert_eq!(
            parent_name, "UIParent",
            "Expected `MainActionBar:GetParent():GetName()` to return `UIParent`, got \
             `{parent_name}`. The XML at `{XML_SITE}` declares `parent=\"UIParent\"` \
             literally — `UIParent` is the standard scaled-UI root that `SetUIScale` and \
             the resolution-aware reparenting drive against. A regression that reparents \
             the action bar onto `WorldFrame` (the 3D world root) or some intermediate \
             would break user-set UI scaling and detach the bar from `UIParent.Hide()` \
             cascading."
        );

        let action_buttons_count: i64 = env
            .eval(&format!("return #_G[{FRAME_NAME:?}].actionButtons"))
            .expect("`#MainActionBar.actionButtons` must run cleanly");

        let expected_count = NUM_BUTTONS as i64;
        assert_eq!(
            action_buttons_count, expected_count,
            "Expected `#MainActionBar.actionButtons` to be `{expected_count}`, got \
             `{action_buttons_count}`. The XML at `{XML_SITE}` declares \
             `<KeyValue key=\"numButtons\" value=\"12\" type=\"number\"/>`, which \
             `ActionBarMixin:ActionBar_OnLoad` at `{ACTION_BAR_LUA_SITE}` reads and uses to \
             drive a `for i=1, self.numButtons do ... table.insert(self.actionButtons, \
             actionButton) end` loop (lua:13/41). The `actionButtons` array is the \
             internal contract every consumer of the bar reads — `UpdateShownButtons()` \
             walks it (lua:200-209), `SetShowGrid()` walks it (lua:144-171), and \
             `ActionBarController` queries it via `bar.actionButtons[i]` for keybind \
             dispatch. A short array means either the OnLoad chunk failed mid-loop or \
             Blizzard moved away from the file-local table pattern."
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn main_action_bar_publishes_action_button_globals_and_button_containers() {
    wrath_action_case!(callbacks::main_buttons);
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for slot in 1..=NUM_BUTTONS {
            let action_button_name = format!("ActionButton{slot}");
            let container_name = format!("MainActionBarButtonContainer{slot}");

            let button_object_type: String = env
                .eval(&format!(
                    "return type(_G[{action_button_name:?}]) == \"table\" and \
                            _G[{action_button_name:?}]:GetObjectType() or \
                            type(_G[{action_button_name:?}])"
                ))
                .expect("ActionButton<N> object-type probe must run cleanly");

            assert_eq!(
                button_object_type, "CheckButton",
                "Expected `_G[{action_button_name:?}]:GetObjectType()` to return `CheckButton` \
                 after `{ROOT}` loads, got `{button_object_type}`. \
                 `ActionBarMixin:ActionBar_OnLoad` at `{ACTION_BAR_LUA_SITE}` line 19-20 \
                 special-cases `MainActionBar` to name its buttons `ActionButton<N>` (NOT \
                 `MainActionBarButton<N>` — see the absence test \
                 `main_action_bar_does_not_publish_plan_named_button_globals` for the \
                 inverse pin). The XML at `{XML_SITE}` declares \
                 `<KeyValue key=\"buttonTemplate\" value=\"MainBarActionBarButtonTemplate\" \
                 type=\"string\"/>`, and `MainBarActionBarButtonTemplate` at \
                 `Mainline/MainActionBar.xml:3` inherits `ActionBarButtonTemplate` which \
                 is itself a `<CheckButton>`. A non-CheckButton or nil reading means \
                 either the special-case branch at lua:19-20 broke (so the button name \
                 fell through to the generic suffix) or the OnLoad loop failed before \
                 reaching slot {slot}. Bindings.xml `ActionButton<N>` keybinds and the \
                 legacy `ActionButton<N>:Click()` call sites all assume this name."
            );

            let button_parent_name: String = env
                .eval(&format!(
                    "return _G[{action_button_name:?}]:GetParent():GetName()"
                ))
                .expect("ActionButton<N>:GetParent():GetName() must run cleanly");

            assert_eq!(
                button_parent_name, container_name,
                "Expected `_G[{action_button_name:?}]:GetParent():GetName()` to return \
                 `{container_name}`, got `{button_parent_name}`. \
                 `ActionBarMixin:ActionBar_OnLoad` at `{ACTION_BAR_LUA_SITE}` line 14 \
                 creates each button's container as \
                 `CreateFrame(\"Frame\", actionBarName..\"ButtonContainer\"..i, self, \
                 \"ActionBarButtonContainerTemplate\", i)`, then line 31 creates the \
                 button as `CreateFrame(\"CheckButton\", buttonName, buttonContainer, \
                 self.buttonTemplate, i)` — so the button's parent IS the container, NOT \
                 `MainActionBar` directly. A different parent name means either the \
                 wrapping container was elided (so `ActionButton{slot}` would parent \
                 directly onto `MainActionBar` and break the `buttonContainer:SetSize` \
                 sizing layer at lua:43) or the container's name was changed."
            );

            let container_parent_name: String = env
                .eval(&format!(
                    "return _G[{container_name:?}]:GetParent():GetName()"
                ))
                .expect("MainActionBarButtonContainer<N>:GetParent():GetName() must run cleanly");

            assert_eq!(
                container_parent_name, FRAME_NAME,
                "Expected `_G[{container_name:?}]:GetParent():GetName()` to return \
                 `{FRAME_NAME}`, got `{container_parent_name}`. \
                 `ActionBarMixin:ActionBar_OnLoad` at `{ACTION_BAR_LUA_SITE}` line 14 \
                 creates each button container with `self` as the parent (where `self` \
                 is `MainActionBar` for this bar's invocation of the OnLoad), so the \
                 container's parent must be `MainActionBar`. A different parent name \
                 means the OnLoad ran on a different frame (e.g. the `for i=1, \
                 self.numButtons` loop self-reference broke) or the container was \
                 reparented post-load — neither has a consumer in Blizzard's own code so \
                 either would be a simulator-side regression."
            );
        }
    });
}

/// PLAN-named multi-bars and their XML declaration sites in
/// `Shared/MultiActionBars.xml`. All seven inherit
/// `EditModeActionBarTemplate`, all have `parent="UIParent"`, all have
/// `frameStrata="MEDIUM"`, all are `hidden="true"`, all declare
/// `numButtons=12`. They differ in button template
/// (`MultiBar<N>ButtonTemplate`), command-name prefix
/// (`MULTIACTIONBAR<N>`), orientation (BottomLeft/BottomRight/5/6/7
/// horizontal, Left/Right vertical), and EditMode system index.
///
/// Unlike `MainActionBar` (which special-cases its button names to
/// `ActionButton<N>` per `Shared/ActionBar.lua:19-20`), the multi-bars
/// fall through to the generic suffix branch at
/// `Shared/ActionBar.lua:28` (`buttonName = actionBarName..\"Button\"..i`).
/// So `MultiBarBottomLeft` publishes `MultiBarBottomLeftButton1`..`12`,
/// `MultiBar5` publishes `MultiBar5Button1`..`12`, etc. — each bar's
/// button globals share the bar's own name as a prefix.
const PLAN_NAMED_MULTI_BARS: &[(&str, &str)] = &[
    ("MultiBarBottomLeft", "Shared/MultiActionBars.xml:45"),
    ("MultiBarBottomRight", "Shared/MultiActionBars.xml:75"),
    ("MultiBarLeft", "Shared/MultiActionBars.xml:104"),
    ("MultiBarRight", "Shared/MultiActionBars.xml:133"),
    ("MultiBar5", "Shared/MultiActionBars.xml:162"),
    ("MultiBar6", "Shared/MultiActionBars.xml:191"),
    ("MultiBar7", "Shared/MultiActionBars.xml:220"),
];

/// Pin each of the seven multi-bars: exists as a table global, parent
/// is `UIParent`, and `actionButtons` array holds exactly 12
/// `CheckButton`s. The PLAN line covers all seven bars in one entry —
/// a single test loops them so a regression on any one bar fails fast
/// with the bar name in the assertion message.
///
/// Why parent-chain is pinned here too: every multi-bar's XML declares
/// `parent="UIParent"` literally, but `EditMode` reparents bars when
/// the player drags them onto attached frames (e.g.
/// `MainActionBarMixin:AttachToFrame` at
/// `Shared/MainActionBar.lua:38-46` does `self:SetParent(frame)` on
/// MainActionBar; while no equivalent exists for the multi-bars in
/// the current codebase, the EditMode test fixtures could exercise
/// reparenting). At rest / immediately after addon load, every
/// multi-bar must be parented to `UIParent` so `SetUIScale` and the
/// `UIParent.Hide()` cascade reach them. A different parent name
/// here flags a regression in either the XML declaration or the
/// EditMode reparenting logic firing pre-load.
#[cfg(feature = "optional-tests")]
#[test]
fn multi_bars_publish_expected_panel_identity() {
    wrath_action_case!(callbacks::multibar_identity);
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for (bar_name, xml_site) in PLAN_NAMED_MULTI_BARS {
            let frame_type: String = env
                .eval(&format!("return type(_G[{bar_name:?}])"))
                .expect("multi-bar global probe must run cleanly");

            assert_eq!(
                frame_type, "table",
                "Expected `_G[{bar_name:?}]` to be a table after `{ROOT}` loads, got \
                 `{frame_type}`. The XML at `{xml_site}` declares this frame with \
                 `name=\"{bar_name}\"` and `parent=\"UIParent\"`, so the named-frame \
                 registration runs at XML-load time. A nil reading means either the XML \
                 did not execute (a regression in the load pipeline) or the frame failed \
                 to register its name. Every downstream consumer that names `{bar_name}` \
                 directly would surface a nil-table-method error — including the \
                 `MultiBar<N>_IsVisible()` trampolines (lua:140-164 in MultiActionBars.lua) \
                 that call `_G[barName]:IsShown()` indirectly via the file-local \
                 `IsMultibarVisible(index)` helper, EditMode's bar-list builder that \
                 reaches into `MultiBarBottomLeft`/`Right`/`Left`/`Right` directly, and \
                 the per-page driver in `MultiActionBar_GetBarForPage` (lua:131) that \
                 returns `bars[page].bar` from the file-local `GetMultiActionBars()` \
                 table at lua:58-71."
            );

            let parent_name: String = env
                .eval(&format!("return _G[{bar_name:?}]:GetParent():GetName()"))
                .expect("multi-bar GetParent():GetName() must run cleanly");

            assert_eq!(
                parent_name, "UIParent",
                "Expected `_G[{bar_name:?}]:GetParent():GetName()` to return `UIParent`, \
                 got `{parent_name}`. The XML at `{xml_site}` declares \
                 `parent=\"UIParent\"` literally — `UIParent` is the standard scaled-UI \
                 root that `SetUIScale` and the resolution-aware reparenting drive \
                 against. A different parent name means either the XML changed or \
                 EditMode's reparenting (analogous to `MainActionBarMixin:AttachToFrame` \
                 at `Shared/MainActionBar.lua:38-46`) fired pre-load and stranded the bar \
                 outside the UIParent scaling chain — `UIParent.Hide()` cascading and \
                 user-set UI scaling would both miss the bar."
            );

            let action_buttons_count: i64 = env
                .eval(&format!("return #_G[{bar_name:?}].actionButtons"))
                .expect("multi-bar #actionButtons must run cleanly");

            let expected_count = NUM_BUTTONS as i64;
            assert_eq!(
                action_buttons_count, expected_count,
                "Expected `#_G[{bar_name:?}].actionButtons` to be `{expected_count}`, got \
                 `{action_buttons_count}`. The XML at `{xml_site}` declares \
                 `<KeyValue key=\"numButtons\" value=\"12\" type=\"number\"/>`, which \
                 `ActionBarMixin:ActionBar_OnLoad` at `{ACTION_BAR_LUA_SITE}` reads and \
                 uses to drive a `for i=1, self.numButtons do ... \
                 table.insert(self.actionButtons, actionButton) end` loop (lua:13/41). \
                 The `actionButtons` array is the internal contract every consumer of \
                 the bar reads — `UpdateShownButtons()` walks it (lua:200-209), \
                 `SetShowGrid()` walks it (lua:144-171), and `MultiActionButtonDown` / \
                 `MultiActionButtonUp` (Shared/MultiActionBars.lua:35/44) reach into \
                 `_G[barName].actionButtons[id]` for keybind dispatch. A short array \
                 means either the OnLoad chunk failed mid-loop (fewer than 12 buttons \
                 created) or Blizzard moved away from the file-local table pattern."
            );
        }
    });
}

/// Pin the per-bar button publishing for all seven multi-bars: each
/// slot N has `_G[<BarName>Button<N>]` as a CheckButton with the
/// parent chain `<BarName>Button<N>` → `<BarName>ButtonContainer<N>`
/// → `<BarName>`. 7 bars × 12 slots × 3 assertions = 252 checks per
/// run, but the failure is bounded to the first slot/bar pair that
/// regresses (the loop `assert_eq!` exits the test on first
/// mismatch).
///
/// This is the inverse of the MainActionBar absence test: the
/// multi-bars fall through to the generic suffix branch at
/// `Shared/ActionBar.lua:28` (`buttonName = actionBarName..\"Button\"..i`)
/// so the PLAN-meant naming pattern (`<BarName>Button<N>`) IS the
/// actual published surface. A nil reading on any of the buttons would
/// prove either (a) a regression added a special case for one of these
/// bars that mirrors the MainActionBar branch (which would silently
/// rename the buttons and break every Bindings.xml entry that names
/// `MultiBarBottomLeftButton1` and friends), OR (b) the OnLoad loop
/// at `Shared/ActionBar.lua:13-44` failed before reaching slot N for
/// the bar in question.
#[cfg(feature = "optional-tests")]
#[test]
fn multi_bars_publish_their_button_globals_and_button_containers() {
    wrath_action_case!(callbacks::multibar_buttons);
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for (bar_name, xml_site) in PLAN_NAMED_MULTI_BARS {
            for slot in 1..=NUM_BUTTONS {
                let button_name = format!("{bar_name}Button{slot}");
                let container_name = format!("{bar_name}ButtonContainer{slot}");

                let button_object_type: String = env
                    .eval(&format!(
                        "return type(_G[{button_name:?}]) == \"table\" and \
                                _G[{button_name:?}]:GetObjectType() or \
                                type(_G[{button_name:?}])"
                    ))
                    .expect("multi-bar button object-type probe must run cleanly");

                assert_eq!(
                    button_object_type, "CheckButton",
                    "Expected `_G[{button_name:?}]:GetObjectType()` to return `CheckButton` \
                     after `{ROOT}` loads, got `{button_object_type}`. The XML at \
                     `{xml_site}` declares `<KeyValue key=\"buttonTemplate\" \
                     value=\"...\"/>` referencing one of the `MultiBar<N>ButtonTemplate` \
                     CheckButton templates at `Shared/MultiActionBars.xml:3-43`, all of \
                     which inherit `ActionBarButtonTemplate` (a CheckButton). \
                     `ActionBarMixin:ActionBar_OnLoad` at `{ACTION_BAR_LUA_SITE}` line 28 \
                     falls through to the generic-suffix branch for the multi-bars (the \
                     special-case branch at lua:19-26 only matches MainActionBar, \
                     StanceBar, PetActionBar, PossessActionBar — none of the multi-bars), \
                     so `buttonName = actionBarName..\"Button\"..i` produces \
                     `{button_name}`. A non-CheckButton or nil reading means either (a) \
                     a regression added a `self == {bar_name}` special-case branch at \
                     lua:19-26 that renamed this bar's buttons (silently breaking every \
                     Bindings.xml entry that names `{button_name}` directly), OR (b) \
                     the OnLoad loop failed before reaching slot {slot}, OR (c) the \
                     `buttonTemplate` KeyValue was changed to a different widget kind."
                );

                let button_parent_name: String = env
                    .eval(&format!("return _G[{button_name:?}]:GetParent():GetName()"))
                    .expect("multi-bar button GetParent():GetName() must run cleanly");

                assert_eq!(
                    button_parent_name, container_name,
                    "Expected `_G[{button_name:?}]:GetParent():GetName()` to return \
                     `{container_name}`, got `{button_parent_name}`. \
                     `ActionBarMixin:ActionBar_OnLoad` at `{ACTION_BAR_LUA_SITE}` line 14 \
                     creates each button's container as `CreateFrame(\"Frame\", \
                     actionBarName..\"ButtonContainer\"..i, self, \
                     \"ActionBarButtonContainerTemplate\", i)`, then line 31 creates the \
                     button as `CreateFrame(\"CheckButton\", buttonName, buttonContainer, \
                     self.buttonTemplate, i)` — so the button's parent IS the container, \
                     NOT the bar directly. A different parent name means either the \
                     wrapping container was elided (button parented directly onto \
                     `{bar_name}`, which would break the `buttonContainer:SetSize` \
                     sizing layer at lua:43) or the container's name was changed."
                );

                let container_parent_name: String = env
                    .eval(&format!(
                        "return _G[{container_name:?}]:GetParent():GetName()"
                    ))
                    .expect("multi-bar container GetParent():GetName() must run cleanly");

                assert_eq!(
                    container_parent_name, *bar_name,
                    "Expected `_G[{container_name:?}]:GetParent():GetName()` to return \
                     `{bar_name}`, got `{container_parent_name}`. \
                     `ActionBarMixin:ActionBar_OnLoad` at `{ACTION_BAR_LUA_SITE}` line 14 \
                     creates each button container with `self` as the parent (where \
                     `self` is `{bar_name}` for this bar's invocation of the OnLoad), so \
                     the container's parent must be `{bar_name}`. A different parent name \
                     means the OnLoad ran on a different frame (e.g. the `for i=1, \
                     self.numButtons` loop self-reference broke) or the container was \
                     reparented post-load — neither has a consumer in Blizzard's own \
                     code so either would be a simulator-side regression."
                );
            }
        }
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn main_action_bar_does_not_publish_plan_named_button_globals() {
    wrath_action_case!(callbacks::main_alias_absence);
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for slot in 1..=NUM_BUTTONS {
            let plan_named_button = format!("MainActionBarButton{slot}");

            let plan_named_type: String = env
                .eval(&format!("return type(_G[{plan_named_button:?}])"))
                .expect("MainActionBarButton<N> nil-probe must run cleanly");

            assert_eq!(
                plan_named_type, "nil",
                "Expected `_G[{plan_named_button:?}]` to be nil after `{ROOT}` loads, got \
                 `{plan_named_type}`. The PLAN.md line for this task names the bar's 12 \
                 buttons as `MainActionBarButton1`..`MainActionBarButton12`, but \
                 `ActionBarMixin:ActionBar_OnLoad` at `{ACTION_BAR_LUA_SITE}` line 19-20 \
                 special-cases `MainActionBar` to use the legacy name `ActionButton<N>` \
                 instead. The branch is mutually exclusive with the fall-through at \
                 lua:28 (`buttonName = actionBarName..\"Button\"..i`) that would have \
                 produced the PLAN names — so a non-nil reading proves either Blizzard \
                 removed the special case (and MainActionBar's buttons fell through to \
                 the generic name, which would simultaneously null `_G.ActionButton<N>` — \
                 the sibling presence test \
                 `main_action_bar_publishes_action_button_globals_and_button_containers` \
                 would also fail), OR the simulator double-published the buttons under \
                 both names (which would shadow `_G.ActionButton<N>` consumers — every \
                 Bindings.xml entry, every `ActionButton1:Click()` call site — via a \
                 stale reference left over from an aborted OnLoad rerun). Either way the \
                 PLAN spec needs revisiting before this assertion is mutated. Note: \
                 StanceBar / PetActionBar / PossessActionBar follow the same pattern \
                 (special-case names `StanceButton<N>`, `PetActionButton<N>`, \
                 `PossessButton<N>` per lua:21-26) — only the Multi-bars (`MultiBarBottomLeft` \
                 etc.) and EditMode-only bars (MultiBar5/6/7) fall through to the \
                 `actionBarName..\"Button\"..i` suffix and thus DO publish a \
                 `<BarName>Button<N>` global."
            );
        }
    });
}

/// PLAN-named event-routing global frames. Each is a parentless,
/// invisible singleton declared at `Shared/ActionButtonComponentTemplate.xml:89-121`
/// whose only job is to route engine events / OnUpdate ticks to a
/// fan-out list of action buttons. Action buttons opt into a router by
/// calling `Router:RegisterFrame(self)` (or `(action, self)` for the
/// per-action routers); the router demuxes the event and dispatches
/// back to each registered button. The mixin-pinned method on each
/// router is `RegisterFrame` because that is the public surface every
/// caller in Blizzard's own code uses, and a regression that breaks
/// `RegisterFrame` would silently disable every action button's
/// reaction to the corresponding event class.
///
/// Note the deliberate inconsistency in the `ActionBarButtonEventsFrame`
/// row: the XML at xml:89 declares `mixin="ActionBarButtonEventsDerivedFrameMixin"`,
/// NOT `ActionBarButtonEventsFrameMixin`. The base `Mixin = {}` is
/// declared at `Shared/ActionButton.lua:201` but the XML uses the
/// derived subclass declared at `Shared/ActionButton.lua:1443`
/// (`ActionBarButtonEventsDerivedFrameMixin = CreateFromMixins(ActionBarButtonEventsFrameMixin)`)
/// so that flavor-specific overrides (the WoWLabs-only `OnWorldLootObjectTooltipShown`
/// at `WoWLabs/ActionButtonOverrides.lua:26-30`) can drop into the
/// subclass without disturbing the base contract that `RegisterFrame`
/// callers depend on. `CreateFromMixins` is implemented as
/// `Mixin({}, ...)` in `src/lua_api/env_init/shared_bootstrap.lua` —
/// it shallow-copies the parent mixin's keys onto a fresh table — so
/// `RegisterFrame` shows up on the derived mixin AND on the frame.
const PLAN_NAMED_EVENT_ROUTING_FRAMES: &[(&str, &str)] = &[
    (
        "ActionBarButtonEventsFrame",
        "Shared/ActionButtonComponentTemplate.xml:89",
    ),
    (
        "ActionBarActionEventsFrame",
        "Shared/ActionButtonComponentTemplate.xml:96",
    ),
    (
        "ActionBarButtonUpdateFrame",
        "Shared/ActionButtonComponentTemplate.xml:103",
    ),
    (
        "ActionBarButtonRangeCheckFrame",
        "Shared/ActionButtonComponentTemplate.xml:110",
    ),
    (
        "ActionBarButtonUsableWatcherFrame",
        "Shared/ActionButtonComponentTemplate.xml:117",
    ),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn plan_named_event_routing_frames_exist_with_register_frame_surface() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for (frame_name, xml_site) in PLAN_NAMED_EVENT_ROUTING_FRAMES {
            let frame_type: String = env
                .eval(&format!("return type(_G[{frame_name:?}])"))
                .expect("event-routing frame existence probe must run cleanly");

            assert_eq!(
                frame_type, "table",
                "Expected `_G[{frame_name:?}]` to be a table after `{ROOT}` loads, got \
                 `{frame_type}`. The XML at `{xml_site}` declares this frame with \
                 `<Frame name=\"{frame_name}\" mixin=\"...\">` — a parentless invisible \
                 singleton. A nil reading means the addon's XML chunk failed to execute \
                 past this declaration, which would also prevent the four sibling \
                 routing frames in the same XML file from registering (so this test \
                 fails first on whichever frame appears earliest in the file). Every \
                 caller of `{frame_name}:RegisterFrame(...)` in `Shared/ActionButton.lua` \
                 (the 5 RegisterFrame call sites at lua:459, 564, 957, etc.) would \
                 nil-call on the `:` indexer."
            );

            let register_frame_type: String = env
                .eval(&format!("return type(_G[{frame_name:?}].RegisterFrame)"))
                .expect("RegisterFrame method probe must run cleanly");

            assert_eq!(
                register_frame_type, "function",
                "Expected `_G[{frame_name:?}].RegisterFrame` to be a function after \
                 `{ROOT}` loads, got `{register_frame_type}`. Every routing frame's \
                 mixin (declared at `Shared/ActionButton.lua:201/243/346/366/404`) defines \
                 `function Mixin:RegisterFrame(...)` as the public fan-in API. The XML \
                 codegen at `src/loader/xml_frame_codegen.rs:155-173` expands the \
                 `mixin=` attribute into `Mixin(frame, MixinName)`, and the shared \
                 `Mixin(object, ...)` impl in `src/lua_api/env_init/shared_bootstrap.lua` \
                 walks `pairs(mixin)` to copy `RegisterFrame` onto the frame. A \
                 nil/non-function reading means either (a) the mixin source did not \
                 execute past its `function Mixin:RegisterFrame(...)` line, OR (b) the \
                 codegen Mixin call did not run for this frame. Either way every \
                 ActionButton that calls `{frame_name}:RegisterFrame(self)` from its \
                 OnLoad/OnEvent path would crash with a nil-method error during the \
                 first event tick after addon load."
            );
        }
    });
}
}

post_wrath_ui_fixture! {
/// Pin `SpellFlyout` identity. Declared at `Shared/SpellFlyout.xml:62`
/// as a `toplevel="true"` `frameStrata="DIALOG"` `hidden="true"` Frame
/// inheriting `SecureFrameTemplate, ResizeLayoutFrame, FlyoutPopupTemplate`
/// with `mixin="SpellFlyoutMixin"`. The DIALOG strata is what lifts
/// the flyout above the action bar (which lives in the default
/// MEDIUM strata for non-Main bars and HIGH for MainActionBar's
/// EditMode-bumped frame level), so a regression that drops the
/// strata to MEDIUM/LOW would visually clip the flyout under the
/// action bar's own buttons.
///
/// The hidden=true initial state matters because the flyout is a
/// pop-out that the action button reveals on click via
/// `SpellFlyout:Toggle(...)` (`Shared/SpellFlyout.lua`); a default-
/// shown flyout would render at startup over whatever's behind it
/// and steal mouse-over events from action buttons until the player
/// clicked elsewhere.
///
/// Pin three facts: (1) `SpellFlyout` exists as a table global, (2)
/// `:GetFrameStrata()` returns `"DIALOG"`, (3) `:IsShown()` returns
/// `false` (the explicit Show/Hide state — `IsVisible()` would also
/// be false because the parent chain matters, but `IsShown()` is the
/// more direct pin on the XML's `hidden="true"`).
#[cfg(feature = "optional-tests")]
#[test]
fn spell_flyout_publishes_dialog_strata_hidden_by_default() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval("return type(_G.SpellFlyout)")
            .expect("SpellFlyout existence probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G.SpellFlyout` to be a table after `{ROOT}` loads, got \
             `{frame_type}`. The XML at `Shared/SpellFlyout.xml:62` declares \
             `<Frame name=\"SpellFlyout\" toplevel=\"true\" hidden=\"true\" \
             frameStrata=\"DIALOG\" frameLevel=\"10\" \
             inherits=\"SecureFrameTemplate, ResizeLayoutFrame, FlyoutPopupTemplate\" \
             mixin=\"SpellFlyoutMixin\">`. A nil reading means the addon's XML chunk \
             failed to execute or the frame failed to register its name; every \
             ActionButton that calls `SpellFlyout:Toggle(...)` from `Shared/SpellFlyout.lua` \
             would nil-call."
        );

        let frame_strata: String = env
            .eval("return _G.SpellFlyout:GetFrameStrata()")
            .expect("SpellFlyout:GetFrameStrata() must run cleanly");

        assert_eq!(
            frame_strata, "DIALOG",
            "Expected `SpellFlyout:GetFrameStrata()` to return `DIALOG`, got \
             `{frame_strata}`. The XML at `Shared/SpellFlyout.xml:62` declares \
             `frameStrata=\"DIALOG\"` literally — the DIALOG layer sits above MEDIUM \
             (the default strata for the non-Main bars) and HIGH, so the flyout pops \
             out OVER the action bar's own buttons. A regression that drops the strata \
             would cause the flyout's spell icons to clip behind the action bar slots \
             at runtime."
        );

        let is_shown: bool = env
            .eval("return _G.SpellFlyout:IsShown() == true")
            .expect("SpellFlyout:IsShown() must run cleanly");

        assert!(
            !is_shown,
            "Expected `SpellFlyout:IsShown()` to return `false` (i.e. the frame is \
             hidden by default). The XML at `Shared/SpellFlyout.xml:62` declares \
             `hidden=\"true\"`, and the flyout is meant to be a pop-out that the \
             action button reveals on click via `SpellFlyout:Toggle(...)` in \
             `Shared/SpellFlyout.lua`. A default-shown flyout would render at \
             startup over whatever's behind it and steal mouse-over events from \
             action buttons until the player clicked elsewhere."
        );
    });
}
}

post_wrath_ui_fixture! {
/// Pin `StatusTrackingBarManager` identity together with its two
/// `parentKey` child containers. Declared at
/// `Mainline/StatusTrackingBar.xml:35-62` as a `parent="UIParent"`
/// `frameStrata="MEDIUM"` Frame with `mixin="StatusTrackingManagerMixin"`,
/// containing two `<Frame>` entries with
/// `parentKey="MainStatusTrackingBarContainer"` and
/// `parentKey="SecondaryStatusTrackingBarContainer"` respectively
/// (both inherit `StatusTrackingBarContainerTemplate` plus an
/// `EditModeStatusTrackingBar<N>SystemTemplate`).
///
/// `parentKey` auto-attaches the child onto its parent under that
/// key — separate from the optional `name=` global.
/// `StatusTrackingManagerMixin` at `Mainline/StatusTrackingBarManager.lua`
/// reads both keys directly, so a regression that severs `parentKey`
/// wiring while keeping the global name would crash every UpdateAll
/// pass. The dual-axis pin below (global + field lookup) catches it.
#[cfg(feature = "optional-tests")]
#[test]
fn status_tracking_bar_manager_publishes_main_and_secondary_parent_key_children() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let manager_type: String = env
            .eval("return type(_G.StatusTrackingBarManager)")
            .expect("StatusTrackingBarManager existence probe must run cleanly");

        assert_eq!(
            manager_type, "table",
            "Expected `_G.StatusTrackingBarManager` to be a table after `{ROOT}` loads, got \
             `{manager_type}`. The XML at `Mainline/StatusTrackingBar.xml:35` declares \
             `<Frame name=\"StatusTrackingBarManager\" parent=\"UIParent\" \
             frameStrata=\"MEDIUM\" mixin=\"StatusTrackingManagerMixin\">`. A nil reading \
             means either the XML chunk failed to execute or the named-frame registration \
             didn't run. Every consumer that calls `StatusTrackingBarManager:UpdateBarsShown()` \
             from `Mainline/StatusTrackingBarManager.lua` would surface a nil-method error."
        );

        for parent_key in [
            "MainStatusTrackingBarContainer",
            "SecondaryStatusTrackingBarContainer",
        ] {
            let container_type: String = env
                .eval(&format!(
                    "return type(_G.StatusTrackingBarManager[{parent_key:?}])"
                ))
                .expect("parentKey child probe must run cleanly");

            assert_eq!(
                container_type, "table",
                "Expected `StatusTrackingBarManager.{parent_key}` to be a table (the \
                 `parentKey=\"{parent_key}\"` child auto-attached by XML at \
                 `Mainline/StatusTrackingBar.xml:41/49`), got `{container_type}`. \
                 `parentKey` on a child `<Frame>` is what wires the child onto its parent \
                 under that key — separate from the optional `name=` global. A nil reading \
                 here while `_G.{parent_key}` is non-nil would mean the global-name path \
                 ran but the parentKey wiring did not, leaving `StatusTrackingManagerMixin` \
                 method bodies in `Mainline/StatusTrackingBarManager.lua` (which read \
                 `self.{parent_key}:Show()` / `self.{parent_key}:UpdateBarsShown()` \
                 directly) to crash with nil-method errors on every UpdateAll pass."
            );
        }
    });
}
}
