//! Mixin application surface for the `Blizzard_ActionBar` lane —
//! pins that `StanceBar`, `PetActionBar`, `PossessActionBar`,
//! `ExtraActionButton1`, and `MainMenuBarVehicleLeaveButton` each
//! received their expected mixin's methods via the XML `mixin=`
//! attribute (or via a template's `mixin=` for `ExtraActionButton1`).
//! `ExtraActionBarFrame` is split off into its own absence test —
//! it has NO `mixin=` and the PLAN line is technically wrong for that
//! one frame.
//!
//! PLAN.md task: `StanceBar`, `PetActionBar`, `PossessActionBar`,
//! `ExtraActionBarFrame`, `ExtraActionButton1`,
//! `MainMenuBarVehicleLeaveButton` exist with expected mixins applied.
//!
//! Pulled out of `surface_frames.rs` because that file passed the
//! 750-line readability budget. Split is along the frames-vs-mixin-
//! shape aspect boundary.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_ActionBar";

/// PLAN-named frames with a `mixin=` attribute in their XML. Codegen
/// at `src/loader/xml_frame_codegen.rs:155-173` expands the attribute
/// into `Mixin(frame, MixinName)`; the shared impl at
/// `src/lua_api/env_init/shared_bootstrap.lua` does `object[k] = v` for
/// every mixin key. After load, `frame.method == MixinName.method`.
struct MixinPin {
    frame_name: &'static str,
    frame_xml_site: &'static str,
    mixin_name: &'static str,
    mixin_lua_site: &'static str,
    methods: &'static [&'static str],
}

const PLAN_NAMED_MIXIN_FRAMES: &[MixinPin] = &[
    MixinPin {
        frame_name: "StanceBar",
        frame_xml_site: "Mainline/StanceBar.xml:12",
        mixin_name: "StanceBarMixin",
        mixin_lua_site: "Shared/StanceBar.lua",
        methods: &[
            "OnLoad",
            "OnEvent",
            "OnShow",
            "ShouldShow",
            "Update",
            "ShouldShowBackgroundArt",
            "SetBackgroundArtShown",
            "UpdateBackgroundArt",
            "UpdateState",
            "Select",
        ],
    },
    MixinPin {
        frame_name: "PetActionBar",
        frame_xml_site: "Mainline/PetActionBar.xml:33",
        mixin_name: "PetActionBarMixin",
        mixin_lua_site: "Shared/PetActionBar.lua",
        methods: &[
            "ClearPetActionHighlightMarks",
            "UpdatePetActionHighlightMarks",
            "OnHide",
            "OnLoad",
            "OnEvent",
            "OnUpdate",
            "Update",
            "UpdateCooldowns",
            "PetActionButtonDown",
            "PetActionButtonUp",
            "LockPetActionBar",
            "UnlockPetActionBar",
            "ShouldShowBackgroundArt",
            "SetBackgroundArtShown",
        ],
    },
    MixinPin {
        frame_name: "PossessActionBar",
        frame_xml_site: "Mainline/PossessActionBar.xml:13",
        mixin_name: "PossessActionBarMixin",
        mixin_lua_site: "Shared/PossessActionBar.lua",
        methods: &[
            "PossessActionBar_OnLoad",
            "Update",
            "UpdateState",
            "ShouldShowBackgroundArt",
            "SetBackgroundArtShown",
        ],
    },
    MixinPin {
        frame_name: "ExtraActionButton1",
        frame_xml_site: "Shared/ExtraActionBar.xml:116",
        mixin_name: "ExtraActionButtonMixin",
        mixin_lua_site: "Shared/ExtraActionBar.lua",
        methods: &["ExtraActionButton_OnLoad"],
    },
    MixinPin {
        frame_name: "MainMenuBarVehicleLeaveButton",
        frame_xml_site: "Shared/VehicleLeaveButton.xml:4",
        mixin_name: "MainMenuBarVehicleLeaveButtonMixin",
        mixin_lua_site: "Shared/VehicleLeaveButton.lua",
        methods: &[
            "OnLoad",
            "OnEnter",
            "OnEvent",
            "CanExitVehicle",
            "UpdateShownState",
            "Update",
            "OnClicked",
        ],
    },
];

post_wrath_ui_fixture! {
/// Pin per-frame mixin application. The double-pin (frame.method is
/// function AND frame.method == mixin.method) catches two regressions:
/// missing `Mixin(frame, MixinName)` codegen call vs. mixin source-load
/// failure. `ExtraActionButton1` uses an INHERITED mixin via
/// `inherits="ExtraActionButtonTemplate"` (`Shared/ExtraActionBar.xml:3`
/// has `mixin="ExtraActionButtonMixin"`), exercising the
/// template-mixin code path; the other four entries exercise the
/// direct `mixin=` codegen path.
#[cfg(feature = "optional-tests")]
#[test]
fn plan_named_frames_have_their_mixins_applied() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for pin in PLAN_NAMED_MIXIN_FRAMES {
            let frame_type: String = env
                .eval(&format!("return type(_G[{:?}])", pin.frame_name))
                .expect("frame existence probe must run cleanly");

            assert_eq!(
                frame_type, "table",
                "Expected `_G[{:?}]` to be a table (XML at `{}`), got `{frame_type}`",
                pin.frame_name, pin.frame_xml_site
            );

            for method in pin.methods {
                let probe = format!(
                    "return type({}.{method}) == \"function\" and \
                            type(_G[{:?}].{method}) == \"function\" and \
                            (_G[{:?}].{method} == {}.{method})",
                    pin.mixin_name, pin.frame_name, pin.frame_name, pin.mixin_name
                );
                let ok: bool = env
                    .eval(&probe)
                    .expect("mixin pin probe must evaluate cleanly");
                assert!(
                    ok,
                    "Expected `{}.{method}` to equal `{}.{method}` (XML `{}` mixin `{}`)",
                    pin.frame_name, pin.mixin_name, pin.frame_xml_site, pin.mixin_lua_site
                );
            }
        }
    });
}
}

#[cfg(not(feature = "client-wrath"))]
#[cfg(feature = "optional-tests")]
#[test]
fn edit_mode_action_bar_seeds_callable_base_method_aliases() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let (aliases_callable, base_preserved): (bool, bool) = env
            .eval(
                r#"
                local originalOnLoad = EditModeActionBarMixin.EditModeActionBar_OnLoad
                EditModeActionBarMixin.EditModeActionBar_OnLoad = function(frame)
                    local aliasesCallable = type(frame.SetScaleBase) == "function"
                        and type(frame.SetPointBase) == "function"
                        and type(frame.ClearAllPointsBase) == "function"
                        and type(frame.SetShownBase) == "function"
                        and type(frame.ShowBase) == "function"
                        and type(frame.HideBase) == "function"
                        and type(frame.IsShownBase) == "function"

                    local originalSetShown = frame.SetShown
                    local originalSetShownBase = frame.SetShownBase
                    frame.SetShown = function() end
                    local basePreserved = type(frame.SetShownBase) == "function"
                        and frame.SetShownBase == originalSetShownBase
                        and frame.SetShown ~= originalSetShownBase
                    frame.SetShown = originalSetShown

                    __editModeAliasesCallable = aliasesCallable
                    __editModeSetShownBasePreserved = basePreserved
                end

                CreateFrame(
                    "Frame",
                    "EditModeActionBarAliasProbe",
                    UIParent,
                    "EditModeActionBarTemplate"
                )
                EditModeActionBarMixin.EditModeActionBar_OnLoad = originalOnLoad

                return __editModeAliasesCallable == true,
                    __editModeSetShownBasePreserved == true
                "#,
            )
            .expect("EditMode ActionBar pre-initialization probe should run cleanly");

        assert!(
            aliases_callable,
            "EditModeSystemMixin should seed all seven callable Base aliases before ActionBar OnLoad"
        );
        assert!(
            base_preserved,
            "SetShownBase should preserve the original callable when SetShown is overridden"
        );
    });
}

post_wrath_ui_fixture! {
/// Pin that `ExtraActionBarFrame` has NO mixin — XML at
/// `Shared/ExtraActionBar.xml:93` lacks `mixin=`, no
/// `ExtraActionBarFrameMixin` global exists; behavioral wiring is the
/// FREE function `ExtraActionBar_OnLoad` (`Shared/ExtraActionBar.lua:5`).
/// Inverse of the mixin-applied test: pinning the empty mixin set
/// guards against accidental mixin injection.
#[cfg(feature = "optional-tests")]
#[test]
fn extra_action_bar_frame_publishes_no_mixin_only_a_script_handler() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval("return type(_G.ExtraActionBarFrame)")
            .expect("ExtraActionBarFrame probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G.ExtraActionBarFrame` table (XML `Shared/ExtraActionBar.xml:93`), got `{frame_type}`"
        );

        let convention_named_mixin_type: String = env
            .eval("return type(_G.ExtraActionBarFrameMixin)")
            .expect("ExtraActionBarFrameMixin nil-probe must run cleanly");
        assert_eq!(
            convention_named_mixin_type, "nil",
            "Expected `_G.ExtraActionBarFrameMixin` nil (no `mixin=` on the XML frame), got `{convention_named_mixin_type}`"
        );

        let on_load_handler_type: String = env
            .eval("return type(_G.ExtraActionBar_OnLoad)")
            .expect("ExtraActionBar_OnLoad probe must run cleanly");
        assert_eq!(
            on_load_handler_type, "function",
            "Expected `_G.ExtraActionBar_OnLoad` function (`Shared/ExtraActionBar.lua:5`), got `{on_load_handler_type}`"
        );
    });
}
}

const ACTION_BAR_MIXIN_LUA_SITE: &str = "Shared/ActionBar.lua:1";

/// PLAN-named methods that DO exist on `ActionBarMixin`. Source order
/// (lua:3/57/144/93/198): `ActionBar_OnLoad` and `ActionBar_OnEvent` are
/// the prefixed entry points the bar XML wires via
/// `<OnLoad function="ActionBar_OnLoad"/>`-style chains; `SetShowGrid`,
/// `UpdateGridLayout`, `UpdateShownButtons` are the three grid/visibility
/// helpers each bar's mixin calls into.
const ACTION_BAR_MIXIN_PLAN_NAMED_METHODS: &[&str] = &[
    "ActionBar_OnLoad",
    "ActionBar_OnEvent",
    "SetShowGrid",
    "UpdateGridLayout",
    "UpdateShownButtons",
];

/// Source-additional methods on `ActionBarMixin` that PLAN omits.
/// Grid-cache (`CacheGridSettings`/`ShouldUpdateGrid` lua:65/76),
/// strata raise/lower (`GetShowAllButtons`/`ShouldRaise`/
/// `UpdateFrameStrata` lua:173/183/194), and spell-flyout direction
/// (`UpdateSpellFlyoutDirection`/`GetSpellFlyoutDirection` lua:221/246).
const ACTION_BAR_MIXIN_SOURCE_ADDITIONAL_METHODS: &[&str] = &[
    "CacheGridSettings",
    "ShouldUpdateGrid",
    "GetShowAllButtons",
    "ShouldRaise",
    "UpdateFrameStrata",
    "UpdateSpellFlyoutDirection",
    "GetSpellFlyoutDirection",
];

/// PLAN-named methods absent from `ActionBarMixin` — negative tripwires.
/// Source has no `SetSpellFlyoutDirection` (only `Update`/`Get` at
/// lua:221/246); `Layout` lives on `ResizeLayoutMixin`
/// (`Blizzard_SharedXML/LayoutFrame.lua:486`), reaching bar frames via
/// `ResizeLayoutFrame` template inheritance.
const ACTION_BAR_MIXIN_PLAN_NAMED_ABSENT_METHODS: &[&str] = &["SetSpellFlyoutDirection", "Layout"];

post_wrath_ui_fixture! {
/// Pin `ActionBarMixin`'s method-surface contract. **Bidirectional
/// spec/source mismatch.** PLAN names 7; source declares 12 (5 match,
/// 7 source-additional, 2 PLAN-named absent). 15 assertions total.
#[cfg(feature = "optional-tests")]
#[test]
fn action_bar_mixin_publishes_plan_named_and_source_additional_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval("return type(_G.ActionBarMixin)")
            .expect("ActionBarMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G.ActionBarMixin` to be a table after `{ROOT}` loads, got \
             `{mixin_type}`. Source declares it at `{ACTION_BAR_MIXIN_LUA_SITE}` \
             (`ActionBarMixin = {{}}`). Nil reading: source file failed to load before \
             line 1, or the global was overwritten by a later addon."
        );

        for method in ACTION_BAR_MIXIN_PLAN_NAMED_METHODS {
            let method_type: String = env
                .eval(&format!("return type(_G.ActionBarMixin.{method})"))
                .expect("ActionBarMixin method probe must run cleanly");

            assert_eq!(
                method_type, "function",
                "Expected `ActionBarMixin.{method}` to be a function after `{ROOT}` \
                 loads, got `{method_type}`. PLAN names this method; source declares \
                 `function ActionBarMixin:{method}(...)` in `Shared/ActionBar.lua`. \
                 False reading: source file failed to execute past the declaration, or \
                 the method was renamed/removed. Each per-bar mixin's OnLoad calls into \
                 these via `self:ActionBar_OnLoad()`-style invocations, so a nil \
                 reading would nil-call at frame-OnLoad time."
            );
        }

        for method in ACTION_BAR_MIXIN_SOURCE_ADDITIONAL_METHODS {
            let method_type: String = env
                .eval(&format!("return type(_G.ActionBarMixin.{method})"))
                .expect("ActionBarMixin method probe must run cleanly");

            assert_eq!(
                method_type, "function",
                "Expected `ActionBarMixin.{method}` to be a function after `{ROOT}` \
                 loads, got `{method_type}`. PLAN omits this method, but source \
                 declares it as a direct `function ActionBarMixin:{method}(...)` in \
                 `Shared/ActionBar.lua`. Pinned as a tripwire so the spec recognises \
                 source drift if the method is removed: the grid-cache path \
                 (`CacheGridSettings`/`ShouldUpdateGrid`), strata raise/lower path \
                 (`GetShowAllButtons`/`ShouldRaise`/`UpdateFrameStrata`), and \
                 spell-flyout direction path (`UpdateSpellFlyoutDirection`/\
                 `GetSpellFlyoutDirection`) all depend on these declarations."
            );
        }

        for method in ACTION_BAR_MIXIN_PLAN_NAMED_ABSENT_METHODS {
            let method_type: String = env
                .eval(&format!("return type(_G.ActionBarMixin.{method})"))
                .expect("ActionBarMixin absent-method probe must run cleanly");

            assert_eq!(
                method_type, "nil",
                "Expected `ActionBarMixin.{method}` to be nil after `{ROOT}` loads, \
                 got `{method_type}`. PLAN names this method but source does NOT \
                 declare it on `ActionBarMixin` (`Shared/ActionBar.lua` has \
                 `UpdateSpellFlyoutDirection` lua:221 and `GetSpellFlyoutDirection` \
                 lua:246 — no `SetSpellFlyoutDirection`; and `Layout` lives on \
                 `ResizeLayoutMixin` at `Blizzard_SharedXML/LayoutFrame.lua:486`, \
                 reaching bar frames only via the `ResizeLayoutFrame` template \
                 inheritance at `Shared/ActionBarTemplate.xml:7`). Non-nil reading: \
                 source added the method on `ActionBarMixin` directly — the spec \
                 needs review (a directly-declared `Layout` on `ActionBarMixin` \
                 would shadow the inherited `ResizeLayoutMixin:Layout` and silently \
                 change layout-pass behavior across every bar)."
            );
        }
    });
}
}

const ACTION_BUTTON_LUA_SITE: &str = "Shared/ActionButton.lua";

/// PLAN names 10 methods as living on `ActionBarButtonMixin /
/// BaseActionButtonMixin`. Source disagrees on 9: only `UpdateButtonArt`
/// is on `BaseActionButtonMixin` (lua:1546 stub + `Mainline/ActionButtonOverrides.lua:2`
/// real impl). The remaining 9 (`OnLoad`, `OnEvent`, `OnEnter`,
/// `OnLeave`, `UpdateUsable`, `UpdateState`, `UpdateAction`,
/// `SetTooltip`, `MatchesActiveButtonSpellID`) all live on the sibling
/// `ActionBarActionButtonMixin` (declared lua:442) — a third mixin PLAN
/// does not name. The plain script handlers (`OnLoad`, `OnEnter`,
/// `OnLeave`) on the named mixins use prefixed variants
/// (`BaseActionButtonMixin_OnLoad` lua:1502, `ActionBarButtonMixin_OnLoad`
/// lua:1605, etc.) so the chain `ActionBarButtonTemplate ->
/// ActionButtonTemplate` (`Mainline/ActionButtonTemplate.xml:189` ->
/// `xml:4`) can compose Mixin OnLoads without name collision.
const PLAN_NAMED_BUTTON_METHODS: &[&str] = &[
    "OnLoad",
    "OnEvent",
    "OnEnter",
    "OnLeave",
    "UpdateUsable",
    "UpdateState",
    "UpdateAction",
    "SetTooltip",
    "MatchesActiveButtonSpellID",
    "UpdateButtonArt",
];

/// The single PLAN-named method that IS on `BaseActionButtonMixin`.
/// Stub at lua:1546, overridden at `Mainline/ActionButtonOverrides.lua:2`
/// — the real Mainline body shows/hides `SlotArt`/`SlotBackground` and
/// switches the normal+pushed atlases between `UI-HUD-ActionBar-IconFrame*`
/// and `UI-HUD-ActionBar-IconFrame-AddRow*` based on `self.bar.hideBarArt`.
const BASE_ACTION_BUTTON_PRESENT_PLAN_METHOD: &str = "UpdateButtonArt";

/// Sample of source-additional methods on `BaseActionButtonMixin` that
/// PLAN omits — pinned so a regression that drops the chain entry
/// points (`BaseActionButtonMixin_OnLoad` etc.) or the grid-attribute
/// helpers (`GetShowGrid`/`SetShowGrid`/`UpdateFlyout`) surfaces with a
/// clear cause.
const BASE_ACTION_BUTTON_SOURCE_ADDITIONAL_METHODS: &[&str] = &[
    "BaseActionButtonMixin_OnLoad",
    "BaseActionButtonMixin_OnEnter",
    "BaseActionButtonMixin_OnLeave",
    "BaseActionButtonMixin_OnDragStart",
    "BaseActionButtonMixin_OnAttributeChanged",
    "GetShowGrid",
    "SetShowGrid",
    "UpdateFlyout",
];

/// Source-additional methods on `ActionBarButtonMixin` (declared
/// lua:1603). All four are prefixed-name forwarders that delegate to
/// both `BaseActionButtonMixin` and `ActionBarActionButtonDerivedMixin`
/// (lua:1606-1607 etc.) — they exist precisely so the
/// `ActionBarButtonTemplate` XML can compose two parent mixins without
/// name collision on `OnLoad`/`OnEnter`/`OnLeave`/`OnDragStart`.
const ACTION_BAR_BUTTON_SOURCE_ADDITIONAL_METHODS: &[&str] = &[
    "ActionBarButtonMixin_OnLoad",
    "ActionBarButtonMixin_OnEnter",
    "ActionBarButtonMixin_OnLeave",
    "ActionBarButtonMixin_OnDragStart",
];

post_wrath_ui_fixture! {
/// Pin `ActionBarButtonMixin` and `BaseActionButtonMixin` method
/// surfaces. **Spec/source mismatch — PLAN names 10 methods, but only
/// 1 (`UpdateButtonArt`) actually lives on the named mixins.** The
/// other 9 live on the sibling `ActionBarActionButtonMixin` (declared
/// `Shared/ActionButton.lua:442`) — a mixin PLAN does NOT name and
/// reaches frames via `ActionBarActionButtonDerivedMixin = CreateFromMixins(...)`
/// at lua:1444 + a function-call apply at lua:1607. Test pins 30
/// assertions: 2 mixin-table existence + 1 PLAN-named PRESENT on
/// `BaseActionButtonMixin` (`UpdateButtonArt`) + 9 PLAN-named ABSENT on
/// `BaseActionButtonMixin` + 10 PLAN-named ABSENT on
/// `ActionBarButtonMixin` + 8 source-additional functions on
/// `BaseActionButtonMixin` (`BaseActionButtonMixin_*` chain entries +
/// `GetShowGrid`/`SetShowGrid`/`UpdateFlyout`).
#[cfg(feature = "optional-tests")]
#[test]
fn action_bar_button_and_base_action_button_mixins_pin_plan_named_and_source_additional_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for mixin in ["BaseActionButtonMixin", "ActionBarButtonMixin"] {
            let mixin_type: String = env
                .eval(&format!("return type(_G.{mixin})"))
                .expect("mixin global probe must run cleanly");

            assert_eq!(
                mixin_type, "table",
                "Expected `_G.{mixin}` to be a table after `{ROOT}` loads, got \
                 `{mixin_type}`. Source declares both at `{ACTION_BUTTON_LUA_SITE}` \
                 (`BaseActionButtonMixin = {{}}` lua:1500, `ActionBarButtonMixin = {{}};` \
                 lua:1603). Nil reading: source file failed to load, or one global was \
                 overwritten by a later addon."
            );
        }

        let present_method_type: String = env
            .eval(&format!(
                "return type(_G.BaseActionButtonMixin.{BASE_ACTION_BUTTON_PRESENT_PLAN_METHOD})"
            ))
            .expect("BaseActionButtonMixin.UpdateButtonArt probe must run cleanly");

        assert_eq!(
            present_method_type, "function",
            "Expected `BaseActionButtonMixin.{BASE_ACTION_BUTTON_PRESENT_PLAN_METHOD}` to be a \
             function — stub at `{ACTION_BUTTON_LUA_SITE}:1546`, overridden by \
             `Mainline/ActionButtonOverrides.lua:2`. False reading: stub or override failed \
             to load."
        );

        for method in PLAN_NAMED_BUTTON_METHODS {
            if *method == BASE_ACTION_BUTTON_PRESENT_PLAN_METHOD {
                continue;
            }

            let method_type: String = env
                .eval(&format!("return type(_G.BaseActionButtonMixin.{method})"))
                .expect("BaseActionButtonMixin absent-method probe must run cleanly");

            assert_eq!(
                method_type, "nil",
                "Expected `BaseActionButtonMixin.{method}` to be nil after `{ROOT}` \
                 loads, got `{method_type}`. PLAN names this method, but source places \
                 it on the sibling `ActionBarActionButtonMixin` (lua:442) — NOT on \
                 `BaseActionButtonMixin`. Non-nil reading: source moved the method onto \
                 `BaseActionButtonMixin`, which would shadow the sibling's contract."
            );
        }

        for method in PLAN_NAMED_BUTTON_METHODS {
            let method_type: String = env
                .eval(&format!("return type(_G.ActionBarButtonMixin.{method})"))
                .expect("ActionBarButtonMixin absent-method probe must run cleanly");

            assert_eq!(
                method_type, "nil",
                "Expected `ActionBarButtonMixin.{method}` to be nil after `{ROOT}` loads, \
                 got `{method_type}`. `ActionBarButtonMixin` (lua:1603) only declares 4 \
                 prefixed forwarders (`ActionBarButtonMixin_OnLoad`/`OnEnter`/`OnLeave`/\
                 `OnDragStart`); the PLAN-named methods reach frames through \
                 `ActionBarActionButtonDerivedMixin`. Non-nil reading: spec drift."
            );
        }

        for method in BASE_ACTION_BUTTON_SOURCE_ADDITIONAL_METHODS {
            let method_type: String = env
                .eval(&format!("return type(_G.BaseActionButtonMixin.{method})"))
                .expect("BaseActionButtonMixin source-additional probe must run cleanly");

            assert_eq!(
                method_type, "function",
                "Expected `BaseActionButtonMixin.{method}` to be a function after \
                 `{ROOT}` loads, got `{method_type}`. Source declares it directly in \
                 `{ACTION_BUTTON_LUA_SITE}` (lua:1502-1551). Tripwire: prefixed chain \
                 entries are XML-script targets; grid helpers are called from \
                 `ActionBarMixin:ActionBar_OnLoad` and `BaseActionButtonMixin_OnLoad`."
            );
        }

        for method in ACTION_BAR_BUTTON_SOURCE_ADDITIONAL_METHODS {
            let method_type: String = env
                .eval(&format!("return type(_G.ActionBarButtonMixin.{method})"))
                .expect("ActionBarButtonMixin source-additional probe must run cleanly");

            assert_eq!(
                method_type, "function",
                "Expected `ActionBarButtonMixin.{method}` to be a function after \
                 `{ROOT}` loads, got `{method_type}`. Source declares the prefixed \
                 forwarder in `{ACTION_BUTTON_LUA_SITE}` (lua:1605-1620), each delegating \
                 to both `BaseActionButtonMixin_*` and \
                 `ActionBarActionButtonDerivedMixin_*` siblings."
            );
        }
    });
}
}

/// One per-mixin row. PLAN says "are tables with the mixin methods
/// documented in the analyzer inventory" — the contract is the FULL
/// method set per mixin, not a subset.
struct MixinInventory {
    name: &'static str,
    lua_site: &'static str,
    methods: &'static [&'static str],
}

const MIXIN_INVENTORY: &[MixinInventory] = &[
    MixinInventory {
        name: "StanceBarMixin",
        lua_site: "Shared/StanceBar.lua:4",
        methods: &[
            "OnLoad",
            "OnEvent",
            "OnShow",
            "ShouldShow",
            "Update",
            "ShouldShowBackgroundArt",
            "SetBackgroundArtShown",
            "UpdateBackgroundArt",
            "UpdateState",
            "Select",
        ],
    },
    MixinInventory {
        name: "StanceButtonMixin",
        lua_site: "Shared/StanceBar.lua:107",
        methods: &[
            "StanceButtonMixin_OnLoad",
            "StanceButtonMixin_OnClick",
            "StanceButtonMixin_OnEnter",
            "StanceButtonMixin_OnLeave",
            "HasAction",
        ],
    },
    MixinInventory {
        name: "PetActionBarMixin",
        lua_site: "Shared/PetActionBar.lua:16",
        methods: &[
            "ClearPetActionHighlightMarks",
            "UpdatePetActionHighlightMarks",
            "OnHide",
            "OnLoad",
            "OnEvent",
            "OnUpdate",
            "Update",
            "UpdateCooldowns",
            "PetActionButtonDown",
            "PetActionButtonUp",
            "LockPetActionBar",
            "UnlockPetActionBar",
            "ShouldShowBackgroundArt",
            "SetBackgroundArtShown",
        ],
    },
    MixinInventory {
        name: "PossessActionBarMixin",
        lua_site: "Shared/PossessActionBar.lua:4",
        methods: &[
            "PossessActionBar_OnLoad",
            "Update",
            "UpdateState",
            "ShouldShowBackgroundArt",
            "SetBackgroundArtShown",
        ],
    },
    MixinInventory {
        name: "PossessButtonMixin",
        lua_site: "Shared/PossessActionBar.lua:65",
        methods: &["OnLoad", "OnClick", "OnEnter", "OnLeave", "HasAction"],
    },
    MixinInventory {
        name: "ExtraActionButtonMixin",
        lua_site: "Shared/ExtraActionBar.lua:82",
        methods: &["ExtraActionButton_OnLoad"],
    },
    MixinInventory {
        name: "MainActionBarMixin",
        lua_site: "Shared/MainActionBar.lua:3",
        methods: &[
            "OnLoad",
            "OnShow",
            "OnHide",
            "SetYOffset",
            "GetYOffset",
            "OnEvent",
            "AttachToFrame",
            "DetachFromFrame",
            "IsInDefaultPosition",
            "SetQuickKeybindModeEffectsShown",
            "UpdateEndCaps",
            "EditModeSetScale",
            "UpdateDividers",
            "GetEndCapsFrameLevel",
        ],
    },
];

post_wrath_ui_fixture! {
/// Pin the analyzer-inventory method surface for the seven bar/button
/// mixins PLAN names. 61 assertions = 7 mixin-table existence + 54
/// methods. `MainActionBarMixin:UpdateEndCaps` is a Shared stub
/// (`Shared/MainActionBar.lua:72`) overridden by
/// `Mainline/MainActionBarOverrides.lua:2` — both are valid `function`
/// readings. `StanceButtonMixin` uses prefixed forwarders
/// (`StanceButtonMixin_OnLoad` etc.); `PossessButtonMixin` uses plain
/// handler names — both pinned verbatim.
#[cfg(feature = "optional-tests")]
#[test]
fn analyzer_inventory_mixins_publish_their_documented_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for inventory in MIXIN_INVENTORY {
            let mixin_type: String = env
                .eval(&format!("return type(_G.{})", inventory.name))
                .expect("mixin global probe must run cleanly");

            assert_eq!(
                mixin_type, "table",
                "Expected `_G.{}` to be a table after `{ROOT}` loads (declared at `{}`), \
                 got `{mixin_type}`. Every method assertion below depends on it.",
                inventory.name, inventory.lua_site
            );

            for method in inventory.methods {
                let method_type: String = env
                    .eval(&format!("return type(_G.{}.{method})", inventory.name))
                    .expect("mixin method probe must run cleanly");

                assert_eq!(
                    method_type,
                    "function",
                    "Expected `{mixin}.{method}` to be a function after `{ROOT}` loads \
                     (declared at `{lua_site}`), got `{method_type}`. PLAN names the \
                     analyzer inventory as the contract — full per-mixin method set, \
                     not a PLAN subset.",
                    mixin = inventory.name,
                    lua_site = inventory.lua_site,
                );
            }
        }
    });
}
}

/// Status-tracking and spell-flyout mixins PLAN names as "tables".
/// Existence-only contract — no method inventory.
const STATUS_AND_FLYOUT_BAR_MIXINS: &[(&str, &str)] = &[
    ("SpellFlyoutPopupButtonMixin", "Shared/SpellFlyout.lua:8"),
    ("SpellFlyoutMixin", "Shared/SpellFlyout.lua:151"),
    ("StatusTrackingBarMixin", "Shared/StatusTrackingBar.lua:1"),
    ("ExpBarMixin", "Shared/ExpBar.lua:2"),
    ("ExhaustionTickMixin", "Shared/ExpBar.lua:111"),
    ("ReputationStatusBarMixin", "Shared/ReputationBar.lua:40"),
    ("HonorBarMixin", "Mainline/HonorBar.lua:5"),
    ("AzeriteBarMixin", "Mainline/AzeriteBar.lua:12"),
    ("ArtifactBarMixin", "Mainline/ArtifactBar.lua:5"),
    ("ArtifactTickMixin", "Mainline/ArtifactBar.lua:110"),
    ("HouseFavorBarMixin", "Mainline/HouseFavorBar.lua:11"),
];

post_wrath_ui_fixture! {
/// Pin the 11 PLAN-named status-tracking/spell-flyout mixins as tables.
/// Single test loops `STATUS_AND_FLYOUT_BAR_MIXINS` and asserts each
/// global's `type` is `"table"` after addon load. 11 assertions total.
#[cfg(feature = "optional-tests")]
#[test]
fn status_tracking_and_flyout_bar_mixins_publish_as_tables() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for (name, lua_site) in STATUS_AND_FLYOUT_BAR_MIXINS {
            let mixin_type: String = env
                .eval(&format!("return type(_G.{name})"))
                .expect("mixin global probe must run cleanly");
            assert_eq!(
                mixin_type, "table",
                "Expected `_G.{name}` to be a table after `{ROOT}` loads (declared at \
                 `{lua_site}`), got `{mixin_type}`. Nil reading: source file failed to \
                 load, or the `MixinName = {{}}` line was dropped."
            );
        }
    });
}
}

/// Pin `AssistedCombatManager` as a table with all 8 PLAN-named
/// methods. Declared at `Mainline/AssistedCombatManager.lua:3`. Source
/// has 11 additional methods PLAN omits (`ProcessCVars` etc.) — the
/// PLAN list is the surface contract for this task.
#[cfg(not(feature = "client-wrath"))]
#[cfg(feature = "optional-tests")]
#[test]
fn assisted_combat_manager_publishes_plan_named_methods() {
    let plan_methods = [
        "Init",
        "OnSpellsChanged",
        "SetActionSpell",
        "IsAssistedHighlightActive",
        "IsRotationSpell",
        "HasActionSpell",
        "GetActionSpellID",
        "ShouldHighlightSpellbookSpell",
    ];
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let manager_type: String = env
            .eval("return type(_G.AssistedCombatManager)")
            .expect("AssistedCombatManager global probe must run cleanly");
        assert_eq!(
            manager_type, "table",
            "Expected `_G.AssistedCombatManager` to be a table after `{ROOT}` loads \
             (declared at `Mainline/AssistedCombatManager.lua:3`), got `{manager_type}`."
        );
        for method in plan_methods {
            let method_type: String = env
                .eval(&format!("return type(_G.AssistedCombatManager.{method})"))
                .expect("AssistedCombatManager method probe must run cleanly");
            assert_eq!(
                method_type, "function",
                "Expected `AssistedCombatManager.{method}` to be a function after \
                 `{ROOT}` loads, got `{method_type}`. Source declares it in \
                 `Mainline/AssistedCombatManager.lua`."
            );
        }
    });
}

/// Pin `ActionButtonSpellAlertManager` table + 3 PLAN-named methods
/// at `Shared/ActionButtonSpellAlerts.lua` lua:1/114/125/132.
#[cfg(not(feature = "client-wrath"))]
#[cfg(feature = "optional-tests")]
#[test]
fn action_button_spell_alert_manager_publishes_plan_named_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let manager: String = env
            .eval("return type(_G.ActionButtonSpellAlertManager)")
            .expect("ActionButtonSpellAlertManager probe must run cleanly");
        assert_eq!(manager, "table", "expected table, got `{manager}`");
        for method in ["ShowAlert", "HideAlert", "HasAlert"] {
            let m: String = env
                .eval(&format!(
                    "return type(_G.ActionButtonSpellAlertManager.{method})"
                ))
                .expect("method probe must run cleanly");
            assert_eq!(m, "function", "expected `{method}` function, got `{m}`");
        }
    });
}

/// Event-routing/refresh-watcher mixins PLAN names as "tables". Five
/// live in `Shared/ActionButton.lua` (lua:201/243/346/366/404 — per-
/// button event router + three refresh-frame watchers); the sixth at
/// `Shared/VehicleLeaveButton.lua:2`. Existence-only contract.
const EVENT_AND_WATCHER_FRAME_MIXINS: &[&str] = &[
    "ActionBarButtonEventsFrameMixin",
    "ActionBarActionEventsFrameMixin",
    "ActionBarButtonUpdateFrameMixin",
    "ActionBarButtonRangeCheckFrameMixin",
    "ActionBarButtonUsableWatcherFrameMixin",
    "MainMenuBarVehicleLeaveButtonMixin",
];

post_wrath_ui_fixture! {
/// Pin the 6 event-routing/refresh-watcher mixins as tables. 6
/// assertions; nil reading on any row means the source file failed to
/// load past the `MixinName = {}` line.
#[cfg(feature = "optional-tests")]
#[test]
fn event_and_watcher_frame_mixins_publish_as_tables() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for name in EVENT_AND_WATCHER_FRAME_MIXINS {
            let mixin_type: String = env
                .eval(&format!("return type(_G.{name})"))
                .expect("mixin global probe must run cleanly");
            assert_eq!(
                mixin_type, "table",
                "Expected `_G.{name}` to be a table, got `{mixin_type}`"
            );
        }
    });
}
}
