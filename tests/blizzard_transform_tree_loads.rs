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

fn transform_tree_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_TransformTree")
}

fn transform_tree_toc() -> PathBuf {
    transform_tree_dir().join("Blizzard_TransformTree.toc")
}

const ALL_FOUR_SCREENS: &[ScreenKind] = &[
    ScreenKind::Game,
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const PUBLISHED_MIXINS: &[(&str, usize)] = &[
    ("TransformTreeBaseNodeMixin", 25),
    ("TransformTreeFrameNodeMixin", 25),
    ("TransformTreeTextureNodeMixin", 25),
    ("TransformTreeMixin", 5),
];

const PUBLISHED_GLOBAL_FUNCTIONS: &[&str] = &[
    "CreateTransformTreeNode",
    "CreateTransformTreeNodeFromWidget",
    "TransformTreeFrameNode_Reset",
    "CreateTransformFrameNodePool",
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

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
retail_ptr_forever_toc_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&transform_tree_dir()).expect("TransformTree TOC resolves");
    assert_eq!(
        resolved,
        transform_tree_toc(),
        "Bare TOC — no flavor suffix; the transform-tree library is a \
         flavor-agnostic LoD utility resolved via the bare-TOC path in \
         find_toc_file at src/loader/mod.rs:65-95"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_is_load_on_demand_with_no_dependencies() {
    let toc = TocFile::from_file(&transform_tree_toc()).expect("TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "`## LoadOnDemand: 1` — TransformTree is a generic transform-\
         hierarchy library that loads only when a consumer addon \
         declares it as a hard dep. Today the lone consumer is \
         Blizzard_AzeriteUI (itself LoD), which uses transform nodes \
         to position azerite-power buttons inside the empowered-item \
         clip frame"
    );
    assert!(
        toc.dependencies().is_empty(),
        "No `## Dependencies:` directive — TransformTree depends only \
         on always-loaded SharedXML utilities (CreateFromMixins, Mixin, \
         CreateVector2D, AreVector2DEqual, CreateSecureFramePool, \
         CallErrorHandler, xpcall). Got: {:?}",
        toc.dependencies()
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        !toc.is_game_type_restricted(),
        "AllowLoadGameType absent → not restricted (false)"
    );
    assert!(
        toc.default_enabled(),
        "`## DefaultState: enabled` — explicit opt-in default"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn allow_load_both_permits_every_screen() {
    let toc = TocFile::from_file(&transform_tree_toc()).expect("TOC parses");

    for screen in ALL_FOUR_SCREENS {
        assert!(
            toc.allows_screen(*screen),
            "`## AllowLoad: Both` → toc.rs:305-313 returns true for \
             {screen:?}. The library is a pure-Lua transform-math \
             helper with no UI surface, so it is safe on glue and game \
             alike — Blizzard tags it Both so that future glue-screen \
             consumers (e.g. character preview) could pick it up \
             without TOC churn"
        );
    }
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_raw_bytes_pin_five_directives_and_five_lua_files() {
    let raw = std::fs::read_to_string(transform_tree_toc()).expect("TOC reads utf-8");

    let expected_lines = [
        "## Title: Blizzard_TransformTree",
        "## Author: Blizzard Entertainment",
        "## DefaultState: enabled",
        "## LoadOnDemand: 1",
        "## AllowLoad: Both",
        "TransformTreeBaseNodeMixin.lua",
        "TransformTreePools.lua",
        "TransformTreeTextureNodeMixin.lua",
        "TransformTreeFrameNodeMixin.lua",
        "TransformTreeMixin.lua",
    ];

    for line in expected_lines {
        assert!(
            raw.contains(line),
            "Raw TOC must pin `{line}` — 5 metadata directives + 5 \
             pure-lua body files (no XML — the library has no UI \
             templates, just transform math and frame-pool helpers)"
        );
    }

    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("## Dependencies"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains(".xml"));
}
}

post_wrath_ui_fixture! {
#[test]
fn body_lists_five_lua_files_in_dependency_order() {
    let toc = TocFile::from_file(&transform_tree_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert_eq!(
        body,
        vec![
            "TransformTreeBaseNodeMixin.lua".to_string(),
            "TransformTreePools.lua".to_string(),
            "TransformTreeTextureNodeMixin.lua".to_string(),
            "TransformTreeFrameNodeMixin.lua".to_string(),
            "TransformTreeMixin.lua".to_string(),
        ],
        "Body must be exactly 5 entries in this order — Base first \
         (publishes TransformTreeBaseNodeMixin + the 2 \
         CreateTransformTreeNode constructors used by Pools), then \
         Pools (which calls CreateSecureFramePool with a PostCreate \
         hook), then Texture/Frame node specializations (each \
         CreateFromMixins(TransformTreeBaseNodeMixin)), and finally \
         TransformTreeMixin (whose OnLoad calls \
         CreateTransformTreeNode(TransformTreeBaseNodeMixin) so the \
         base mixin must already exist). Got: {body:?}"
    );
}
}

#[test]
fn absent_from_every_screen_eager_discovery() {
    for screen in ALL_FOUR_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_TransformTree");
        assert!(
            !found,
            "Blizzard_TransformTree must be absent from {screen:?} \
             eager discovery — `## LoadOnDemand: 1` excludes LoD \
             addons from the eager sweep, even with `AllowLoad: Both`"
        );
    }
}

post_wrath_ui_fixture! {
#[test]
fn azerite_ui_declares_transform_tree_as_hard_dependency() {
    let azerite_toc = blizzard_ui_dir()
        .join("Blizzard_AzeriteUI")
        .join("Blizzard_AzeriteUI.toc");
    let toc = TocFile::from_file(&azerite_toc).expect("Blizzard_AzeriteUI TOC parses");

    assert!(
        toc.dependencies()
            .iter()
            .any(|d| d == "Blizzard_TransformTree"),
        "Blizzard_AzeriteUI's `## Dependencies: Blizzard_TransformTree` \
         line is the SOLE consumer wiring — when AzeriteUI loads \
         (itself LoD) it pulls TransformTree first via \
         pull_required_lod_addons. AzeriteUI uses TransformTree to \
         position azerite-power buttons inside the empowered-item \
         clip frame (Blizzard_AzeriteEmpoweredItemUI.lua:26 \
         `self.transformTree = CreateFromMixins(TransformTreeMixin)` + \
         line 56 `CreateTransformFrameNodePool(\"BUTTON\", ..., \
         \"AzeriteEmpoweredItemPowerTemplate\", PowerReset)`). Got \
         deps: {:?}",
        toc.dependencies()
    );
    assert!(toc.is_load_on_demand());
}
}

post_wrath_ui_fixture! {
#[test]
fn no_other_addon_declares_transform_tree_as_dependency() {
    let entries = std::fs::read_dir(blizzard_ui_dir()).expect("BlizzardUI dir reads");
    let mut declarers: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let addon_dir = entry.path();
        if !addon_dir.is_dir() {
            continue;
        }
        let Some(toc_path) = find_toc_file(&addon_dir) else {
            continue;
        };
        let Ok(toc) = TocFile::from_file(&toc_path) else {
            continue;
        };
        let declared = toc
            .dependencies()
            .iter()
            .any(|d| d == "Blizzard_TransformTree")
            || toc
                .optional_deps()
                .iter()
                .any(|d| d == "Blizzard_TransformTree");
        if declared {
            let name = addon_dir.file_name().unwrap().to_string_lossy().to_string();
            declarers.push(name);
        }
    }

    assert_eq!(
        declarers,
        vec!["Blizzard_AzeriteUI".to_string()],
        "Exactly one Blizzard addon may declare Blizzard_TransformTree \
         as a hard dep — Blizzard_AzeriteUI. The library is otherwise \
         a forward-declared utility for future consumers. Got \
         declarers: {declarers:?}"
    );
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_four_mixins_with_inheritance(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &transform_tree_toc())
        .expect("Blizzard_TransformTree must load via Rust loader");

    for (mixin, expected_methods) in PUBLISHED_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(kind, "table", "{mixin} must be a table after LoD load");

        let count_probe = format!(
            "local n = 0 for k, v in pairs({mixin}) do if type(v) == 'function' then n = n + 1 end end return n"
        );
        let actual: i64 = env
            .eval(&count_probe)
            .unwrap_or_else(|err| panic!("{mixin} method count probe failed: {err}"));
        assert_eq!(
            actual, *expected_methods as i64,
            "{mixin} must publish {expected_methods} functions via \
             pairs(). TransformTreeBaseNodeMixin = 25: OnLoad, \
             SetParentTransform, GetParentTransform, Unlink, \
             CreateAndAddChild, CreateNodeFromTexture, \
             CreateNodeFromFrame, FindChildIndex, EnumerateChildren, \
             SetLocalScale, GetLocalScale, GetGlobalScale, \
             SetLocalRotation, GetLocalRotation, GetGlobalRotation, \
             SetLocalPosition, GetLocalPosition, GetGlobalPosition, \
             OnTransformResolved, RequiresPushedResolutions, \
             MarkDirty, SetParentTree, GetParentTree, \
             ResolveTransform, CheckResolvingError. FrameNode and \
             TextureNode = 25 each because \
             `CreateFromMixins(TransformTreeBaseNodeMixin)` shallow-\
             copies all 25 base keys and then the 2 own overrides \
             (OnTransformResolved + RequiresPushedResolutions) \
             OVERWRITE existing keys rather than adding new ones — \
             the count stays at 25. TransformTreeMixin = 5: OnLoad, \
             GetRoot, ResolveTransforms, AddDirtyTransform, \
             RemoveDirtyTransform. Got {actual}"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_global_constructor_and_pool_helpers(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &transform_tree_toc())
        .expect("Blizzard_TransformTree must load via Rust loader");

    for fn_name in PUBLISHED_GLOBAL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type({fn_name})"))
            .unwrap_or_else(|err| panic!("{fn_name} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "Global function `{fn_name}` must be defined after LoD \
             load. CreateTransformTreeNode (Base.lua:3) builds a fresh \
             node via CreateFromMixins; \
             CreateTransformTreeNodeFromWidget (Base.lua:9) Mixin's \
             into an existing widget so a Frame/Texture gains \
             transform-tree behavior; TransformTreeFrameNode_Reset \
             (Pools.lua:1) is the default reset func that unlinks + \
             clears + hides; CreateTransformFrameNodePool (Pools.lua:7) \
             wraps CreateSecureFramePool with a PostCreate hook that \
             attaches TransformTreeFrameNodeMixin to each pooled \
             frame. Got type={kind} for {fn_name}"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_no_named_top_level_frames(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &transform_tree_toc())
        .expect("Blizzard_TransformTree must load via Rust loader");

    let probe = "
        local count = 0
        for _, name in ipairs({'TransformTree','TransformTreeFrame','TransformTreeRoot'}) do
            if _G[name] ~= nil then count = count + 1 end
        end
        return count
    ";
    let count: i64 = env.eval(probe).expect("named-frame probe");
    assert_eq!(
        count, 0,
        "TransformTree publishes ZERO named top-level frames — the \
         body has 5 lua files and no XML, so there are no XML-declared \
         names. Consumers MUST instantiate the tree via \
         `CreateFromMixins(TransformTreeMixin)` and then call \
         `:OnLoad()` to wire the root node (see \
         Blizzard_AzeriteEmpoweredItemUI.lua:26). Got count={count}"
    );
}
}

prefork_full_ui_case! {
fn frame_node_inherits_base_node_methods_via_create_from_mixins(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &transform_tree_toc())
        .expect("Blizzard_TransformTree must load via Rust loader");

    let inherited_method: String = env
        .eval("return type(TransformTreeFrameNodeMixin.SetLocalPosition)")
        .expect("inherited method probe");
    assert_eq!(
        inherited_method, "function",
        "TransformTreeFrameNodeMixin.SetLocalPosition must be a \
         function — inherited from TransformTreeBaseNodeMixin via \
         `TransformTreeFrameNodeMixin = \
         CreateFromMixins(TransformTreeBaseNodeMixin)` at \
         FrameNode.lua:1. CreateFromMixins shallow-copies all base \
         keys, so the frame node specialization gains the full \
         transform-math API (Set/Get Local/Global Position/Rotation/\
         Scale, MarkDirty, ResolveTransform, etc.) without manual \
         redeclaration"
    );

    let override_method: String = env
        .eval("return type(TransformTreeFrameNodeMixin.OnTransformResolved)")
        .expect("override method probe");
    assert_eq!(
        override_method, "function",
        "TransformTreeFrameNodeMixin.OnTransformResolved must override \
         the no-op base impl — FrameNode.lua:3-13 ClearAllPoints + \
         SetPoint(\"CENTER\", parent, \"BOTTOMLEFT\", \
         globalPosition.x/scale, globalPosition.y/scale) + \
         SetScale(globalScale). Frames cannot be rotated directly so \
         rotation is intentionally ignored (note in source line 12)"
    );

    let requires_push: bool = env
        .eval("return TransformTreeFrameNodeMixin:RequiresPushedResolutions()")
        .expect("RequiresPushedResolutions probe");
    assert!(
        requires_push,
        "TransformTreeFrameNodeMixin:RequiresPushedResolutions must \
         return true — frame nodes link to external Frame state, so \
         the parent tree's dirty-set must actively push resolutions \
         to them on every ResolveTransforms tick \
         (FrameNode.lua:15-17 override). The base default is false \
         (Base.lua:131-135) since math-only nodes resolve lazily on \
         first GetGlobalX query"
    );
}
}

prefork_full_ui_case! {
fn explicit_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &transform_tree_toc())
        .expect("Blizzard_TransformTree must load via Rust loader");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_TransformTree") || e.contains("TransformTree"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Re-loading TransformTree over a fully-loaded game env must \
         emit zero addon-specific errors — load only publishes mixin \
         tables and 4 global helper functions; no frames are \
         instantiated, no events registered, no side effects beyond \
         table assignment. Found {}: {:#?}",
        addon_specific.len(),
        addon_specific
    );
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_TransformTree, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_transformtree_toc_is_absent_from_classic_caches() {
    let dir = transform_tree_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_TransformTree"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
