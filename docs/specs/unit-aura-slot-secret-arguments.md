# Unit aura slot secret arguments

Bounded Retail 12.0.5 contract for **row376 only**, `global api-C_UnitAuras-GetAuraDataBySlot-376`: `SecretArguments AllowedWhenTainted -> AllowedWhenUntainted`. Fixtures target `C_UnitAuras.GetAuraDataBySlot` over existing player/party stores. The bounded C API producer implements the argument boundary; lookup/DTO/store behavior is unchanged. [Lua API system](../wiki/systems/lua-api.md#retail-1205-aura-slot-arguments) describes ownership. Parent compiled RED precedes this producer; [saved parent GREEN](#reconciled-batch44-parent-green--2026-10-02) establishes bounded executed coverage. Parent accepts independent353 for exact row376; chosen behavior checkboxes below are proven bounded simulator behavior, not native parity. No native parity claim.

## What it must do

### Source-grounded argument boundary

Retained [source extract](../../data/patch-api/sources/12.0.5-api-changes.txt), lines375–376, and [register](../../data/patch-api/sources/12.0.5-register.json) identify the single caller-policy delta. Source plaintext SHA256: `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`.

Actual retail cache: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua`, lines207–222; cached file SHA256 `39405809f92e74a945034529d2a2962ebebe9a70fe581b2f411586fc2b72d5d5`. Complete declaration: required `unit: UnitTokenRestrictedForAddOns`, **`NeverSecret = true`**; required `slot: number`; one nullable `AuraData` return; `SecretArguments = "AllowedWhenUntainted"`, `RequiresUnitAuraAccess = true`, `SecretWhenUnitAuraRestricted = true`.

- [x] Reject authentic secret STRING unit even for secure callers: this declaration retains NeverSecret. Indexed-row unit-marker removals do not apply here. Recognize actual VM secrets; do not treat arbitrary userdata as a secret marker.
- [x] Authenticate authentic secret NUMBER slot for untainted callers and use its actual stored ID value; tainted callers must reject secret slot before lookup, including unknown units/IDs. Do not declassify the original input or reset caller taint.
- [x] Reject secret unit independently of slot security, including combined secret arguments and secret unknown-unit strings. Public populated/missing-result recovery must succeed inside the same tainted closure.

### Inferred representations; retained modeled lookup

- [x] **Inferred:** require an actual string unit and finite integral signed-i32 number slot. Reject missing/nil required arguments, wrong types, numeric coercion, numeric-string slot, fractions, nonfinite and out-of-range values before lookup, including unknown unit. Native validation ordering and exact error messages are unproved; fixtures require a nonempty error, not particular wording.
- [x] **Inferred representation/miss policy:** negative/zero/past-unknown signed-i32 slots and unknown units return exactly one nil. No invented positive-only restriction or cap. Current slots are 1:1 `AuraInfo.aura_instance_id`, not list ordinals.
- [x] Preserve actual `GetAuraSlots` roundtrip for player/party helpful and harmful records. Current enumeration returns all visible slots in one batch with nil continuation even when requested batch size is one; this is a retained control, not new pagination/API coverage.
- [x] Preserve blocked-inclusive slot lookup: enumeration omits blocked records but slot retrieval still finds them. Preserve existing helpful-then-harmful traversal across player buffs and seeded party buff/debuff stores, without changing shared helpers.
- [x] Switching the AuraUtil provider suppresses its instance-ID query, not C slot retrieval; query paths must leave provider state unchanged.
- [x] Preserve complete existing DTO fields: identity, count aliases, timing, source, polarity/player-source flags, remaining boolean flags, nilable dispel name and independent empty points table. Player DTO source remains normalized to `player` despite stored pet/party1 source; party DTO source remains stored pet/party1. DTO mutations must not affect later reads or records.

### Security lifetime and isolation

- [x] Host-created STRING/NUMBER secrets are rooted before global insertion and survive forced GC with identical references/security. Secure → tainted denial/public recovery → secure queries preserve original input secrecy and caller taint; tainted `secretunwrap` remains denied.
- [x] Across successes, misses, malformed inputs, secure secret slots and tainted denials, all stored aura fields/order and block-table identity/content remain unchanged. Block/provider/global secret roots remain isolated per environment.

## How it works

- [Lua API system](../wiki/systems/lua-api.md).
- [Aura classification flags](aura-classification-flags.md): existing DTO context.
- [Indexed aura arguments](unit-aura-index-secret-arguments.md): separate contract; unit NeverSecret removal does **not** transfer to slots.

## Implementation inventory

Producer inventory over unchanged fixture commit `fc84ffc0203741b1db9961cbc53b24804aa97af4`:

- [`src/c_api/c_unit_aura_slot_query.rs`](../../src/c_api/c_unit_aura_slot_query.rs): sole `retail-12-0-5` slot boundary; VM `is_secret_value` rejects NeverSecret unit before strict UTF-8 string decoding; VM `unwrap_secret` authenticates slot before strict finite integral signed-i32 validation. Both arguments validate before lookup, with no taint/input mutation.
- `src/c_api/mod.rs` and `src/lua_api/globals/register.rs`: epoch-gated declaration and registration after aura namespace/state initialization, alongside indexed query registration.
- `src/lua_api/globals/auras.rs`: original slot provider/registration now earlier-epoch-only, not a fallback. `push_aura_by_instance_id` gains only crate visibility; blocked-inclusive `find_aura_by_instance_id`, helpful-then-harmful collection and `build_aura_table` remain unchanged. `GetAuraSlots` uses visible collection; AuraUtil alone checks provider switch.
- `tests/unit_aura_slot_secret_arguments.rs`: new epoch-gated fixtures, automatically discovered by existing grouped integration harness; no Cargo/build/registration edits.
- `docs/specs/unit-aura-slot-secret-arguments.md`: row identity, contract, inference labels and proof boundary.

## Tests asserting this spec

`tests/unit_aura_slot_secret_arguments.rs`: **12 concrete fixtures**, existing `integration` target, filter `unit_aura_slot_secret_arguments::`, requires `retail-12-0-5`. Producer slice ran only owned-file formatting; saved parent compiled GREEN is reconciled below, with independent353 now parent-accepted below.

| Fixture(s) | Exact coverage | Proof level |
| --- | --- | --- |
| `player_enumerated_instance_slots_keep_full_dto_and_single_batch`, `party_enumerated_helpful_harmful_slots_keep_full_dto` | Actual enumerated IDs101/102/103/104 and201/202/203/204, both polarities, nonordinal/cross-unit misses, full DTO incl. dispel nilability, nil continuation/current one batch | Saved parent RED PASS; retained controls |
| `blocked_records_disappear_from_enumeration_but_remain_slot_retrievable`, `switched_aura_util_provider_does_not_disable_c_slot_lookup`, `valid_signed_i32_misses_return_exactly_one_nil_without_positive_cap` | Blocked101/104/201/204, visible later slots, switched AuraUtil control, signed extremes/zero/unknown exact nullable arity | Saved parent RED PASS; retained lookup plus inferred miss domain |
| `required_unit_is_actual_string_without_default_or_numeric_coercion`, `required_slot_is_finite_integral_signed_i32_before_unknown_lookup` | Required/missing/nil/types/coercion, fractions/nonfinite/range, validation before unknown-unit result | Saved parent RED FAIL; representation inferred |
| `secure_authentic_secret_slot_numbers_use_actual_stored_ids`, `never_secret_unit_rejects_authentic_secret_strings_even_when_secure`, `tainted_secret_denials_precede_lookup_and_public_recovery_preserves_taint`, `gc_rooted_secret_identity_survives_secure_tainted_public_secure_roundtrip` | Authentic secret IDs101/103/201/203/99999, secure populated/missing lookup, NeverSecret unit secure+tainted rejection, same-closure public recovery, input/taint preservation, rooted GC identity | Saved parent RED FAIL; authentic NUMBER slots rejected by old conversion |
| `query_paths_preserve_records_dto_block_provider_and_environment_isolation` | All stored fields/order, independent DTO/points mutation, block identity/content, provider state, per-env isolation across public/error/secret/tainted paths | Saved parent RED FAIL at old secret-slot conversion; state acceptance pending |

## Saved batch44 RED — 2026-10-02

Date correction: actual RED and compilation commands occurred October 2, 2026; the prior October 1 heading was incorrect.

At unchanged fixture commit `fc84ffc0203741b1db9961cbc53b24804aa97af4`, parent reports compilation exit0 in258.470s. Saved run `/tmp/patch-12.0.5-batch44-red-run.{json,stdout,stderr}` records actual12 selected, **5 PASS / 7 FAIL**, exit101, 1.908s. Enumeration/fullDTO, provider, blocked retrieval and signed miss controls PASS. Required-unit/slot cases fail with `slot argument rejection`; secure authentic NUMBER slot and dependent security/lifetime/state cases fail with `expected number, got userdata at argument 2`. No demonstrably invalid fixture identified; tests unchanged.

Run command: `timeout 90 target/debug/deps/integration-a11e89d240f9bd0c unit_aura_slot_secret_arguments:: --test-threads=1`. Executable SHA256 `84f1226fc85c07909aefbe8751292913ca4115495f6df4821c5632dfede22ef9`. Parent manifest binds fixture revision plus preserved unowned dirty duration scope (diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`), not clean-revision proof. That artifact is parent evidence only; disputed duration source was not read or changed by this slice.

## Reconciled batch44 parent GREEN — 2026-10-02

Saved parent commands occurred October 2, 2026. Producer `054525aff9736be762f69fe7479b16e8383d48e1`, wiki follow-up/tested revision `a92d0a71702faa5b8e7823c373aa11c35ed97e7c`, unchanged fixture `fc84ffc0203741b1db9961cbc53b24804aa97af4`. Compilation and execution bind that revision **plus preserved unowned dirty source**, diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`; not clean-revision proof. Unowned duration source and its diff artifact were not read, edited, formatted or staged by this reconciliation.

Saved compile command: `cargo test --test integration --no-run --message-format=json`; exit0, **270.583s**. Manifest `/tmp/patch-12.0.5-batch44-green-build-result.json`; Cargo output `/tmp/patch-12.0.5-batch44-green-build.jsonl` and `.log`; parent source provenance `/tmp/patch-12.0.5-batch44-green-build-source-diff.txt` (reference only). Integration executable `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c`, SHA256 `3e416f486a13ff23ceb3f15c16351ba08434666867bb6f0b1cafa1679d0d7af9`.

Each saved serial command is `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c <filter> --test-threads=1`. Exact argv/provenance/results: `/tmp/patch-12.0.5-batch44-green-runs.json`; full stdout/stderr: `/tmp/patch-12.0.5-batch44-green-run-<n>.stdout` and `.stderr`, with n matching this table. **106 unique PASS; all seven exit0**, no ignored tests.

| n / filter | PASS | Seconds | Concrete capability / proof boundary |
| --- | ---: | ---: | --- |
| 0 / `unit_aura_slot_secret_arguments::` | 12 | 4.866 | All twelve named fixtures above GREEN: strict required unit/slot before unknown lookup; secure authentic NUMBER slot IDs and misses; NeverSecret unit secure/tainted denial; tainted public recovery; rooted GC identity/taint/secrecy; record/DTO/points/block/provider/environment isolation. Retained player/party enumeration/full DTO, blocked-inclusive lookup and signed misses also PASS. |
| 1 / `aura_application_display_count::` | 14 | 5.612 | Display-count regression controls; no transfer of display-count accounting/acceptance to row376. |
| 2 / `next125aura::` | 12 | 4.084 | Indexed aura argument regression controls. |
| 3 / `unit_aura_filter_query::` | 14 | 4.268 | Instance filter-query regression controls. |
| 4 / `aura_table_shape::` | 7 | 2.518 | Existing aura DTO shape controls. |
| 5 / `aura_api::` | 29 | 8.856 | Existing aura API controls. |
| 6 / `admin_buff_api::` | 18 | 3.232 | Existing admin buff controls. |

The earlier fixture matrix records historical RED; every listed fixture now has saved parent GREEN, now accepted under independent353 below. Argument caller-policy/NeverSecret declarations are source-grounded and authentic VM-secret behavior is simulator-executed. Strict representations, signed/no-positive-cap miss policy, validation ordering and traversal/DTO policies remain inferred or retained modeled behavior, not native-verified semantics. Current one-batch enumeration does not prove pagination.

Saved startup command: `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/wow-sim --no-addons --no-saved-vars lua-errors`; exit0, stdout `[]`, **13.061s**. Executable SHA256 `08427777550109036c6066cf0d60a8a1cec8cd3996d037b3351b96a5cc9f79d2`. Manifest `/tmp/patch-12.0.5-batch44-green-startup-run.json`; full outputs `/tmp/patch-12.0.5-batch44-green-startup.stdout` and `.stderr`. Startup establishes this selected no-addon/no-saved-vars loading path only.

Independent verifier remains active; security/readability/Rust gates and parent acceptance are **pending**, not supplied by these saved runtime runs. Native access permission, restricted output secrecy, exact errors/validation order and earlier/all-profile execution remain unproved. **Row376 pending; totals unchanged: 242 pending / 106 bounded / 14 partial = 362.** No accounting, PLAN or data modification. No new build/test/check/delegation run for this docs-only reconciliation.

## Independent bounded acceptance — 2026-10-02

Parent fully reviewed and accepts independent353, `/tmp/patch-12.0.5-aura-slot-independent-proof.md`, for **exact row376 only**. This supersedes historical pending checkpoints above. Source plaintext SHA256 `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`; register SHA256 `eaea58ae8adf215587cea6de12349b3586fcb2520a4c8aefd4d7cee5406046ed`; cached declaration SHA256 `39405809f92e74a945034529d2a2962ebebe9a70fe581b2f411586fc2b72d5d5`. Unit retains NeverSecret; only slot AllowedWhenUntainted delta is credited.

Full ten saved runs: **150 unique PASS =12 slot +138 controls**, all exits0. Runs0–6 above supply106; supplemental7 `aura_refresh_duration::`18/3.7817744370549917s,8 `aura_spell_identifier::`12/2.4825490301009268s,9 `c_unit_auras_admin::`14/2.5201916430378333s. Same exact bound executable/argv and artifact pattern above; no duplicates or zero-selection runs. Runtime42.221559187048115s +startup13.061459569027647s =**55.28301875607576s**, below60–300s target. Retain missed target: no padding, reruns or invented minimum-duration pass. Bounded development acceptance **is not final whole-goal acceptance**.

Producer `054525aff9736be762f69fe7479b16e8383d48e1`, compiled `a92d0a71702faa5b8e7823c373aa11c35ed97e7c`, fixtures `fc84ffc0203741b1db9961cbc53b24804aa97af4`; binary/startup hashes above unchanged. Dirty source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`: dirty-combined proof, never clean revision. Later batch45 `tests/c_unit_auras_admin.rs` edits are NOT covered by earlier binary controls; later combined diff `b0ba04f53034bc0f279fb983419713c785f05aa0142a7e2636cf1c634636c70b` is not execution/check proof. Two historical plain CreateColor successes violate curve-input contract: old public DTO/color observations only, **not row378 evidence**.

Fresh gates recorded by353: scoped `rustfmt --check --edition 2024 --config skip_children=true`0/0.03640089696273208s; dirty-combined `cargo check`0/22.98355918098241s; global `cargo fmt --check`1/19.20256610191427s on preserved unowned source. No global-format pass. Check stderr SHA256 `61fcf5ae4997c49f88bfa9354b4112f84349cebeefa3fadb6b85342f0109fa00`; global-fmt stdout SHA256 `6ffbeac5e67d2dc2d7a6d37c4cbe48c6e478491a62f1ac5860fc549598f437b5`. Full commands/output hashes and82/82 artifact checks: `/tmp/patch-12.0.5-aura-slot-independent/{gates,artifact-audit,binding-conclusions}.json`; source bindings before/after/final and compiled fixture copies retained there. Actual pinned VM `6044544b960cd68b4b0c58bb3373412757c2caee`; security/wiring/static audit accepted, not native-client verification.

Owned file SHA256: slot producer `a756fb0589ccfb537767547f1104d1fade10f5d893d11c640307d34bf12f2102`; c_api/mod `dc07f07449b9ebac5b7a3a6169978a6983ce8d356f44a8a0684f673e323d8d59`; globals/auras `f9248253898305f1efcc9add7ad3610fa1dbd01510b6ad861863403e24c93047`; globals/register `7edb3342a25b8c4206f3082007d1c5c6ad5e302a34712503325b81ea3091281e`; slot fixture `9758ad91f80912c9c9278de3303edf24611394db944a277d51ed54ad6f48a8c6`.

Four readability suggestions deferred: numeric guard extraction, fixture seeding/assertion split, record field-group assertions, registrar grouping. No behavior counterexample; adjacent refactor unauthorized. Native access/output secrecy/errors/validation order, broader consumers and earlier/all-profile execution remain excluded; strict representation and miss policies remain inferred.

Exact accounting: add `unit-aura-slot-secret-arguments`; promote only `global api-C_UnitAuras-GetAuraDataBySlot-376`. **242 pending/106 bounded/14 partial →241 pending/107 bounded/14 partial =362**. Before `/tmp/patch-12.0.5-batch44-accounting-before.json`; preserve ordered362 IDs,361 unrelated rows, prior50 capabilities and top-level source/register hashes. Parent owns postcommit validation. No PLAN/code/build/tests/checks/operations here.

## Known gaps (current cycle)

- [x] Independent security/readability/Rust/acceptance gates and exact-row accounting remain pending despite saved producer GREEN. No batch43 proof transfers to these fixtures.
- [ ] Native validation ordering, exact errors, access permission and conditional output secrecy remain unproved. Earlier-profile providers are preserved in source only, not newly executed.

## Out of scope

- Native `RequiresUnitAuraAccess` permission and `SecretWhenUnitAuraRestricted` conditional output policy: backing restriction/access state unmodeled. Public modeled DTO checks do not prove permission enforcement or restricted-output secrecy.
- Indexed/display/duration producers, shared lookup/DTO/store behavior changes, Cargo/build changes, batching redesign, invented visibility predicates, native error/parity and all-profile coverage.
- Batch43 source/spec/wiki/accounting, PLAN, unowned `src/c_api/aura_duration.rs` changes, push/deploy/operations. This producer slice changes only its C API module, declaration/registrar, narrow helper visibility and earlier-provider gate, owned spec and Lua API wiki implementation links. Fixtures remain unchanged; parent owns remaining proof and credit. This docs-only reconciliation owns the slot spec and Lua API system/index/log links only; audit paths remain excluded.
