use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn unit_frame_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_UnitFrame")
}

fn unit_frame_toc() -> PathBuf {
    unit_frame_dir().join("Blizzard_UnitFrame_Mainline.toc")
}

fn unit_frame_classic_toc() -> PathBuf {
    unit_frame_dir().join("Blizzard_UnitFrame_Classic.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOC_DEPENDENCIES: &[&str] = &[
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_BuffFrame",
    "Blizzard_UIPanels_Game",
    "Blizzard_ActionBar",
    "Blizzard_TextStatusBar",
    "Blizzard_SpellDiminishUI",
    "Blizzard_PingUI",
    "Blizzard_ColorPickerFrame",
    "Blizzard_ManagedFrameSystem",
    "Blizzard_StatusTrayManager",
];

const REPRESENTATIVE_BODY_FILES: &[&str] = &[
    "Mainline\\BuilderSpenderFrame.lua",
    "Mainline\\PowerBarColorUtil.lua",
    "Shared\\StatusBarOverlaySegment.lua",
    "Shared\\UnitFrame.lua",
    "Mainline\\UnitFrame.lua",
    "Shared\\UnitFrame.xml",
    "Shared\\PlayerFrame.lua",
    "Mainline\\PlayerFrame.xml",
    "Shared\\TargetFrame.lua",
    "Mainline\\TargetFrame.xml",
    "Mainline\\RuneFrame.xml",
    "Mainline\\ClassPowerBar.xml",
    "Shared\\CompactUnitFrame.lua",
    "Shared\\CompactRaidGroup.xml",
    "Mainline\\CompactArenaFrame.lua",
    "DemonHunterSoulFragmentsBar.lua",
    "DevourerFuryBar.lua",
    "Mainline\\Localization.lua",
];

const FAMILY_PLACEHOLDER_BODY_FILE: &str = "[Family]\\CompactUnitFrameOptions.lua";
const FAMILY_PLACEHOLDER_RESOLVED_RELATIVE: &str = "Mainline/CompactUnitFrameOptions.lua";

const REPRESENTATIVE_FRAME_MIXINS: &[&str] = &[
    "PlayerBottomManagedFrameContainerMixin",
    "TargetFrameMixin",
    "TargetFrameStatusBarMixin",
    "TargetFrameHealthBarMixin",
    "TargetOfTargetMixin",
    "FocusFrameMixin",
    "PetFrameMixin",
    "PetHealthBarMixin",
    "PetManaBarMixin",
    "PetCastingBarMixin",
    "PartyMemberFrameMixin",
    "PartyMemberPetFrameMixin",
    "BossTargetFrameMixin",
    "BossTargetFrameContainerMixin",
    "BossSpellBarMixin",
    "TargetSpellBarMixin",
    "EncounterBarMixin",
    "RuneFrameMixin",
    "RuneButtonMixin",
];

const REPRESENTATIVE_RESOURCE_MIXINS: &[&str] = &[
    "ClassResourceBarMixin",
    "DruidComboPointBarMixin",
    "DruidComboPointMixin",
    "RogueComboPointBarMixin",
    "RogueComboPointMixin",
    "WarlockShardMixin",
    "ArcaneChargeMixin",
    "MonkLightEnergyMixin",
    "MonkStaggerBarMixin",
    "EssencePointButtonMixin",
    "EvokerEbonMightBarMixin",
    "PlayerFrameEvokerEbonMightBarMixin",
    "AlternatePowerBarMixin",
    "AlternatePowerBarBaseMixin",
    "PlayerFrameAlternatePowerBarBaseMixin",
    "PlayerPowerBarAltMixin",
    "TotemFrameMixin",
    "TotemButtonMixin",
];

const REPRESENTATIVE_COMPACT_AND_AURA_MIXINS: &[&str] = &[
    "CompactPartyFrameMixin",
    "CompactArenaFrameMixin",
    "CompactBuffMixin",
    "CompactDebuffMixin",
    "CompactAuraTooltipMixin",
    "CompactUnitFrameCenterStatusIconMixin",
    "CompactUnitFrameReadyCheckMixin",
    "CompactUnitIndividualPrivateAuraAnchorMixin",
    "BasePrivateAuraBehaviorMixin",
    "ContainerPrivateAuraBehaviorMixin",
    "PrivateAuraAnchorSettingsContainerMixin",
    "ArenaPreMatchFramesContainerMixin",
    "PreMatchArenaUnitFrameMixin",
    "StealthedArenaUnitFrameMixin",
    "ArenaUnitFrameDebuffMixin",
    "ArenaUnitFrameCcRemoverMixin",
    "PartyMemberBuffTooltipMixin",
    "PartyAuraFrameMixin",
    "ResurrectableIndicatorMixin",
    "AnimatedHealthLossMixin",
    "TempMaxHealthLossMixin",
    "TempMaxHealthLossDividerMixin",
    "StatusBarOverlaySegmentMixin",
];

const REPRESENTATIVE_VIRTUAL_TEMPLATES: &[&str] = &[
    "ArenaUnitFrameCastingBarTemplate",
    "ArenaUnitFrameCcRemoverTemplate",
    "ArenaUnitFrameCooldownTemplate",
    "ArenaUnitFrameDebuffTemplate",
    "BossSpellBarTemplate",
    "BossTargetFrameTemplate",
    "BuilderSpenderFrame",
    "ClassPowerBarFrame",
    "ClassResourceBarTemplate",
    "ComboPointTemplate",
    "CompactArenaFrameTemplate",
    "CompactPartyFrameTemplate",
    "CompactPartyPetUnitFrameTemplate",
    "CompactRaidGroupTemplate",
    "CompactRaidGroupUnitFrameTemplate",
    "CompactUnitFrameTemplate",
    "DruidComboPointBarTemplate",
    "DruidComboPointTemplate",
    "EssencePlayerFrameTemplate",
    "EssencePointButtonTemplate",
    "EvokerEbonMightBarTemplate",
    "FullResourcePulseFrame",
    "HealAbsorbBarTemplate",
    "MonkHarmonyBarFrameTemplate",
    "MonkLightEnergyTemplate",
    "MonkStaggerBarTemplate",
    "MyHealPredictionBarTemplate",
    "OtherHealPredictionBarTemplate",
    "PaladinPowerBarFrameTemplate",
    "PlayerBottomManagedFrameTemplate",
    "PlayerManagedContainerTemplate",
];

const REPRESENTATIVE_NON_VIRTUAL_FRAMES: &[&str] = &[
    "PlayerFrame",
    "TargetFrame",
    "FocusFrame",
    "PetFrame",
    "PetCastingBarFrame",
    "PartyFrame",
    "ComboFrame",
    "RuneFrame",
    "EncounterBar",
    "TotemFrame",
    "BossTargetFrameContainer",
    "Boss1TargetFrame",
    "Boss2TargetFrame",
    "Boss3TargetFrame",
    "Boss4TargetFrame",
    "Boss5TargetFrame",
    "EssencePlayerFrame",
    "InsanityBarFrame",
    "MageArcaneChargesFrame",
    "MonkHarmonyBarFrame",
    "PaladinPowerBarFrame",
    "RogueComboPointBarFrame",
    "DruidComboPointBarFrame",
    "WarlockPowerFrame",
    "PlayerPowerBarAlt",
    "PlayerBottomManagedFrameContainer",
    "PartyMemberBuffTooltip",
    "PlayerBuffTimerManager",
];

const MODULE_LOAD_NUMBER_CONSTANTS: &[(&str, f64)] = &[
    ("MAX_PARTY_MEMBERS", 4.0),
    ("MAX_PARTY_BUFFS", 4.0),
    ("MAX_PARTY_DEBUFFS", 4.0),
    ("MAX_PARTY_TOOLTIP_BUFFS", 16.0),
    ("MAX_PARTY_TOOLTIP_BUFFS_PER_ROW", 8.0),
    ("MAX_PARTY_TOOLTIP_DEBUFFS", 8.0),
    ("MAX_COMBO_POINTS", 5.0),
    ("MAX_TARGET_DEBUFFS", 16.0),
    ("MAX_TARGET_BUFFS", 32.0),
    ("MAX_BOSS_FRAMES", 5.0),
    ("RAID_TARGET_TEXTURE_COLUMNS", 4.0),
    ("RAID_TARGET_TEXTURE_ROWS", 4.0),
    ("REQUIRED_REST_HOURS", 5.0),
    ("HOLY_POWER_SPELL_READY", 3.0),
    ("COMBOFRAME_FADE_IN", 0.3),
    ("COMBOFRAME_FADE_OUT", 0.5),
    ("COMBOFRAME_HIGHLIGHT_FADE_IN", 0.4),
    ("CUF_READY_CHECK_DECAY_TIME", 11.0),
    ("CUF_NAME_SECTION_SIZE", 15.0),
];

fn fresh_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = fresh_game_env();

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn find_toc_file_resolves_mainline_variant() {
    let resolved = find_toc_file(&unit_frame_dir()).expect("UnitFrame TOC resolves");
    assert_eq!(
        resolved,
        unit_frame_toc(),
        "find_toc_file at src/loader/mod.rs:65-95 prefers \
         `<addon>_Mainline.toc`. Blizzard_UnitFrame ships TWO TOCs \
         (Mainline + Classic) — the Mainline variant has 7 hard deps \
         and ~53 body files arranged across Shared / Mainline subdirs, \
         while the Classic companion drops Blizzard_ActionBar + \
         Blizzard_SpellDiminishUI deps and adds Blizzard_ReadyCheck + \
         Blizzard_UIParentPanelManager (different dispatch chain), \
         pulls in Wrath/Cata/Mists/Vanilla flavor-suffixed body files \
         via inline `[AllowLoadGameType ...]` annotations, and \
         restricts gametype to `classic` so it's filtered out on a \
         mainline build"
    );
}

#[test]
fn toc_is_eager_with_ten_dependencies() {
    let toc = TocFile::from_file(&unit_frame_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` directive → eagerly loaded. PlayerFrame \
         and TargetFrame must be alive at PLAYER_ENTERING_WORLD so \
         UNIT_HEALTH / UNIT_POWER_UPDATE / UNIT_AURA / UNIT_SPELLCAST_* \
         events have somewhere to dispatch"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps, TOC_DEPENDENCIES,
        "Retail 12.1.0.69497 declares ten UnitFrame dependencies in published order, \
         including PingUI, ColorPickerFrame, ManagedFrameSystem, and StatusTrayManager. \
         Got: {deps:?}"
    );

    assert!(toc.optional_deps().is_empty());
    assert!(toc.load_with().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(toc.default_enabled());
}

#[test]
fn allow_load_game_restricts_to_in_world() {
    let toc = TocFile::from_file(&unit_frame_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Game` (titlecase here, but matched \
         case-insensitively at toc.rs:308 via eq_ignore_ascii_case) \
         hits → Game-only. Unit frames target real units; glue \
         screens have no UnitId concept"
    );
    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "Glue screen {screen:?} must be excluded — `AllowLoad: Game` \
             matches only the Game variant via toc.rs:308"
        );
    }
}

#[test]
fn allow_load_game_type_mainline_is_not_restricted() {
    let toc = TocFile::from_file(&unit_frame_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "`## AllowLoadGameType: mainline` hits the non-restricting \
         branch at toc.rs:294-302 (standard|mainline accepted)"
    );
}

#[test]
fn classic_companion_is_restricted_on_mainline_build() {
    let classic = TocFile::from_file(&unit_frame_classic_toc()).expect("Classic TOC parses");

    assert!(
        classic.is_game_type_restricted(),
        "Companion `_Classic.toc` carries `## AllowLoadGameType: classic` \
         which hits toc.rs:294-302 as a non-{{standard,mainline}} value \
         → restricted on a mainline build. The Classic body files (with \
         Wrath/Cata/Mists/Vanilla flavor inline annotations like \
         `Wrath\\AlternatePowerBar.lua [AllowLoadGameType wrath, cata, mists]`) \
         only matter on Classic-flavor builds; the Mainline build \
         silently skips this TOC at discovery time"
    );
}

#[test]
fn toc_raw_bytes_pin_directives_and_representative_body_files() {
    let raw = std::fs::read_to_string(unit_frame_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard_UnitFrame",
        "## Author: Blizzard Entertainment",
        "## Dependencies: Blizzard_SettingsDefinitions_Frame, \
         Blizzard_BuffFrame, Blizzard_UIPanels_Game, Blizzard_ActionBar, \
         Blizzard_TextStatusBar, Blizzard_SpellDiminishUI, Blizzard_PingUI, \
         Blizzard_ColorPickerFrame, Blizzard_ManagedFrameSystem, Blizzard_StatusTrayManager",
        "## AllowLoad: Game",
        "## AllowLoadGameType: mainline",
    ];

    for line in expected_directives {
        assert!(raw.contains(line), "Raw TOC must pin directive `{line}`");
    }

    for path in REPRESENTATIVE_BODY_FILES {
        assert!(
            raw.contains(path),
            "Raw TOC must pin body path `{path}`. Body order layers \
             Shared\\UnitFrame.lua (cross-flavor base) BEFORE \
             Mainline\\UnitFrame.lua (mainline-only behaviour overlay) \
             so the Mainline override can patch fields the shared file \
             populated. Same pattern for PlayerFrame / PartyMemberFrame \
             / TargetFrame / PartyFrameTemplates"
        );
    }

    assert!(
        raw.contains(FAMILY_PLACEHOLDER_BODY_FILE),
        "Raw TOC must pin `{FAMILY_PLACEHOLDER_BODY_FILE}` — the \
         `[Family]` placeholder is substituted at toc.rs:145 to the \
         build's family directory (`Mainline` for the Mainline build, \
         `Classic` for the Classic build). This lets one TOC line \
         reference whichever family-specific copy of \
         CompactUnitFrameOptions.lua exists. The Classic TOC reuses \
         the same `[Family]\\CompactUnitFrameOptions.lua` line and gets \
         the `Classic\\` resolution"
    );

    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## LoadWith"));
    assert!(!raw.contains("## OptionalDeps"));
    assert!(!raw.contains("## SavedVariables"));
}

#[test]
fn family_placeholder_resolves_to_mainline_directory() {
    let resolved = unit_frame_dir().join(FAMILY_PLACEHOLDER_RESOLVED_RELATIVE);
    assert!(
        resolved.is_file(),
        "`{FAMILY_PLACEHOLDER_BODY_FILE}` resolves to \
         `{resolved:?}` on disk. toc.rs:145 substitutes \
         `[Family]` → `Mainline` literally; the Mainline build's body \
         file must exist at the resolved path or load_addon will fail \
         with a missing-file error"
    );
}

#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons.iter().any(|(name, _)| name == "Blizzard_UnitFrame");
    assert!(
        found,
        "Blizzard_UnitFrame must appear in Game eager discovery — \
         PlayerFrame, TargetFrame, PartyFrame, RuneFrame, etc. are \
         the canonical visible-on-PLAYER_ENTERING_WORLD frames"
    );
}

#[test]
fn absent_from_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_UnitFrame");
        assert!(
            !found,
            "Blizzard_UnitFrame must NOT appear on {screen:?} — \
             AllowLoad:Game restricts to in-world via toc.rs:308, \
             checked at loader/mod.rs:527 BEFORE pool partitioning"
        );
    }
}

#[test]
fn dep_directories_exist_on_disk() {
    for dep in TOC_DEPENDENCIES {
        let dir = blizzard_ui_dir().join(dep);
        assert!(
            dir.is_dir(),
            "Hard-dep directory `{dep}` must exist on disk"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "{dep} must have a discoverable TOC"
        );
    }
}

prefork_full_ui_case! {
fn full_game_load_publishes_frame_mixins(env: &WowLuaEnv) {

    for mixin in REPRESENTATIVE_FRAME_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. The frame \
             mixins front the toplevel unit frames: PlayerFrame chassis \
             (with PlayerBottomManagedFrameContainer hosting AlternatePowerBar / PetFrame), \
             TargetFrame + FocusFrame (both share TargetFrameTemplate but FocusFrame is \
             clamped-to-screen), PetFrame (parent=PlayerFrame, mixes \
             PlayerBottomManagedFrameTemplate + SecureUnitButtonTemplate), Boss[1-5]TargetFrame \
             array container, EncounterBar (parent=UIParent owning the scenario power-bar \
             widgets — the parent of UIWidgetPowerBarContainerFrame from \
             Blizzard_UIWidgets), RuneFrame for DK runes"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_resource_mixins(env: &WowLuaEnv) {

    for mixin in REPRESENTATIVE_RESOURCE_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. The resource \
             mixin family backs each spec's secondary-resource bar: \
             Druid combo points, Rogue combo points, Warlock soul \
             shards, Mage arcane charges, Monk light/stagger/harmony, \
             Paladin holy power (via PaladinPowerBarFrame template), \
             Evoker ebon might, Death Knight runes, plus the generic \
             AlternatePowerBar (server-driven Encounter mechanic bars) \
             and PlayerPowerBarAlt for unit-power-bar-alt events. \
             TotemFrame + TotemButton power the Shaman totem indicator"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_compact_and_aura_mixins(env: &WowLuaEnv) {

    for mixin in REPRESENTATIVE_COMPACT_AND_AURA_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. \
             CompactPartyFrameMixin + CompactArenaFrameMixin (extends \
             CompactPartyFrameMixin) back the raid-style compact frames; \
             CompactBuff/Debuff extend CompactAuraTooltipMixin to share \
             aura tooltip plumbing; the PrivateAura family \
             (BasePrivateAuraBehavior / Container / IndividualAnchor) \
             handles the Shadowlands private-aura system per-unit; \
             ArenaPreMatchFramesContainer + PreMatchArenaUnitFrame + \
             StealthedArenaUnitFrame implement the arena-prep gates; \
             AnimatedHealthLossMixin + TempMaxHealthLossMixin + \
             TempMaxHealthLossDividerMixin power the absorb-loss / \
             temp-max-hp visual layer; \
             ResurrectableIndicatorMixin shows the rezz-pending icon"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_module_load_number_constants(env: &WowLuaEnv) {

    for (name, expected) in MODULE_LOAD_NUMBER_CONSTANTS {
        let value: f64 = env
            .eval(&format!("return {name}"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert!(
            (value - expected).abs() < 1e-9,
            "{name} must equal {expected} (got {value}). \
             MAX_PARTY_MEMBERS=4 (the canonical 5-player party = self \
             + 4 others); MAX_COMBO_POINTS=5 / MAX_TARGET_DEBUFFS=16 / \
             MAX_TARGET_BUFFS=32 are the historical pool sizes the \
             TargetFrame buff/debuff arrays size against; \
             MAX_BOSS_FRAMES=5 caps the Boss1-5 array; \
             COMBOFRAME_FADE_IN/OUT are the legacy combo-frame anim \
             durations; CUF_READY_CHECK_DECAY_TIME=11 is the seconds \
             until the ready-check icon fades after the answer window; \
             REQUIRED_REST_HOURS=5 is the offline-time threshold for \
             `Resting` state on PlayerFrame"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_registers_representative_virtual_templates(env: &WowLuaEnv) {
    let _ = env;

    for template in REPRESENTATIVE_VIRTUAL_TEMPLATES {
        let resolved = wow_ui_sim::xml::get_template(template);
        assert!(
            resolved.is_some(),
            "Virtual template `{template}` must be registered. \
             Blizzard_UnitFrame ships ~76 virtual templates total: the \
             CompactUnitFrameTemplate / CompactRaidGroupTemplate / \
             CompactPartyFrameTemplate trio (the raid/party UI is \
             rendered through pooled instances of these), the \
             ClassResourceBar* family for spec resources, the \
             Combo/Druid/Rogue/Mage/Monk/Paladin/Evoker per-spec \
             templates, the HealAbsorb/MyHealPrediction/\
             OtherHealPredictionBar segment templates that overlay the \
             health bar, plus the BossSpellBar + BossTargetFrame + \
             ArenaUnitFrame* family for instanced-PvE / PvP frames"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_named_top_level_frames(env: &WowLuaEnv) {

    for name in REPRESENTATIVE_NON_VIRTUAL_FRAMES {
        let frame_kind: String = env
            .eval(&format!("return type(_G[{name:?}])"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert!(
            frame_kind == "table" || frame_kind == "userdata",
            "Named top-level frame `{name}` must exist as a global \
             after Blizzard_UnitFrame loads (got type={frame_kind}). \
             Unit-frame instances span: PlayerFrame chassis (with \
             PlayerBottomManagedFrameContainer hosting PlayerPowerBarAlt + PetFrame); \
             TargetFrame + FocusFrame (both inherit TargetFrameTemplate, both \
             EditMode-managed); PetCastingBarFrame parent=UIParent (StatusBar widget for \
             the pet's spellcast); BossTargetFrameContainer + Boss[1-5]TargetFrame for \
             raid-encounter boss bars; EncounterBar (frameStrata=MEDIUM frameLevel=70 \
             VerticalLayoutFrame, owns scenario power-bar widgets — the parent referenced \
             by Blizzard_UIWidgets's UIWidgetPowerBarContainerFrame); RuneFrame for DK \
             runes; TotemFrame for Shaman; the per-spec resource bars \
             (PaladinPowerBarFrame, MonkHarmonyBarFrame, RogueComboPointBarFrame, \
             DruidComboPointBarFrame, EssencePlayerFrame for Evoker, InsanityBarFrame for \
             Shadow Priest, MageArcaneChargesFrame, WarlockPowerFrame); PartyFrame anchor; \
             PartyMemberBuffTooltip; PlayerBuffTimerManager"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_UnitFrame/"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Full game-screen load must emit zero Blizzard_UnitFrame body \
         errors. The 53 body files (~16868 lua+xml lines, the largest \
         addon analyzed so far) span PlayerFrame / TargetFrame / \
         FocusFrame / PetFrame / PartyMemberFrame / CompactUnitFrame \
         pools / CompactRaidGroup / CompactArenaFrame plus 14 per-spec \
         resource bars; any failure cascades into the entire visible \
         in-world UI. Found: {addon_specific:?}"
    );
}
}
