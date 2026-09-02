use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn zone_ability_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ZoneAbility")
}

fn zone_ability_toc() -> PathBuf {
    zone_ability_dir().join("Blizzard_ZoneAbility_Mainline.toc")
}

const REQUIRED_DEPS: &[&str] = &[
    "Blizzard_UIPanels_Game",
    "Blizzard_ActionBarController",
    "Blizzard_ActionBar",
    "Blizzard_PingUI",
];

const BODY_FILES: &[&str] = &["ZoneAbility.lua", "ZoneAbility.xml"];

const FRAME_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnEvent",
    "SetVariablesLoaded",
    "MarkDirty",
    "UpdateDisplayedZoneAbilities",
    "CheckForTutorial",
    "CanShowTutorial",
    "CheckShowZoneAbilityTutorial",
];

const SPELL_BUTTON_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnHide",
    "OnEvent",
    "OnEnter",
    "OnLeave",
    "OnClick",
    "OnDragStart",
    "Refresh",
    "CheckForTutorial",
    "SetSpellID",
    "GetSpellID",
    "GetOverrideSpellID",
    "SetContent",
];

const UPDATER_METHODS: &[&str] = &["AddDirtyFrame", "Clean"];

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[test]
fn find_toc_file_resolves_mainline_only_variant() {
    let resolved =
        find_toc_file(&zone_ability_dir()).expect("Blizzard_ZoneAbility TOC should resolve");
    assert_eq!(
        resolved,
        zone_ability_toc(),
        "Blizzard_ZoneAbility ships exactly one mainline-flavored TOC \
         (`Blizzard_ZoneAbility_Mainline.toc`) — NO bare `Blizzard_ZoneAbility.toc`, NO \
         `_Mists.toc`, NO `_Classic.toc`. find_toc_file probes the `_Mainline.toc` variant FIRST \
         and resolves on the first hit. The mainline-only TOC matches the AllowLoadGameType \
         restriction: zone-ability is a Shadowlands+ covenant/dragonriding/timerunning concept \
         that does not exist on Classic flavors"
    );
}

#[test]
fn toc_declares_eager_load_with_two_required_deps() {
    let toc =
        TocFile::from_file(&zone_ability_toc()).expect("Blizzard_ZoneAbility TOC should parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_ZoneAbility omits `## LoadOnDemand` — eagerly loaded with `## DefaultState: \
         enabled`. The frame must exist before the player enters a zone with active zone \
         abilities, so it cannot be LoD"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_ZoneAbility omits `## LoadFirst` — does NOT need to load before other addons. \
         It registers events in OnLoad and reacts to zone changes; sequence with other addons \
         does not matter"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_ZoneAbility omits `## UseSecureEnvironment` — file-scope globals land in `_G`, \
         not `__secureenv`"
    );

    let deps: Vec<String> = toc.dependencies().to_vec();
    assert_eq!(
        deps,
        REQUIRED_DEPS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "Blizzard_ZoneAbility declares four dependencies in source order: \
         Blizzard_UIPanels_Game, Blizzard_ActionBarController, Blizzard_ActionBar, and \
         Blizzard_PingUI. The ZoneAbility frame depends on the panel host, action-bar \
         controller and button surface, plus the Ping UI integration"
    );

    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
}

#[test]
fn toc_restricts_to_game_screen_and_mainline_gametype() {
    let toc =
        TocFile::from_file(&zone_ability_toc()).expect("Blizzard_ZoneAbility TOC should parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` permits the addon on the Game screen — zone abilities only exist \
         in the world, never on glue screens"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "`## AllowLoad: game` rejects glue screen {screen:?}"
        );
    }

    assert!(
        !toc.is_game_type_restricted(),
        "is_game_type_restricted at toc.rs:294-302 returns FALSE for `mainline` and `standard` \
         and TRUE for any other gametype value (plunderstorm / classic / wrath / cata / mists). \
         FIRST analyzed addon with `## AllowLoadGameType: mainline` — the directive is present \
         but does not flip the restriction flag because mainline IS the simulator's gametype. \
         The directive is still meaningful as a marker: it documents that the addon is \
         retail-only (Shadowlands covenant abilities, Dragonriding race buttons, Timerunning \
         extras are all mainline-only mechanics) even though the gametype-restricted shortcut \
         only fires for the inverse Classic-flavor case"
    );
}

#[test]
fn toc_lists_lua_then_xml_in_body() {
    let toc =
        TocFile::from_file(&zone_ability_toc()).expect("Blizzard_ZoneAbility TOC should parse");

    let body_files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    assert_eq!(
        body_files,
        BODY_FILES.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "TOC body lists `ZoneAbility.lua` (322 lines) FIRST and `ZoneAbility.xml` (69 lines) \
         SECOND. The order matters because the lua declares `ZoneAbilityFrameMixin = {{}}`, \
         `ZoneAbilityFrameSpellButtonMixin = CreateFromMixins(ContentFrameMixin)`, and \
         `ZoneAbilityFrameUpdater = {{}}` at file scope BEFORE the XML element \
         `<Frame mixin=\"ZoneAbilityFrameMixin\">` resolves the mixin name. If the XML loaded \
         first, the mixin reference would resolve to nil and OnLoad/OnEvent script handlers \
         would dispatch to nothing"
    );
}

#[test]
fn toc_raw_bytes_pin_directives() {
    let raw =
        std::fs::read_to_string(zone_ability_toc()).expect("Blizzard_ZoneAbility TOC should read");

    for directive in [
        "## Title: Blizzard_ZoneAbility",
        "## Author: Blizzard Entertainment",
        "## DefaultState: enabled",
        "## Dependencies: Blizzard_UIPanels_Game, Blizzard_ActionBarController, Blizzard_ActionBar, Blizzard_PingUI",
        "## AllowLoad: game",
        "## AllowLoadGameType: mainline",
        "ZoneAbility.lua",
        "ZoneAbility.xml",
    ] {
        assert!(
            raw.contains(directive),
            "TOC raw bytes must contain `{directive}` — eager mainline-only zone-ability addon"
        );
    }

    for absent_directive in [
        "## Version:",
        "## Notes:",
        "## RequiredDep:",
        "## RequiredDeps:",
        "## OptionalDeps:",
        "## LoadFirst:",
        "## LoadWith:",
        "## LoadOnDemand:",
        "## SavedVariables:",
        "## UseSecureEnvironment:",
    ] {
        assert!(
            !raw.contains(absent_directive),
            "TOC raw bytes must NOT contain `{absent_directive}`"
        );
    }
}

#[test]
fn directory_holds_three_entries() {
    let entries: Vec<String> = std::fs::read_dir(zone_ability_dir())
        .expect("Blizzard_ZoneAbility directory should exist")
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        3,
        "Blizzard_ZoneAbility directory must hold exactly 3 entries (mainline toc + lua + xml). \
         No flavor subdirectory, no Localization.lua, no separate Mixins.lua. Got: {entries:?}"
    );
}

#[test]
fn dep_directories_exist_on_disk() {
    for dep in REQUIRED_DEPS {
        let dep_dir = blizzard_ui_dir().join(dep);
        assert!(
            dep_dir.is_dir(),
            "Required dep `{dep}` must exist at `Interface/BlizzardUI/{dep}/` — one of the \
             four eager-loaded dependencies declared by the active retail TOC"
        );
    }
}

#[test]
fn appears_in_game_eager_discovery() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let names: Vec<&str> = addons.iter().map(|(name, _)| name.as_str()).collect();

    assert!(
        names.contains(&"Blizzard_ZoneAbility"),
        "Blizzard_ZoneAbility must appear in Game eager discovery — `## AllowLoad: game` + \
         `## AllowLoadGameType: mainline` + `## DefaultState: enabled` + no LoadOnDemand all \
         combine to make it an eagerly-discovered Game-screen addon on mainline. Got: {names:?}"
    );
}

#[test]
fn absent_from_glue_screen_auto_discovery() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let names: Vec<&str> = addons.iter().map(|(name, _)| name.as_str()).collect();

        assert!(
            !names.contains(&"Blizzard_ZoneAbility"),
            "Blizzard_ZoneAbility must NOT appear in {screen:?} eager discovery — \
             `## AllowLoad: game` excludes glue screens"
        );
    }
}

prefork_full_ui_case! {
fn loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("ZoneAbility")
                || message.contains("ZoneAbilityFrame")
                || message.contains("ZoneAbilityFrameMixin")
                || message.contains("ZoneAbilityFrameSpellButtonMixin")
                || message.contains("ZoneAbilityFrameUpdater")
        })
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_ZoneAbility emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_eager_pass(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ZoneAbility')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_ZoneAbility') must return true after the eager pass — \
         proves the mainline-restricted Game-screen addon brings up successfully"
    );

    for dep in REQUIRED_DEPS {
        let dep_loaded: bool = env
            .eval(&format!("return C_AddOns.IsAddOnLoaded('{dep}')"))
            .unwrap_or_else(|err| panic!("IsAddOnLoaded({dep}) probe failed: {err}"));
        assert!(
            dep_loaded,
            "Required dep `{dep}` must also be loaded — both are DefaultState: enabled core addons"
        );
    }
}
}

prefork_full_ui_case! {
fn zone_ability_frame_mixin_publishes_with_methods(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(ZoneAbilityFrameMixin)")
        .expect("ZoneAbilityFrameMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "ZoneAbilityFrameMixin must publish as a table at `_G` — declared at file scope on \
         line 63 of ZoneAbility.lua. The mixin is attached to the named ZoneAbilityFrame via \
         `<Frame mixin=\"ZoneAbilityFrameMixin\">` in the template"
    );

    for method in FRAME_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(ZoneAbilityFrameMixin.{method})"))
            .unwrap_or_else(|err| panic!("ZoneAbilityFrameMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "ZoneAbilityFrameMixin.{method} must be a function — declared via \
             `function ZoneAbilityFrameMixin:Method(...)` syntax in the lua file"
        );
    }
}
}

prefork_full_ui_case! {
fn zone_ability_spell_button_mixin_inherits_content_frame_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(ZoneAbilityFrameSpellButtonMixin)")
        .expect("ZoneAbilityFrameSpellButtonMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "ZoneAbilityFrameSpellButtonMixin must publish as a table at `_G` — declared at line \
         230 of ZoneAbility.lua via `CreateFromMixins(ContentFrameMixin)`. This is the FIRST \
         analyzed addon where a published mixin EXPLICITLY inherits via CreateFromMixins at \
         declaration time (prior addons either defined plain `{{}}` mixins or invoked \
         CreateFromMixins inside OnLoad). The inheritance pulls in ContentFrameMixin's \
         SetContent/GetContent/Refresh contract used by ManagedHorizontalLayoutFrameTemplate \
         to populate the SpellButtonContainer's children"
    );

    for method in SPELL_BUTTON_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!(
                "return type(ZoneAbilityFrameSpellButtonMixin.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("ZoneAbilityFrameSpellButtonMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            method_kind, "function",
            "ZoneAbilityFrameSpellButtonMixin.{method} must be a function — XML wires 8 script \
             handlers (OnLoad/OnShow/OnHide/OnEvent/OnClick/OnDragStart/OnEnter/OnLeave) to the \
             mixin, plus the lua adds 6 helpers (Refresh/CheckForTutorial/SetSpellID/GetSpellID/ \
             GetOverrideSpellID/SetContent). GetOverrideSpellID delegates to \
             C_SpellBook.FindSpellOverrideByID so passive talent overrides are picked up at \
             cast/tooltip time"
        );
    }
}
}

prefork_full_ui_case! {
fn zone_ability_frame_updater_publishes_dirty_batcher(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(ZoneAbilityFrameUpdater)")
        .expect("ZoneAbilityFrameUpdater probe should succeed");
    assert_eq!(
        kind, "table",
        "ZoneAbilityFrameUpdater must publish as a table at `_G` — declared at line 39 of \
         ZoneAbility.lua. The updater is a singleton dirty-frame batcher that coalesces \
         repeated MarkDirty() calls within a single tick into a single \
         UpdateDisplayedZoneAbilities call via `C_Timer.After(0, function() self:Clean() end)`. \
         Without this batcher, every UNIT_AURA / SPELLS_CHANGED / ACTIONBAR_SLOT_CHANGED / \
         vehicle event would trigger a full table.sort + atlas-info lookup pass — at-once \
         coalescence keeps the heavy work on the next-frame boundary"
    );

    for method in UPDATER_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(ZoneAbilityFrameUpdater.{method})"))
            .unwrap_or_else(|err| panic!("ZoneAbilityFrameUpdater.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "ZoneAbilityFrameUpdater.{method} must be a function — AddDirtyFrame seeds the \
             dirty-set and arms the timer; Clean drains the dirty-set and clears the isDirty \
             flag for the next tick"
        );
    }
}
}

prefork_full_ui_case! {
fn frame_template_registers_as_virtual(env: &WowLuaEnv) {
    let _env = env;

    assert!(
        wow_ui_sim::xml::get_template("ZoneAbilityFrameTemplate").is_some(),
        "ZoneAbilityFrameTemplate (`<Frame virtual=\"true\">` from ZoneAbility.xml:3) must be \
         registered in the template registry. The template carries the OVERLAY-layer Style \
         texture (atlas display backing, swapped per textureKit), the SpellButtonContainer \
         child (ManagedHorizontalLayoutFrameTemplate with spacing=4, fixedHeight=52), and the \
         OnLoad/OnEvent script bindings to ZoneAbilityFrameMixin"
    );
}
}

prefork_full_ui_case! {
fn spell_button_template_registers_as_virtual(env: &WowLuaEnv) {
    let _env = env;

    assert!(
        wow_ui_sim::xml::get_template("ZoneAbilityFrameSpellButtonTemplate").is_some(),
        "ZoneAbilityFrameSpellButtonTemplate (`<Button virtual=\"true\">` from \
         ZoneAbility.xml:31) must be registered in the template registry. Used at lua:79 via \
         `self.SpellButtonContainer:SetTemplate(\"Button\", \
         \"ZoneAbilityFrameSpellButtonTemplate\")` — the ManagedHorizontalLayoutFrameTemplate \
         host pool calls CreateFrame with this template name when the dirty-frame batcher \
         lands a non-empty zone-abilities array"
    );
}
}

prefork_full_ui_case! {
fn zone_ability_frame_publishes_hidden_with_template_chrome(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(ZoneAbilityFrame)")
        .expect("ZoneAbilityFrame probe should succeed");
    assert_eq!(
        kind, "table",
        "ZoneAbilityFrame must publish at `_G` — `<Frame name=\"ZoneAbilityFrame\" \
         inherits=\"ZoneAbilityFrameTemplate\" hidden=\"true\"/>` at xml:68 resolves through \
         the template registry"
    );

    let frame_name: String = env
        .eval("return ZoneAbilityFrame:GetName()")
        .expect("GetName probe should succeed");
    assert_eq!(frame_name, "ZoneAbilityFrame");

    let object_type: String = env
        .eval("return ZoneAbilityFrame:GetObjectType()")
        .expect("GetObjectType probe should succeed");
    assert_eq!(
        object_type, "Frame",
        "ZoneAbilityFrame is a Frame (NOT a Button) — clicking is delegated to the per-spell \
         child Buttons created via ZoneAbilityFrameSpellButtonTemplate, not to the container \
         itself"
    );

    let is_shown: bool = env
        .eval("return ZoneAbilityFrame:IsShown()")
        .expect("IsShown probe should succeed");
    assert!(
        !is_shown,
        "ZoneAbilityFrame must start hidden — `hidden=\"true\"` in the XML attribute. The \
         frame is only made visible by ExtraAbilityContainer:AddFrame at lua:174 when \
         UpdateDisplayedZoneAbilities lands a non-empty zone-abilities list (and \
         RemoveFrame'd at lua:170 when the list goes empty)"
    );
}
}

prefork_full_ui_case! {
fn zone_ability_frame_inherits_template_children(env: &WowLuaEnv) {

    for child_key in ["Style", "SpellButtonContainer"] {
        let child_kind: String = env
            .eval(&format!(
                "return type(ZoneAbilityFrame and ZoneAbilityFrame.{child_key})"
            ))
            .unwrap_or_else(|err| panic!("ZoneAbilityFrame.{child_key} probe failed: {err}"));
        assert!(
            matches!(child_kind.as_str(), "table" | "userdata"),
            "ZoneAbilityFrame.{child_key} must resolve to a child object via parentKey \
             inheritance from the ZoneAbilityFrameTemplate — got `{child_kind}`. Style is the \
             OVERLAY-layer Texture that swaps atlas via SetAtlas (textureKit-driven \
             covenant/zone backing) or SetTexture fallback; SpellButtonContainer is the \
             ManagedHorizontalLayoutFrameTemplate host that pools the spell buttons"
        );
    }
}
}
