# ReloadUI

`ReloadUI` records a deferred request. The owner retires the old Lua VM and UI registries, then repeats addon bootstrap against the retained simulator game state. The [contract](../../specs/reload-ui.md) owns requirements and proof status.

## Request and ownership

Both `C_UI.Reload` and legacy `ReloadUI` call the same Rust request producer. Blizzard SharedXML replaces the public global with a wrapper that calls `C_UI.Reload`; wiring only the legacy global does not cover real startup. Lua calls only set `SimState::ui_reload_requested`; repeated calls coalesce. App services requests outside Lua borrows, at update boundaries and between IPC commands. An Exec response captures its output and acknowledges the completed invoking chunk before expensive rebootstrap. IPC transport and queued commands remain process-owned.

Dump-tree, screenshot and lua-errors owners also service requests after startup, script execution and settling callbacks, before observing their output. Headless owners settle rebuilt startup synchronously.

Wrath's original forbidden-action popup first calls legacy `DisableAddOn`. That endpoint now registers the same Rust model function as `C_AddOns.DisableAddOn`; the [contract](../../specs/reload-ui.md#wrath-12340-requirements) records the seven-profile source check and prerequisite RED. No namespace alias table or vendor modification is involved. Wrath owner service now leaves the request pending during supplied cinematic playback; other profiles retain their prior eligibility policy. The original popup first passed its disable/query control, then reproduced the cinematic failure before this guard. Taint-gate production/feedback remain evidence-blocked.

The GUI shortcut records the same request. App discards frame-ID input, hit-testing, quad and pending-texture caches, then restarts yielded GUI startup using current canvas dimensions. Texture manager, font system and glyph atlas allocations survive.

## Environment retirement

Under Wrath, `WowLuaEnv::service_reload` opens the temporary protected-action window, synchronously emits argless PLAYER_REGEN_ENABLED, conditionally leaves world with argless PLAYER_LEAVING_WORLD, then emits argless PLAYER_LOGOUT. Only after handlers finish does it write declared SavedVariables, before VM destruction. An explicit in-world field is separate from IsLoggedIn. The combat bit stays intact; world entry ends the allowance and conditionally notifies PLAYER_REGEN_DISABLED so the original PlayerFrame rebuilds its hate-list display. The finer entry-helper slot is INFERRED simulator policy. Forever uses the researched shutdown policy described under session finalization below; other profiles retain their previous shutdown behavior. The persistence manager forgets old `Val` handles and registrations; each enabled addon re-registers through normal TOC loading. Previously saved addons bypass WTF import and read simulator storage. Their just-written data bypasses the external import-size cap and retains scalar globals as well as tables; ordinary initial imports keep the existing limit. Disabled persistence stays disabled. Imported WTF files are never written.

The old VM's self-reference and font handle are cleared before it drops. A fresh VM receives fresh app data, registries, API registration, templates and built-in frames. `SimState::discard_ui_environment` discards UI callbacks, animations, frames and addon runtime records without replacing player/world/party/session data. Selected target/focus and physical-input model fields (`mouse_position`, `mouse_buttons`, `pressed_keys`) are retained. Native keyboard/mouse persistence is INFERRED and unasserted pending lifecycle evidence; old frame-ID input caches are still discarded.

The launch pipeline installs a reload loader on the environment. It reuses Blizzard/third-party addon loading and post-load bootstrap, preserving original launch options. The initial EditMode layout name and a copied WTF config seed a clean cache reader with no old `Val` handles; the file-backed cache is reread for the fresh VM even when Lua SavedVariables are disabled. Native reread timing/ordering is INFERRED and unasserted. ServerSnapshot character/enable imports and initial party reseeding are still skipped on reload. No addon-specific sentinel cleanup exists.

## Constant dispatch factories

[`load_constant_factory`](../../../src/lua_api/script_helpers.rs) stores an unexecuted Lua-function anchor in the active VM registry, indexed by exact chunk name and source. The anchor roots the prepared prototype and its interned constants in that VM; no prototype or Lua handle lives in a process-global cache. HookScript and direct/indirect attribute dispatch retain their original Lua source, chunk names and invocation/error policies.

Each use allocates a new Lua closure from the prototype, binds it to the current global table and initializes independent nil top-level upvalues, matching `load_bytes`. The global table is shared within a VM; each closure's environment binding is independent. Returning/executing the anchor would let callback `setfenv` changes contaminate later calls, so the anchor is never exposed to dispatch. Nested hook closures and their captured arguments are still created by normal Lua execution.

The single constant `template-method-handler` factory also uses this path through [`load_constant_chunk`](../../../src/loader/chunk_cache.rs). Its exact source-derived generated chunk name is unchanged, and the builder still disables dynamic global slots while loading. Factory execution still creates a new method-name capture for each XML handler; inherited/chained handlers retain their existing execution policy. Generated chunks keep the existing bytecode path; layer instance identities now travel as arguments rather than source literals. No mutable closure cache or arbitrary Lua-text normalization is introduced.

Reload replaces the VM registry, so the prototype cache begins empty without a model-state reset or extra cleanup path. The [contract](../../specs/reload-ui.md#dispatch-factory-isolation) records matched before/after characterization and profile compilation; paired wall-time and full acceptance remain separate. Factory characterization does not authenticate native lifecycle ordering.

## Generated XML layer arguments

[`XmlLuaArgs`](../../../src/loader/xml_lua_args.rs) binds child names, parent references and resolved relativeTo names into a fresh invocation table. Texture, MaskTexture, Line and FontString producers emit table-slot expressions at those exact identity sites; XML properties and authored handlers remain generated normally. The shared anchor emitter accepts a reference emitter so non-layer callers keep their previous literal references.

[`LoaderEnv`](../../../src/lua_api/loader_env.rs) still loads through the existing exact-source bytecode cache, disables global slots and applies the same secure/scoped loading environment. Each execution receives a new table of strings/numbers, accessed through chunk varargs without introducing lexical locals that could shadow authored handlers. No Lua table, frame, closure or prototype is cached across VMs. The batch retains XML order and all post-exec attachment, animation and text-sync steps; standalone XML FontStrings use the same parameterized producer.

The [contract](../../specs/reload-ui.md#generated-xml-layer-reconstruction) records the Forever16-compilation prerequisite RED and bounded Forever/default Retail GREEN at `23f1f629a`: both reloads compile0 sources on each profile;27 exact cases/profile and both zero-warning checks pass, with shared formatting proof and unchanged original CVars. The initial fixture failure was implicit SetAllPoints against an unsized parent, fixed with an explicit TOPLEFT anchor without weakening size or freshness assertions. This optimization does not establish full-workload savings or native reload chronology.

The cost harness uses a distinct exact libtest selector for reconstruction so profiling/cache OnceLocks cannot inherit state from unrelated tests, and supplies a fresh cache directory. The budget parent has no environment-selected early return. A regression reexecutes that parent with the former child marker set and requires both zero-compile/budget1 observations. Subprocesses use the current test executable with native wait/kill/reap and concurrent pipe draining, not PATH-resolved `timeout`. These harness changes are formatted only; their new behavioral proof remains pending.

## Managed addon restoration

Wrath's loader defers [SavedVariables](../../../src/saved_variables.rs) until addon code finishes. It registers declarations, then reads account and character scopes once each, allowing persisted scalars/tables to overwrite code defaults. Initial WTF imports remain capped and read-only; freshly serialized reload storage bypasses that import cap and stale WTF. Other profiles keep their prior loading policy. The C_AddOns runtime route still passes no persistence manager; LoD restore/registration is UNMODELED, not silently bridged by a copied manager.

Parsed named XML bindings set `AddonInfo.contains_bindings`; conventional unlisted `Bindings.xml` is parsed for this metadata under Retail. The two real-addon notification emitters share payload construction: Retail appends a boolean, Wrath keeps one name argument. Boolean type is authenticated; deriving its value from parsed definitions is INFERRED. Synthetic Retail notifications remain unchanged. [Proof status](../../specs/reload-ui.md#wrath-12340-requirements) records feature RED→GREEN and separate two-sided preservation. The [final gate](../../specs/reload-ui.md#final-implementer-gate--2026-10-19) owns profile coverage and explicitly classified pre-existing failures.

## Wrath placed-frame storage and timer retention

After logout handlers and before registry retirement, a present SavedVariablesManager writes eligible geometry into its character-local `layout-local.txt`. The fresh addon loader runs before this file is read and applied to new named frames. Capture derives one corresponding root anchor from resolved geometry; offsets and size divide by effective scale and round nearest/ties-even. Current movable/resizable flags independently gate anchor/size restoration; scale is not restored. The [contract](../../specs/reload-ui.md#wrath-12340-requirements) owns grammar, exclusions, numeric deviation and proof. No manager means no file reads/writes. Wrath LeaveWorld clears mirror-timer state, not totems/player/session state, and invents no mirror-stop notification.

## Wrath startup readiness

Startup registers an environment-local pending account-variables callback. The two supplied CVar chunk flags invoke it immediately when already ready, or from the completion setter once both become ready. The callback clears its pending bit before synchronous VARIABLES_LOADED dispatch, preventing duplicate completion. Reload retires this callback while retaining account readiness. Wrath emits argless PEW and omits the synthetic WoWUISim addon notification. Other profiles keep their prior policy; this is not account-data networking. The [contract](../../specs/reload-ui.md#wrath-12340-requirements) owns current proof and separately blocked original-consumer controls.

## Lifecycle evidence and failure

GUI close, Ctrl-Q and Lua Quit share the fallible
[session finalizer](../../specs/session-finalization.md). Initialization and
shutdown diagnostics survive reload while current failed-owner data remains
protected. Intact failures can be retried after repair; incomplete VMs can only
recover through reconstruction before saving. GUI destruction does not save.
Forever runs its researched
shutdown callbacks before saving, with `ADDONS_UNLOADING(false)` on reload and
`true` on finalization. Its native chronology remains unmeasured. Wrath retains
its separate existing sequence.

The [storage and process contract](../../specs/session-finalization.md#storage-and-process-results)
describes configured output, interactive exit status and partial-save limits.
ReloadUI uses the same persistence storage without completing the session.

The [Wrath 12340 requirements](../../specs/reload-ui.md#wrath-12340-requirements) now distinguish authenticated static native facts from implementation proof. Cinematic eligibility has bounded original-popup GREEN. Shutdown-before-save, post-code addon SV restore, readiness-driven VARIABLES_LOADED and selective placed-frame/mirror state retain separate proof rows. The supplied cinematic state is not a cinematic playback engine. Unnamed taint-gate production and feedback remain evidence-blocked. Retail and other-profile chronology remain UNKNOWN; do not copy Wrath ordering into them. Failed reconstruction prevents a partial VM from overwriting valid SavedVariables; an explicit retry can reconstruct from valid inputs and storage.

## Sources

- [Request producer](../../../src/c_api/c_ui.rs) and [environment](../../../src/lua_api/env.rs) — deferred request and VM retirement.
- [Dispatch helpers](../../../src/lua_api/script_helpers.rs) — per-VM prototype rooting and per-use closure construction.
- [UI-owned state](../../../src/lua_api/state/ui_reload.rs) — reset boundary.
- [Launch bootstrap](../../../src/bin/wow_sim/startup.rs) — real addon reload loader.
- [App](../../../src/iced_app/app.rs) and [IPC boundary](../../../src/iced_app/update_servers.rs) — cache/startup reset and response-before-rebuild sequencing.
- [Persistence](../../../src/saved_variables.rs) — SavedVariables registration and storage.
- [Session finalization](../../specs/session-finalization.md) — fallible close/Quit, retained failures and Forever shutdown policy.

## See Also

- [[event-system]] — existing startup dispatch.
- [[addon-loading]] — addon graph reconstruction.
- [[texture-source-resolution-cache]] — generation invalidation without resource replacement.
