# Taint System

The taint system enforces WoW's secure/insecure execution boundary. The simulator keeps the practical pieces that addons rely on: protected-frame gating, the dual-environment split (`genv` vs `secureenv`), SecureHandler fallbacks, and shallow state/attribute driver application. Per-call taint tracking comes from the rilua VM, which implements Elune-compatible taint natively.

## Design Scope

Full retail taint simulation remains out of scope. What the simulator does implement:

- **Protected-frame gating**: `can_change_protected_state_for()` blocks protected mutations when the caller is insecure and the player is in combat
- **Dual Lua environment**: `genv` (`_G`) for addon code vs `secureenv` for Blizzard secure code. `secureenv` is a separate shallow copy, not a live overlay over `_G`.
- **VM taint**: `debug.*taint*` helpers come from rilua; `issecure`, `securecall`, `issecurevariable` and `forceinsecure` follow Elune semantics
- **securecallmethod()**: simulator-provided helper
- **Secret values**: simulator-owned identity values and tainted `CallMethod` payloads/results are tracked by fallback accessors
- **SecureHandler fallback**: `SecureHandlerSetFrameRef`, `SecureHandlerGetFrameRef`, `SecureHandlerExecute`, `SecureHandlerWrapScript`, and `SecureHandlerUnwrapScript` are backed by the Lua-side fallback in `src/lua_api/globals/security/`
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

## Aura Access Restrictions (`src/c_api/unit_aura_access.rs`)

Retail 12.1.0 `frame_id_from_stack` (native script-object resolution) rejects tainted callers when the object carries `DenyTaintedAccessWhenAurasAreSecret` and `SimState.unit_auras_restricted` (auras secret) is set; the error is the client's fixed `Attempt to access forbidden object from code tainted by an AddOn` (template NATIVE; its use on this path best-guess, see [taint error text](../../specs/taint-error-text.md)). The restriction lifts as soon as auras become public again. `CanBeAccessedInContext` follows the same rule; `IsForbidden`, `CanBeAccessedInContext`, `HasAccessConstraints` and the access-restriction mask methods resolve through `frame_id_from_stack_unrestricted` so any caller can query them. Blizzard Lua owns the timing: the AuraContainer frame provider applies the restriction after `initializeFrame`, deferred to PLAYER_ENTERING_WORLD before login. The aura-secret state itself is an explicit input; combat/encounter/M+/PvP activation is not derived. Plain Lua field reads on the frame table are not gated, and the `forbidden` flag is still not enforced on method calls. Proof: `tests/patch_12_1_0_aura_access_restrictions.rs`.

## Dual Lua Environment (`src/lua_api/globals/security/secure_env.rs`)

Two environments share one Lua state:

- **genv** (`_G`): addon code; `Blizzard_EnvironmentCleanup` nils selected secure APIs here.
- **secureenv** (`__secureenv`): one shared table for secure addon chunks, not one table per addon.

**VERIFIED (Retail 12.1.5.70688, R88g.1):** the client creates the shared environment before addon files, seeds it from exactly 39 names, shallow-copies values that are tables, and installs no `__index = _G` fallback. R88y later verifies per-member destinations for a bounded inventory; neither finding proves a blanket ordinary-global copy.

**VERIFIED (simulator source):** `create_secure_environment` uses the fixed 39-name base, records that exact base-name set privately, gives secureenv its own `_G` reference, and installs no ordinary-global fallback. **VERIFIED (Retail 12.1.5.70688, R88y.y2):** the native initializer opens libraries and registers their unflagged aliases before this table copy; the simulator installs the verified string aliases and ordinary `fastrandom` at the same pre-copy boundary. Later global registration alone does not create a plain-name lookup in secureenv. The R88y publisher reads only existing handlers after bootstrap and after screen-specific registration, and it tracks prior values when it reconciles late registrations. The [R88y publication contract](../../specs/addon-load-phases.md#r88y-verified-secure-publication) records the native function-identity boundary. **VERIFIED (simulator source):** direct flag-3 RustFn registration creates one zero-upvalue `RustClosure` per destination; the generic publisher creates a separate closure when a verified flag-3 Rust handler exists on only one side. The standard-library shallow-copy path retains the original function values. **UNVERIFIED:** native function identity after later script replacement or deliberate aliasing.

**VERIFIED (Retail 12.1.5.70688, R88y.y1; source CSV SHA-256 `3a8922d84756171f666fe79b61068d3344ce2db866250dede1de384fa9878f82`):** the working CSV has 6,763 distinct namespace/member rows: 6,554 VERIFIED and 209 INFERRED. Effective flags select ordinary-only (1), secure-only (2), both (3), secure-only plus an unknown extra action (6), or both plus an unknown extra action (7). The simulator table contains only VERIFIED rows and routes a member only when an implementation exists. The row context is retained; existing screen-aware registrars decide which members exist. The simulator does not create absent C_* functions. `GetForbiddenObjectTable` is flag 6; its internal Lua helpers keep a local closure while the global name is moved to secureenv.

**VERIFIED (Retail 12.1.5.70688, R88y.y2):** `string.concat`, `string.split`, and `string.join` alias the existing `strconcat`, `strsplit`, and `strjoin` handlers before the fixed-table copy. The native table startup/extension entries do not include `table.join`. The ordinary global `fastrandom` retains the original base `math.random` handler and is not copied into secureenv; the replacement `math.random` handler's behavior and identity remain UNKNOWN. The simulator keeps its current handler and tests only library membership and function identity, not random output. The [Retail `string.split` alias investigation](../investigations/post-load-workaround-audit.md#retail-r88y-stringsplit-alias-collision) records the later bootstrap overwrite and profile boundary.

**VERIFIED (Retail 12.1.5.70688, R88y.y3):** successful `SwapToGlobalEnvironment` changes the caller function environment to ordinary `_G` and returns zero values; it does not populate secureenv. XML `addToSecureEnv` publishes eligible named frames, not Lua-defined globals. Automatic global copying, complete `securecopy` behavior, and frame-callback access remain UNKNOWN. The [add-on load-phase contract](../../specs/addon-load-phases.md#runtime-loading) records these bounds.

**VERIFIED (Retail 12.1.5.70688, R88g.2):** the examined native registrars publish `Enum` members and `C_Timer.After`, `C_UIColor.GetColors`, `Mixin`, and `CreateFromMixins` into the shared secure environment when their native checks pass. **VERIFIED (merged simulator source at `a941395885655f56edfcc1f57cac3fa1f0c853c7`):** Retail `C_UIColor.GetColors` reads the generated `GlobalColor` catalog. The five-row temporary `C_UIColor` payload remains only for non-Retail profiles. **VERIFIED (R88i.2 BASELINE-39; simulator source):** `geterrorhandler` is a copied base name; because simulator native registration follows secure-table construction, its registrar now publishes to both environments.

**VERIFIED (native Retail 12.1.5.70688, R88t.t2):** `AddSourceLocationExclude` and `table.count` use flag 3 and separate Lua function objects; the table libraries are separate. The six `C_ColorOverrides` members use separate namespace tables and function objects. `Constants.*` tables, including `Constants.AccountStoreConsts`, use a no-flag publication route. The UIParent bootstrap uses named-frame policy flag 32 and gives the same frame-table object to both destinations when both names are empty; it preserves existing non-nil names. See the [add-on load-phase contract](../../specs/addon-load-phases.md#r88t-secure-dependency-and-named-publication-routes) for the exact member names and constant values.

**VERIFIED (Retail 12.1.5.70688, R88ac.ac2):** accepted GlobalStrings DB2 records write the same name and value to ordinary `_G` and the separately referenced shared secure table after the Game/Glue Flags mask check. This is direct row publication, not a copy of `_G`. **INFERRED (current simulator source):** Retail's generated GlobalStrings PHF map has no Flags field, so its current row set is mirrored into secureenv. This does not establish individual row-mask membership; `RAID_TARGET_1` remains INFERRED.

**VERIFIED (simulator source at `49e4d091d`):** Retail evaluates generated `CONSTANTS_VALUES_LUA` data in both environments. It uses a temporary secure `Constants` metatable to create the generated tables, then clears that metatable. Missing and compatibility datasets remain ordinary-only. **UNVERIFIED:** the new `InventoryConstants` consumer test has not run.

**VERIFIED (simulator source and test at `adf8d25f9`, Retail 12.1.5):** `AddSourceLocationExclude` is a real global with separate ordinary and secure function objects. Both update one shared `SimState.source_location_exclusions` vector. `CreateTexture` selects the first non-excluded Lua file and line from existing `CallInfo` and `Proto` debug data, then stores `file:line` on the texture; `GetSourceLocation` returns that saved value. No rilua tracking changes were needed. The two-file caller/wrapper probe remains INFERRED and is not asserted.

**VERIFIED (simulator source at `5cd3cc29e`):** Retail `C_ColorOverrides` registers its six modeled handlers after secureenv construction, then uses the generic R88y publisher. The two temporary handlers and custom secure-publication bridge are removed on Retail; non-Retail defaults remain. No unimplemented member is published. **UNVERIFIED (merged-head runtime):** the exact six-member secure table test is still pending. Event mapping, persisted-string encoding, and the reload/logout chain remain unmodeled. No global was added to the fixed 39-name base, and no blanket ordinary-to-secure copy was added.

**VERIFIED (Retail 12.1.5 pinned source and simulator run at `7f81a8853`):** `Blizzard_SharedXMLBase/Color.lua:80-88` builds color globals from `C_UIColor.GetColors()` in `GetCurrentEnvironment()`. Runtime diagnostics showed `Color.lua` loaded in secureenv and `GetCurrentEnvironment() == _G` from secure code. **VERIFIED (merged simulator source):** the generated 70783 catalog contains `TOOLTIP_DEFAULT_BACKGROUND_COLOR` with packed value `0xff171730`, and the flag-3 registration routes `GetColors` to secureenv. **INFERRED:** applying 70688 `GetColors` behavior to the 70783 catalog follows the pinned-source rules documented in [[global-color-catalog]]. The prior fallback-disabled PrivateAuras test was RED at pre-merge SHA `7f81a8853`; **UNVERIFIED:** its result against the merged catalog. The hand-set table in `colors.rs:291-292` remains ordinary-only under the existing preservation rule; do not directly publish the color.

**VERIFIED (Retail simulator run, fallback cleared):** secure dependency execution provides the original `TooltipDataProcessor` from `TooltipDataHandler.lua:177`; `CopyTable` and `GetFrameMetatable` are present in secureenv. R88i.1 verifies that `GetFrameMetatable` uses the native name/handler pair-list registration path without a per-name flag; the simulator publishes its Rust implementation through the shared helper. The next observed dependency error is `Blizzard_SharedXMLBase/TextureUtil.lua:122` (`Constants` is nil); the later `Blizzard_PrivateAurasUI.lua:1398` consumer then reports nil `TextureKitConstants`. **INFERRED:** `TextureUtil.lua` does not reach its assignment to `TextureKitConstants` at line 325 in this secure pass. This does not prove full PrivateAurasUI acceptance. **VERIFIED (current source):** the Retail TooltipDataProcessor fallback remains installed until all cited consumers pass through their original owners.

Retail probe result, 2026-06-20: wrapping `_G.CooldownFrame_Set` and `_G.CooldownFrame_Clear` printed for normal CooldownViewer callers but did not print from `Blizzard_PrivateAurasUI` during a Mythic+ private-aura display. PrivateAurasUI is `UseSecureEnvironment: 1`, so its cooldown code did not reach the live `_G` override. That falsifies the simulator's previous `secureenv` metatable fallback model.

Blizzard source also matches this: outbound bridge files explicitly capture `local secureEnv = GetCurrentEnvironment()`, call `SwapToGlobalEnvironment()`, then assign bridge tables back into `secureEnv` (for example `secureEnv.WowTokenOutbound = WowTokenOutbound`). In the current retail contract, inbound Wow Token callbacks and `RedeemFailed` publish to `_G`, while `WowTokenOutbound` is stored only in `__secureenv`; these are explicit source-level exports, not generic mirroring.

For shared Blizzard libraries, secure dependency files run in the shared environment when the consumer uses `UseSecureEnvironment: 1`; selected non-secure consumers also use the explicit library allowlist. Neither path mirrors all of `_G`. See [[addon-loading]] for dependency replay, allowlist, and lifecycle details.

`C_ClickBindings` has a per-environment interaction profile. Its unmodified defaults are inferred simulator policy, not native-probed behavior; `ExecuteBinding()` remains an inert temporary gap. The [click-binding interaction profile](../../specs/click-binding-interaction-profile.md) is the current contract and proof SSOT.

`src/c_api/helpers.rs` keeps ordinary and secure namespace tables separate. Flag-3 Rust function registration constructs one closure per table. Value registration can copy an existing `Val`; the R88y publisher constructs a separate zero-upvalue Rust closure when it publishes a flag-3 handler to the other table.

## Runtime Taint Functions

rilua provides the `debug.*taint*` helpers natively. The simulator registers or overrides `issecurevariable`, `forceinsecure`, `issecure`, `hooksecurefunc` and `secureexecuterange` on top of that VM state; see the Taint System section of the repo `AGENTS.md`.

`securecallmethod(obj, name, ...)` calls `obj[name](obj, ...)` via protected pcall dispatch and returns `nil` on missing/non-function/error paths.

Frame method `CallMethod` is registered in `src/lua_api/frame/methods/text_attribute_event/mod.rs`. It preserves return values and marks tainted caller arguments/results as simulator secret values so insecure data cannot be laundered through secure snippets.

## Secret Values

`src/lua_api/globals/security/` provides fallback accessors for secret values:

- `issecretvalue(value)` returns true for values explicitly marked with the simulator secret marker, Elune-tainted functions, tables with tainted slots, and tables containing nested secret keys or values.
- `canaccessvalue(value)`, `canaccessallvalues(...)`, and `canaccesstable(table)` return false when those same checks find a secret value.
- `scrub(...)` replaces secret values and every non-boolean/number/string argument with nil; `scrubsecretvalues(...)` replaces only secret values. Both keep the argument count. Sources: decomp S18 §12 and `FrameScriptDocumentation.lua` (two routes). `issecretvalue()` / `canaccessvalue()` with no argument treat the missing value as nil (`tests/security_api.rs`).

Current simulator-owned secret values include party/raid identity strings returned through unit/group APIs and tainted `CallMethod` payloads/results.

## SecureHandler Fallback

`src/lua_api/globals/security/` installs a Lua-side fallback for the SecureHandler APIs before `Blizzard_RestrictedAddOnEnvironment` arrives.

- `SecureHandlerSetFrameRef(frame, label, refFrame)` stores frame refs in weak-keyed registries
- `SecureHandlerGetFrameRef(frame, label)` reads those stored refs back
- `SecureHandlerExecute(frame, body, ...)` compiles `body` into a restricted closure and runs it with `pcall`
- `SecureHandlerWrapScript(frame, script, header, preBody, postBody)` wraps the original handler with pre/original/post callbacks
- `SecureHandlerUnwrapScript(frame, script)` restores the original handler

`SecureHandlerExecute` snippets run in a locked restricted environment, not the full `_G`. The fallback exposes utility functions/tables such as `assert`, `error`, `ipairs`, `math`, `next`, `pairs`, `print`, `select`, `string`, `tonumber`, `tostring`, `type`, and `unpack`; the `math` and `string` tables are read-only copies.

## State Drivers

Before Blizzard's restricted runtime loads, the fallback `RegisterStateDriver`, `UnregisterStateDriver`, `RegisterAttributeDriver` and `UnregisterAttributeDriver` use `SimState.secure_attribute_drivers`.

- `RegisterStateDriver(frame, "visibility", ...)` maps to the `state-visibility` special case and toggles visibility plus `statehidden`
- Other state/attribute drivers resolve the final clause of the driver string and write that value directly onto the frame
- `Unregister*` removes the stored driver text but leaves the last applied frame state in place

Driver limitations:

- Conditional grammar is not fully evaluated in this path.
- Driver values are not automatically reevaluated on every relevant state transition.
- This is a compatibility fallback for addon bootstrap, not a full `SecureStateDriverManager`.

The vendor state template supplies a restricted handle so `RunAttribute` and
frame references use the managed environment. Its untainted declaration
retires the raw-frame `_onstate-*` fallback for that VM.
[State snippet ownership](../../specs/secure-attribute-delegation.md#state-snippet-ownership)

## Blizzard `issecure()` Call-Sites

These are the real Blizzard Lua paths the simulator executes today that branch on `issecure()` or pass its value into secure APIs. They are the practical end-to-end checks for Elune taint integration. The important runtime question is whether existing tests exercise the secure branch, the insecure branch, or only load-time registration.

The secure-file and restricted-environment startup-test citations below have [profile-specific applicability](../../specs/client-profiles.md#startup-warning-test-applicability); they do not establish all-profile runtime acceptance.

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

- `tests/security_api.rs` covers the base Elune-compatible contract: `issecure()`, `forceinsecure()`, `loadstring()` tainting, and `securecall()` restoring secure execution.
- `tests/protected_frame_enforcement.rs` covers the combat/insecure gates that many `issecure()` branches are protecting.
- `tests/secure_handler_fallback.rs` covers the SecureHandler fallback path that runs before `Blizzard_RestrictedAddOnEnvironment` loads.
- `tests/secure_group_headers.rs` covers the secure group-header path after `Blizzard_RestrictedAddOnEnvironment` loads.
- `tests/startup_warnings.rs` and `tests/load_order.rs` cover the Blizzard addon startup/load path where these call sites are exercised together.
- Startup, panel, tooltip, action-bar, keybinding, and click-targeting tests exercise several interaction branches through real Blizzard Lua.

Remaining audit gaps are branch-specific coverage gaps, not known missing implementation by themselves: tainted calls into StaticPopup secure edit boxes, GroupFinder pending-search confirmation, UnitPopup protected actions, and NamePlate secure flag behavior.

## Sources

- [protected-frame-enforcement.md](../../protected-frame-enforcement.md) — protected-frame behavior and remaining gaps
- `src/lua_api/frame/methods/methods_helpers.rs` — protected-state gating and `ADDON_ACTION_BLOCKED`
- `src/lua_api/globals/security/secure_env.rs` — fixed secure base, environment publication, and secure chunk retargeting
- `src/c_api/secure_publication.rs`, `src/c_api/secure_publication_data.rs` — verified R88y member routing and late-registration reconciliation
- `data/secure-publication/retail-12-1-5-70688.csv`, `data/secure-publication/provenance.json`, `tools/gen_r88y_secure_publication.py` — source inventory, hash/count manifest, and table generator
- `src/lua_api/globals/real/math_extensions.rs`, `src/lua_api/globals/real/table_extensions.rs`, `src/lua_api/env_init/shared_bootstrap.lua` — pre-copy aliases, distinct `table.count` registration, and the internal forbidden-object closure
- `src/lua_api/globals/security/environment.rs` — `SwapToGlobalEnvironment` success route
- `src/c_api/helpers.rs` and `crates/sim-lua-bridge/src/table_builder.rs` — paired namespace creation and flag-3 Rust function closure construction
- R88 series, Retail 12.1.5.70688 decomp line 13165 — separate no-capture native C closure objects for flag-3 publication
- `src/lua_api/globals/security/` — taint helpers, `securecallmethod`, SecureHandler fallback, state/attribute drivers
- `src/c_api/c_click_bindings.rs` — modeled per-environment interaction profile
- `src/lua_api/workarounds/temporary/click_bindings_defaults.rs` — inert `ExecuteBinding` and remaining temporary helpers
- `src/lua_api/state.rs` — `secure_attribute_drivers` storage
- `src/loader/lua_file.rs` — per-addon compiled-closure taint stamping
- `src/lua_api/env.rs` — frame script-handler taint stamping
- `tests/protected_frame_enforcement.rs` — combat lockdown coverage
- `tests/secure_handler_fallback.rs` — SecureHandler fallback coverage
- `tests/security_api.rs` — state driver and `securecallmethod` coverage
- `tests/blizzard_private_auras_ui_loads.rs` — secure TooltipDataProcessor provenance and the exact `TextureUtil.lua:122` / `PrivateAurasUI.lua:1398` boundary
- `src/loader/addon.rs` — secure dependency replay and the separate library allowlist
- `src/lua_api/workarounds/temporary/color_defaults.rs` — Retail C_UIColor fallback payload publication
- `src/lua_api/globals/real/secure_helpers.rs` — simulator `CreateSecureDelegate` registration
- [Forever state dispatch](../../../tests/forever_secure_state_dispatch.rs) —
  selected vendor handle execution and pre-vendor fallback lifetime
- [Pinned Forever secure handlers](https://github.com/Gethe/wow-ui-source/blob/aff0d8cd53bf4d9ef1c1dd2786b94461a276fb16/Interface/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureHandlers.lua)
  and [restricted frame handles](https://github.com/Gethe/wow-ui-source/blob/aff0d8cd53bf4d9ef1c1dd2786b94461a276fb16/Interface/AddOns/Blizzard_RestrictedAddOnEnvironment/RestrictedFrames.lua)
  — state callback signature, managed execution, `RunAttribute` and frame references
- [Pinned state template](https://github.com/Gethe/wow-ui-source/blob/aff0d8cd53bf4d9ef1c1dd2786b94461a276fb16/Interface/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureHandlerTemplates.xml)
  — vendor-owned callback installation and template-readiness boundary
- [click-binding interaction profile](../../specs/click-binding-interaction-profile.md) — bounded profile contract and current proof

Removed/stale paths that older docs may mention:

- `src/lua_api/globals/security_api.rs`
- `src/lua_api/secure_env.rs`
- `src/lua_api/frame/methods/combat_lockdown.rs`

## See Also

- [[lua-api]] — Lua-facing compatibility surfaces and modeled boundaries
- [[addon-loading]] — dependency secure replay, the shared-library allowlist, and addon load lifecycle
- [[post-load-workaround-audit]] — cleanup restoration ownership
- [[event-system]] — ADDON_ACTION_BLOCKED event firing
- [[frame-data-flow]] — frame is_protected field and Protect() method
- [[protected-frames]] — focused protected-frame enforcement notes
