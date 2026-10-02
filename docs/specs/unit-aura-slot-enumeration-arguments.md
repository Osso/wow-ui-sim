# Unit aura slot enumeration arguments

Batch46 covers only `global api-C_UnitAuras-GetAuraSlots-382`: `C_UnitAuras.GetAuraSlots` / `- arg1 NeverSecret`. Retained [plaintext](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 381–382 and [register](../../data/patch-api/sources/12.0.5-register.json) identify this single consolidated delta. Plaintext SHA256: `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`. This does not change pagination or [slot retrieval](unit-aura-slot-secret-arguments.md).

Actual Retail runtime cache `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:284–310` declares `RequiresUnitAuraAccess=true`, unchanged `SecretArguments="AllowedWhenUntainted"`, required restricted unit token, nullable filter/number maxSlots/number continuationToken, and nullable numeric continuation followed by stride-1 numeric slots. Unit no longer has `NeverSecret`. The declaration has no output secrecy annotation. Architecture: [Lua API wiki](../wiki/systems/lua-api.md).

## What it must do

### Argument authentication

- [ ] Accept authentic host-secret STRING units for untainted callers using their actual payload, including player, seeded party, empty and unknown units. Authenticate unknown units before absence handling. This is the literal row382 removal axis.
- [ ] Apply unchanged AllowedWhenUntainted to all four supplied arguments: secret STRING filter, secret NUMBER maxSlots and continuationToken, individually and mixed with secret unit. Do not bypass authentication because maxSlots is unused or a token would terminate.
- [ ] Authenticate all four arguments before type/representation checks, store lookup or token termination. A tainted call with a malformed public earlier argument and secret later argument must fail at the VM caller guard. Tests match the simulator VM's `untainted caller` diagnostic, not native error wording.
- [ ] Preserve original secret identity/secrecy and caller taint through secure success, tainted denial, same-closure public recovery, and forced GC. Root actual host STRING/NUMBER wrappers before global insertion; do not emulate secrets with Lua tables.

### Inferred representation policy — not native-verified

- [ ] **INFERRED:** unit must be a required actual UTF-8 STRING; no nil default or numeric coercion. Unknown/empty valid strings are lookup misses, not vocabulary errors.
- [ ] **INFERRED:** nullable filter defaults to empty; nonnil filter must be an actual UTF-8 STRING. Do not redesign accepted filter vocabulary.
- [ ] **INFERRED:** nullable maxSlots/token must be actual finite f64 NUMBERs; reject numeric strings, booleans, tables, functions, NaN and infinities even for unknown units or supplied terminating tokens. Accept zero, negative, fractional and large finite values without an integral/positive cap. Securely authenticated NaN still fails validation.

### Retained enumeration, not native pagination

- [ ] Return exactly nil continuation plus every visible stored aura instance ID in stored order, including maxSlots=1. IDs are nonordinal; duplicate names do not merge records. Unknown/empty units or empty visible selection return exactly **one nil**, not zero values.
- [ ] Any nonnil valid finite token returns exactly **zero values**, including zero, negative, fractional and large tokens. No new pagination requirement is introduced by row382; maxSlots remains unused after authentication/validation.
- [ ] Preserve the actual existing filter collector: uppercase substring precedence MAW → EXTERNAL_DEFENSIVE → HARMFUL → otherwise Helpful. MAW/external select nothing; `PLAYER` and `RAID` do not add source/raid filtering on this path. Player uses polarity within its buff store; seeded party uses the buff/debuff stores. This is narrower than [instance enumeration](unit-aura-instance-enumeration.md), whose additional filtering is not imported here.
- [ ] Blocked IDs disappear and compact the visible tuple; returned IDs round-trip through real C slot getter DTOs. Blocked records remain C-slot-retrievable. Preserve complete DTO fields, source normalization, flags and nullable dispel value.
- [ ] Preserve C enumeration/getter independence from the AuraUtil provider switch. Query success, denial, miss and token termination must not alter block-table identity/content, provider flag, stored AuraInfo records or returned tuple/DTO ownership. Two environments isolate block/provider/secret globals; mutating a captured tuple or DTO does not change subsequent results.

## How it works

- [Lua API wiki](../wiki/systems/lua-api.md)
- [Lua API architecture](../lua-api.md)
- [Separate slot getter contract](unit-aura-slot-secret-arguments.md)

## Implementation inventory

- `src/c_api/c_unit_aura_slot_enumeration.rs` — sole `retail-12-0-5` GetAuraSlots boundary: VM-authenticate all four arguments first, then strict strings/nullable finite f64 validation and retained one-batch output.
- `src/c_api/mod.rs`, `src/lua_api/globals/register.rs` — epoch-gated module/registration after existing aura namespace/state initialization.
- `src/lua_api/globals/auras.rs` — original body/registration earlier-epoch-only, without runtime fallback; only collector visibility becomes `pub(crate)`. Collector, predicates, DTOs, stores and provider behavior unchanged.
- Original wrappers stay rooted on the VM argument stack. `unwrap_secret(&LuaState, Val)` reads actual payloads through the VM caller guard without allocating, replacing wrappers or changing caller taint; strings are copied before mutable collector/output work.
- `build.rs` — existing top-level test discovery groups this fixture into the integration harness; no new target or Cargo change.
- `tests/unit_aura_slot_enumeration_arguments.rs` — retail-12-0-5-only concrete fixtures.
- Cached `Blizzard_FrameXMLUtil/AuraUtil.lua:85–125` — inspected provider-dispatched GetAuraSlots/GetAuraDataBySlot continuation handshake; no vendor edit.

## Tests asserting this spec

All 12 unchanged fixtures live in `tests/unit_aura_slot_enumeration_arguments.rs`. Saved parent pre-producer RED below covers this matrix; this implementation slice runs formatting only. GREEN/independent acceptance remain parent-owned.

| Fixture | Exact capability | Proof level |
|---|---|---|
| `player_batches_keep_nonordinal_duplicate_named_ids_and_dto_roundtrip` | Player IDs307/811 helpful,419/907 harmful, exact arity/defaults/DTO/source | Saved RED PASS (control) |
| `seeded_party_batches_retain_filter_substrings_without_player_or_raid_redesign` | Party IDs1207/1801 helpful,1409/1907 harmful; actual substring selection/empty domains | Saved RED PASS (control) |
| `blocked_ids_compact_visible_tuple_but_c_slot_getter_remains_blocked_inclusive` | Block compaction, empty batch, getter contrast | Saved RED PASS (control) |
| `provider_switch_only_changes_aura_util_provider_not_c_enumeration` | Real provider switch/reset vs C enumeration/getter | Saved RED PASS (control) |
| `inferred_required_utf8_unit_and_optional_actual_string_filter_validate_before_absence` | Required strict unit/filter, UTF-8, unknown/empty one-nil tuple | Saved RED FAIL; policy inferred |
| `inferred_optional_finite_f64_controls_retain_unused_max_and_any_token_termination` | Finite actual numeric controls, negatives/fractions/large, ignored max, zero-return token | Saved RED FAIL; representation inferred |
| `secure_authentic_secret_units_enumerate_actual_store_and_unknown_before_absence` | Literal row382 unit secret acceptance | Saved RED FAIL |
| `secure_optional_secret_filter_max_token_and_mixed_arguments_keep_payloads` | All optional secret inputs, mixed units, secure invalid numeric payload | Saved RED FAIL |
| `tainted_each_argument_and_mixed_secrets_deny_with_public_recovery_in_same_closure` | Each argument caller guard, unknown/NaN/token, same-closure recovery | Saved RED FAIL |
| `all_four_authenticate_before_wrong_public_representation_or_token_short_circuit` | Malformed public earlier argument + later secret guard precedence | Saved RED FAIL |
| `rooted_secret_identity_and_caller_taint_survive_forced_gc_roundtrips` | Rooted host secrets, rawequal, secure/tainted/secure GC lifecycle | Saved RED FAIL |
| `enumeration_preserves_records_tuple_dto_block_provider_and_two_environment_isolation` | Record/block/provider/tuple/DTO immutability, two-env isolation | Saved RED FAIL |

## Saved parent RED — pre-producer

Actual fixture commit (resolved by `git rev-parse 5fb039e8f^{commit}`) and saved artifact revision: `5fb039e8f0dc5cd9e688553fb94b1cd939f0537b`. `/tmp/patch-12.0.5-batch46-red-run.json`, `.stdout`, `.stderr` record **4 PASS / 8 FAIL**, exit101/2.847786s. Four retained controls pass; strict representation, actual secret payloads, guard-before-malformed/token and GC/state fixtures genuinely fail under the old producer.

Exact command: `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c unit_aura_slot_enumeration_arguments:: --test-threads=1`. Binary SHA256 `02889f968c43086c0ba2dbc0fa6764769c71a42e16bc9a3368fd95663b705f96`; same compilation as `/tmp/patch-12.0.5-batch45-green-build-result.json`: `cargo test --test integration --no-run --message-format=json`, exit0/233.146580s. Saved proof includes preserved unowned dirty source (diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`), not clean-revision proof. No rerun or credit inferred from that control build.

## Known gaps (current cycle)

- [ ] Parent owns combined batch45 corrected GREEN + batch46 GREEN and one independent acceptance gate. No tests, builds, checks, readability gates, operations or delegation performed in this implementation slice. All contract checkboxes remain unverified by GREEN.
- [ ] Exact row382 accounting remains unchanged; fixture existence is not behavioral completion or clean-revision proof.

## Out of scope

- Native permission enforcement (`RequiresUnitAuraAccess`), restricted output secrecy, exact native validation/errors and native secret acceptance parity: annotations alone do not prove those behaviors.
- Native pagination/max-slot fidelity: retained one-batch/any-token termination is expressly simulator behavior, not a claim about native pages.
- Native cached AuraUtil consumer closure: inspected Lua forwards four arguments through a data provider and retrieves DTOs by slots; current Rust AuraUtil.ForEachAura directly visits the store and does not establish that handshake. No consumer closure claim or optional consumer fixture.
- Other source rows, indexed/display/duration APIs, Cargo/new targets, audit accounting/PLAN, dispel-color producer/spec/corrected tests and independently owned slot-acceptance work. Only this contract, minimal Lua API wiki inventory/index/log and enumeration producer wiring change.
