#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_addons_for_screen, find_toc_file, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn move_pad_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_MovePad")
}

fn move_pad_toc() -> PathBuf {
    move_pad_dir().join("Blizzard_MovePad.toc")
}

const MOVE_PAD_TOC_FILES: &[&str] = &[
    "Blizzard_MovePad_Bootstrap.lua",
    "Blizzard_MovePad.lua",
    "Blizzard_MovePad.xml",
];

const PUBLIC_MIXINS: &[&str] = &[
    "MovePadMixin",
    "MovePadCheckboxMixin",
    "MovePadForwardMixin",
    "MovePadBackwardMixin",
    "MovePadRotateLeftMixin",
    "MovePadRotateRightMixin",
    "MovePadStrafeLeftMixin",
    "MovePadStrafeRightMixin",
    "MovePadJumpMixin",
];

const NAMED_FRAMES: &[&str] = &[
    "MovePadFrame",
    "MovePadForward",
    "MovePadJump",
    "MovePadBackward",
    "MovePadRotateLeft",
    "MovePadRotateRight",
    "MovePadStrafeLeft",
    "MovePadStrafeRight",
];

fn load_move_pad(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &move_pad_toc())
        .expect("Blizzard_MovePad on-demand load succeeds");
}

#[test]
fn blizzard_move_pad_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&move_pad_dir()).expect("Blizzard_MovePad TOC resolves");
    assert_eq!(
        resolved,
        move_pad_toc(),
        "Blizzard_MovePad ships exactly one bare TOC — no `_Mainline.toc` and no \
         `_Classic.toc`. The clickable movement-pad surface is a controller / accessibility \
         feature with no flavor-specific divergence; one bare TOC carries the addon"
    );

    let mainline = move_pad_dir().join("Blizzard_MovePad_Mainline.toc");
    let classic = move_pad_dir().join("Blizzard_MovePad_Classic.toc");
    assert!(
        !mainline.exists() && !classic.exists(),
        "There must be NO flavor-suffixed TOC variants — the addon is flavor-agnostic at \
         the TOC layer"
    );
}

#[test]
fn blizzard_move_pad_toc_declares_load_on_demand_with_no_dependencies() {
    let toc = TocFile::from_file(&move_pad_toc()).expect("Blizzard_MovePad TOC parses");
    assert!(
        toc.is_load_on_demand(),
        "TOC must declare `## LoadOnDemand: 1` — the move-pad surface is a deferred-load \
         accessibility frame. The user toggles it via the `enableMovePad` setting; only \
         when that setting flips true does the loader pull the addon out of the lod_pool. \
         It does NOT eager-load on Game-screen boot"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        toc.dependencies().is_empty(),
        "Zero `## Dependencies:` — the move-pad addon is dependency-free at the TOC layer. \
         It calls Settings.SetOnValueChangedCallback (only IF Settings is non-nil at OnLoad), \
         CVarCallbackRegistry methods, FrameUtil.RegisterForTopLevelParentChanged, \
         SquareButton_SetIcon, RunBinding, ValidateFramePosition, MoveForwardStart / \
         MoveBackwardStart / TurnLeftStart / etc — all foundational SharedXML / SharedXMLGame \
         globals or hard-coded movement-binding C entry points"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — the move-pad's locked / press-and-hold modes persist via \
         CVars (movePadLocked / movePadInPressAndHoldMode), not Lua-side SavedVariables. \
         The visibility toggle is a Settings system callback against the `enableMovePad` \
         setting"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC omits `## AllowLoadGameType:` — the addon is flavor-agnostic. \
         is_game_type_restricted() returns false when the directive is absent"
    );
}

#[test]
fn blizzard_move_pad_toc_declares_load_on_demand_in_raw_bytes() {
    let raw = std::fs::read_to_string(move_pad_toc()).expect("Blizzard_MovePad TOC reads");
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly. The numeric truthiness keyword (1) \
         is the canonical Blizzard spelling — alternatives like `## LoadOnDemand: true` are \
         accepted by the parser but the live tree uses `1` consistently for LoD addons"
    );
    assert!(
        !raw.contains("## DefaultState"),
        "TOC must NOT declare `## DefaultState:` — DefaultState is meaningless on a \
         LoadOnDemand addon (the lod_pool routing pre-empts the eager-load decision)"
    );
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare `## Dependencies:` — the move-pad addon walks only \
         foundational globals; no sibling addons are required"
    );
}

#[test]
fn blizzard_move_pad_toc_lists_bootstrap_lua_and_xml() {
    let toc = TocFile::from_file(&move_pad_toc()).expect("Blizzard_MovePad TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, MOVE_PAD_TOC_FILES,
        "Retail 12.1.0.69497 lists the bootstrap first, then Blizzard_MovePad.lua and \
         Blizzard_MovePad.xml. The main Lua must precede XML so its mixin tables exist when \
         XML resolves per-frame mixins"
    );
}

#[test]
fn blizzard_move_pad_does_not_appear_in_eager_discovery_on_any_screen() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_MovePad");
        assert!(
            !found,
            "Blizzard_MovePad must NOT appear in `discover_blizzard_addons_for_screen` on \
             any screen — `## LoadOnDemand: 1` routes the addon to the lod_pool at \
             src/loader/mod.rs:530-534. The lod_pool is consulted only when a non-LOD addon \
             declares MovePad as a dependency (none does) or when load_addon is called \
             explicitly (the Settings system's enableMovePad-flip path). (Screen tested: \
             {screen:?})"
        );
    }
}

#[test]
fn blizzard_move_pad_appears_in_discover_all_blizzard_addons() {
    let all_addons = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = all_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_MovePad");
    assert!(
        found,
        "Blizzard_MovePad MUST appear in `discover_all_blizzard_addons` — that function \
         walks every `Blizzard_*` directory regardless of LoadOnDemand routing or screen \
         restrictions. The `## LoadOnDemand: 1` directive only excludes the addon from the \
         eager screen-specific sweep; the all-addons enumeration does not honor LoD"
    );
}

prefork_full_ui_case! {
fn blizzard_move_pad_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_move_pad(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_MovePad")
                || message.contains("MovePadMixin")
                || message.contains("MovePadFrame")
                || message.contains("MovePadCheckbox")
                || message.contains("MovePadForward")
                || message.contains("MovePadBackward")
                || message.contains("MovePadJump")
                || message.contains("MovePadRotate")
                || message.contains("MovePadStrafe")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_MovePad emitted addon-specific Lua errors during on-demand load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_move_pad_is_addon_loaded_after_explicit_load_addon_call(env: &WowLuaEnv) {
    load_move_pad(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_MovePad')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_MovePad') must return true after the explicit \
         load_addon call — proves the LoD routing through the simulator's loader registers \
         the addon in the loaded-set the same way an eager-discovery load would, even \
         though the discovery sweep skipped it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_move_pad_publishes_nine_mixins_as_tables(env: &WowLuaEnv) {
    load_move_pad(env);
    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type({mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "Mixin `{mixin}` must publish at `_G` as a table after Blizzard_MovePad loads. \
             The 9 mixins partition responsibility — MovePadMixin orchestrates the parent \
             frame, MovePadCheckboxMixin is the shared 6-checkbox handler base, and the 7 \
             per-direction mixins (Forward / Backward / RotateLeft / RotateRight / \
             StrafeLeft / StrafeRight / Jump) carry the icon-loading OnLoad bindings"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_move_pad_publishes_eight_named_xml_frames_as_globals(env: &WowLuaEnv) {
    load_move_pad(env);
    for frame in NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type({frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "Named XML frame `{frame}` must publish at `_G` after on-demand load. \
             MovePadFrame is the parent (parent=UIParent, frameStrata=BACKGROUND, \
             movable=true, registerForDrag=LeftButton); the 7 named children — Forward / \
             Backward / RotateLeft / RotateRight / StrafeLeft / StrafeRight (CheckButtons) \
             plus Jump (Button) — anchor relative to each other in a 3x3 D-pad grid \
             centered on the Jump button"
        );
    }

    let dropdown_kind: String = env
        .eval("return type(MovePadFrame.SettingsDropdown)")
        .expect("MovePadFrame.SettingsDropdown probe succeeds");
    assert_eq!(
        dropdown_kind, "table",
        "MovePadFrame.SettingsDropdown must resolve as a table — the XML declares an \
         anonymous DropdownButton child with parentKey=\"SettingsDropdown\" inheriting \
         UIPanelIconDropdownButtonTemplate. The parentKey path is how SetupDropdownMenu at \
         line 76 reaches the dropdown to populate it with 2 checkbox menu entries \
         (MOVE_PAD_LOCKED + MOVE_PAD_PRESS_AND_HOLD_MODE)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_move_pad_checkbox_template_does_not_leak_as_global(env: &WowLuaEnv) {
    load_move_pad(env);
    let kind: String = env
        .eval("return type(_G.MovePadCheckboxTemplate)")
        .expect("MovePadCheckboxTemplate _G probe succeeds");
    assert_eq!(
        kind, "nil",
        "_G.MovePadCheckboxTemplate must remain nil — the XML declares it as `virtual=true` \
         at line 3, which registers the template in the XML template registry but MUST NOT \
         publish a `_G.<TemplateName>` global. Virtual templates are pure-XML inheritance \
         vehicles; runtime addon code addresses them only by string name to the \
         CreateFrame inherits parameter"
    );
}
}

prefork_full_ui_case! {
fn blizzard_move_pad_parent_array_collects_six_check_buttons(env: &WowLuaEnv) {
    load_move_pad(env);
    let count: i64 = env
        .eval("return #MovePadFrame.MoveButtons")
        .expect("MovePadFrame.MoveButtons length probe succeeds");
    assert_eq!(
        count, 6,
        "MovePadFrame.MoveButtons must hold exactly 6 entries. The parentArray=\"MoveButtons\" \
         attribute on MovePadCheckboxTemplate (XML line 3) collects every CheckButton \
         instantiated from the template into the parent's `MoveButtons` array. The 6 \
         instances are MovePadForward / MovePadBackward / MovePadRotateLeft / \
         MovePadRotateRight / MovePadStrafeLeft / MovePadStrafeRight (the Jump button uses \
         the plain UIPanelSquareButton template — it is NOT a checkbox and does NOT join \
         the array). MovePadMixin:ResetMoveButtons iterates this array to clear opposing \
         button state on press-and-hold transitions; got {count}"
    );
}
}

prefork_full_ui_case! {
fn blizzard_move_pad_opposing_button_pairs_are_wired_after_onload(env: &WowLuaEnv) {
    load_move_pad(env);

    let pairs = [
        ("MovePadForward", "MovePadBackward"),
        ("MovePadBackward", "MovePadForward"),
        ("MovePadRotateLeft", "MovePadRotateRight"),
        ("MovePadRotateRight", "MovePadRotateLeft"),
        ("MovePadStrafeLeft", "MovePadStrafeRight"),
        ("MovePadStrafeRight", "MovePadStrafeLeft"),
    ];
    for (button, expected_opposite) in pairs {
        let probe = format!("return rawequal({button}.opposingMoveButton, {expected_opposite})");
        let wired: bool = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("opposingMoveButton probe for {button}: {err}"));
        assert!(
            wired,
            "{button}.opposingMoveButton must reference {expected_opposite}. \
             MovePadMixin:OnLoad lines 20-27 wires the 3 opposing-direction pairs symmetrically \
             — Forward ↔ Backward, RotateLeft ↔ RotateRight, StrafeLeft ↔ StrafeRight. The \
             OnMovePadCheckboxClick handler at line 113 reads opposingMoveButton to reset \
             the opposite direction's checkbox state when the press toggles, preventing the \
             user from being 'locked' moving in two opposing directions at once"
        );
    }
}
}
