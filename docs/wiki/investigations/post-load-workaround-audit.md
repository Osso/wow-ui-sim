# Post-Load Workaround Audit

The retail startup workaround audit separates duplicate per-addon patches from
temporary compatibility shims that still represent missing simulator state or
lifecycle behavior.

## Content

### Retired duplicates

These hooks previously ran from `src/loader/addon.rs` even though
`patch_runtime_surface_for_addon_load` already applies the same per-addon
surface:

| Addon | Retired loader hook | Remaining source of behavior |
|-------|---------------------|------------------------------|
| `Blizzard_AccountStore` | `patch_account_store_set_storefront` | `patch_runtime_feature_addon_surfaces` for `Blizzard_AccountStore` |
| `Blizzard_MapCanvas` | `patch_map_canvas_scroll_container` | `patch_runtime_map_addon_surfaces` for `Blizzard_MapCanvas` |
| `Blizzard_FrameXMLUtil` | `patch_quest_objective_defaults` | `patch_runtime_core_addon_surfaces` for `Blizzard_FrameXMLUtil` |

The retirement criterion was exact duplicate call path, not visual symptom
masking. The remaining runtime hooks still isolate the temporary Lua surfaces in
`src/lua_api/workarounds/temporary/`.

### Retail Glue CharacterCreate fallbacks

**VERIFIED (Retail 12.1.5 cache and selector):** the original Glue-selected
CharacterCreate Lua/XML owns selected-state updates, overlay textures and alpha,
and Back/Forward labels. Retail no longer installs those fallback wrappers in
`character_create_defaults.rs`; other profiles retain them pending separate
consumer evidence. The test and XML `parentArray` root-cause fix are recorded in
[Lua-owned fallback retirement](lua-owned-fallback-retirement.md#retail-character-create-fallback-retirement-queue-462).
Authentication parenting, secure/public binding and Submit behavior are outside
this retirement.

### Retail R88y string.split alias collision Evidence: pr46-49-decomp-answers.md, sections R88y.

**VERIFIED (Retail 12.1.5.70688, R88y.y2):** `string.split` aliases the `strsplit` handler before secure-table construction. Evidence: pr46-49-decomp-answers.md, sections R88y.y2.

**VERIFIED (simulator RED at source `f3564d89a`):** after startup, ordinary `string.split` did not share identity with `strsplit`, while `__secureenv.string.split` did. The temporary formatting-default script wrote a `split` method into the string metatable's `__index` table after the secure copy. Evidence: pr46-49-decomp-answers.md, sections R88y.y2.

**VERIFIED (simulator source and Retail tests at `8f12a1a21`):** Retail skips this compatibility method, and both ordinary and secure aliases retain `strsplit` identity after startup and defaults reapplication. **VERIFIED (simulator source only):** other profiles still execute the existing compatibility path; no non-Retail runtime result is claimed. Evidence: pr46-49-decomp-answers.md, sections R88y.y2.

### Store Glue VAS fallback replacement

[R88q Store Glue state](../../specs/store-glue-r88q.md) replaces the five guessed
`store_glue_state.rs` getters/request functions on modeled profiles with explicit
empty host inputs and native-backed returning paths. Other profiles retain their
compatibility surface. Product/group and unrelated Store compatibility remain
outside this retirement. R88w also replaces the unconditional empty-table
`C_StoreSecure.GetVASErrors` override in `runtime_surface_bootstrap.lua` on modeled
profiles; its scoped legacy branch remains for other profiles. Shared-list copy,
error/state notification payloads, and bounded ASCII GUID parsing now follow the
native evidence. Unresolved GUID forms and service/lifecycle predicates remain
neutral instead of receiving guessed behavior.

### Remaining temporary hooks

These hooks are not duplicate call paths in the current loader. They stay as
temporary workarounds until the named simulator subsystem is modeled directly.

| Addon | Hook | Current rationale | Retirement path |
|-------|------|-------------------|-----------------|
| `Blizzard_EnvironmentCleanup` | `restore_post_cleanup_globals` | Blizzard cleanup nils globals that later addons still need in the simulator bootstrap; preserve-mode UI-string restoration fills only missing constants and strings, and guarded gamepad cursor defaults remain available for the real `ToggleGameMenu` → `CloseAllWindows` escape path, without overwriting Blizzard assignments. | Replace with a cleanup/global-publication model that matches Blizzard's preserved runtime surface. |
| `Blizzard_SharedXML` | `patch_callback_registry_defaults`, `patch_shared_xml_anim_mixins` | Partial addon loads and animation mixins need callback defaults and `SetPlaying` behavior before the simulator's callback/animation lifecycle fully matches retail. | Implement callback registry and animation group lifecycle semantics simulator-side. |
| `Blizzard_UIParent` | `patch_uiparent_managed_frame_mixin` | Managed-frame mixin methods expect UIParent-managed state that is not fully modeled. | Model UIParent managed-frame registration and layout semantics directly. |
| `Blizzard_GlueParent` | `patch_glueparent_uiparent_attributes` | GlueParent aliases UIParent in glue screens, losing attributes expected by shared UI code. | Model glue-screen parent aliases without clobbering UIParent-compatible attributes. |
| `Blizzard_SharedMapDataProviders` | `patch_unit_position_frame_mixin` | Map data-provider mixins need unit-position defaults against incomplete map/provider state. | Implement the map/provider backing state rather than repairing mixin methods after load. |
| `Blizzard_UIPanels_Game` | `patch_quest_log_mixin` | Quest log mixin defaults paper over missing quest model fields. | Replace with modeled quest-log state and objective defaults. |
| `Blizzard_ActionBar` | `patch_action_bar_button_event_fanout` | The real action-bar event frame fans events out to registered button frames; simulator does not model that setup path yet. | Model `ActionBarButtonEventsFrame` registration/fanout directly. |
| `Blizzard_PlayerSpells` | `patch_playerspells_onload_backfill` | PlayerSpells child frames can be used before Blizzard OnLoad initialized per-tab state. | Fix PlayerSpells load/lifecycle ordering so child frames initialize through Blizzard handlers. |
| `Blizzard_Dispatcher` | `patch_dispatcher_surface_for_addon_load` | Dispatcher bootstrap has a dedicated isolated surface and boundary tests. | Replace when the dispatcher model no longer needs Lua compatibility helpers. |
| `Blizzard_AchievementUI` | `patch_achievement_search_preview_for_addon_load` | Achievement search preview selection needs post-load wiring against current incomplete preview state. | Model the achievement search preview data/state directly. |

`restore_post_cleanup_globals` calls `restore_missing_ui_strings` after Blizzard cleanup. The preserve policy fills nil entries across the shared string, integer, float, and font-color tables; it restores required autocomplete priorities and combat-log raid-target constants while retaining Blizzard reassignment and table extensions. It also reapplies the guarded `CanAutoSetGamePadCursorControl` and `SetGamePadCursorControl` defaults, which GameMenu calls before the close-window stack. This is targeted runtime restoration, not generic global mirroring.

`chat_window_defaults.rs` remains a separate temporary compatibility surface: its existing `__wow_chat_window_state` table now stores chat-window names and docked flags for `SetChatWindowName()` / `SetChatWindowDocked()` and returns them through `GetChatWindowInfo()`. Retire this table when saved chat-layout state is modeled; no `SimState` persistence is claimed.

### Runtime-surface buckets

`patch_runtime_surface_for_addon_load` also runs broader buckets before the
loader's explicit match table. These are intentionally grouped by missing
runtime subsystem instead of by one visible addon symptom:

| Bucket | Examples | Current rationale | Retirement path |
|--------|----------|-------------------|-----------------|
| Core bootstrap surfaces | SharedXMLBase pool constructor sync, chat/voice button surface, paged content page text, PlayerSpells/PvP talent defaults | Keeps foundational Blizzard helpers available in secure/addon load contexts where the simulator bootstrap does not yet mirror Blizzard's publication order. | Move each helper into the backing runtime system and delete the Lua compatibility function once startup still passes without it. |
| Journal/load-on-demand surfaces | Collections journal namespace, Encounter Journal toggle, Adventure Map surface | Demand-loaded panels expect globals and toggle helpers before the simulator has a complete panel/panel-manager model. | Model the journal/panel registration paths directly. |
| Feature panel surfaces | Artifact item quality colors, Auction House aliases/events, Auth Challenge parent repair, Catalog Shop defaults, Damage Meter scroll extent | Feature panels expose missing C/API or frame initialization surfaces during partial and full startup. | Replace with modeled C/API data and frame initialization instead of post-load patching. |
| Map surfaces | fog-of-war pin methods, map exploration pin methods, data-provider attachment, MapCanvas scroll-container methods | Map canvas providers need method and attachment repairs until map/provider frames are modeled at the same lifecycle points as retail. | Implement provider attachment and pin behavior in simulator state/layout code. |

## Sources

- [loader/addon.rs](../../../src/loader/addon.rs) — per-addon post-load hook table
- [workarounds/mod.rs](../../../src/lua_api/workarounds/mod.rs) — runtime addon-surface dispatch and bootstrap hooks
- [workarounds/temporary](../../../src/lua_api/workarounds/temporary) — isolated temporary workaround implementations and tests
- [formatting_utility_defaults.rs](../../../src/lua_api/workarounds/temporary/formatting_utility_defaults.rs) — profile-scoped colon-split compatibility and Retail alias preservation
- [secure_env.rs](../../../src/lua_api/globals/security/secure_env.rs) — secure-library copy and string alias identity tests
- [env_init/mod.rs](../../../src/lua_api/env_init/mod.rs) — library alias initialization before secure-table construction
- [strings/mod.rs](../../../src/lua_api/globals/strings/mod.rs) — replace versus preserve UI-string registration policies
- [environment_cleanup_restore.rs](../../../src/lua_api/workarounds/temporary/environment_cleanup_restore.rs) — post-cleanup restore ordering and regression coverage
- [gamepad_cursor_control_defaults.rs](../../../src/lua_api/workarounds/temporary/gamepad_cursor_control_defaults.rs) — guarded gamepad cursor-control defaults restored after cleanup
- [chat_window_defaults.rs](../../../src/lua_api/workarounds/temporary/chat_window_defaults.rs) — temporary chat-window name/docking state and round-trip defaults
- [collections_escape.rs](../../../tests/collections_escape.rs) — real Blizzard Collections Escape close-stack coverage
- [recent_runtime_bootstrap_boundaries.rs](../../../tests/recent_runtime_bootstrap_boundaries.rs) — dispatcher bootstrap boundary coverage
- [achievement_search_bootstrap_boundaries.rs](../../../tests/achievement_search_bootstrap_boundaries.rs) — achievement search bootstrap boundary coverage

## See Also

- [[lua-api]] — Lua-facing compatibility surfaces restored during startup
- [[taint-system]] — secure/public publication boundaries used by Blizzard startup
- [[addon-startup-settings-and-item-load]] — related startup shim boundary examples
- [[store-secure-pool-constructors]] — runtime-surface sync pattern used for Store startup fixes
- [[playerspells-runtime-load]] — PlayerSpells lifecycle/load background
