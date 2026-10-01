# Party ping restrictions

Bounded Retail 12.0.5 `C_PartyInfo.GetRestrictPings` / `SetRestrictPings` contract. The provider registers both methods through existing `src/c_api/c_party_info.rs`, with epoch-gated implementation in `c_party_info/ping_restrictions.rs` over the explicit per-environment numeric enum input. Seven ping fixtures now have saved parent GREEN; independent bounded acceptance is recorded below; no Lua workaround or new backing abstraction is added. See [Lua API state architecture](../lua-api.md).

## Evidence

Final retained [`prose-2026-03-31-169`](../../data/patch-api/sources/12.0.5-register.json), source line 169, states: “The recent restrictions to countdown, ready check, ping and loot method APIs have been loosened to only apply when in chat messaging lockdown rather than in all combat.” Superseded March 25 prose is not this contract.

Profile runtime cache `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua`:

- Line 348: `GetRestrictPings` returns one required `RestrictPingsTo` enum, `restrictTo`.
- Line 579: `SetRestrictPings` takes one required `RestrictPingsTo` enum, `restrictTo`, declares `HasRestrictions = true` and `SecretArguments = "AllowedWhenUntainted"`, and declares no returns.

Current cached `Blizzard_CompactRaidFrames/Mainline/Blizzard_CompactRaidFrameManager.lua:164–180` compares `GetRestrictPings()` with the selected enum, calls `SetRestrictPings(enum)`, and resets a repeated dropdown selection to `Enum.RestrictPingsTo.None`. Four radio options are None=0, Lead=1, Assist=2, TankHealer=3. The reset is caller logic, not setter behavior. Legacy unused mixin code at line 1289 passes `GetChecked()`; its boolean call is not evidence of accepted boolean input and is not emulated.

## What it must do

- [x] Return exactly one required numeric enum from this environment's explicit `party_ping_restriction` input. **INFERRED SIMULATOR DEFAULT:** a fresh environment starts at None=0; native initial/reset policy is unknown.
- [x] Public setter accepts all four enum values, stores the selection, and returns zero values. Repeating a setter selection preserves it; an explicit None argument resets it. Public changes in one environment cannot affect another.
- [x] With explicit [chat messaging lockdown](chat-messaging-lockdown.md) true, reject the setter before mutation; getter remains readable. With lockdown false, setter succeeds. Cover all four combinations of combat and lockdown; combat itself neither blocks nor activates the restriction.
- [x] **SIMULATOR POLICY:** blocked calls fail fast with an explicit nonempty runtime error detectable by `pcall`; exact native error convention/text is unknown. Provider reuses existing `reject_chat_messaging_lockdown` rather than inferring lockdown from combat.
- [x] **STRICT SIMULATOR VALIDATION POLICY:** missing/nil, fractional, unknown numeric, string, boolean, table and function enum arguments reject atomically, without coercion. This is not native malformed-input characterization.
- [x] **CONSERVATIVE SIMULATOR SECURITY POLICY:** reject actual secret enum arguments in both untainted and tainted callers without unwrapping, mutation, declassification or clearing/replacing caller taint. Cached `AllowedWhenUntainted` semantics are **not modeled** by this stricter policy; untainted-secret acceptance remains a known gap.

Input, provider and fixtures are gated by `retail-12-0-5`; no native filtering, permissions, event or native-state producer claim follows. Checked requirements mean independently accepted bounded saved fixture proof, not native parity.

## How it works

- [Chat messaging lockdown input contract](chat-messaging-lockdown.md)
- [Lua API environment/state architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs` — public primitive `u8` input `party_ping_restriction`, explicitly identified as C_PartyInfo backing state.
- `src/lua_api/state.rs` — initializes the epoch-gated input to inferred None=0.
- `tests/party_ping_restrictions.rs` — seven autodiscovered fixtures in existing grouped integration target; no new Cargo target.
- `src/c_api/c_party_info.rs` — registers only the two ping slots in the existing namespace through the epoch-gated helper module.
- `src/c_api/c_party_info/ping_restrictions.rs` — numeric getter, strict public enum setter and conservative secret rejection before mutation.
- `src/c_api/c_chat_info.rs` — existing lockdown guard reused before mutation; unchanged.

## Tests asserting this spec

`tests/party_ping_restrictions.rs`: fresh numeric getter/arity; direct explicit input; four public enum selections/zero setter returns/caller-controlled reset; environment isolation; lockdown atomicity/readability on both combat axes; malformed argument atomicity; actual secret argument rejection with preserved caller taint.

Parent filter: `cargo test --test integration party_ping_restrictions::`.

**Proof ledger, 2026-10-01:** parent compiled RED at `12e4a1a28` (inputs/tests `f71d4d832`, import repair `12e4a1a28`): `/tmp/patch-12.0.5-batch32-red-fixed-build-result.json` exit 0; `-run.log` reports seven selected FAIL, numeric getter assertions and accepted malformed/secret setters. Fixtures establish callable slots: lazy stubs are not absent methods. No concrete previous ping owner was found in simulator source. Initial missing-trait compilation failure is excluded from behavioral RED. No build, test, check or native probe ran in this docs reconciliation.

## Reconciled batch32 parent proof — 2026-10-01

Producer `77ab2785f5544ebed9958fc937c34900b82f2cea` has saved parent compile exit **0** in **213.82s** (`/tmp/patch-12.0.5-batch32-green-build-result.json`, build log/JSONL). Integration executable SHA256 `a3fc155dd66e3c967246e6e2226a58dcceab8dfae1d42b2ece8592e815352e1c` identifies all four runs in `/tmp/patch-12.0.5-batch32-green-runs.json`: **7 ping + 6 ready-check + 5 predicate + 11 group = 29 selected PASS**, each exit0, with actual summaries in `-run-0.log` through `-run-3.log`. Seven ping fixtures establish only the checked bounded policies above; controls do not expand ping coverage.

Parent startup at the same producer is exit **0**, JSON **[]** (`/tmp/patch-12.0.5-batch32-green-startup-run.json`, `-startup.json`, `-startup.log`), executable SHA256 `caec6acd9c8129ae3d5baa9d88370242d578ae7bc1e814ad57722f43f80cdcec`. These are inspected saved parent artifacts, not fresh runs. Full `/tmp/patch-12.0.5-ping-restrictions-independent-proof.md` accepts bounded **29/29 PASS** and startup **0 []**, with source/wiring/security/readability review. Fresh default `cargo fmt --check` exit **0** (24.236441s, `77ab2785f` → `303de9af7`) and `cargo check` exit **0** (63.061756s, `303de9af7` → `ee9984ec6`) retain identical relevant ping/guard/wiring/build and control-fixture hashes matching producer blobs. Concurrent warning fixtures and docs advanced HEAD; this is not immutable whole-tree proof or warning-runtime credit. Checks ledger: `/tmp/patch-12.0.5-ping-independent-checks.json`; scope identities: `-before.json`, `-after.json`, `-final.json`.

Current row169 status: **bounded PREDICATE coverage**, accepted using all four independent family proofs. [Chronological decision and coverage matrix](party-countdown.md#exact-row169-decision--bounded-predicate-acceptance) owns scope, current **260 pending / 88 bounded / 14 partial = 362** accounting and unknown native producer/security/error/permissions/exhaustive API/all-profile limits. Ping evidence is `SetRestrictPings` restriction-setting, not a `C_Ping` action or delivery claim. Earlier pending/count milestones were historical; pending because ping delivery was missing was an assistant-added condition, now withdrawn. This slice alone never established the aggregate decision.

Historical batch32 checkpoint: row169 was audit-pending with ready-check and ping subset proofs; countdown and loot were still open. Accounting then was **264 pending / 84 bounded / 14 partial = 362**; IDs/source SHA unchanged. That snapshot does not describe current aggregate coverage.

## Known gaps (current cycle)

- [x] Compile and execute the seven fixtures as RED before producer implementation (saved parent evidence above).
- [x] Prove the implemented bounded provider with saved parent compilation and seven-fixture GREEN.
- [x] Independent bounded acceptance from the actual verifier report; saved runtime proof plus fresh source-identical fmt/check.
- [ ] Native `AllowedWhenUntainted` secret acceptance and native malformed/blocked error conventions remain unverified.

## Out of scope

- **Ping behavior:** actual ping filtering/content, group-role permissions, event production and native ping-system state/reset are not implemented or claimed.
- **Other named families:** separate countdown, loot-method and [ready-check proofs](chat-lockdown-ready-checks.md) supply aggregate bounded predicate acceptance linked above. Ping proof alone does not establish their behavior or native parity.
- **Native lockdown producer:** activation/reset, encounter/M+/PvP inputs and ordering remain open; explicit test input is not a native producer.
- **Channels/macros:** channel join/leave/mutation, messaging and macro execution restrictions remain open.
- **Other-owned work:** ready-check/predicate code, their specs and audit accounting remain unchanged by this slice; no whole-audit or all-profile acceptance.
