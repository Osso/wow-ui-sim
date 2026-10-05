# 12.0.5 non-sweep pending rows follow-up

Verified: 2026-10-04. Worktree `/home/osso/.worktrees/wow-ui-sim-p1205-pending`, baseline `7f22dc66a`, implementation `71a694e82`. Coverage JSON is unchanged; main session owns ledger decisions. This report supersedes stale implementation observations in [blocked-row evidence](blocked-rows-ledger.md), not its retained historical source material. [Audit page](../../../../docs/wiki/investigations/patch-12-0-5-api-audit.md).

## Result

One row has new bounded unchanged-vendor behavioral proof (099); one has implemented/tested leadership-subset behavior but remains pending in aggregate (074). Remaining rows have specific blockers. No native historical regression or exhaustive restriction-policy claim follows from these fixtures.

## Pinned rilua finding

`Cargo.toml` and `Cargo.lock` pin `a76ffa83f548da94b72777c96d51de38b7f9ef3c`. Read-only source inspection used `/home/osso/.cargo/git/checkouts/rilua-fd5a0715e46b5888/a76ffa8` from commands rooted in the authorized worktree.

- `git merge-base --is-ancestor 690fc3b a76ffa8` exits 0. The reported branch head is an ancestor, but ancestry does not establish capability.
- `git show --stat 690fc3b` identifies `690fc3b8ff8463fc926e1635f6bbded0cf031738`, subject **Wire table security integration suites**. It changes only `tests/integration.rs`, adding eight lines. No execution-limit implementation is in that commit.
- `src/vm/state.rs:401–418` exposes `set_alloc_limit` / `check_alloc_limit`: allocation/GC limits, not execution budgets. `HookState` at 475 has the ordinary instruction-count debug hook counter.
- `src/vm/execute.rs:494–535` calls a Lua debug hook, disabling recursive hooks during the callback. `src/stdlib/debug.rs:1540–1584` exposes `debug.sethook`, including clearing/replacing the hook. This is not trusted addon-budget metering.
- Searches of pinned `src/` and `tests/` found no execution-budget/limit/throttle owner, per-addon accounting, exhaustion policy or scoped exemption primitive. The allocation, call-depth and debug-hook limits do not provide that contract.
- Simulator `ReloadUI` still only dispatches `PLAYER_ENTERING_WORLD(false, true)` (`state_backed_queries.rs:110–117`). `GetScriptBucketThrottleLimits` is a later-epoch zero-valued mock, explicitly not enforcement.

Therefore no execution-limit integration is possible under this pin. 050/095 remain pending. No dependency edits, invented zero budgets, replaceable-hook enforcement or synthetic exemptions were added.

## Per-source coverage matrix

| source_id | Result and exact remaining boundary |
|---|---|
| prose-2026-03-12-038 | **still-pending.** `Ambiguate` now accepts secrets from addons per source. Current `real/ambiguate.rs` returns input unchanged for `none`, but other contexts invoke `string.match` on an opaque VM wrapper. Pinned rilua `Userdata::secret_value` is crate-private (`vm/value.rs:116`) and `unwrap_secret` requires secure caller; opaque string formatting is specialized inside the VM, not a generic host transform API. A secure-only unwrap or clearing addon taint would violate the contract. Need trusted opaque name transformation preserving output secrecy for addon callers. |
| prose-2026-03-12-049 | **still-pending.** Exhaustive cached `UnitTokenNamePlate` inventory is `C_NamePlate.GetNamePlateForUnit`, `C_NamePlateManager.IsNamePlateUnitBehindCamera`, `SetNamePlateSimplified`. Current first provider is a zero-return permanent shim, with no unit-to-frame association; the other consumers lack an equivalent live model. Prose establishes rejected party/raid/target-of-target categories, but not full accepted alias grammar or native rejection shape. Need real association/consumer state plus category and authenticated-secret tests, not a new rejecting shim. |
| prose-2026-03-12-050 | **still-pending.** Pinned VM lacks trusted execution-limit capability; see finding above. Normal-budget denial cannot be produced, so lifecycle exemption/restoration tests would be fictitious. |
| prose-2026-03-12-056 | **still-pending.** Mixed historical/content claim: MOST encounter debuffs were private, displays lacked unspecified parity, nameplates unsupported. `c_unit_aura_classification.rs` has explicit private classification; anchors exist. No epoch-specific encounter population or named display-parity contract establishes these aggregate claims. |
| prose-2026-03-25-074 | **still-pending; leadership subset proven.** Four current modeled namespace successors deny addon-tainted calls during combat without mutation/events, permit secure combat and addon out-of-combat calls, and recover after denial. Tests: `leadership_addon_combat_denial_preserves_roles_leader_events_and_caller`, `leadership_secure_combat_and_addon_out_of_combat_calls_mutate_live_state`. March 31 line169 supersedes combat-only restrictions for countdown/ready-check/ping/loot; their existing lockdown fixtures pass. The three Convert/ConfirmConvert operations have no real provider/category-conversion model; historical global publication differs from current namespaces. Do not reintroduce removed globals or claim aggregate closure. |
| prose-2026-03-25-076 | **still-pending.** `targeting_verbs.rs` has live GUID-keyed marker assignment, but no captured process-lifetime ordering that causes random sticky denial until restart. Repeated happy-path writes are not a reproduction. Need failing lifecycle/permission transition and recovery sequence. |
| prose-2026-03-25-085 | **still-pending.** Same category restriction and three-consumer inventory as 049; no real nameplate association/consumer proof. |
| prose-2026-03-25-093 | **still-pending.** VM table security/freeze errors exist, but source names neither changed operation nor replacement diagnostic. Current pin differs from the old evidence pin; rewording existing errors does not prove the historical diagnostic delta. Need operation-specific authoritative text or native pre/post failures. |
| prose-2026-03-25-095 | **still-pending.** Same actual execution-meter blocker as 050. The old claim that the branch is unavailable is obsolete; the available ancestor contains table-security test wiring, not the requested meter. |
| prose-2026-03-25-099 | **proven-by-test, bounded existing behavior.** Unchanged cached `RestrictedFrames.lua:324–370` is the actual `HANDLE:IsUnderMouse` API, not a new frame-method alias. Tests `parent_under_mouse_uses_rectangle_not_hover_focus`, `recursive_parent_under_mouse_checks_visible_protected_child`, `scaled_parent_under_mouse_normalizes_cursor_by_effective_scale` prove anchored parent success without hovered focus, outside/boundary movement, protected child outside parent, hidden/show child transitions, unprotected-child exclusion and scale normalization. No production/vendor patch. Exact native historical failing frame and pre/post regression remain unknown; main session decides whether this bounded fixture coverage suffices for row credit. |
| prose-2026-03-31-172 | **still-pending.** Cached `EditModeManager.lua:697–717` already uses a secure delegate for secure/out-of-combat calls and direct cleanup for insecure combat calls. `FrameScriptDocumentation.lua:98–114` specifies protected secure-taint invocation and direct argument/results; current simulator delegate is identity. Missing real primitive, creation restriction semantics and untrusted callback/error restoration. A wrapper alone or invented combat selection preservation would not implement the unchanged consumer. |
| prose-2026-04-10-196 | **still-pending.** “Various chat channel APIs” does not inventory the affected membership/moderation operations. Existing channel mutation and explicit messaging-lockdown models do not establish that inventory. `RunMacro`/`RunMacroText` lack a scoped executable Lua /run channel-call provenance boundary. Need affected API list and real scoped macro execution with denial-before-mutation/restoration; no guessed all-channel guard. |
| prose-2026-04-10-200 | **still-pending; applicability audit completed.** Reload-proof taint logging requires VM lifecycle and a persistent logging sink. Simulator only retains `taintLog` CVar defaults; no writer/sink/reload restoration. `ReloadUI` emits an event in the same VM, not a reload. Pinned VM taint tags are real but not a logging pipeline. Need actual VM teardown/recreation and logging ownership; firing PLAYER_ENTERING_WORLD is insufficient. |
| prose-2026-04-10-201 | **still-pending; applicability audit completed.** Correct tainted-table-read log messages require instrumentation of actual VM table reads plus caller/key/slot provenance and a logging sink. Current simulator has no taint logger; pinned `stdlib/taint.rs` implements tags/debug state, not read-log emission. Simulator cannot instrument every Lua GETTABLE through a global wrapper. VM work is outside authorized editable paths. |
| prose-2026-04-10-202 | **still-pending.** Corrected old evidence: `src/c_api/aura_entry.rs` DOES exist and `env_events.rs:246` invokes it. Host-staged IDs are atomically rekeyed; without a staged batch, entry delivers with IDs unchanged. No automatic post-rekey UNIT_AURA notification or loaded cooldown-viewer rebinding/tooltip-identity fixture establishes encounter-entry ordering. Isolated rekey/query authentication is not consumer proof. |
| prose-2026-04-17-215 | **still-pending.** Upstream Classic documentation cleanup/general validation cannot be completed by local simulator edits. Runtime non-secret correction is separately credited by rows213/214; no extra named runtime delta is supplied here. No new Classic build/proof claimed. |
| global api-C_CatalogShop-GetProductInfo-247 | **still-pending.** Real product reads exist (`c_catalog_shop_products.rs:155–179`). Cached `CatalogShopDocumentation.lua:226–239` declares HasRestrictions and AllowedWhenUntainted separately but supplies no execution-restriction predicate. A guessed secure-only/combat/purchase-permission guard would conflate distinct contracts. |
| global api-C_CatalogShop-PurchaseProduct-249 | **still-pending.** Cached declaration at421–433 additionally requires a bool result; current `housing_catalog_state.lua` provider remains no-op. Need real purchase request/state transition plus evidenced restriction predicate. Adding a can-purchase constant/flag does not establish this delta. |
| global api-C_ChatInfo-ReplaceIconAndGroupExpressions-253 | **still-pending; applicability audit completed.** Cached ChatInfo485–499 expressly marks arguments2/3 NeverSecret and input AllowedWhenTainted. Current temporary default returns input unchanged and ignores both flags. Real cached callers use each flag to control icon/group expansion, including TextToSpeech. `ChatFrameConstants.lua` provides icon/group dictionaries but not native full expansion/output-secrecy behavior. Pinned VM lacks exported opaque secret-string transformation; no taint-clearing unwrap or shim-only argument guard was added. Need owned real expansion plus original flag-wrapper rejection before transformations. |
| global api-Localization CreateAbbreviateConfig-413 | **still-pending.** Proxy factory still copies arbitrary initial data. `LocalizationSharedDocumentation.lua:24–38` describes recommended pairs/divisor products and Error failure mode, not the exact restricted accepted/rejected numeric rule. The separate typed AbbreviatedNumberFormatter is not AbbreviateConfig and cannot establish this precondition. Need authoritative numeric triples and shared atomic constructor/setter validation. |
| global api-PlayerScript Ambiguate-416 | **still-pending; applicability audit completed.** Exact AllowedWhenUntainted → AllowedWhenTainted delta needs successful opaque shortening with tainted caller, secret output, nonsecret context validation and caller restoration. Same VM transform boundary as038; existing public-name tests and `none` identity passthrough do not prove shortening of secret names. |
| scriptobjects-AbbreviateConfig-SetAbbreviateNumberData-522 | **still-pending.** Same unknown restricted numeric rule as413; current proxy setter stores arbitrary data. Error failure mode is known, numeric predicate is not. |
| widgets-FontString-GetFont-544 | **still-pending.** Cached FontString121–132 names FontAsset but does not define its Lua representations/canonicalization. Current getter at formatting.rs389–420 returns path string/default. A string roundtrip does not prove FontAsset domain equivalence; no numeric-ID representation was invented. |
| widgets-FontString-SetFont-546 | **still-pending.** Cached FontString500–515 names FontAsset and RequiresValidFontAsset; current setter authenticates arguments then converts/stores string paths. No asset-type definition or native valid/invalid representation probes establish a different domain or atomic validation rule. |

## INFERRED choices

Only new production choice: historical leadership restriction applies to the four current C_PartyInfo successors; denial raises a nonempty runtime error before argument decoding. Secure callers remain permitted, consistent with the source's addon-specific wording. This choice is documented in code and [spec](../../../../docs/specs/patch-12-0-5-pending-leadership.md). No inferred metering, CatalogShop, FontAsset, abbreviation, nameplate-token grammar, or channel-API inventory was added.

## Proof ledger

All commands ran from the authorized worktree; all build-host calls explicitly used local. Test invocation prefix is `python3 scripts/build-host.py --build-host local --test --test integration`; suffix is `-- --nocapture`. No relevant source changes followed GREEN/check/startup proof; later documentation does not invalidate these scopes.

| Revision / scope | Command filter | Result | Artifact |
|---|---|---|---|
| e74034ec9; unchanged restricted-frame/layout/input implementation | patch_12_0_5_pending_mouse:: | 3 PASS | pending-mouse-test-output.txt |
| 20d68671a; production/build/dependency sources identical to baseline | patch_12_0_5_pending_party:: | expected RED: 1 PASS / 1 FAIL; missing combat denial | pending-party-red.txt |
| detached 7f22dc66a, same new party fixture temporarily supplied in this worktree | patch_12_0_5_pending_party:: | same 1 PASS / 1 FAIL; confirms baseline defect, not regression | pending-party-master.txt |
| 71a694e82 | patch_12_0_5_pending_party:: | 2 PASS | pending-patch_12_0_5_pending_party-green.txt |
| 71a694e82 | party_1207_audit:: | 11 PASS | pending-party_1207_audit-green.txt |
| 71a694e82 | party_ping_restrictions:: | 7 PASS | pending-party_ping_restrictions-green.txt |
| 71a694e82 | party_countdown:: | 10 PASS | pending-party_countdown-green.txt |
| 71a694e82 | party_loot_method:: | 13 PASS | pending-party_loot_method-green.txt |
| 71a694e82 | ready_check:: | 0 selected; NOT coverage | pending-ready_check-green.txt |
| 71a694e82 | chat_lockdown_ready_checks:: | 6 PASS | pending-chat_lockdown_ready_checks-green.txt |
| 71a694e82 | p1207_ready_check_auth:: | 3 PASS | pending-p1207_ready_check_auth-green.txt |
| 71a694e82 | cargo fmt --check | exit0 | pending-format.txt |
| 71a694e82 | python3 scripts/build-host.py --build-host local --check | exit0 | pending-check.txt |
| 71a694e82 | python3 scripts/build-host.py --build-host local | build exit0 before timed run | pending-build.txt |
| 71a694e82 | python3 scripts/build-host.py --build-host local --run -- --no-addons --no-saved-vars lua-errors | exit0, JSON [] | pending-startup.txt |

55 selected GREEN tests total (including three earlier mouse tests with unchanged relevant implementation). No publication sweep/enum/full-UI-preload run is claimed. Existing six deprecated Clippy-key manifest warnings in `iced-wgpu-patched/Cargo.toml` occur on baseline and GREEN; vendor edits are prohibited. No new Rust compiler warning or warning suppression. Manual changed-line readability audit found no issue in the new guard or two fixture files. No agents/models/CLIs, push or merge.

## Commits

- `e74034ec9` — Test restricted parent IsUnderMouse geometry.
- `20d68671a` — Add leadership combat restriction fixtures.
- `71a694e82` — Block addon leadership mutations during combat.
- Evidence/spec reconciliation commit follows; inspect branch log for its final hash.
