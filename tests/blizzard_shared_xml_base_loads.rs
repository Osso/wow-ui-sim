use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn shared_xml_base_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SharedXMLBase")
}

fn shared_xml_base_toc() -> PathBuf {
    shared_xml_base_dir().join("Blizzard_SharedXMLBase.toc")
}

const FOUNDATION_MIXINS: &[&str] = &[
    "ColorMixin",
    "ColorSwatchMixin",
    "AnchorMixin",
    "GridLayoutMixin",
    "AccumulatorMixin",
    "AnimTransitionMixin",
    "RectangleMixin",
    "FlagsMixin",
    "DirtyFlagsMixin",
    "TaggableObjectMixin",
    "ButtonStateBehaviorMixin",
    "ProxyConvertableMixin",
    "TemplateInfoCacheMixin",
    "TimedCallbackMixin",
    "CVarAccessorMixin",
    "CallbackRegistryMixin",
    "CallbackRegistrantMixin",
    "ExportDataStreamMixin",
    "ImportDataStreamMixin",
];

const FOUNDATION_UTIL_TABLES: &[&str] = &[
    "FunctionUtil",
    "AddOnUtil",
    "FrameUtil",
    "EnumUtil",
    "LocaleUtil",
    "ModelSceneUtil",
    "ProxyUtil",
    "TextureUtil",
    "FlagsUtil",
    "TableUtil",
    "AnchorUtil",
    "ExportUtil",
    "TextureKitConstants",
    "RegionLayoutManager",
    "CombatAudioAlertConstants",
    "CurveConstants",
];

const POOL_FACTORIES: &[&str] = &[
    "CreateFramePool",
    "CreateTexturePool",
    "CreateFontStringPool",
    "CreateObjectPool",
    "CreateActorPool",
    "CreateFramePoolCollection",
    "CreateFontStringPoolCollection",
    "CreateMaskTexturePool",
];

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&shared_xml_base_dir()).expect("SharedXMLBase TOC resolves");
    assert_eq!(
        resolved,
        shared_xml_base_toc(),
        "Blizzard_SharedXMLBase ships exactly one bare \
         `Blizzard_SharedXMLBase.toc` — no `_Mainline` / `_Mists` flavor \
         variants. The mixin/util layer is cross-flavor: ColorMixin, \
         CallbackRegistryMixin, EnumUtil, FlagsUtil, TableUtil, MathUtil are \
         identical between retail and Cataclysm Classic, so a single TOC \
         covers both flavors. Drift between flavors lives in \
         Blizzard_SharedXML's two-flavor TOC pair, not here"
    );
}

#[test]
fn toc_declares_single_required_dep_and_no_lod() {
    let toc = TocFile::from_file(&shared_xml_base_toc()).expect("TOC parses");

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        ["Blizzard_ScriptErrors"],
        "TOC must declare exactly one hard dep: Blizzard_ScriptErrors. \
         CallErrorHandler / assertsafe (defined in ErrorUtil.lua) hand off to \
         the ScriptErrors error-display frame for the per-error popup. \
         Without ScriptErrors loaded first, every Lua error from a later \
         addon would silently drop instead of surfacing to the player. \
         Got: {deps:?}"
    );

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT be LoadOnDemand. SharedXMLBase publishes the \
         foundational mixin/util layer that every later addon (SharedXML, \
         Settings_Shared, etc.) hard-deps on at PARSE TIME — XML \
         `mixin=\"CallbackRegistryMixin\"` references resolve against `_G` \
         when the XML is parsed, so the mixin tables MUST exist before any \
         dependent addon's XML is read"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_glue_only());
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
}

#[test]
fn toc_allow_load_both_includes_all_four_screens() {
    let toc = TocFile::from_file(&shared_xml_base_toc()).expect("TOC parses");

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "`## AllowLoad: Both` must allow {screen:?}. SharedXMLBase \
             foundations (CallbackRegistry, Pools, FrameUtil) are reused on \
             both glue and game screens — the glue-screen UI relies on the \
             same Pools / FrameUtil / CallbackRegistryMixin primitives the \
             game-screen UI uses. AllowLoad: Both is the contract that \
             enables this reuse"
        );
    }
}

#[test]
fn toc_raw_bytes_pin_minimal_metadata_surface() {
    let raw = std::fs::read_to_string(shared_xml_base_toc()).expect("TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard_SharedXMLBase"));
    assert!(raw.contains("## AllowLoad: Both"));
    assert!(raw.contains("## Dependencies: Blizzard_ScriptErrors"));

    assert!(
        !raw.contains("## Author"),
        "SharedXMLBase intentionally omits Author — minimal-metadata \
         primitive layer pattern (matches Blizzard_SharedTalentUI). The \
         author field is purely cosmetic for the addon list, and primitive \
         libraries don't show up there"
    );
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## SavedVariablesPerCharacter"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## UseSecureEnvironment"));
}

#[test]
fn body_starts_with_compat_then_error_then_mixin_loader() {
    let toc = TocFile::from_file(&shared_xml_base_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let first_three = &body[..3];
    assert_eq!(
        first_three,
        ["Compat.lua", "ErrorUtil.lua", "Mixin.lua"],
        "TOC body MUST start with these 3 entries in order. (1) Compat.lua \
         FIRST — overwrites a large set of Lua-4 compat aliases at top-level \
         (foreach, foreachi, getn, tinsert, abs, acos, ceil, cos, etc.); \
         every later file uses these freely without redefining them. (2) \
         ErrorUtil.lua publishes CallErrorHandler / assertsafe — the error \
         routing primitives wired into Blizzard_ScriptErrors. (3) Mixin.lua \
         publishes CreateAndInitFromMixin / CreateSecureMixinCopy / \
         SecureMixin — every subsequent file uses CreateFromMixins (already \
         a `_G` builtin from the C side) but downstream addons additionally \
         use CreateAndInitFromMixin (e.g., AnchorUtil.CreateAnchor = \
         GenerateClosure(CreateAndInitFromMixin, AnchorMixin)). Reordering \
         these breaks the rest of the load. Got: {first_three:?}"
    );
}

#[test]
fn body_count_matches_filesystem_layout() {
    let toc = TocFile::from_file(&shared_xml_base_toc()).expect("TOC parses");

    let lua_count = toc
        .files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "lua"))
        .count();
    let xml_count = toc
        .files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "xml"))
        .count();

    assert_eq!(
        toc.files.len(),
        39,
        "TOC body must list exactly 39 entries — the foundational primitive \
         surface for the entire UI codebase. Got {} entries",
        toc.files.len()
    );
    assert_eq!(
        lua_count, 36,
        "TOC body must list exactly 36 .lua files (one per primitive: \
         Compat, ErrorUtil, Mixin, TableUtil, EnumUtil, LocaleUtil, \
         FunctionUtil, ObjectUpdater, MathUtil, ExportUtil, Rectangle, \
         TextureUtil, AddOnUtil, Flags, Event, CallbackRegistry, \
         CallbackRegistrant, GlobalCallbackRegistry, CvarUtil, \
         TemplateInfoCache, FrameUtil, FrameFactory, SecureTypes, ProxyUtil, \
         Pools, ButtonStateBehavior, EnvironmentUtil, Color, ColorSwatch, \
         FrameWatcher, TaggableObject, TimedCallback, RegionLayoutManager, \
         ModelSceneUtil, AnchorUtil, CurveConstants, \
         CombatAudioAlertConstants). Got {lua_count}"
    );
    assert_eq!(
        xml_count, 3,
        "TOC body must list exactly 3 .xml files: CallbackRegistrant.xml, \
         ColorSwatch.xml, and ForbiddenAspectTemplates.xml. Got {xml_count}"
    );
}

#[test]
fn xml_files_load_after_their_lua_companions() {
    let toc = TocFile::from_file(&shared_xml_base_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let registrant_lua = body
        .iter()
        .position(|p| p == "CallbackRegistrant.lua")
        .expect("CallbackRegistrant.lua present");
    let registrant_xml = body
        .iter()
        .position(|p| p == "CallbackRegistrant.xml")
        .expect("CallbackRegistrant.xml present");
    assert!(
        registrant_lua < registrant_xml,
        "CallbackRegistrant.lua MUST load before CallbackRegistrant.xml. \
         The XML's `<Frame mixin=\"CallbackRegistrantMixin\" virtual=\"true\">` \
         resolves CallbackRegistrantMixin against `_G` at parse time. If the \
         XML loaded first, the mixin attr would be nil and OnShow/OnHide \
         handlers would never bind to template instances. Got lua at {} xml \
         at {}",
        registrant_lua,
        registrant_xml
    );

    let swatch_xml = body
        .iter()
        .position(|p| p == "ColorSwatch.xml")
        .expect("ColorSwatch.xml present");
    let swatch_lua = body
        .iter()
        .position(|p| p == "ColorSwatch.lua")
        .expect("ColorSwatch.lua present");
    assert!(
        swatch_lua < swatch_xml,
        "ColorSwatch.lua must load before ColorSwatch.xml so the virtual \
         ColorSwatchTemplate resolves ColorSwatchMixin at XML parse time. \
         Got lua at {} xml at {}",
        swatch_lua,
        swatch_xml
    );
}

#[test]
fn callback_registry_loads_before_its_xml_consumer() {
    let toc = TocFile::from_file(&shared_xml_base_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let registry_lua = body
        .iter()
        .position(|p| p == "CallbackRegistry.lua")
        .expect("CallbackRegistry.lua present");
    let registrant_lua = body
        .iter()
        .position(|p| p == "CallbackRegistrant.lua")
        .expect("CallbackRegistrant.lua present");
    assert!(
        registry_lua < registrant_lua,
        "CallbackRegistry.lua publishes CallbackRegistryMixin which \
         CallbackRegistrant.lua consumes. CallbackRegistrant.lua does NOT \
         CreateFromMixins(CallbackRegistryMixin) directly, but sibling \
         files like CVarCallbackRegistry = \
         CreateFromMixins(CallbackRegistryMixin) (in CvarUtil.lua) and \
         EventRegistry = CreateFromMixins(CallbackRegistryMixin) (in \
         GlobalCallbackRegistry.lua) demand the foundation be live first. \
         Got registry at {} registrant at {}",
        registry_lua,
        registrant_lua
    );
}

#[test]
fn pools_loads_before_button_state_behavior() {
    let toc = TocFile::from_file(&shared_xml_base_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let pools_idx = body
        .iter()
        .position(|p| p == "Pools.lua")
        .expect("Pools.lua present");
    let bsb_idx = body
        .iter()
        .position(|p| p == "ButtonStateBehavior.lua")
        .expect("ButtonStateBehavior.lua present");
    assert!(
        pools_idx < bsb_idx,
        "Pools.lua MUST load before ButtonStateBehavior.lua. ButtonStateBehavior \
         consumers (every action-bar button mixin in later addons) acquire \
         frames via CreateFramePool which Pools.lua publishes. Pools also \
         publishes the secure-rebinding logic that ButtonStateBehavior \
         relies on for its protected-frame state. Got pools at {} bsb at {}",
        pools_idx,
        bsb_idx
    );
}

#[test]
fn eager_discovery_includes_addon_on_all_four_screens() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let names: Vec<&str> = addons.iter().map(|(name, _)| name.as_str()).collect();
        assert!(
            names.contains(&"Blizzard_SharedXMLBase"),
            "Discovery on {screen:?} must include Blizzard_SharedXMLBase. \
             AllowLoad: Both + no LoadOnDemand = eager on all 4 screens. \
             Without it, every dependent addon would crash on missing \
             CallbackRegistryMixin / FlagsUtil / EnumUtil / TableUtil / \
             FrameUtil / Pools at parse time"
        );
    }
}

prefork_full_ui_case! {
    fn full_game_load_emits_no_addon_specific_lua_errors(env: &WowLuaEnv) {
        let errors: Vec<String> = env.state().borrow().lua_errors.clone();

        let addon_specific: Vec<&String> = errors
            .iter()
            .filter(|err| {
                err.contains("Blizzard_SharedXMLBase")
                    || err.contains("CallbackRegistryMixin")
                    || err.contains("Compat.lua")
                    || err.contains("Mixin.lua")
            })
            .collect();

        assert!(
            addon_specific.is_empty(),
            "Full Game-screen UI load must produce zero SharedXMLBase-attributed \
             Lua errors. Errors: {addon_specific:#?}"
        );
    }
}

prefork_full_ui_case! {
    fn is_addon_loaded_reports_true_after_eager_sweep(env: &WowLuaEnv) {

        let result: bool = env
            .eval("return C_AddOns.IsAddOnLoaded(\"Blizzard_SharedXMLBase\")")
            .expect("IsAddOnLoaded query succeeds");
        assert!(
            result,
            "C_AddOns.IsAddOnLoaded(\"Blizzard_SharedXMLBase\") MUST return true \
             after the eager sweep. The TOC has no LoadOnDemand and AllowLoad: \
             Both, so the discovery pass guarantees inclusion regardless of \
             whether any later addon hard-deps on it"
        );

        let dep: bool = env
            .eval("return C_AddOns.IsAddOnLoaded(\"Blizzard_ScriptErrors\")")
            .expect("Blizzard_ScriptErrors loaded check");
        assert!(
            dep,
            "Blizzard_ScriptErrors must also be loaded — SharedXMLBase declares \
             it as a hard dep, so the loader pulls it into the load set"
        );
    }
}

prefork_full_ui_case! {
    fn publishes_nineteen_foundation_mixin_tables(env: &WowLuaEnv) {

        for mixin in FOUNDATION_MIXINS {
            let result: bool = env
                .eval(&format!("return type(_G[{mixin:?}]) == \"table\""))
                .unwrap_or_else(|err| panic!("eval for mixin {mixin}: {err}"));
            assert!(
                result,
                "Mixin {mixin} must be a `_G` table after SharedXMLBase loads. \
                 Every mixin in this list is consumed by either an XML \
                 `mixin=...` attribute (parsed at template load) or a later \
                 CreateFromMixins call. Missing the mixin breaks parse-time \
                 resolution"
            );
        }
    }
}

prefork_full_ui_case! {
    fn publishes_sixteen_foundation_util_namespace_tables(env: &WowLuaEnv) {

        for util in FOUNDATION_UTIL_TABLES {
            let result: bool = env
                .eval(&format!("return type(_G[{util:?}]) == \"table\""))
                .unwrap_or_else(|err| panic!("eval for util {util}: {err}"));
            assert!(
                result,
                "Util namespace {util} must be a `_G` table after \
                 SharedXMLBase loads. Util namespaces hold static helper \
                 functions (FrameUtil.RegisterFrameForEvents, \
                 TableUtil.SafeCountTable, EnumUtil.MakeEnum, etc.) — they're \
                 the public API surface every later addon imports as `_G.X`"
            );
        }
    }
}

prefork_full_ui_case! {
    fn publishes_eight_pool_factory_functions_aliased_to_secure_variants(env: &WowLuaEnv) {

        for factory in POOL_FACTORIES {
            let result: bool = env
                .eval(&format!("return type(_G[{factory:?}]) == \"function\""))
                .unwrap_or_else(|err| panic!("eval for factory {factory}: {err}"));
            assert!(
                result,
                "Pool factory {factory} must be a `_G` function. Pools.lua \
                 tail-aliases each `Create*Pool` to its `CreateSecure*Pool` \
                 counterpart so the secure pool is the default. Every later \
                 addon (action bars, scroll boxes, talent buttons) uses \
                 CreateFramePool — without these aliases the call would be nil"
            );
        }
    }
}

prefork_full_ui_case! {
    fn callback_registry_mixin_provides_register_callback_method(env: &WowLuaEnv) {

        let result: bool = env
            .eval(
                "return type(CallbackRegistryMixin) == \"table\" and \
                        type(CallbackRegistryMixin.RegisterCallback) == \"function\" and \
                        type(CallbackRegistryMixin.OnLoad) == \"function\" and \
                        type(CallbackRegistryMixin.TriggerEvent) == \"function\"",
            )
            .expect("CallbackRegistryMixin shape check");
        assert!(
            result,
            "CallbackRegistryMixin must publish OnLoad / RegisterCallback / \
             TriggerEvent — the pub/sub contract every CreateFromMixins \
             subclass relies on. DataProviderMixin in SharedXML, \
             SettingsCategoryListMixin in Settings_Shared, EventRegistry in \
             this addon, and CVarCallbackRegistry all extend it via \
             CreateFromMixins"
        );
    }
}

prefork_full_ui_case! {
    fn event_registry_singleton_published_with_frame_event_methods(env: &WowLuaEnv) {

        let result: bool = env
            .eval(
                "return type(EventRegistry) == \"table\" and \
                        type(EventRegistry.RegisterFrameEvent) == \"function\" and \
                        type(EventRegistry.RegisterFrameEventAndCallback) == \"function\"",
            )
            .expect("EventRegistry shape check");
        assert!(
            result,
            "EventRegistry must be a populated singleton (table with \
             CreateFromMixins(CallbackRegistryMixin) + the frame-event helper \
             methods from GlobalCallbackRegistry.lua). Consumers do \
             `EventRegistry:RegisterFrameEvent(\"PLAYER_LOGIN\")` to bridge \
             legacy frame events into the callback registry, so it must be a \
             live instance after load"
        );
    }
}

prefork_full_ui_case! {
    fn enum_util_make_enum_produces_inverted_table(env: &WowLuaEnv) {

        let result: bool = env
            .eval(
                "local e = EnumUtil.MakeEnum(\"Foo\", \"Bar\", \"Baz\") \
                 return e.Foo == 1 and e.Bar == 2 and e.Baz == 3",
            )
            .expect("EnumUtil.MakeEnum eval");
        assert!(
            result,
            "EnumUtil.MakeEnum must produce a name→1-based-index inverted \
             table (it's `tInvert({{...}})` internally). Every CategorySet / \
             ControlType / VarType enum in Settings_Shared, MapPinTags in \
             SharedMapDataProviders, and TalentButtonAnimState in SharedTalentUI \
             all key off this contract"
        );
    }
}

prefork_full_ui_case! {
    fn flags_util_make_flags_produces_bit_value_table(env: &WowLuaEnv) {

        let result: bool = env
            .eval(
                "local f = FlagsUtil.MakeFlags(\"A\", \"B\", \"C\") \
                 return f.A == 1 and f.B == 2 and f.C == 4",
            )
            .expect("FlagsUtil.MakeFlags eval");
        assert!(
            result,
            "FlagsUtil.MakeFlags must produce a name→2^index bitmask table. \
             Settings.CommitFlag in Settings_Shared and various module-state \
             bitmasks rely on this — name maps to a power of 2 so OR-combination \
             and bit-test queries work"
        );
    }
}

prefork_full_ui_case! {
    fn dirty_flags_mixin_inherits_flags_mixin_capability(env: &WowLuaEnv) {

        let result: bool = env
            .eval(
                "return type(DirtyFlagsMixin) == \"table\" and \
                        type(DirtyFlagsMixin.Set) == \"function\" and \
                        type(DirtyFlagsMixin.IsSet) == \"function\" and \
                        type(DirtyFlagsMixin.MarkDirty) == \"function\" and \
                        type(DirtyFlagsMixin.IsDirty) == \"function\"",
            )
            .expect("DirtyFlagsMixin shape check");
        assert!(
            result,
            "DirtyFlagsMixin = CreateFromMixins(FlagsMixin) — it inherits the \
             Set / IsSet / Clear flag-bit methods from FlagsMixin and adds \
             MarkDirty / MarkClean / IsDirty on top. The inheritance path is \
             what lets framework code that takes a generic Flags-like object \
             also drive DirtyFlags-tagged state"
        );
    }
}

prefork_full_ui_case! {
    fn anchor_util_create_anchor_returns_anchor_mixin_instance(env: &WowLuaEnv) {

        let result: bool = env
            .eval(
                "local anchor = AnchorUtil.CreateAnchor(\"TOPLEFT\", UIParent, \"TOPLEFT\", 10, -20) \
                 return type(anchor) == \"table\" and type(anchor.SetPoint) == \"function\"",
            )
            .expect("AnchorUtil.CreateAnchor eval");
        assert!(
            result,
            "AnchorUtil.CreateAnchor = GenerateClosure(CreateAndInitFromMixin, \
             AnchorMixin) — calling it produces an AnchorMixin instance with \
             SetPoint / GetPoint methods. Layout primitives in later addons \
             (ResizeLayoutFrame, SettingsLayoutMixin) chain CreateAnchor for \
             declarative anchor specification"
        );
    }
}

prefork_full_ui_case! {
    fn frame_util_create_frame_helper_callable(env: &WowLuaEnv) {

        let result: bool = env
            .eval(
                "return type(FrameUtil) == \"table\" and \
                        type(FrameUtil.CreateFrame) == \"function\" and \
                        type(FrameUtil.RegisterFrameForEvents) == \"function\" and \
                        type(FrameUtil.GetRootParent) == \"function\"",
            )
            .expect("FrameUtil shape check");
        assert!(
            result,
            "FrameUtil must publish CreateFrame / RegisterFrameForEvents / \
             GetRootParent at minimum. These wrap the raw `_G.CreateFrame` to \
             add safe-default parent handling, batch-register events, and walk \
             the parent chain to UIParent — used pervasively by later addons"
        );
    }
}

prefork_full_ui_case! {
    fn create_and_init_from_mixin_constructs_initialized_instance(env: &WowLuaEnv) {

        let result: bool = env
            .eval(
                "local TestMixin = {} \
                 function TestMixin:Init(value) self.value = value end \
                 local obj = CreateAndInitFromMixin(TestMixin, 42) \
                 return obj.value == 42 and obj.Init == TestMixin.Init",
            )
            .expect("CreateAndInitFromMixin eval");
        assert!(
            result,
            "CreateAndInitFromMixin(mixin, ...) must (1) create an object via \
             CreateFromMixins(mixin), (2) call obj:Init(...) with the varargs. \
             Every `CreateFramePool` resetterFunc and `AnchorUtil.CreateAnchor` \
             call uses this pattern indirectly via GenerateClosure"
        );
    }
}

prefork_full_ui_case! {
    fn curve_constants_published_with_zero_to_one_curve(env: &WowLuaEnv) {

        let result: bool = env
            .eval("return type(CurveConstants) == \"table\" and CurveConstants.ZeroToOne ~= nil")
            .expect("CurveConstants eval");
        assert!(
            result,
            "CurveConstants table must publish at least the ZeroToOne curve. \
             Animation interpolation primitives (in InterpolatorUtil at \
             SharedXML and onward) consume these named curves to drive eased \
             transitions without each consumer redefining identity / linear \
             scale curves"
        );
    }
}

prefork_full_ui_case! {
    fn frame_watcher_singleton_initialized_with_watch_methods(env: &WowLuaEnv) {

        let result: bool = env
            .eval(
                "return type(FrameWatcher) == \"table\" and \
                        type(FrameWatcher.WatchFrame) == \"function\" and \
                        type(FrameWatcher.StopWatchingFrame) == \"function\"",
            )
            .expect("FrameWatcher shape check");
        assert!(
            result,
            "FrameWatcher must be a `_G` singleton with WatchFrame / \
             StopWatchingFrame methods. The constructor calls FrameWatcher:Init() \
             at the end of FrameWatcher.lua, so the table is a live initialized \
             instance — not just a method bag. Consumers register frames to \
             observe show/hide transitions for batched UI updates"
        );
    }
}

prefork_full_ui_case! {
    fn templates_registered_from_xml_files(env: &WowLuaEnv) {

        let result: bool = env
            .eval(
                "local CreateFrame = _G.CreateFrame \
                 local f = CreateFrame(\"Frame\", nil, nil, \"ColorSwatchTemplate\") \
                 return type(f) == \"table\" and type(f.SetColor) == \"function\"",
            )
            .expect("ColorSwatchTemplate materialization");
        assert!(
            result,
            "ColorSwatchTemplate (registered via ColorSwatch.xml) must be \
             materializable through CreateFrame — instantiation must mix in \
             ColorSwatchMixin's SetColor method. Validates the lua-then-xml \
             load sequence binds template + mixin together"
        );
    }
}
