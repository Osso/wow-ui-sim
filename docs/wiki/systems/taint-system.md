# Taint System

The taint system enforces WoW's secure/insecure execution boundary. The simulator keeps the practical pieces that addons rely on: protected-frame gating, the dual-environment split (`genv` vs `secureenv`), SecureHandler fallbacks, and shallow state/attribute driver application. Per-call taint tracking still comes from the Elune Lua runtime.

## Design Scope

Full retail taint simulation remains out of scope. What the simulator does implement:

- **Protected-frame gating**: `can_change_protected_state_for()` blocks protected mutations when the caller is insecure and the player is in combat
- **Dual Lua environment**: `genv` (`_G`) for addon code vs `secureenv` for Blizzard secure code. `secureenv` is a separate shallow copy, not a live overlay over `_G`.
- **Elune runtime taint**: `issecure`, `securecall`, `issecurevariable`, `forceinsecure`, and `debug.*taint*` helpers come from Elune
- **securecallmethod()**: simulator-provided helper that Elune omits
- **Secret values**: simulator-owned identity values and tainted `CallMethod` payloads/results are tracked by fallback accessors
- **SecureHandler fallback**: `SecureHandlerSetFrameRef`, `SecureHandlerGetFrameRef`, `SecureHandlerExecute`, `SecureHandlerWrapScript`, and `SecureHandlerUnwrapScript` are backed by the Lua-side fallback in `src/lua_api/globals/security.rs`
- **State/attribute drivers**: `RegisterStateDriver`, `UnregisterStateDriver`, `RegisterAttributeDriver`, `UnregisterAttributeDriver` store raw driver text in `SimState.secure_attribute_drivers` and eagerly apply the resolved state

## Protected Frame Gating (`src/lua_api/frame/methods/methods_helpers.rs`)

`can_change_protected_state_for(state, id)` returns true when the caller is secure or the player is out of combat. In combat, it blocks if `frame_blocks_protected_state()` says the target frame is protected, has a protected descendant, or anchors to protected state.

The frame methods that use this gate live under `src/lua_api/frame/methods/`, primarily:

- `core_state/visibility.rs`
- `core_state/size.rs`
- `core_state/scale.rs`
- `core_state/strata_level.rs`
- `button_anchor_hierarchy/anchors.rs`
- `button_anchor_hierarchy/hierarchy.rs`
- `text_attribute_event/attributes.rs`

When blocked, callers emit `ADDON_ACTION_BLOCKED` via `emit_addon_action_blocked()` and return without mutating the frame.

`Protect()` is implemented in `src/lua_api/frame/methods/misc/secret.rs`. It only sets `frame.is_protected` for secure callers; insecure callers silently fail.

## Dual Lua Environment (`src/lua_api/globals/security.rs`)

Two environments share the same Lua state:

- **genv** (`_G`): addon code; `Blizzard_EnvironmentCleanup` nils secure APIs here
- **secureenv**: shallow copy of `_G` at startup with no `__index = _G` fallback; retains secure APIs after cleanup; addons with `UseSecureEnvironment: 1` in TOC run here via `setfenv`. Blizzard Lua reaches it only by replay: the library addons listed in `is_secure_replay_library_addon` (`src/loader/addon.rs`) have their Lua loaded a second time in the secure pass (`[AllowLoadEnvironment Global]` files excluded). `Blizzard_CombatLogBase` is on that list because secure `Blizzard_CombatLogProcessor` reads `CombatLogUtil`, `COMBATLOG_FILTER_MINE` and `COMBATLOG_DEFAULT_COLORS` from it. Replaying every declared dependency of a secure addon instead is not viable: secure TOCs also declare `Blizzard_FrameXMLBase`, `Blizzard_ObjectAPI`, `Blizzard_SharedXMLGame`, `Blizzard_VisualAlerts` and similar UI addons as `## Dep`, and re-running those in secureenv re-executes their top-level code (measured on PTR: `Constants.lua` and `VisualAlertTemplates.lua` load errors doubled and `PrivateVisualAlertsManager` OnLoad gained `attempt to call a nil value`).

The shallow copy preserves table references that existed at initialization: `tests/secureenv_isolation.rs::shared_table_mutation_propagates_both_ways` dynamically finds a table present in both environments and verifies a secure-side mutation through the shared reference is visible from `_G`. Primitive values and later `_G` bindings remain separate; later global tables are not visible to secureenv without explicit export.

Retail probe result, 2026-06-20: wrapping `_G.CooldownFrame_Set` and `_G.CooldownFrame_Clear` printed for normal CooldownViewer callers but did not print from `Blizzard_PrivateAurasUI` during a Mythic+ private-aura display. PrivateAurasUI is `UseSecureEnvironment: 1`, so its cooldown code did not reach the live `_G` override. That falsifies the simulator's previous `secureenv` metatable fallback model.

Blizzard source also matches this: outbound bridge files explicitly capture `local secureEnv = GetCurrentEnvironment()`, call `SwapToGlobalEnvironment()`, then assign bridge tables back into `secureEnv` (for example `secureEnv.WowTokenOutbound = WowTokenOutbound`). In the current retail contract, inbound Wow Token callbacks and `RedeemFailed` publish to `_G`, while `WowTokenOutbound` is stored only in `__secureenv`; these are explicit source-level exports, not generic mirroring.

For shared Blizzard libraries, publication into secureenv is an explicit loader allowlist, not generic global mirroring. The allowlist replays `Blizzard_CombatLogBase` and `Blizzard_CatalogShopSharedUtil` so secure consumers receive `CombatLogUtil` and `CatalogShopUtil`; focused tests verify both `_G` and `__secureenv` bindings. See [[addon-loading]] for the complete allowlist and replay lifecycle.

When no click-binding profile is modeled, `C_ClickBindings.GetBindingType()` reports `Enum.ClickBindingType.None` and `ExecuteBinding()` does nothing. This leaves Blizzard secure-button `type`/`*typeN` attributes in control; fabricated fallback targeting would bypass focus/assist dispatch.

`set_in_both_envs_rilua(key, value)` registers named frames in both environments so frame globals are visible from both.

## Elune Runtime Functions

Elune provides `issecure`, `issecurevariable`, `securecall`, `securecallfunction`, `forceinsecure`, `hooksecurefunc`, `secureexecuterange`, and the `debug.*taint*` helpers. The simulator relies on these VM-level functions instead of replacing them.

`securecallmethod(obj, name, ...)` calls `obj[name](obj, ...)` via protected pcall dispatch and returns `nil` on missing/non-function/error paths.

Frame method `CallMethod` is registered in `src/lua_api/frame/methods/text_attribute_event/mod.rs`. It preserves return values and marks tainted caller arguments/results as simulator secret values so insecure data cannot be laundered through secure snippets.

## Secret Values

`src/lua_api/globals/security.rs` provides fallback accessors for secret values:

- `issecretvalue(value)` returns true for values explicitly marked with the simulator secret marker, Elune-tainted functions, tables with tainted slots, and tables containing nested secret keys or values.
- `canaccessvalue(value)`, `canaccessallvalues(...)`, and `canaccesstable(table)` return false when those same checks find a secret value.
- `scrub()` and `scrubsecretvalues()` are still pass-throughs.

Current simulator-owned secret values include party/raid identity strings returned through unit/group APIs and tainted `CallMethod` payloads/results.

## SecureHandler Fallback

`src/lua_api/globals/security.rs` installs a Lua-side fallback for the SecureHandler APIs before `Blizzard_RestrictedAddOnEnvironment` arrives.

- `SecureHandlerSetFrameRef(frame, label, refFrame)` stores frame refs in weak-keyed registries
- `SecureHandlerGetFrameRef(frame, label)` reads those stored refs back
- `SecureHandlerExecute(frame, body, ...)` compiles `body` into a restricted closure and runs it with `pcall`
- `SecureHandlerWrapScript(frame, script, header, preBody, postBody)` wraps the original handler with pre/original/post callbacks
- `SecureHandlerUnwrapScript(frame, script)` restores the original handler

`SecureHandlerExecute` snippets run in a locked restricted environment, not the full `_G`. The fallback exposes utility functions/tables such as `assert`, `error`, `ipairs`, `math`, `next`, `pairs`, `print`, `select`, `string`, `tonumber`, `tostring`, `type`, and `unpack`; the `math` and `string` tables are read-only copies.

## State Drivers

`RegisterStateDriver`, `UnregisterStateDriver`, `RegisterAttributeDriver`, `UnregisterAttributeDriver` are backed by `SimState.secure_attribute_drivers`.

- `RegisterStateDriver(frame, "visibility", ...)` maps to the `state-visibility` special case and toggles visibility plus `statehidden`
- Other state/attribute drivers resolve the final clause of the driver string and write that value directly onto the frame
- `Unregister*` removes the stored driver text but leaves the last applied frame state in place

Driver limitations:

- Conditional grammar is not fully evaluated in this path.
- Driver values are not automatically reevaluated on every relevant state transition.
- This is a compatibility fallback for addon bootstrap, not a full `SecureStateDriverManager`.

## Blizzard `issecure()` Call-Sites

These are the real Blizzard Lua paths the simulator executes today that branch on `issecure()` or pass its value into secure APIs. They are the practical end-to-end checks for Elune taint integration. The important runtime question is whether existing tests exercise the secure branch, the insecure branch, or only load-time registration.

| Area | Blizzard paths | Branch behavior | Simulator coverage |
|------|----------------|-----------------|--------------------|
| Secure mixin creation | `Blizzard_SharedXMLBase/Mixin.lua` | `SecureMixin` and `CreateFromSecureMixins` return only for secure callers. | Loaded by nearly every Blizzard integration lane; TOC coverage in `tests/toc_parsing.rs`, global/mixin coverage in `tests/utility_api.rs`, startup/panel/keybinding lanes. |
| Secure slash commands | `Blizzard_ChatFrameBase/Shared/SlashCommands.lua`, `SlashCommandsRegistry.lua` | Secure callers register secure command aliases; insecure callers fall back to normal slash command tables or error for `AddSecureCmd`. | `Blizzard_ChatFrameBase` loads in startup/load-order lanes; secure-file load coverage in `tests/startup_warnings.rs::test_secure_env_annotated_files_load_cleanly`. |
| Action button secure attributes | `Blizzard_ActionBar/Shared/ActionButton.lua`, `Blizzard_ActionBar/WoWLabs/ActionButtonOverrides.lua` | Secure callers update protected `showgrid` attributes; insecure callers skip the protected attribute mutation. | Action bar loading and interaction are covered by `tests/action_bar_drag.rs`, `tests/spell_casting.rs`, `tests/frame_positions.rs`, and ShowUIPanel/keybinding lanes that include `Blizzard_ActionBar`. |
| UI panel lockdown | `Blizzard_UIParentPanelManager/Shared/UIParentPanelManager.lua` | `CheckProtectedFunctionsAllowed` blocks insecure panel show/hide while in combat. | Panel load and interaction coverage in `tests/test_showuipanel.rs`, `tests/test_showuipanel_lod*.rs`, `tests/panel_harness_runtime.rs`, and keybinding panel tests. |
| Static popup secure text | `Blizzard_StaticPopup/StaticPopup.lua` | `editBoxSecureText` dialogs error when shown from tainted context. | StaticPopup loads in panel/click/profession lanes; no focused tainted secure-editbox dialog test yet. |
| CVar cache hygiene | `Blizzard_SharedXMLBase/CvarUtil.lua` | CVar values are cached only when execution is secure to avoid tainting later reads. | CVar API coverage in `tests/test_cvar_display_settings.rs`, `tests/set_cvar_global.rs`, startup warning coverage for registered CVars. |
| Tooltip callback tables | `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua` | Secure callbacks are stored in secure tables; insecure callbacks are wrapped with `forceinsecure()`. | Full tooltip lanes in `tests/tooltip_hover.rs` and `src/loader/tests/wow_api_tooltip.rs`. |
| Edit Mode secure delegate | `Blizzard_EditMode/Shared/EditModeManager.lua` | `ClearSelectedSystem` uses a secure delegate when secure or out of combat, otherwise runs direct cleanup. | `Blizzard_EditMode` is in startup, panel, keybinding, action-bar, and frame-position lanes. |
| Group Finder protected searches | `Blizzard_GroupFinder/Mainline/LFGList.lua` | Secure pending quest/scenario searches start directly; insecure paths show confirmation popups. | `Blizzard_GroupFinder` loads in full load-order and keybinding panel lanes; no focused insecure pending-search branch test yet. |
| Unit popup protected actions | `Blizzard_UnitPopupShared/UnitPopupSharedButtonMixins.lua` | Target, Battle.net target, and raid-role buttons are hidden/disabled for insecure callers. | `Blizzard_UnitPopupShared` / `Blizzard_UnitPopup` load in full load-order and interaction lanes; no focused tainted unit-popup branch test yet. |
| Restricted add-on environment | `Blizzard_RestrictedAddOnEnvironment/RestrictedExecution.lua`, `RestrictedInfrastructure.lua`, `SecureHandlers.lua`, `SecureHoverDriver.lua` | Secure callers can create restricted closures/tables/frame handles and direct auto-hide operations; insecure callers error or route through attribute-mediated updates. | Load and exported-surface coverage in `tests/startup_warnings.rs::test_secure_env_annotated_files_load_cleanly` and `test_restricted_addon_environment_exposes_execution_surface`; fallback API coverage in `tests/secure_handler_fallback.rs`; state/driver/security coverage in `tests/security_api.rs`. |
| Nameplate secure flag forwarding | `Blizzard_NamePlates/Blizzard_NamePlates.lua`, `Blizzard_NewPlayerExperience/Blizzard_TutorialTutorials.lua` | Current secure state is passed to `C_NamePlate.GetNamePlateForUnit` / `GetNamePlates`. | NamePlate addon is in full load-order; startup API stubs cover nameplate API presence. NewPlayerExperience file is present in the source tree but not part of the current normal load-order snapshot. |
| Script error registration | `Blizzard_ScriptErrors/Blizzard_ScriptErrors.lua`, `Blizzard_ScriptErrorsFrame/Blizzard_ScriptErrorsFrame.lua` | Internal handler registration asserts secure execution. | ScriptErrorsFrame is in full load-order; startup/load warning lanes catch load-time assertion failures. |
| Debug/menu probes | `Blizzard_DebugTools/DebugObjectUtil.lua`, `Blizzard_Menu/MenuTemplates.lua` | Debug object access permits secure callers or non-forbidden objects; menu debug path prints current secure state. | DebugTools and Menu load in full/panel/click lanes. These are diagnostic/passive branches, not core compatibility gates. |

End-to-end coverage summary:

- `tests/security_api.rs` covers the base Elune contract: `issecure()`, `forceinsecure()`, `loadstring()` tainting, and `securecall()` restoring secure execution.
- `tests/protected_frame_enforcement.rs` covers the combat/insecure gates that many `issecure()` branches are protecting.
- `tests/secure_handler_fallback.rs` covers the SecureHandler fallback path that runs before `Blizzard_RestrictedAddOnEnvironment` loads.
- `tests/secure_group_headers.rs` covers the secure group-header path after `Blizzard_RestrictedAddOnEnvironment` loads.
- `tests/startup_warnings.rs` and `tests/load_order.rs` cover the Blizzard addon startup/load path where these call sites are exercised together.
- Startup, panel, tooltip, action-bar, keybinding, and click-targeting tests exercise several interaction branches through real Blizzard Lua.

Remaining audit gaps are branch-specific coverage gaps, not known missing implementation by themselves: tainted calls into StaticPopup secure edit boxes, GroupFinder pending-search confirmation, UnitPopup protected actions, and NamePlate secure flag behavior.

## Sources

- [protected-frame-enforcement.md](../../protected-frame-enforcement.md) — protected-frame behavior and remaining gaps
- `src/lua_api/frame/methods/methods_helpers.rs` — protected-state gating and `ADDON_ACTION_BLOCKED`
- `src/lua_api/globals/security.rs` — taint helpers, `securecallmethod`, SecureHandler fallback, state/attribute drivers, secure environment
- `src/lua_api/workarounds/temporary/click_bindings_defaults.rs` — no-profile click-binding behavior
- `src/lua_api/state.rs` — `secure_attribute_drivers` storage
- `src/loader/lua_file.rs` — per-addon compiled-closure taint stamping
- `src/lua_api/env.rs` — frame script-handler taint stamping
- `tests/protected_frame_enforcement.rs` — combat lockdown coverage
- `tests/secure_handler_fallback.rs` — SecureHandler fallback coverage
- `tests/security_api.rs` — state driver and `securecallmethod` coverage

Removed/stale paths that older docs may mention:

- `src/lua_api/globals/security_api.rs`
- `src/lua_api/secure_env.rs`
- `src/lua_api/frame/methods/combat_lockdown.rs`

## See Also

- [[lua-api]] — Lua-facing compatibility surfaces and modeled boundaries
- [[addon-loading]] — explicit secure replay allowlist and addon load lifecycle
- [[post-load-workaround-audit]] — cleanup restoration ownership
- [[event-system]] — ADDON_ACTION_BLOCKED event firing
- [[frame-data-flow]] — frame is_protected field and Protect() method
- [[protected-frames]] — focused protected-frame enforcement notes
