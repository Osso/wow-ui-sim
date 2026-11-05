//! Frame-shape surface pins for the `Blizzard_AccountStore` lane.
//!
//! Spec/source mismatch finding (PLAN.md task for `AccountStoreFrame`
//! parentKey children): the plan named `Inset`, `CategoryList`,
//! `ItemDisplay`, `Footer` — but `Blizzard_AccountStore.xml:78-141`
//! declares six Frame children with parentKeys: `LeftInset`,
//! `RightInset`, `LeftDisplay`, `RightDisplay`, `CategoryList`,
//! `StoreDisplay`. The mismatches are:
//!
//! | PLAN name      | Actual XML name(s)                                | Notes |
//! |----------------|---------------------------------------------------|-------|
//! | `Inset`        | `LeftInset` + `RightInset`                        | The PLAN-named singular `Inset` is split into two halves; both inherit `AccountStoreInsetFrameTemplate`. |
//! | `CategoryList` | `CategoryList`                                    | Matches verbatim — inherits `AccountStoreCategoryListTemplate`. |
//! | `ItemDisplay`  | `StoreDisplay`                                    | Renamed to `StoreDisplay` but still inherits `AccountStoreItemDisplayTemplate`, so the underlying surface is the same. |
//! | `Footer`       | (does not exist)                                  | No `Footer` parentKey anywhere in `Blizzard_AccountStore.xml` — the PLAN-named child is genuinely absent. |
//!
//! The `account_store_frame_publishes_expected_parentkey_children`
//! test pins what actually exists (the six real parentKey children
//! plus `frameStrata == "HIGH"` and the `PageText` FontString
//! parentKey on the ARTWORK layer) plus three positive nil assertions
//! for the PLAN-named keys (`Inset`, `ItemDisplay`, `Footer`) — the
//! mismatches surface as tripwires that flip if Blizzard ever renames
//! the children to match the PLAN shape.
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `AccountStoreInsetFrameTemplate`): the plan wrote that the
//! template "resolves and is referenced by `AccountStoreFrame.Inset`",
//! but `AccountStoreFrame.Inset` does not exist (the same mismatch
//! pinned by the negative-assertion list in the prior test). The
//! actual references are at `Blizzard_AccountStore.xml:93` and `:100`,
//! both inheriting the template via `inherits="AccountStoreInsetFrameTemplate"`
//! on Frame children with parentKeys `LeftInset` and `RightInset`.
//! Virtual templates never escape into `_G`, so the resolution proof
//! comes from the structural fingerprint the template leaves on every
//! inheritor: a `Bg` Texture parentKey on the BACKGROUND layer plus
//! eight border-tile Texture parentKeys on the BORDER layer
//! (`InsetBorderTopLeft`, `InsetBorderTopRight`, `InsetBorderBottomLeft`,
//! `InsetBorderBottomRight`, `InsetBorderTop`, `InsetBorderBottom`,
//! `InsetBorderLeft`, `InsetBorderRight`). The
//! `account_store_inset_frame_template_resolves_into_both_inheritors`
//! test pins the virtual-template absence in `_G`, then asserts the
//! nine-parentKey fingerprint on BOTH `LeftInset` and `RightInset` —
//! the dual-inheritor pin is the strongest proof that the template
//! resolves: a regression that broke template inheritance would lose
//! the fingerprint on both at once.
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `FullscreenAccountStoreContainer.LeaveButton`): the plan named the
//! parentKey child as `LeaveButton` inheriting `MagicButtonTemplate`,
//! but `Blizzard_AccountStore.xml:167` declares it as
//! `parentKey="LeaveStoreButton"` inheriting
//! `BigRedThreeSliceButtonTemplate` (which itself inherits
//! `ThreeSliceButtonTemplate`, defined at
//! `Blizzard_SharedXML/Shared/Button/ThreeSliceButtonTemplate.xml:4`).
//! Both halves of the spec mismatch the source: the parentKey is
//! `LeaveStoreButton` not `LeaveButton`, and the inherited template is
//! a three-slice red button (red gradient, 441x128 atlas-driven
//! visual) not a `MagicButtonTemplate` (a `UIPanelButtonTemplate`
//! variant with `MagicButton_OnLoad`). The
//! `fullscreen_account_store_container_publishes_expected_button_with_three_slice_inheritance`
//! test pins what actually exists (the container as a table, the
//! `LeaveStoreButton` parentKey, the structural fingerprint of the
//! three-slice template — `Left` / `Right` / `Center` textures plus
//! `Controller` frame — that distinguishes it from `MagicButtonTemplate`)
//! plus a positive nil assertion on `LeaveButton` as the spec/source
//! mismatch tripwire.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";
const FRAME_NAME: &str = "AccountStoreFrame";

const ACTUAL_PARENT_KEY_CHILDREN: &[(&str, &str)] = &[
    ("LeftInset", "Blizzard_AccountStore.xml:93"),
    ("RightInset", "Blizzard_AccountStore.xml:100"),
    ("LeftDisplay", "Blizzard_AccountStore.xml:106"),
    ("RightDisplay", "Blizzard_AccountStore.xml:119"),
    ("CategoryList", "Blizzard_AccountStore.xml:128"),
    ("StoreDisplay", "Blizzard_AccountStore.xml:135"),
];

const PLAN_NAMED_BUT_ABSENT: &[(&str, &str)] = &[
    (
        "Inset",
        "split into `LeftInset` + `RightInset` in the actual XML",
    ),
    ("ItemDisplay", "renamed to `StoreDisplay` in the actual XML"),
    (
        "Footer",
        "no Footer parentKey exists anywhere in `Blizzard_AccountStore.xml`",
    ),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_frame_publishes_expected_parentkey_children() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{FRAME_NAME:?}])"))
            .expect("AccountStoreFrame global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{FRAME_NAME:?}]` to be a table after `{ROOT}` loads, got \
             `{frame_type}`. The frame is declared at `Blizzard_AccountStore.xml:78` with \
             `name=\"AccountStoreFrame\"` and `parent=\"UIParent\"`, so the named-frame \
             registration runs at XML load time. A nil reading means either the XML did not \
             execute (a regression in the load pipeline) or the frame failed to register its \
             name (a regression in the named-frame routing inside `CreateFrame`). Either way, \
             every downstream consumer that reaches `AccountStoreFrame.X` would surface a \
             nil-table-index error."
        );

        let frame_strata: String = env
            .eval(&format!("return _G[{FRAME_NAME:?}]:GetFrameStrata()"))
            .expect("`GetFrameStrata` must run cleanly on AccountStoreFrame");

        assert_eq!(
            frame_strata, "HIGH",
            "Expected `AccountStoreFrame:GetFrameStrata()` to return `HIGH` after `{ROOT}` \
             loads, got `{frame_strata}`. The XML at `Blizzard_AccountStore.xml:78` declares \
             `frameStrata=\"HIGH\"` literally. A regression dropping or rewriting this attribute \
             would change the panel's render order — HIGH places it above MEDIUM (the default \
             UIPanel stratum) so the account-store panel appears on top of normal world-frame \
             chrome but below TOOLTIP / DIALOG / FULLSCREEN strata."
        );

        for (parent_key, defining_site) in ACTUAL_PARENT_KEY_CHILDREN {
            let child_type: String = env
                .eval(&format!("return type(_G[{FRAME_NAME:?}][{parent_key:?}])"))
                .unwrap_or_else(|error| {
                    panic!("failed to probe `AccountStoreFrame.{parent_key}`: {error}")
                });

            assert_eq!(
                child_type, "table",
                "Expected `AccountStoreFrame.{parent_key}` to be a table after `{ROOT}` loads \
                 (declared at `{defining_site}`), got `{child_type}`. The XML element registers \
                 the parentKey via `parentKey=\"{parent_key}\"` on a `<Frame>` child, which the \
                 simulator's `__newindex` metamethod syncs to both the Lua-side property and the \
                 Rust-side `children_keys` HashMap. A nil reading means either the XML element \
                 was dropped (a regression in this lane's XML) or the parentKey-sync routing \
                 broke (a regression in `register_new_frame` / `__newindex`). The six real \
                 parentKey children together cover the panel's entire visible chrome: the \
                 LeftInset / RightInset pair frames the two-pane layout, LeftDisplay / \
                 RightDisplay host the inset backgrounds, CategoryList drives the left-pane \
                 category scroll-box, and StoreDisplay drives the right-pane focused-card \
                 detail panel."
            );
        }

        let page_text_type: String = env
            .eval(&format!("return type(_G[{FRAME_NAME:?}].PageText)"))
            .expect("`AccountStoreFrame.PageText` probe must run cleanly");

        assert_eq!(
            page_text_type, "table",
            "Expected `AccountStoreFrame.PageText` to be a table (FontString userdata that \
             type()s as table) after `{ROOT}` loads. Declared at `Blizzard_AccountStore.xml:85` \
             as a `<FontString parentKey=\"PageText\" inherits=\"GameFontHighlight\">` on the \
             ARTWORK layer, anchored BOTTOMRIGHT — used by `AccountStoreMixin` to render the \
             current page indicator (e.g. `2/5`) on the panel's bottom-right corner. A nil \
             reading means the FontString registration was dropped, leaving the panel without \
             page indication."
        );

        for (absent_key, mismatch_reason) in PLAN_NAMED_BUT_ABSENT {
            let absent_type: String = env
                .eval(&format!("return type(_G[{FRAME_NAME:?}][{absent_key:?}])"))
                .unwrap_or_else(|error| {
                    panic!("failed to probe absence of `AccountStoreFrame.{absent_key}`: {error}")
                });

            assert_eq!(
                absent_type, "nil",
                "Expected `AccountStoreFrame.{absent_key}` to be nil after `{ROOT}` loads \
                 (PLAN.md spec/source mismatch tripwire — {mismatch_reason}), got \
                 `{absent_type}`. The PLAN.md task names `{absent_key}` as a parentKey child of \
                 AccountStoreFrame, but the actual XML at `Blizzard_AccountStore.xml:78-141` \
                 does not declare a child with this parentKey. This positive nil assertion \
                 flips if Blizzard ever renames a child to match the PLAN shape, forcing a \
                 re-pin against the new parentKey set."
            );
        }
    });
}
}

const FULLSCREEN_CONTAINER_NAME: &str = "FullscreenAccountStoreContainer";
const ACTUAL_LEAVE_BUTTON_KEY: &str = "LeaveStoreButton";
const PLAN_NAMED_LEAVE_BUTTON_KEY: &str = "LeaveButton";

const THREE_SLICE_PARENTKEY_FINGERPRINT: &[(&str, &str)] = &[
    (
        "Left",
        "Texture parentKey on BACKGROUND layer (ThreeSliceButtonTemplate.xml:23)",
    ),
    (
        "Right",
        "Texture parentKey on BACKGROUND layer (ThreeSliceButtonTemplate.xml:28)",
    ),
    (
        "Center",
        "Texture parentKey with horizTile=true on BACKGROUND layer (ThreeSliceButtonTemplate.xml:33)",
    ),
    (
        "Controller",
        "Frame parentKey with ButtonControllerMixin (ThreeSliceButtonTemplate.xml:42)",
    ),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn fullscreen_account_store_container_publishes_expected_button_with_three_slice_inheritance() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let container_type: String = env
            .eval(&format!("return type(_G[{FULLSCREEN_CONTAINER_NAME:?}])"))
            .expect("FullscreenAccountStoreContainer global probe must run cleanly");

        assert_eq!(
            container_type, "table",
            "Expected `_G[{FULLSCREEN_CONTAINER_NAME:?}]` to be a table after `{ROOT}` loads, \
             got `{container_type}`. The frame is declared at `Blizzard_AccountStore.xml:149` \
             with `name=\"FullscreenAccountStoreContainer\"`, `parent=\"UIParent\"`, \
             `mixin=\"FullscreenAccountStoreContainerMixin\"`, and `setAllPoints=true` via \
             TOPLEFT/BOTTOMRIGHT anchors. It hosts the fullscreen-mode chrome that wraps \
             `AccountStoreFrame` when the lane runs in WoW Labs / Plunderstorm fullscreen mode \
             (the `SetFullscreenMode(true)` path at `Blizzard_AccountStore.lua:52-53` reparents \
             AccountStoreFrame onto this container)."
        );

        let leave_button_type: String = env
            .eval(&format!(
                "return type(_G[{FULLSCREEN_CONTAINER_NAME:?}][{ACTUAL_LEAVE_BUTTON_KEY:?}])"
            ))
            .expect("FullscreenAccountStoreContainer.LeaveStoreButton probe must run cleanly");

        assert_eq!(
            leave_button_type, "table",
            "Expected `FullscreenAccountStoreContainer.{ACTUAL_LEAVE_BUTTON_KEY}` to be a table \
             after `{ROOT}` loads, got `{leave_button_type}`. The button is declared at \
             `Blizzard_AccountStore.xml:167` as a `<Button parentKey=\"LeaveStoreButton\" \
             inherits=\"BigRedThreeSliceButtonTemplate\" \
             mixin=\"FullscreenLeaveAccountStoreButtonMixin\">`, anchored BOTTOM y=32 with \
             height 32. A nil reading means either the XML element was dropped or the \
             parentKey-sync routing broke — the user would have no way to leave the account \
             store fullscreen mode and return to the WoW Labs match-details panel."
        );

        let is_button: bool = env
            .eval(&format!(
                "return _G[{FULLSCREEN_CONTAINER_NAME:?}][{ACTUAL_LEAVE_BUTTON_KEY:?}]:IsObjectType(\"Button\")"
            ))
            .expect("`IsObjectType(\"Button\")` probe must run cleanly");

        assert!(
            is_button,
            "Expected `FullscreenAccountStoreContainer.{ACTUAL_LEAVE_BUTTON_KEY}:IsObjectType(\"Button\")` \
             to return true after `{ROOT}` loads. The XML declares the element as `<Button>` \
             (not `<Frame>`), so the simulator's widget-type registration MUST land in the \
             Button branch. A false reading means the XML-to-widget conversion misrouted the \
             element type — every Button-specific method (`SetText`, `Enable`, `Disable`, \
             `Click`, etc.) would surface a missing-method error."
        );

        for (parent_key, defining_role) in THREE_SLICE_PARENTKEY_FINGERPRINT {
            let child_type: String = env
                .eval(&format!(
                    "return type(_G[{FULLSCREEN_CONTAINER_NAME:?}][{ACTUAL_LEAVE_BUTTON_KEY:?}][{parent_key:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe `LeaveStoreButton.{parent_key}` (three-slice fingerprint): {error}"
                    )
                });

            assert_eq!(
                child_type, "table",
                "Expected `FullscreenAccountStoreContainer.{ACTUAL_LEAVE_BUTTON_KEY}.{parent_key}` \
                 to be a table after `{ROOT}` loads (`{defining_role}`), got `{child_type}`. \
                 The four parentKeys `Left` / `Right` / `Center` / `Controller` are the \
                 structural fingerprint of `ThreeSliceButtonTemplate` (the parent template that \
                 `BigRedThreeSliceButtonTemplate` inherits via \
                 `Blizzard_SharedXML/Shared/Button/ThreeSliceButtonTemplate.xml:61`). PLAN.md \
                 names `MagicButtonTemplate` as the inherited template, but \
                 `MagicButtonTemplate` (declared at `SharedUIPanelTemplates.xml:722`, \
                 inheriting `UIPanelButtonTemplate`) does not declare any of these four \
                 parentKeys — its visual structure comes from `MagicButton_OnLoad` setting up \
                 `UI-Panel-Button-*` stretched textures, not three-slice atlases. So all four \
                 fingerprint assertions present and accounted for IS the proof that the actual \
                 inheritance is `BigRedThreeSliceButtonTemplate`, NOT `MagicButtonTemplate`."
            );
        }

        let plan_named_button_type: String = env
            .eval(&format!(
                "return type(_G[{FULLSCREEN_CONTAINER_NAME:?}][{PLAN_NAMED_LEAVE_BUTTON_KEY:?}])"
            ))
            .expect("FullscreenAccountStoreContainer.LeaveButton absence probe must run cleanly");

        assert_eq!(
            plan_named_button_type, "nil",
            "Expected `FullscreenAccountStoreContainer.{PLAN_NAMED_LEAVE_BUTTON_KEY}` to be nil \
             after `{ROOT}` loads, got `{plan_named_button_type}`. PLAN.md's spec names \
             `LeaveButton` as the parentKey, but the actual XML at \
             `Blizzard_AccountStore.xml:167` uses `parentKey=\"LeaveStoreButton\"` — the \
             PLAN-named key is genuinely absent. This positive nil assertion is the spec/source \
             mismatch tripwire: if Blizzard ever renames the button to match the PLAN shape, \
             this test flips and forces a re-pin against the new parentKey."
        );
    });
}
}

const INSET_TEMPLATE_NAME: &str = "AccountStoreInsetFrameTemplate";

const INSET_TEMPLATE_FINGERPRINT: &[(&str, &str)] = &[
    (
        "Bg",
        "BACKGROUND-layer tiled Texture (Blizzard_AccountStore.xml:9 — \
         BlackMarketBackground-Tile, horizTile + vertTile)",
    ),
    (
        "InsetBorderTopLeft",
        "BORDER-layer corner Texture inheriting UI-Frame-InnerTopLeft \
         (Blizzard_AccountStore.xml:30)",
    ),
    (
        "InsetBorderTopRight",
        "BORDER-layer corner Texture inheriting UI-Frame-InnerTopRight \
         (Blizzard_AccountStore.xml:35)",
    ),
    (
        "InsetBorderBottomLeft",
        "BORDER-layer corner Texture inheriting UI-Frame-InnerBotLeftCorner \
         (Blizzard_AccountStore.xml:40)",
    ),
    (
        "InsetBorderBottomRight",
        "BORDER-layer corner Texture inheriting UI-Frame-InnerBotRight \
         (Blizzard_AccountStore.xml:45)",
    ),
    (
        "InsetBorderTop",
        "BORDER-layer top-edge tile inheriting _UI-Frame-InnerTopTile \
         (Blizzard_AccountStore.xml:50)",
    ),
    (
        "InsetBorderBottom",
        "BORDER-layer bottom-edge tile inheriting _UI-Frame-InnerBotTile \
         (Blizzard_AccountStore.xml:56)",
    ),
    (
        "InsetBorderLeft",
        "BORDER-layer left-edge tile inheriting !UI-Frame-InnerLeftTile \
         (Blizzard_AccountStore.xml:62)",
    ),
    (
        "InsetBorderRight",
        "BORDER-layer right-edge tile inheriting !UI-Frame-InnerRightTile \
         (Blizzard_AccountStore.xml:68)",
    ),
];

const INSET_TEMPLATE_INHERITORS: &[(&str, &str)] = &[
    ("LeftInset", "Blizzard_AccountStore.xml:93"),
    ("RightInset", "Blizzard_AccountStore.xml:100"),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_inset_frame_template_resolves_into_both_inheritors() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let virtual_template_global_type: String = env
            .eval(&format!("return type(_G[{INSET_TEMPLATE_NAME:?}])"))
            .expect("AccountStoreInsetFrameTemplate global probe must run cleanly");

        assert_eq!(
            virtual_template_global_type, "nil",
            "Expected `_G[{INSET_TEMPLATE_NAME:?}]` to be nil after `{ROOT}` loads, got \
             `{virtual_template_global_type}`. Virtual templates (declared with \
             `virtual=\"true\"` at `Blizzard_AccountStore.xml:5`) live in the simulator's \
             template registry only — they do not register as named globals because they are \
             never instantiated as standalone frames; they only contribute their declarative \
             shape to inheritors via the XML loader's template-resolution path. A non-nil \
             reading here means a regression converted the template into a concrete frame \
             registration (probably by a missing `virtual=\"true\"` attribute), which would \
             leak the template into `_G` and likely break the inheritance chain that drives \
             `LeftInset` / `RightInset`."
        );

        for (inheritor_key, inheritor_site) in INSET_TEMPLATE_INHERITORS {
            for (fingerprint_key, fingerprint_role) in INSET_TEMPLATE_FINGERPRINT {
                let child_type: String = env
                    .eval(&format!(
                        "return type(_G[{FRAME_NAME:?}][{inheritor_key:?}][{fingerprint_key:?}])"
                    ))
                    .unwrap_or_else(|error| {
                        panic!(
                            "failed to probe `AccountStoreFrame.{inheritor_key}.{fingerprint_key}` \
                             (template fingerprint): {error}"
                        )
                    });

                assert_eq!(
                    child_type, "table",
                    "Expected `AccountStoreFrame.{inheritor_key}.{fingerprint_key}` to be a \
                     table after `{ROOT}` loads. The {inheritor_key} parentKey at \
                     `{inheritor_site}` inherits `AccountStoreInsetFrameTemplate` via \
                     `inherits=\"{INSET_TEMPLATE_NAME}\"`, which contributes the \
                     `{fingerprint_key}` parentKey ({fingerprint_role}). A nil reading means \
                     either (a) the virtual-template registration was dropped (the registry \
                     never recorded the template body), or (b) the template-resolution path \
                     failed to copy the parentKey children onto the inheritor (a regression in \
                     the XML loader's inherit-merge routing). Either case would leave the \
                     inset frame without its background/border chrome, so the panel's two \
                     content panes would render against the bare frame backdrop."
                );
            }
        }
    });
}
}
