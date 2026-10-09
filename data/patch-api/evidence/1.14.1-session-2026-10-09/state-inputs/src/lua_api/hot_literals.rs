//! Pre-intern whitelist for hot Lua string literals used during Blizzard
//! UI startup.
//!
//! # What lives here
//!
//! A static, hand-curated list of byte-slices that the VM sees many times
//! during `--no-addons --no-saved-vars` startup. These are candidates for
//! `Gc::intern_string_static(&'static [u8])` so the pointer-keyed intern
//! cache short-circuits them on every lookup.
//!
//! The list is organized by semantic category (globals, namespaces, method
//! / property keys, metatable keys, loader sentinels) so later sub-items
//! (registry module, hot-path conversions) can opt into categories
//! selectively.
//!
//! # What does NOT live here
//!
//! - The registry itself (sub-item 2 in Track 1).
//! - Any large consumer-specific hot-path logic — sub-items 3 convert the
//!   hot paths to consume the registry. This file stays mostly data plus
//!   a tiny registry accessor helper.
//! - Runtime-discovered literals. The spirit of the PLAN task is "static
//!   and versioned, not runtime-discovered", so additions to this list
//!   should be deliberate and reviewed.
//!
//! # Versioning
//!
//! Bump [`WHITELIST_VERSION`] whenever entries change in a way that could
//! invalidate downstream invariants (e.g. the Track 3 slotted-global
//! fast-path is keyed by index into this list). Adding new entries at the
//! end is a soft bump; reordering or removing entries requires a hard bump
//! and invalidates any on-disk bytecode keyed on the old version.
//!
//! # Source of truth
//!
//! Entries originate from grepping the existing rilua wow-ui-sim codebase
//! for repeated string literals in startup paths (namespace names,
//! frame-method identifiers, metatable keys, chunk tags) plus first-party
//! knowledge of the Blizzard addon API surface. The `wow_ui_sim` runtime
//! will compare its own `intern_string` traffic against this list in a
//! measurement step (Track 1 sub-item 4).

/// Increment on any change that could invalidate downstream ABI (e.g.
/// Track 3's slotted global vector is indexed off [`HOT_GLOBALS`]).
pub const WHITELIST_VERSION: u32 = 1;

// ── Global symbols ─────────────────────────────────────────────────────────
//
// Bare globals the Blizzard UI reads many times during startup. Most enter
// via `SETGLOBAL` at addon-load time and are then read every frame by the
// layout / event / script dispatch paths.

pub const HOT_GLOBALS: &[&[u8]] = &[
    b"_G",
    b"UIParent",
    b"WorldFrame",
    b"GameTooltip",
    b"UIErrorsFrame",
    b"ChatFrame1",
    b"MainMenuBar",
    b"MainActionBar",
    b"PlayerFrame",
    b"TargetFrame",
    b"MinimapCluster",
    b"Minimap",
    b"SettingsPanel",
    b"PlayerSpellsFrame",
    b"QuestFrame",
    b"GossipFrame",
    b"MerchantFrame",
    b"CharacterFrame",
    b"FriendsFrame",
    b"LFGListFrame",
    b"Constants",
    // Blizzard "util" globals that load early and are called from many
    // addons' OnLoad handlers.
    b"Mixin",
    b"CreateFromMixins",
    b"CreateFrame",
    b"CopyTable",
    b"Clamp",
    b"SetParentFrameLevel",
    b"GetTime",
    b"GetCVar",
    b"SetCVar",
    b"SecureHandlerExecute",
    b"securecall",
    b"issecure",
    b"issecurevariable",
    b"hooksecurefunc",
];

// ── Namespace tables ───────────────────────────────────────────────────────
//
// All C_* namespaces plus Enum/Constants. These appear both as global
// lookups (`C_Foo.Bar(...)`) and as table-key interns during the
// registration phase. Full set is 83 entries (grepped from
// `grep -rEo '"(C_[A-Za-z]+)"' src/lua_api/`); listed exhaustively because
// Track 3 wants a stable slot per namespace.

pub const HOT_NAMESPACES: &[&[u8]] = &[
    b"Enum",
    b"Constants",
    // C_* namespaces, alphabetized. Additions here are an ABI bump candidate.
    b"C_AchievementInfo",
    b"C_ActionBar",
    b"C_AddOns",
    b"C_AdventureMap",
    b"C_AreaPoiInfo",
    b"C_AuctionHouse",
    b"C_Bank",
    b"C_BattleNet",
    b"C_BehavioralMessaging",
    b"C_CVar",
    b"C_CampaignInfo",
    b"C_CharacterCreation",
    b"C_CharacterServices",
    b"C_ChatBubbles",
    b"C_ChatInfo",
    b"C_ChromieTime",
    b"C_ClassColor",
    b"C_ClassTrial",
    b"C_Club",
    b"C_ClubFinder",
    b"C_Commentator",
    b"C_Console",
    b"C_Container",
    b"C_ContributionCollector",
    b"C_Covenants",
    b"C_CreatureInfo",
    b"C_CurrencyInfo",
    b"C_DateAndTime",
    b"C_DeathRecap",
    b"C_DelvesUI",
    b"C_DurationUtil",
    b"C_EncounterJournal",
    b"C_EquipmentSet",
    b"C_EventUtils",
    b"C_FogOfWar",
    b"C_FriendList",
    b"C_GMTicketInfo",
    b"C_GameRules",
    b"C_Garrison",
    b"C_GossipInfo",
    b"C_GuildInfo",
    b"C_Heirloom",
    b"C_Housing",
    b"C_InvasionInfo",
    b"C_IslandsQueue",
    b"C_Item",
    b"C_ItemSocketInfo",
    b"C_ItemUpgrade",
    b"C_LFGInfo",
    b"C_LFGList",
    b"C_Loot",
    b"C_LootHistory",
    b"C_LootJournal",
    b"C_Mail",
    b"C_Map",
    b"C_MapExplorationInfo",
    b"C_MerchantFrame",
    b"C_ModelInfo",
    b"C_MountJournal",
    b"C_Navigation",
    b"C_NewItems",
    b"C_PaperDollInfo",
    b"C_PartyInfo",
    b"C_PartyPose",
    b"C_PetBattles",
    b"C_PetInfo",
    b"C_PetJournal",
    b"C_PhotoSharing",
    b"C_PlayerInfo",
    b"C_ProfSpecs",
    b"C_PvP",
    b"C_QuestLine",
    b"C_QuestLog",
    b"C_RaidLocks",
    b"C_RecruitAFriend",
    b"C_Reputation",
    b"C_ScenarioInfo",
    b"C_ScriptedAnimations",
    b"C_Seasons",
    b"C_SharedCharacterServices",
    b"C_Social",
    b"C_Soulbinds",
    b"C_Spell",
    b"C_SpellBook",
    b"C_StorePublic",
    b"C_SummonInfo",
    b"C_System",
    b"C_TaskQuest",
    b"C_Texture",
    b"C_Timer",
    b"C_TooltipInfo",
    b"C_ToyBox",
    b"C_TradeSkillUI",
    b"C_Transmog",
    b"C_Tutorial",
    b"C_UI",
    b"C_UIWidgetManager",
    b"C_UnitAuras",
    b"C_VideoOptions",
    b"C_VoiceChat",
    b"C_WowEntitlementInfo",
    b"C_WowTokenSecure",
    b"C_ZoneAbility",
];

// ── Frame method / property keys ──────────────────────────────────────────
//
// The most-called method names on FrameRef userdata during startup.
// Sourced from grep over `define_methods!` call sites plus the set of
// method names the XML loader synthesizes for setters (SetXxx) and
// getters (GetXxx) on every frame it instantiates.

pub const HOT_FRAME_METHODS: &[&[u8]] = &[
    // Identity / lifecycle.
    b"GetName",
    b"GetObjectType",
    b"IsObjectType",
    b"GetID",
    b"SetID",
    b"Hide",
    b"Show",
    b"IsShown",
    b"IsVisible",
    b"SetShown",
    // Layout / geometry.
    b"SetPoint",
    b"SetAllPoints",
    b"ClearAllPoints",
    b"GetPoint",
    b"GetNumPoints",
    b"SetWidth",
    b"SetHeight",
    b"GetWidth",
    b"GetHeight",
    b"SetSize",
    b"GetSize",
    b"GetRect",
    b"GetScaledRect",
    b"GetLeft",
    b"GetRight",
    b"GetTop",
    b"GetBottom",
    b"GetCenter",
    b"SetScale",
    b"GetScale",
    b"SetFrameStrata",
    b"GetFrameStrata",
    b"SetFrameLevel",
    b"GetFrameLevel",
    // Script dispatch.
    b"SetScript",
    b"GetScript",
    b"HookScript",
    b"RegisterEvent",
    b"UnregisterEvent",
    b"UnregisterAllEvents",
    b"IsEventRegistered",
    b"RegisterForDrag",
    b"RegisterForClicks",
    // Parent / children.
    b"GetParent",
    b"SetParent",
    b"GetChildren",
    b"GetNumChildren",
    b"GetRegions",
    b"GetNumRegions",
    // Attributes.
    b"SetAttribute",
    b"GetAttribute",
    // Visuals.
    b"SetAlpha",
    b"GetAlpha",
    FRAME_METHOD_SET_TEXT.as_bytes(),
    b"GetText",
    b"SetFont",
    b"SetFontObject",
    b"GetFontObject",
    b"SetTexture",
    b"GetTexture",
    b"SetAtlas",
    b"GetAtlas",
    b"SetTexCoord",
    b"SetVertexColor",
    b"SetColorTexture",
    b"SetDesaturated",
    b"SetDrawLayer",
    b"GetDrawLayer",
    b"CreateTexture",
    b"CreateFontString",
    b"CreateLine",
    b"CreateAnimationGroup",
];

// ── Lua metatable keys ────────────────────────────────────────────────────
//
// The 5.1 metamethod names plus Blizzard-convention `__*` registry keys
// the rilua runtime uses for its own bookkeeping.

// Named `&str` constants for metatable keys. [`HOT_METATABLE_KEYS`] below
// references these via `.as_bytes()` so the whitelist and call sites
// share one source of truth.

pub const METATABLE_INDEX: &str = "__index";
pub const METATABLE_NEWINDEX: &str = "__newindex";
pub const METATABLE_TOSTRING: &str = "__tostring";
pub const METATABLE_GC: &str = "__gc";
pub const METATABLE_EQ: &str = "__eq";
pub const METATABLE_LT: &str = "__lt";
pub const METATABLE_LE: &str = "__le";
pub const METATABLE_ADD: &str = "__add";
pub const METATABLE_SUB: &str = "__sub";
pub const METATABLE_MUL: &str = "__mul";
pub const METATABLE_DIV: &str = "__div";
pub const METATABLE_MOD: &str = "__mod";
pub const METATABLE_POW: &str = "__pow";
pub const METATABLE_UNM: &str = "__unm";
pub const METATABLE_CONCAT: &str = "__concat";
pub const METATABLE_LEN: &str = "__len";
pub const METATABLE_CALL: &str = "__call";
pub const METATABLE_METATABLE: &str = "__metatable";

// rilua / wow-ui-sim registry keys.
pub const REGISTRY_RILUA_FRAME_MT: &str = "__rilua_frame_mt";
pub const REGISTRY_RILUA_FRAME_REFS: &str = "__rilua_frame_refs";
pub const REGISTRY_SIM_PRINT: &str = "__sim_print";
pub const REGISTRY_SECUREENV: &str = "__secureenv";
pub const REGISTRY_CVARS: &str = "__cvars";
pub const REGISTRY_ORIGINAL_STRING_FORMAT: &str = "__original_string_format";

pub const HOT_METATABLE_KEYS: &[&[u8]] = &[
    METATABLE_INDEX.as_bytes(),
    METATABLE_NEWINDEX.as_bytes(),
    METATABLE_TOSTRING.as_bytes(),
    METATABLE_GC.as_bytes(),
    METATABLE_EQ.as_bytes(),
    METATABLE_LT.as_bytes(),
    METATABLE_LE.as_bytes(),
    METATABLE_ADD.as_bytes(),
    METATABLE_SUB.as_bytes(),
    METATABLE_MUL.as_bytes(),
    METATABLE_DIV.as_bytes(),
    METATABLE_MOD.as_bytes(),
    METATABLE_POW.as_bytes(),
    METATABLE_UNM.as_bytes(),
    METATABLE_CONCAT.as_bytes(),
    METATABLE_LEN.as_bytes(),
    METATABLE_CALL.as_bytes(),
    METATABLE_METATABLE.as_bytes(),
    // rilua / wow-ui-sim registry keys.
    REGISTRY_RILUA_FRAME_MT.as_bytes(),
    REGISTRY_RILUA_FRAME_REFS.as_bytes(),
    REGISTRY_SIM_PRINT.as_bytes(),
    REGISTRY_SECUREENV.as_bytes(),
    REGISTRY_CVARS.as_bytes(),
    REGISTRY_ORIGINAL_STRING_FORMAT.as_bytes(),
];

// ── Loader / compiler sentinels ───────────────────────────────────────────
//
// Chunk-name prefixes and registry tags the XML loader / template chain
// and secure-env bootstrap pass to `load_template` and friends. These are
// intern-cache hot because every template install synthesises one and
// compares against prior registrations.

// Individual `&str` constants for the chunk tags passed to `load_template`.
// Exposed as a single source of truth: [`HOT_LOADER_SENTINELS`] references
// these via `.as_bytes()`, and the template-chain builders import them by
// name so there's no risk of a call-site literal drifting from the
// whitelist entry.

pub const TEMPLATE_INLINE_FUNCTION_NOARGS: &str = "template-inline-function-noargs";
pub const TEMPLATE_INLINE_FUNCTION_SELF_ID: &str = "template-inline-function-self-id";
pub const TEMPLATE_INLINE_FUNCTION_EVENT_VARARGS: &str = "template-inline-function-event-varargs";
pub const TEMPLATE_INLINE_FUNCTION_BUTTON: &str = "template-inline-function-button";
pub const TEMPLATE_INLINE_FUNCTION_ELAPSED: &str = "template-inline-function-elapsed";
pub const TEMPLATE_INLINE_FUNCTION_SELF_STRING: &str = "template-inline-function-self-string";
pub const TEMPLATE_INLINE_FUNCTION_STRING_ARG: &str = "template-inline-function-string-arg";
pub const TEMPLATE_INLINE_FUNCTION_GLOBAL_ARG: &str = "template-inline-function-global-arg";
pub const TEMPLATE_INLINE_FUNCTION_TWO_GLOBAL_ARGS: &str =
    "template-inline-function-two-global-args";
pub const TEMPLATE_GLOBAL_METHOD_HANDLER: &str = "template-global-method-handler";

// Named `&str` constants for frame-method names that appear as
// hardcoded literals in the template-chain builders. Entries here mirror
// a position in [`HOT_FRAME_METHODS`] so the whitelist and call sites
// share one source of truth. Grow this list only as new builder-site
// consumers land — a frame-method name that is only ever passed through
// from parsed XML (dynamic `&str`) stays in `HOT_FRAME_METHODS` without
// a named constant.

pub const FRAME_METHOD_SET_TEXT: &str = "SetText";

pub const HOT_LOADER_SENTINELS: &[&[u8]] = &[
    b"getfenv",
    b"setfenv",
    b"MARK_SECURE_PROBE",
    b"from-secureenv",
    // Common `template-inline-*` chunk tags emitted by the template chain
    // builders. The full set is large and regenerated; the ten entries
    // below cover the highest-count handlers seen on `--no-addons` startup.
    TEMPLATE_INLINE_FUNCTION_NOARGS.as_bytes(),
    TEMPLATE_INLINE_FUNCTION_SELF_ID.as_bytes(),
    TEMPLATE_INLINE_FUNCTION_EVENT_VARARGS.as_bytes(),
    TEMPLATE_INLINE_FUNCTION_BUTTON.as_bytes(),
    TEMPLATE_INLINE_FUNCTION_ELAPSED.as_bytes(),
    TEMPLATE_INLINE_FUNCTION_SELF_STRING.as_bytes(),
    TEMPLATE_INLINE_FUNCTION_STRING_ARG.as_bytes(),
    TEMPLATE_INLINE_FUNCTION_GLOBAL_ARG.as_bytes(),
    TEMPLATE_INLINE_FUNCTION_TWO_GLOBAL_ARGS.as_bytes(),
    TEMPLATE_GLOBAL_METHOD_HANDLER.as_bytes(),
];

/// Total count of whitelisted literals, sum of all category slices.
pub const HOT_LITERAL_COUNT: usize = HOT_GLOBALS.len()
    + HOT_NAMESPACES.len()
    + HOT_FRAME_METHODS.len()
    + HOT_METATABLE_KEYS.len()
    + HOT_LOADER_SENTINELS.len();

// ── Registry (Track 1 sub-item 2) ────────────────────────────────────────

use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::string::LuaString;

/// Interned handles for every entry in every category.
///
/// Built once during VM bootstrap via [`HotLiteralRegistry::install`] and
/// stashed on [`crate::lua_api::env::WowLuaAppData`]. Each `GcRef<LuaString>`
/// is a pointer into rilua's string arena; rilua's `intern_string_static`
/// cache holds a parallel entry that keeps the string alive as a GC root,
/// so the handles here remain valid for the life of the VM.
///
/// Indexed accessors mirror position in the underlying [`HOT_GLOBALS`],
/// [`HOT_NAMESPACES`], etc. slices. Callers that want compile-time-checked
/// symbolic access can build enums over the category in a follow-up patch
/// (Track 1 sub-item 3).
#[derive(Clone)]
pub struct HotLiteralHandles {
    globals: Box<[GcRef<LuaString>]>,
    namespaces: Box<[GcRef<LuaString>]>,
    frame_methods: Box<[GcRef<LuaString>]>,
    metatable_keys: Box<[GcRef<LuaString>]>,
    loader_sentinels: Box<[GcRef<LuaString>]>,
}

impl HotLiteralHandles {
    /// Total number of handles stored, across all categories. Always equal
    /// to [`HOT_LITERAL_COUNT`] after a successful [`install`].
    pub fn len(&self) -> usize {
        self.globals.len()
            + self.namespaces.len()
            + self.frame_methods.len()
            + self.metatable_keys.len()
            + self.loader_sentinels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Indexed handle for [`HOT_GLOBALS`]. Panics on out-of-range index.
    pub fn global(&self, index: usize) -> GcRef<LuaString> {
        self.globals[index]
    }

    /// Indexed handle for [`HOT_NAMESPACES`]. Panics on out-of-range index.
    pub fn namespace(&self, index: usize) -> GcRef<LuaString> {
        self.namespaces[index]
    }

    /// Indexed handle for [`HOT_FRAME_METHODS`]. Panics on out-of-range index.
    pub fn frame_method(&self, index: usize) -> GcRef<LuaString> {
        self.frame_methods[index]
    }

    /// Indexed handle for [`HOT_METATABLE_KEYS`]. Panics on out-of-range index.
    pub fn metatable_key(&self, index: usize) -> GcRef<LuaString> {
        self.metatable_keys[index]
    }

    /// Indexed handle for [`HOT_LOADER_SENTINELS`]. Panics on out-of-range index.
    pub fn loader_sentinel(&self, index: usize) -> GcRef<LuaString> {
        self.loader_sentinels[index]
    }
}

/// Get a pre-interned metatable key handle if bootstrap already installed
/// the hot-literal registry, otherwise fall back to the static cache.
pub(crate) fn hot_metatable_key(state: &mut LuaState, index: usize) -> GcRef<LuaString> {
    let cached = state
        .app_data::<crate::lua_api::env::WowLuaAppData>()
        .and_then(|app| app.hot_literals.as_ref())
        .map(|handles| handles.metatable_key(index));
    if let Some(key) = cached {
        return key;
    }
    state.gc.intern_string_static(HOT_METATABLE_KEYS[index])
}

// ── Named accessors (Track 1 sub-item 3 foothold) ─────────────────────────
//
// Index constants for the entries that already have hot-path consumers in
// the current codebase. New conversions add entries here so the call sites
// stay symbolic (`handles.metatable_key(idx::RILUA_FRAME_MT)`) instead of
// relying on raw `intern_string_static(b"...")` calls.

/// Index constants into [`HOT_METATABLE_KEYS`]. Kept in lockstep with the
/// slice order — adding or reordering entries there requires updating
/// these and bumping [`WHITELIST_VERSION`].
pub mod metatable_idx {
    /// Position of `b"__index"` in [`super::HOT_METATABLE_KEYS`].
    pub const INDEX: usize = 0;
    /// Position of `b"__newindex"` in [`super::HOT_METATABLE_KEYS`].
    pub const NEWINDEX: usize = 1;
    /// Position of `b"__tostring"` in [`super::HOT_METATABLE_KEYS`].
    pub const TOSTRING: usize = 2;
    /// Position of `b"__gc"` in [`super::HOT_METATABLE_KEYS`].
    pub const GC: usize = 3;
    /// Position of `b"__eq"` in [`super::HOT_METATABLE_KEYS`].
    pub const EQ: usize = 4;
    /// Position of `b"__lt"` in [`super::HOT_METATABLE_KEYS`].
    pub const LT: usize = 5;
    /// Position of `b"__le"` in [`super::HOT_METATABLE_KEYS`].
    pub const LE: usize = 6;
    /// Position of `b"__add"` in [`super::HOT_METATABLE_KEYS`].
    pub const ADD: usize = 7;
    /// Position of `b"__sub"` in [`super::HOT_METATABLE_KEYS`].
    pub const SUB: usize = 8;
    /// Position of `b"__mul"` in [`super::HOT_METATABLE_KEYS`].
    pub const MUL: usize = 9;
    /// Position of `b"__div"` in [`super::HOT_METATABLE_KEYS`].
    pub const DIV: usize = 10;
    /// Position of `b"__mod"` in [`super::HOT_METATABLE_KEYS`].
    pub const MOD: usize = 11;
    /// Position of `b"__pow"` in [`super::HOT_METATABLE_KEYS`].
    pub const POW: usize = 12;
    /// Position of `b"__unm"` in [`super::HOT_METATABLE_KEYS`].
    pub const UNM: usize = 13;
    /// Position of `b"__concat"` in [`super::HOT_METATABLE_KEYS`].
    pub const CONCAT: usize = 14;
    /// Position of `b"__len"` in [`super::HOT_METATABLE_KEYS`].
    pub const LEN: usize = 15;
    /// Position of `b"__call"` in [`super::HOT_METATABLE_KEYS`].
    pub const CALL: usize = 16;
    /// Position of `b"__metatable"` in [`super::HOT_METATABLE_KEYS`].
    pub const METATABLE: usize = 17;
    /// Position of `b"__rilua_frame_mt"` in [`super::HOT_METATABLE_KEYS`].
    pub const RILUA_FRAME_MT: usize = 18;
    /// Position of `b"__rilua_frame_refs"` in [`super::HOT_METATABLE_KEYS`].
    pub const RILUA_FRAME_REFS: usize = 19;
    /// Position of `b"__sim_print"` in [`super::HOT_METATABLE_KEYS`].
    pub const SIM_PRINT: usize = 20;
    /// Position of `b"__secureenv"` in [`super::HOT_METATABLE_KEYS`].
    pub const SECUREENV: usize = 21;
    /// Position of `b"__cvars"` in [`super::HOT_METATABLE_KEYS`].
    pub const CVARS: usize = 22;
    /// Position of `b"__original_string_format"` in [`super::HOT_METATABLE_KEYS`].
    pub const ORIGINAL_STRING_FORMAT: usize = 23;
}

/// Index constants into [`HOT_FRAME_METHODS`]. Grows as builder call
/// sites hardcode a method name and opt into the named constant.
pub mod frame_method_idx {
    /// Position of `b"SetText"` in [`super::HOT_FRAME_METHODS`].
    pub const SET_TEXT: usize = 53;
}

/// Index constants into [`HOT_LOADER_SENTINELS`]. First four entries are
/// non-template helpers (`getfenv`, `setfenv`, `MARK_SECURE_PROBE`,
/// `from-secureenv`), so the template-inline-* indices start at 4.
pub mod loader_sentinel_idx {
    pub const TEMPLATE_INLINE_FUNCTION_NOARGS: usize = 4;
    pub const TEMPLATE_INLINE_FUNCTION_SELF_ID: usize = 5;
    pub const TEMPLATE_INLINE_FUNCTION_EVENT_VARARGS: usize = 6;
    pub const TEMPLATE_INLINE_FUNCTION_BUTTON: usize = 7;
    pub const TEMPLATE_INLINE_FUNCTION_ELAPSED: usize = 8;
    pub const TEMPLATE_INLINE_FUNCTION_SELF_STRING: usize = 9;
    pub const TEMPLATE_INLINE_FUNCTION_STRING_ARG: usize = 10;
    pub const TEMPLATE_INLINE_FUNCTION_GLOBAL_ARG: usize = 11;
    pub const TEMPLATE_INLINE_FUNCTION_TWO_GLOBAL_ARGS: usize = 12;
    pub const TEMPLATE_GLOBAL_METHOD_HANDLER: usize = 13;
}

/// Owns the pre-intern step during VM bootstrap. Call [`install`] once
/// before any addon load so the subsequent hot paths (sub-item 3) find
/// every whitelisted literal already in rilua's static intern cache.
pub struct HotLiteralRegistry;

impl HotLiteralRegistry {
    /// Pre-intern every entry in the whitelist via
    /// `state.gc.intern_string_static(&'static [u8])`. Returns a
    /// [`HotLiteralHandles`] populated with the resulting handles in the
    /// same order as each category slice.
    pub fn install(state: &mut LuaState) -> HotLiteralHandles {
        HotLiteralHandles {
            globals: Self::intern_all(state, HOT_GLOBALS),
            namespaces: Self::intern_all(state, HOT_NAMESPACES),
            frame_methods: Self::intern_all(state, HOT_FRAME_METHODS),
            metatable_keys: Self::intern_all(state, HOT_METATABLE_KEYS),
            loader_sentinels: Self::intern_all(state, HOT_LOADER_SENTINELS),
        }
    }

    fn intern_all(state: &mut LuaState, slice: &[&'static [u8]]) -> Box<[GcRef<LuaString>]> {
        slice
            .iter()
            .map(|entry| state.gc.intern_string_static(entry))
            .collect()
    }
}

#[cfg(test)]
#[path = "hot_literals_tests.rs"]
mod tests;
