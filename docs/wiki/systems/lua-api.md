# Lua API

The Lua API layer bridges Lua addon code with the Rust simulation engine. It provides WoW-compatible globals, 300+ frame methods, C_* namespaces, and a timer system — all backed by `WowLuaEnv` and `SimState`.

## WowLuaEnv (`src/lua_api/env.rs`)

```rust
pub struct WowLuaEnv {
    pub(crate) lua: Lua,
    pub(crate) state: Rc<RefCell<SimState>>,
    on_update_errors: RefCell<HashSet<u64>>,
}
```

Initializes with full Lua stdlib, creates UIParent/WorldFrame via `create_builtin_frames`, then calls `register_globals`. Execution entry points: `exec()`, `exec_public()`, `exec_named()`, `exec_with_varargs()` (addon loading with `addonName, addonTable` varargs), `eval()`.

Enum initialization seeds known `Enum.*` values into existing child tables rather than replacing those tables. This matters after Blizzard Lua extends an enum during addon loading: later runtime-surface restoration reseeds required values without deleting Blizzard-added members such as CooldownViewer categories.

### Loader Environment Boundaries

`LoaderEnv::exec()` runs dynamic loader code in the currently loading addon's environment, including secure-environment and loading-scoped fenv state. `LoaderEnv::exec_public()` deliberately skips that loading fenv state while preserving the addon's secure file execution. Use it only for an addon-specific bridge that must publish a narrow compatibility surface to `_G`; it is not a generic secure-to-public mirroring mechanism. Secure Lua assignments and Rust secure frame exports are tracked in a secure-only publication ledger, so same-addon nil-symbol reconciliation checks the environment in which the name was read. The AuthChallenge workaround uses this boundary to restore five exported callbacks publicly while the `Blizzard_AuthChallengeUI` addon remains secure (`src/lua_api/loader_env.rs`, `src/lua_api/workarounds/temporary/auth_challenge_frame_parent.rs`).

Retail 12.1 removes public `_G.GetInventorySlotInfo`; current `Blizzard_TransmogShared` calls `C_PaperDollInfo.GetInventorySlotInfo` directly. No loader-scoped legacy-global compatibility remains. See [[transmog-inventory-slot-scope]] for the retired stale-source workaround.

## FrameHandle Userdata (`src/lua_api/frame/handle.rs`)

```rust
pub struct FrameHandle { pub id: u64, pub state: Rc<RefCell<SimState>> }
```

Links Lua userdata to the Rust `Frame` via `id`. `__newindex` syncs `parent.Child = frame` assignments into `parent_frame.children_keys`. `__index` resolves methods in priority order: mlua method table → children_keys → `__frame_fields[id]` → fallback stubs.

## Method Categories (14+ submodules)

| Module | Key methods |
|--------|-------------|
| `methods_core` | GetName, SetSize, GetRect, Show/Hide/IsVisible, SetAlpha, SetFrameStrata, EnableMouse, GetEffectiveScale |
| `methods_anchor` | SetPoint, ClearAllPoints, SetAllPoints, GetNumAnchors, GetPoint |
| `methods_event` | RegisterEvent, UnregisterEvent, RegisterAllEvents, IsEventRegistered |
| `methods_script` | SetScript, GetScript, HookScript, WrapScript, ClearScripts, HasScript |
| `methods_create` | CreateTexture, CreateFontString, CreateFrame, CreateAnimationGroup |
| `methods_texture` | SetTexture, SetAtlas, SetTexCoord, SetVertexColor, SetBlendMode, SetDrawLayer |
| `methods_text` | SetText, GetText, SetFont, SetTextColor, SetJustifyH/V, SetWordWrap |
| `methods_button` | SetNormalTexture, SetPushedTexture, GetFontString, SetButtonState, IsPressed |
| `methods_hierarchy` | GetParent, SetParent, GetChildren, GetNumChildren, GetRegions |
| `methods_meta` | `__index`, `__newindex`, `__len`, `__eq` |

Widget-specific: EditBox (SetMultiLine, SetAutoFocus), Slider (SetMinMaxValues, SetValue, SetOrientation), StatusBar (SetStatusBarColor), Cooldown (SetCooldown), Tooltip (SetOwner, AddLine, AddDoubleLine), MessageFrame (AddMessage), Browser (`NavigateTo`, `NavigateHome`). Browser navigation methods are callable no-result compatibility methods; the simulator does not open external content.

Model-family widgets (`Model`, `ModelScene`, `PlayerModel`, and related model frames) expose the Lua surface needed by Blizzard code, but 3D rendering is intentionally out of scope. Visual-only calls such as `ClearFog` are callable no-ops; modeled object state and actor methods remain separately documented where supported.

### Texture identity

For known WoW texture paths resolved by the bundled texture manifest, `Texture:GetTexture()` and `Texture:GetTextureFileID()` return the numeric fileDataID, while `Texture:GetTextureFilePath()` preserves the source path. Use `GetTextureFilePath()` when an assertion needs the authored path rather than the numeric texture identity. Current proofs include `Interface\\TargetingFrame\\UI-Classes-Circles` → `237669` and `Interface\\ICONS\\INV_Misc_QuestionMark` → `134400`.

## Global Functions

**Core overrides** — `print` appends to `SimState.console_output`; `ipairs` iterates frame children; `getmetatable` returns a fake metatable exposing all frame methods; `string.format` maps `%F` → `%f` for LuaJIT compatibility.

**Build options** — retail 12.1 `GetBuildOption("RestrictedAuraAPI")` returns `true`, selecting current forbidden aura templates; unknown options return `nil`.

**CreateFrame** — Parses type/name/parent/template, registers frame, links parent-child, inherits strata/level, creates widget type defaults, applies templates, returns FrameHandle.

**CreateWindow** — Returns a frame-backed `SimpleWindow` for Blizzard external-tool panels. The second argument initializes topmost state; `SetWindowSize`/`SetMinSize` enforce dimensions, `IsTopmost`/`SetTopmost` expose the modeled flag, and `Close` hides the frame. `SetTitle` and `SetFocus` are callable no-ops; popup-style, position, and focus persistence are not modeled. Owner frames use `SetWindow`/`GetWindow` and ordinary anchoring.

**Font system** — `CreateFont()`, standard fonts (GameFontNormal, ChatFontNormal, SystemFont_Small, etc.) stored as Lua tables with canonical `__fontPath`, `__fontHeight`, and `__fontFlags` keys. FontString snapshots prefer those canonical fields and retain legacy aliases (`__font`, `__height`, `__outline`) only as fallback, so XML `inherits="GameFontNormalLarge"` preserves the inherited size and flags.

**Versioned UI strings and enums** — `register_all_ui_strings()` combines generated global strings with static compatibility data. The retail 12.1 table registers 70 strings only under `profile-retail` + `retail-12-1-0`: 45 existing strings, 16 retail-12.1-gated Social UI compatibility labels from `cfec8576b`, seven ChatFrame combat-audio format strings from `43075580b`, and two ChatFrame combat-audio command names from `abcfb395b`. None of those compatibility-string groups was captured by the existing 49-candidate live probe. The Social UI labels keep `Blizzard_SocialUIShared` tag/presence tables populated; the seven ChatFrame strings support unconditional `:format(min, max)` calls, and the two command names are passed to `AddCommand` and normalized with `string.lower` during `TextToSpeechCommands.lua` preload. `cfec8576b` also publishes `Enum.SocialUIPresenceType` (`Unknown=0` through `AppearOffline=5`) and metadata (`0..5`, six values); `8f070a410` publishes `Enum.SocialUIBlockType` (`None=0`, `Ignore=1`, `BattleNetInviteBlock=2`) and metadata (`0..2`, three values). `9555a7649` publishes the generated `Enum.CooldownViewerSound` contract (`0..93`, 94 values) and metadata. `4d90beffd` publishes generated `Enum.RaidDispelOverlayType` (`Disabled=0`, `UseDebuffColor=1`, `UseBlack=2`) and metadata for `CompactUnitFrameOptions`; it does not model raid-dispel overlay rendering. `ec0e21537` publishes `Enum.RecentAlliesFriendTag` (`0..5`) with metadata and sparse `Enum.RolodexType.LegacyFriend=23`, preserving `21`/`22` gaps with metadata (`MinValue=0`, `MaxValue=23`, `NumValues=22`) for separate `Blizzard_RecentAllies` preload tables; Recent Allies service/search behavior remains unmodeled. `6eae179ff` publishes generated `Enum.CustomAuraButtonDispelTypeTextureStyle` (`Border=0` through `CustomAsset=4`) with metadata so `Blizzard_Deprecated/Deprecated_12_1_0.lua` can construct AuraButton border-style aliases when deprecation fallbacks load; texture and rendering behavior remain unmodeled. `e3806caba` publishes generated retail-12.1 `Enum.HousingResult` (`0..111`) with metadata (`MinValue=0`, `MaxValue=111`, `NumValues=112`) before the stale missing-enums fallback, allowing `Blizzard_HousingTemplates` to build `HousingResultToErrorText`; housing operations and result strings remain unmodeled, and the 12.0.0 contract is unchanged. `db678a399` publishes the current wowt-global-dataset `Enum.HouseSettingFlags` bitmask (`None=0` through `BlueprintExportParty=16384`) with metadata (`MinValue=0`, `MaxValue=16384`, `NumValues=16`) before that fallback; `Blizzard_HousingData` uses the four non-`Anyone` blueprint-export flags in `HousingAccessTypeStrings`. Housing access/export behavior and strings remain unmodeled, and the pre-12.1 contract is unchanged. `045c9fa19` publishes generated `Enum.HousingBlueprintType` (`None=0` through `Exterior=4`) with metadata (`MinValue=0`, `MaxValue=4`, `NumValues=5`) for current `Blizzard_HousingData` blueprint-type label tables. `432d357a1` publishes generated `Enum.HousingBlueprintUnmetRequirementFlags` (`InsufficientBudget=1` through `HouseSizeLocked=128`, eight members and no `None`) with metadata (`MinValue=1`, `MaxValue=128`, `NumValues=8`) for its blueprint-requirement text table. `665f5a31b` publishes generated `Enum.HousingBlueprintContentType` (`None=0` through `Other=6`) with metadata (`MinValue=0`, `MaxValue=6`, `NumValues=7`) for its final content label table; housing blueprint validation, operations, and strings remain unmodeled. The 12 probe-proven nil globals remain absent rather than receiving placeholders or aliases.

**Object pools** — `CreateFramePool`, `CreateFrameFactory` (multi-template), `CreateObjectPool` (generic acquire/release).

**Utilities** — `wipe()`, `tinsert/tremove()`, `CopyTable()`, `MergeTable()`, `Mixin()`, `CreateFromMixins()`, `getglobal/setglobal()`, `loadstring()`, `strsplit()`. Both `string.split(...)` and string `:split(...)` accept Blizzard's delimiter-receiver form, including empty and equal-length punctuation inputs, while retaining ordinary input-first behavior.

**Security** — `issecure()`, `securecall()`, `securecallfunction()`, `securecallmethod()`, `forceinsecure()`, `hooksecurefunc()` (from Elune or fallback stubs). The read-only `debug.isglobalindex()` query reports whether the active `_G.__index` invocation came from a syntactic global load; its VM-scoped provenance excludes explicit `_G` table reads and restores across nesting, errors, and coroutine swaps. With no modeled click-binding profile, `C_ClickBindings.GetBindingType()` returns `Enum.ClickBindingType.None` and `ExecuteBinding()` is inert, allowing secure-button `type` attributes to dispatch normally.

**Modeled legacy globals** — `ClearTarget()` clears the current target and returns `true` iff a target existed; it returns `false` when no target was set and preserves the `PLAYER_TARGET_CHANGED` event. `IsTimerunningEnabled()` and `GetRemainingTimerunningSeasonSeconds()` read the simulator's Timerunning season state; the countdown is zero when no season is active. `GetGuildTabardFiles()` is registered on retail as well as classic profiles and returns the modeled guild-tabard file tuple used by Blizzard's guild-bank UI. The temporary `Kiosk` namespace defaults `IsEnabled()` and `IsCompetitiveModeEnabled()` to `false` and `GetKioskLoginInfo()` to three `nil` values while preserving existing members.

**Guild panel toggle ownership** — `ToggleGuildFrame()` remains a simulator fallback on non-retail profiles. Retail leaves the global unpublished by the simulator so `Blizzard_Communities` supplies the canonical implementation after loading; a bare retail environment therefore has no `ToggleGuildFrame`, while reduced fixtures must load that addon before invoking it.

**Settings panel surface** — The post-load temporary workaround in `settings_surface_defaults.rs` reconciles the canonical `_G.SettingsPanel`/`Settings` pair when Blizzard replaces either surface. It adds only absent category IDs and `Settings.OpenToCategory`, preserving existing members; reapplying it is idempotent for the same panel identity. Retail 12.1 also pins the 13 housing Settings labels/tooltips required for the canonical Interface registrant to finish. `C_SettingsUtil.OpenSettingsPanel(categoryID, scrollTarget)` is modeled: it forwards to `SettingsPanelMixin.OpenToCategory(SettingsPanel, categoryID, scrollTarget)`, defaults an absent category ID to `0`, and is a no-op if either required surface is absent. `C_SettingsUtil.NotifySettingsLoaded` remains unsupported; category/element scrolling semantics beyond the supplied target remain unmodeled.

**Focused compatibility boundaries** — `EJ_SetLootFilter()` accepts integer Lua values and numeric strings locally; nil, invalid, and non-integral inputs become zero without changing global argument coercion. Chat startup assigns `DEFAULT_CHAT_FRAME` whenever `ChatFrame1` exists, while `ChatFrame1.editBox` remains conditional on `ChatFrame1EditBox`. Temporary chat-window state also backs `SetChatWindowName()` and `SetChatWindowDocked()` round-trips through `GetChatWindowInfo()`; these fields remain in the compatibility table until saved chat-layout state is modeled.

## C_* Namespaces

C_Timer (After, NewTimer, NewTicker), C_Map (stub), C_Item (`IsConsumableItem`, `IsEquippableItem`, and `IsItemInRange` are state-backed; other listed item methods remain mixed implemented/stubbed), C_Container (`GetBagSlotFlag`, `SetBagSlotFlag`, `GetBackpackAutosortDisabled`, and `GetBackpackSellJunkDisabled` share state-backed bag-slot flags), C_Spell (`GetSpellQueueWindow()` reads the current `SpellQueueWindow` CVar and returns its numeric value when parseable), C_System (GetLocale → "enUS"), C_EditMode (GetLayouts), C_CatalogShop (`GetVCProductInfos()` returns a fresh empty table), C_ChromieTime (retail/PTR empty-state queries and no-op actions), C_SettingsUtil (`OpenSettingsPanel` forwards to the current `SettingsPanelMixin.OpenToCategory` surface), C_StringUtil (`EscapeDecimalNonPrintables` preserves valid UTF-8 and replaces ASCII control bytes except tab/newline/carriage return, plus invalid UTF-8 bytes, with decimal escapes), C_Quest, C_AchievementInfo, C_ClassTalents, C_Guild, C_LFGList, C_Mail, C_ActionBar — most return nil/false/0 stubs.

**Guild management** — `C_GuildInfo.Invite(name)`, `Uninvite(name)`, `Promote(name)`, and `Leave()` share the state-backed handlers used by the deprecated `GuildInvite`, `GuildUninvite`, `GuildPromote`, and `GuildLeave` globals. They update the simulated guild roster/identity and emit the existing roster event behavior. Other `C_GuildInfo` management methods, including `Demote`, `Disband`, `SetLeader`, and `RemoveFromGuild`, remain unsupported because their distinct rank, leadership, disband, or GUID-backed semantics are not modeled.

**Profession specialization boundary** — The temporary `C_ProfSpecs` fixture reports `SkillLineHasSpecialization(164) == true` for the modeled Blacksmithing skill line and `false` for unrelated skill line 165. It does not claim authoritative profession specialization-tree behavior: `GetChildrenForPath()` and the backing path/tree state remain unmodeled, so the full Professions specialization-panel integration test is explicitly ignored rather than supplied with guessed node data.

**Pet-battle breed quality boundary** — Temporary Lua-owned `C_PetBattles` sample state seeds the five displayed pets with `Enum.BattlePetBreedQuality.Rare` (`3`). `GetBreedQuality(owner, petIndex)` returns that numeric seed or `0` for an absent pet, letting current `PetBattleFrame` OnLoad render rarity without a nil value. This is not a pet-battle model: ownership, breeding, capture, combat outcomes, and live battle data remain unmodeled.

### `C_PlayerChoice` (PTR 12.1)

`C_PlayerChoice` is a state-backed deterministic compatibility model. `GetCurrentPlayerChoiceInfo()` returns nil with the default empty state or a documented `PlayerChoiceInfo` table with nested options, buttons, and currency/item/reputation rewards when `SimState.player_choice.current` is seeded. `GetNumRerolls()`, `GetRemainingTime()`, and `IsWaitingForPlayerChoiceResponse()` expose local query state. `SendPlayerChoiceResponse()`, `RequestRerollPlayerChoice()`, and `OnUIClosed()` record local mutator intent. This does not model retail timing, server validation, reroll economics, or live service values.

**Active voice channel type** — `C_VoiceChat.GetActiveChannelType()` reads `SimState.voice_chat.active_channel_id`, finds the matching channel, and returns its numeric `ChatChannelType`. It returns one `nil` result when no channel is active or the active ID is stale; it does not synthesize a channel type. This documents only the state-backed query, not broader voice-service authentication, push-to-talk, or transcription behavior.

**Spell queue window** — `C_Spell.GetSpellQueueWindow()` reads the simulator's current `SpellQueueWindow` CVar, so `SetCVar("SpellQueueWindow", value)` changes the returned numeric value. An unavailable or non-numeric CVar value returns `nil`; the deprecated `GetMaxSpellStartRecoveryOffset` and `GetSpellQueueWindow` globals reference the same C API function object.

**Spell descriptions** — `C_Spell.GetSpellDescription()` and `C_TooltipInfo.GetSpellByID()` both route through `src/spell_description_resolver.rs` before text reaches Lua or tooltip lines. The resolver expands Blizzard DB2-style tokens such as `$s1`, `$<damage>`, `$<dmg>`, `$<shield>`, `${...}` arithmetic, `$STR`, `$INT`, `$AP`, `$MHP`, and simple conditional control tokens against `SimState` player stats and the simulator's spell-effect model. AP-scaled Paladin/Demon Hunter formulas are grounded in the local SimulationCraft dump (`~/Repos/simc/SpellDataDump/allspells.txt`): Avenger's Shield uses `1.55 * AP`, Crusader Strike `1.4 * AP`, Shield of the Righteous `0.95 * AP`, Eye Beam `$<dmg>` uses `10 * 0.4026 * AP`, and Shield of Vengeance `$<shield>` uses `30% max health * (1 + versatility damage)`. This keeps spellbook/tooltips from showing raw `$...` placeholders and prevents tooltip-only one-off replacements from drifting away from the C API surface.

## Timer System

`C_Timer.After(seconds, cb)`, `C_Timer.NewTicker(seconds, cb, iterations)` — backed by `WowLuaEnv.schedule_timer()`. `process_timers()` fires due callbacks each tick. `next_timer_delay()` drives the event loop scheduling.

## Animation System

`CreateAnimationGroup()` returns a group supporting `Play()`, `Stop()`, `Pause()`, `SetLooping()`, and `SetScript("OnFinished")`. Animation types: Alpha, Translation, Scale, Rotation, FlipBook, VertexColor, Path. `fire_on_update()` ticks animation groups after OnUpdate handlers.

## Sources

- [lua-api.md](../../lua-api.md) — WowLuaEnv, FrameHandle, method categories, globals, C_* namespaces, timers
- [spell_description_resolver.rs](../../../src/spell_description_resolver.rs) — shared spell-description token resolver
- [container_portrait_texture.rs](../../../src/lua_api/workarounds/temporary/container_portrait_texture.rs) — retail texture fileDataID proof
- [item_button_helper_defaults.rs](../../../src/lua_api/workarounds/temporary/item_button_helper_defaults.rs) — item-button texture fileDataID proof
- [settings_surface_defaults.rs](../../../src/lua_api/workarounds/temporary/settings_surface_defaults.rs) — additive-only Settings defaults and replacement-surface reconciliation
- [c_settings_util.rs](../../../src/c_api/c_settings_util.rs) — `C_SettingsUtil.OpenSettingsPanel` forwarding boundary
- [c_chromie_time.rs](../../../src/c_api/c_chromie_time.rs) — retail/PTR empty-state C_ChromieTime surface
- [c_container.rs](../../../src/c_api/item_spell/c_container.rs) — state-backed bag-slot flags and backpack queries
- [guild_info.rs](../../../src/lua_api/globals/guild_info.rs) — C_GuildInfo namespace registration and state-backed management methods
- [profession_specs_defaults.rs](../../../src/lua_api/workarounds/temporary/profession_specs_defaults.rs) — temporary seeded profession-specialization fixture and supported skill-line boundary
- [pet_battle_runtime_state.rs](../../../src/lua_api/workarounds/temporary/pet_battle_runtime_state.rs) — temporary seeded pet-battle breed-quality compatibility state
- [c_pet_battles_probes.rs](../../../tests/c_pet_battles_probes.rs) — seeded and absent-pet GetBreedQuality behavior proof
- [test_showuipanel_lod.rs](../../../tests/test_showuipanel_lod.rs) — explicit unsupported Professions path/tree integration boundary
- [guild_verbs.rs](../../../src/lua_api/globals/guild_verbs.rs) — shared guild management handlers
- [blizzard_deprecated_guild_script_loads.rs](../../../tests/blizzard_deprecated_guild_script_loads.rs) — deprecated-global/C_GuildInfo identity and behavior proof
- [voice_chat.rs](../../../src/lua_api/globals/missing_surface/voice_chat.rs) — state-backed active voice-channel queries
- [c_voice_chat_probes.rs](../../../tests/c_voice_chat_probes.rs) — active voice-channel type behavior proofs
- [loader_env.rs](../../../src/lua_api/loader_env.rs) — secure versus public dynamic loader execution
- [c_addons_runtime.rs](../../../src/c_api/c_addons_runtime.rs) — scoped `Blizzard_TransmogShared` loading and environment restoration
- [inventory_slot.rs](../../../src/lua_api/globals/inventory_slot.rs) — retained internal inventory-slot registration
- [blizzard_transmog_shared_loads.rs](../../../tests/blizzard_transmog_shared_loads.rs) — retail public-nil and TransmogUtil regression proof
- [secure_env.rs](../../../src/lua_api/globals/security/secure_env.rs) — isolated secure globals and secure publication tracking
- [precompiled.rs](../../../src/loader/precompiled.rs) — raw `_G.self` lifecycle receiver handling
- [environment_cleanup_restore.rs](../../../src/lua_api/workarounds/temporary/environment_cleanup_restore.rs) — raw `C_StoreSecure` restoration proof
- rilua commits `1a7c9de` and `3630419` — VM-scoped syntactic-global lookup provenance and `debug.isglobalindex()`
- [auth_challenge_frame_parent.rs](../../../src/lua_api/workarounds/temporary/auth_challenge_frame_parent.rs) — addon-specific AuthChallenge public export bridge
- [enums.rs](../../../src/lua_api/env_init/enums.rs) — enum-table reseeding that preserves existing Blizzard extensions
- [strings/mod.rs](../../../src/lua_api/globals/strings/mod.rs) — generated and versioned UI-string registration
- [more_strings.rs](../../../src/lua_api/globals/strings/string_data/more_strings.rs) — live retail 12.1 string data table
- [register.rs](../../../src/lua_api/globals/register.rs) — exact live GlobalStrings contract test
- [timerunning.rs](../../../src/lua_api/globals/real/timerunning.rs) — state-backed legacy Timerunning globals
- [bank_storage_verbs.rs](../../../src/lua_api/globals/bank_storage_verbs.rs) — retail guild-tabard lookup registration
- [c_string_util_decimal.rs](../../../src/c_api/c_string_util_decimal.rs) — decimal escaping for control and invalid UTF-8 bytes
- [font_strings.rs](../../../src/lua_api/frame/methods/button_anchor_hierarchy/font_strings.rs) — canonical Font object field precedence and FontString snapshots
- [chat_window_defaults.rs](../../../src/lua_api/workarounds/temporary/chat_window_defaults.rs) — temporary chat-window name/docking state and public round-trip defaults
- [compat_overrides.rs](../../../src/lua_api/globals/compat_overrides.rs) — table-form `string.split` compatibility
- [formatting_utility_defaults.rs](../../../src/lua_api/workarounds/temporary/formatting_utility_defaults.rs) — string-metatable `:split` compatibility
- [click_bindings_defaults.rs](../../../src/lua_api/workarounds/temporary/click_bindings_defaults.rs) — no-profile click-binding behavior
- [loot.rs](../../../src/lua_api/globals/missing_surface/encounter_journal/loot.rs) — local Encounter Journal filter coercion
- [chat_init.rs](../../../src/lua_api/chat_init.rs) — default chat-frame initialization
- [browser.rs](../../../src/lua_api/frame/methods/widgets/browser.rs) — no-result Browser navigation methods
- [model.rs](../../../src/lua_api/frame/methods/widgets/model.rs) — model-family Lua surface and intentional 3D visual no-ops
- [widget_methods_model.rs](../../../tests/widget_methods_model.rs) — `ClearFog` publication and no-op behavior proof
- [simple_window.rs](../../../src/lua_api/globals/create_frame/simple_window.rs) — frame-backed `CreateWindow` compatibility contract
- [render_layers.rs](../../../src/lua_api/frame/methods/misc/render_layers.rs) — `SetWindow`/`GetWindow` owner attachment
- [kiosk_namespace_defaults.rs](../../../src/lua_api/workarounds/temporary/kiosk_namespace_defaults.rs) — inert Kiosk defaults
- [targeting_verbs.rs](../../../src/lua_api/globals/targeting_verbs.rs) — state-backed targeting globals, including `ClearTarget()`'s boolean result
- [panel_toggle_verbs.rs](../../../src/lua_api/globals/panel_toggle_verbs.rs) — profile-scoped panel toggles, including retail `ToggleGuildFrame()` ownership
- [panel_toggle_verbs.rs tests](../../../tests/panel_toggle_verbs.rs) — bare-environment retail/non-retail registration proof
- [targeting_verbs.rs](../../../tests/targeting_verbs.rs) — targeting global behavior proofs
- [test_showuipanel_toggles.rs](../../../tests/test_showuipanel_toggles.rs) — retail `Blizzard_Communities` load and guild-panel toggle proof
- `/home/osso/Repos/simc/SpellDataDump/allspells.txt` — coefficient and variable formulas used for AP/health-scaled spell text

## See Also

- [[frame-data-flow]] — method lookup order, __index/__newindex, Mixin() application
- [[addon-loading]] — addon execution, idempotent loaded-addon handling, and environment boundaries
- [[taint-system]] — secure/public environments and secure-button publication boundaries
- [[post-load-workaround-audit]] — explicit post-cleanup restoration hooks
- [[event-system]] — fire_event, SetScript, OnUpdate tick mechanism
- [[widget-system]] — Frame struct backing each FrameHandle
- [[texture-atlas]] — texture path resolution, atlas identity, and rendering consumers
