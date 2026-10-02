# Private aura sound removal

Batch49 covers only exact row `global api-C_UnitAuras-RemovePrivateAuraAppliedSound-403` (`- HasRestrictions`) in the [12.0.5 source register](../../data/patch-api/sources/12.0.5-register.json). C API-owned host inputs live in `src/c_api/private_aura_sounds.rs`; fixtures describe a bounded removal transition. The bounded producer registers one removal callback under legacy and epoch-gated modern names; saved combined parent GREEN is recorded below; independent389 verification and acceptance remain pending. Architecture reference: [C API boundary](../wiki/systems/lua-api.md).

## Evidence and inference

The source row removes restrictions from the legacy remover. March 31 source prose (line168) separately retains conditional restrictions on Add; this slice does not change Add. Later cached Retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua` names `RemoveAuraSound`, declares a required numeric `auraSoundID`, no returns, and `SecretArguments = "AllowedWhenUntainted"`. This is cached later-client evidence, **not native 12.0.5 verification**.

Cached Retail `Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua` gates on `GetCVarBool("loadDeprecationFallbacks")` and assigns `C_UnitAuras.RemovePrivateAuraAppliedSound = C_UnitAuras.RemoveAuraSound`. The simulator [CVar defaults](../../src/cvars.yaml) set that CVar to `1`. Executing this actual file after bootstrap can erase legacy-only publication. Thus producer publication exposes legacy under `retail-12-0-5` and modern under `retail-12-1-0`, using the same canonical callback and live ID set. Modern publication is required solely for alias durability, not an expanded audio feature.

The representation (`HashSet<u32>`), inclusive ID domain, missing/repeated-ID behavior, strict validation and conservative secret rejection are **INFERRED simulator policies**, not native semantics. Zero-result removal is supported by later cached documentation but remains unverified for native 12.0.5. Native secret acceptance/access permissions, native error wording, lifecycle and side effects are unknown. No native probe gate applies to this approved bounded model.

## What it must do

### Inputs and transition — INFERRED

- [ ] Start each environment with an empty host-declared live ID set; never fabricate acquired IDs.
- [ ] Accept only an actual public finite integral `u32` NUMBER, including `0` and `4294967295`; no string coercion or spell-alias lookup.
- [ ] Remove an existing host-seeded ID immediately and return exactly zero Lua values; preserve all other IDs.
- [ ] Treat unknown and repeatedly removed valid IDs as zero-result no-ops.
- [ ] Observe host replacement immediately; isolate environments and preserve caller tables, spell aliases, cooldown associations and classification flags.
- [ ] Reject missing, nil, STRING (including `'101'`), bool, plain table, actual FrameTable, function, thread, NaN, infinities, negative, fractional and out-of-range inputs with contextual public errors, no ID mutation and successful subsequent public recovery.

### Security — bounded policy, not native permission parity

- [ ] Ordinary public tainted calls remove live IDs, preserve caller taint and require no caller/combat gate, reflecting the literal restriction-removal row.
- [ ] **INFERRED:** Authenticate secret arguments through actual VM security before type/key access; reject them even in secure context without unwrapping or declassifying their payloads.
- [ ] **INFERRED:** Rooted actual secret NUMBERs (known/unknown), STRINGs (numeric hit/unknown), and a wrapped real frame remain secret and identity-stable through GC and rejection in secure and stamped tainted closures; errors remain public, state unchanged, taint preserved and public recovery works.

### Publication durability

- [ ] Producer publishes legacy at `retail-12-0-5`; modern exists only from `retail-12-1-0`. Both remove from the same ID set with zero results.
- [ ] Under `retail-12-1-0`, executing the complete actual unmodified cached Shared deprecated file after bootstrap with default CVar `1` leaves legacy removal functional, including actual public tainted removal and host-observed state transition.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/c_api/private_aura_sounds.rs` — unchanged default-empty host input struct plus strict public ID reader, one canonical zero-result removal callback and legacy/modern registration.
- `src/lua_api/globals/register.rs` — epoch-gated registration after existing aura surfaces.
- `src/c_api/mod.rs` — public module gated by `retail-12-0-5`.
- `src/lua_api/state/sim_state.rs` — per-environment gated field beside classification/cooldown inputs.
- `src/lua_api/state.rs` — gated empty default initializer.

## Tests asserting this spec

`tests/private_aura_sound_removal.rs`: 13 grouped fixtures (11 at `retail-12-0-5`, two additional publication fixtures at `retail-12-1-0`), discovered by the existing generated integration harness; no new Cargo target.

The publication fixture reads `$HOME/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua` in full and calls `WowLuaEnv::exec_named` with the actual path as chunk name after `WowLuaEnv::new()` bootstrap. Missing cache fails explicitly. No copied alias snippet, synthetic vendor function, namespace substitution, API replacement or fallback is used.

Prerequisites asserted before loading: default `GetCVarBool('loadDeprecationFallbacks')`, real `C_UnitAuras` and `C_SpecializationInfo` tables, existing `Enum.UnitAuraSoundTrigger.Added`, and existing `Enum.CustomAuraButtonDispelTypeTextureStyle.BorderWithIcon`/`PreserveAsset`. The other vendor functions are only defined, not invoked; they need no Add or paper-doll implementation. Its final inspect-specialization assignment may read nil without preventing this chunk's removal alias. Fixture frame arguments use actual `CreateFrame('Frame')` behavior and table representation, never a userdata-shape guess.

### Saved parent compiled RED and producer boundary — 2026-10-02

Input revision `c25a90793aded2f1400c4d59b046212d696de335`. Parent artifacts `/tmp/patch-12.0.5-batch49-red-build-result.json` and `/tmp/patch-12.0.5-batch49-red-run.json` record:

- Compile: `cargo test --test integration --no-run --message-format=json`, exit `0`, `368.9406039819587s`.
- Run: `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c private_aura_sound_removal:: --test-threads=1`, exit `101`, `3.1412803089478984s`; **13 selected, 0 PASS / 13 FAIL**, 9544 filtered out.
- Binary SHA256 `80e0ba9f465061e9e66ab4c91bca4b8854ff7e8af154e03374655c6bfec8af7c`; dirty-source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`. Preserved unowned duration changes mean this is not clean-revision proof.

Full outputs: `/tmp/patch-12.0.5-batch49-red-build.jsonl`, `/tmp/patch-12.0.5-batch49-red-build.log`, `/tmp/patch-12.0.5-batch49-red-run.stdout`, `/tmp/patch-12.0.5-batch49-red-run.stderr`. Saved output shows the actual cached Deprecated chunk and prerequisites succeeded before post-alias zero-result failure; direct modern zero-result assertion also failed. Actual rooted secret/frame setup reached required rejection failures. Parent inspected these boundaries and found no invalid fixture; fixtures remain unchanged.

Producer implementation follows this genuine RED. Both names use the same canonical callback; contextual errors deliberately name legacy `C_UnitAuras.RemovePrivateAuraAppliedSound`, including modern calls, without claiming native wording. Ordinary public calls have no caller/combat gate; secrets are conservatively rejected before type/key access, without unwrap, private-payload inspection or taint mutation. Only current environment live-ID removal is modeled.

No producer GREEN, build/tests/check/lint/readability/coverage/deployment/acceptance was run by this implementation slice. Parent owns subsequent proof. All requirements remain unchecked; no row/accounting acceptance follows.

### Reconciled combined batch49/50 parent GREEN — 2026-10-02

This is the combined proof SSOT; [space-limit scope](string-util-space-limit-security.md) links here rather than duplicating hashes or test names. Sound inputs `c25a90793aded2f1400c4d59b046212d696de335`, producer `100b4d5bb6b5d30cfa5f6ea1dca15ad15c10f387`, and batch50 fixtures/proof revision `5ee8e7327b7ebf4f6827cb9e4e7f0b9b3e04b58c` identify the bounded work. Existing `c_string_util` is unchanged: eight fixtures were already GREEN, not fabricated RED or grounds for a production rewrite. The earlier no-GREEN producer checkpoint is historical; genuine sound RED remains above.

Saved artifacts (read in full, not rerun): `/tmp/patch-12.0.5-batch49-50-green-build-result.json`, `-green-runs.json`, and `-green-startup-run.json` under the same prefix own exact argv, revisions, timing and binary hash records. Full compilation output is `/tmp/patch-12.0.5-batch49-50-green-build.jsonl` and `-green-build.log`; exact test outputs are `-green-run-0.stdout`/`.stderr` through `-green-run-18.stdout`/`.stderr`, in JSON filter order. Startup output is `-green-startup.stdout`/`.stderr`. Full artifact references retain all individual names and hashes without duplicating them here.

- Compile `cargo test --test integration --no-run --message-format=json`: exit0, `196.38996677508112s`.
- Each saved test command is `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c <filter> --test-threads=1`; all19 exits0, **13 sound +8 space-limit +248 controls =269 unique PASS**, no repeated names. Ordered exact filters and summaries live in `-green-runs.json`; stdout confirms the individual results.
- Integration binary SHA256 `404cab9e893e84829b7b0609ae56e00d231ce28e59ef5d37aba7f06e5df9c8ce`. Startup command `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/wow-sim --no-addons --no-saved-vars lua-errors`: exit0, exact stdout `[]\n`, `4.87017765303608s`; binary SHA256 `6b2040fc74259e399510c58ad7f575a94e2fb254df775039efa1550eabd61f68`.
- Saved dirty-source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a` binds preserved unowned changes: **not clean-revision proof**. Source diff contents were not accessed by this docs update. Actual test runtime `40.32936095423065s`, below60 target; no padding or reruns. Startup and compilation are separately timed.

| Boundary | Saved proof | Limit |
|---|---|---|
| Sound host-state removal, inclusive-u32 endpoints, unknown/repeat, isolation/replacement, invalid recovery | 13 sound PASS collectively | Map/domain/no-op policy INFERRED; no acquired-ID lifecycle |
| Real public tainted modeled-combat removal and rooted secure/tainted secret rejection, taint preservation | Sound fixtures PASS | Conservative rejection INFERRED; no native permissions/error parity |
| Actual complete cached Deprecated file, prerequisites/default CVar, durable legacy/modern canonical callback | Publication fixtures PASS after genuine RED post-alias arity failure | Later cached evidence; modern1210 only for alias durability, no Add/playback scope |
| Exact row328 arg2 NeverSecret, opaque secret rejection, GC/state/recovery, byte-preserving ASCII matrix | 8 existing-provider PASS | Raw-NUMBER validation/rejection, not payload authentication or native arg1/result/AllowedWhenTainted permission proof |
| Adjacent controls/startup | 248 unique control PASS; startup0 `[]` | Bounded development partition only |

Independent389 remains pending. No checked requirements, row acceptance, source coverage/accounting, final whole-page/goal or native acceptance follows. No new executions or temporary ledger were produced.

## Known gaps (current cycle)

- [ ] Independent389 verification and parent-owned acceptance of the minimal producer and existing space-limit behavior; saved GREEN is bounded development evidence only.
- [ ] Add acquisition remains unmodeled; host-seeded removal does not establish Add-to-Remove lifecycle.

## Out of scope

Add providers, audio/playback, catalog, listeners, counters, sound records, native encounter matrices, native secret permissions/error parity, CVar0 coverage, other API rows, coverage accounting and full-page acceptance. None are needed for this host-seeded remover or required later alias durability.
