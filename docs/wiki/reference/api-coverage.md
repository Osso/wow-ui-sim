# API Coverage

Current state of WoW API implementation across C_* namespaces, LE_* constants, and global functions.

## Coverage Summary

- **C_* namespaces**: ~97% coverage (~3700 of ~3807 methods used by BlizzardUI)
- **LE_* constants**: ~99% — only 8 missing, all legacy/Classic-era
- **Enum namespaces**: ~99% — 11 missing after scanner false-positive fix, mostly low-usage

## Implementation Layout

C_* support lives in `src/c_api/`, a peer of `src/lua_api/` per the C-API boundary policy. Each namespace is its own module (`c_spell.rs`, `c_spell_book.rs`, `c_map.rs`, `c_addons.rs`, `c_widget.rs`, `c_texture.rs`, `c_paper_doll_info.rs`, `c_addon_profiler.rs`, `c_configuration_warnings.rs`, `c_fog_of_war.rs`, `c_map_exploration_info.rs`, `c_spec.rs`, `c_wowtoken_secure.rs`, `c_xml_util.rs`, `item_spell/`, …).

Permanent C API shims are kept explicit, not hidden:

- `src/c_api/permanent_shims/` — intentionally unsupported domains with documented rationale (currently `c_model_info` for the 3D-model gap and `c_map_api` for legacy map glue).

Temporary compatibility defaults with a retirement path live under `src/lua_api/workarounds/temporary/`, even when they populate `C_*` namespace tables. This keeps unmodeled data shapes visible as simulator workarounds instead of pretending they are state-backed C API implementations.

Default behavior for missing methods (returning nil/false/0) is provided directly from the per-namespace modules, temporary Lua workarounds, and a small set of upgraded global stubs (~25 player/unit/system globals returning correct typed defaults). The previous monolithic `generated_stubs.rs` (~19K lines), `c_stubs_api*.rs` / `c_misc_api*.rs` files, and `src/c_api/temporary_shims/` module have been retired; ownership now lives in the per-namespace files, permanent-shim modules, or explicit Lua workaround modules.

## Well-Implemented Namespaces

| Namespace | Notes |
|-----------|-------|
| C_Timer | After, NewTicker, NewTimer — fully functional |
| C_Item | GetItemInfo, GetItemInfoInstant, item class/subclass lookups, modeled aggregate `GetItemCount`, and ItemLocation `GetStackCount` |
| C_Container | GetContainerItemID/Link/Info, HasContainerItem, bag-slot flags and backpack flag queries |
| C_Map | GetAreaInfo, GetMapInfo, GetWorldPosFromMapPos |
| C_QuestLog | GetNumQuestLogEntries, GetInfo, quest ID lookups |
| C_EditMode | GetLayouts, GetAccountSettings |
| C_Reputation | GetFactionDataByID/Index, GetNumFactions |
| C_ColorOverrides | GetColorForQuality, GetDefaultColorForQuality |
| C_TransmogCollection | PlayerHasTransmog variants |

## Missing APIs (107 methods total)

All missing methods are in niche or expansion-specific systems:

**Store/Token** (39 methods) — C_StoreSecure, C_WowTokenSecure — glue-screen login/purchase flows, not needed for in-game addon testing.

**Housing** (20 methods) — C_HousingCatalog, C_HouseExterior, C_HousingCustomizeMode, C_HousingDecor, C_HousingNeighborhood — new TWW expansion system.

**Delves/Scenarios** (12 methods) — C_DelvesUI, C_ScenarioInfo — expansion-specific content.

**PvP** (6 methods) — C_PvP world PvP area and lockout map queries.

**Miscellaneous** (30 methods) — guild MOTD, transmog outfit management, talent loadout switching, aura data provider management, 1-2 methods each across ~15 namespaces.

## Missing Namespaces (16 total)

Only 16 C_* namespaces from BlizzardUI are completely unregistered. All are either glue-screen (login/character creation) or removed systems:

- **Glue-screen**: C_CharacterCreation (194 refs), C_RealmList, C_StoreGlue, C_PaidServices, C_WowTokenGlue, C_ConfigurationWarnings, C_SocialContractGlue
- **Removed**: C_Reforge (pre-WoD), C_GlyphInfo (pre-Legion), C_ArtifactRelicForgeUI (Legion)
- **Other**: C_LiveEvent, C_MapInternal, C_Barbershop

## Stub Methodology

To implement a missing method, locate (or create) its namespace module under `src/c_api/`, add the method to the registry, and return values matching `Blizzard_APIDocumentationGenerated/*.lua`. State-backed implementations should read from / write to the simulator's state model rather than hardcoding constants.

Per CLAUDE.md, missing or wrong `C_*` behavior should default to **implementing the backing system or state model** — not adding a shim. Permanent C API shims belong in `permanent_shims/` only for intentional unsupported domains. Temporary unmodeled defaults belong in `src/lua_api/workarounds/temporary/` with a retirement path.

The signature audit in `docs/c-api-signature-audit.md` documents the exact parameter/return types for high-priority namespaces. Live gaps come from `wow-cli audit-api --gaps`, which can also emit a PLAN.md-ready checkbox list (`--format plan`).

## Sources

- [docs/c-api-signature-audit.md](../../c-api-signature-audit.md) — official API signatures (Patch 12.0.1)
- [C_Item.GetStackCount location counts](../../specs/c-item-stack-count.md) — bounded ItemLocation contract and focused proof
- [docs/c-api-stub-audit.md](../../c-api-stub-audit.md) — current implementation status per namespace

## See Also

- [[cli-commands]] — `audit-api` command for live gap reports
- [[architecture-overview]] — module layout and C-API boundary
