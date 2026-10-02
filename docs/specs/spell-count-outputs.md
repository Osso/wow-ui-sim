# Spell count outputs

Authored 2026-10-01. Batch67 covers only Retail12.0.5 exact source301 `C_Spell.GetSpellCastCount` and source309 `C_Spell.GetSpellDisplayCount` output-predicate deltas. Public C_* backing inputs are a C API model, not miscellaneous Lua glue; the literal host input is `SimState.spell_cast_counts: HashMap<u32,u32>`. Producers live in `src/c_api/c_spell_counts.rs`. No wrapper type is needed. Architecture: [Lua API](../lua-api.md).

Primary evidence: profile cache `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SpellDocumentation.lua`, lines216–230 and339–355. Cast: `SecretArguments = "AllowedWhenTainted"`, non-nil `SpellIdentifier` → non-nil number; docs say “Returns 0 if spell is not found”. Display: `SecretArguments = "AllowedWhenUntainted"`, identifier, number default9999, cstring default`*` → non-nil string. Both declare `SecretWhenCooldownsRestricted = true`. Display docs say “either the use count or number of charges” and “beyond the display count parameter”. These do not establish native priority, formatting, missing-source, coercion, width, or VM opaque-type behavior.

## What it must do

### Explicit inputs and scalar domains

- [ ] Add only Retail125-gated `spell_cast_counts: HashMap<u32,u32>` with an empty default, shared by Retail/PTR feature composition. No new Cargo feature or per-slot wrapper reuse.
- [ ] Cast returns exactly one number from the public resolved spell identifier's explicit count, otherwise0. Never derive cast count from charges.
- [ ] Display returns exactly one string: valid typed charge state with `max_charges > 0` supplies `current_charges`; otherwise explicit count supplies quantity. This priority is inferred, not native-verified.
- [ ] Inferred display absence is empty string; explicitly supplied0 is `"0"`. Missing cast is0. Local u32 maximum4294967295 must remain exactly representable, not claimed as native maximum.
- [ ] Use the existing alias-first C_Spell identity contract: actual name aliases, fixture alias, numeric-string aliases, numeric alias overrides, then public numeric identity. Do not reverse aliases, synthesize numeric-string aliases, scan slots, or require catalog acquisition for an explicit numeric key.
- [ ] Fixtures19750→7 and642→2 produce both positive cast and display results. Slots17/19 both bind19750 with counts99/3 and must never supply C_Spell counts.
- [ ] Map replacement/clear and charge updates are immediately visible. Reads leave count/charge/alias/slot maps unchanged; environments and replaced Lua results stay isolated.

### Display formatting

- [ ] Default maximum9999 retains9999 and replaces10000. Replace only strictly beyond maximum; equality keeps digits. These precise boundary policies are inferred.
- [ ] Decimal ASCII formatting, nil/omitted defaults, finite f64 thresholds including negative/fractional values, and empty/Unicode replacement exact bytes are inferred simulator policies.
- [ ] Public identifiers inherit strict UTF-8 string or finite integral u32 representations from the existing public helper. Display maximum must be finite number or default; replacement must be UTF-8, NUL-free CString-representable string or default. Validate even when lookup finds no source.

### Authentication and partial input parity

- [ ] Cast uses the existing conservative public identifier helper: reject every authentic secret argument1, including secure callers, before alias/model access. Error names public API and states `secret spell identifier access is not modeled`, without private payload. Public tainted calls remain allowed.
- [ ] Native cast `AllowedWhenTainted` secret permission remains **UNMODELED**. Conservative rejection is not NeverSecret metadata or full secret-input support and earns no input-permission credit. This is an open implementation limit, not a product exclusion.
- [ ] Display authenticates all three original VM arguments, in argument order, before type validation or model lookup. Secure secret NUM/STRING identifiers resolve via Value-based alias extraction; secret NUM threshold, STR replacement and NIL defaults authenticate. No stack mutation.
- [ ] Tainted authentic secret arguments of all six kinds (NUM, STR, NIL, BOOL, ordinary table, frame-backed table) deny at their positions before invalid public identifiers, unresolved identifiers or missing sources. No blanket ban on public tainted calls, no nil-callback guard.
- [ ] Secure wrong-type authenticated display inputs report public API/position without payload. Denial preserves caller trust, frame/table state, secrecy metadata, roots, copies and allocation identity through GC; secure reads recover afterward.

### Output restriction

- [ ] Use only existing explicit `charge_state` cooldown restriction flag/predicate under Retail125/PTR, independent of combat, slot state or query arguments.
- [ ] Unrestricted cast/display are ordinary Lua number/string. Restricted positive cast and display, charges, missing0/empty, supplied0, Unicode/empty replacement are genuine typed host-secret NUM/STR values, not nil or fake callbacks.
- [ ] Meaningful positive cast and display provider assertions precede secretion assertions. Inspect actual `issecretvalue` metadata and fresh secure HOST `unwrap_secret` payloads as exact `Val::Num`/`Val::Str` UTF-8. Opaque private userdata makes no nominal Lua scalar-type claim.
- [ ] Tainted public callers can obtain private results but cannot add, concatenate, compute string length or expose private payload through opaque observations/errors. No secret BOOL equality under taint.
- [ ] Tainted copies preserve secret roots/allocation identity through GC. Live flag-off gives new ordinary results; old wrappers remain private.

## How it works

- [Lua API architecture](../lua-api.md)
- [Existing charge state contract](spell-charge-state.md)
- [Independent action count contract](action-count-outputs.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs`: literal C API spell-keyed input field, Retail125 gated.
- `src/lua_api/state.rs`: matching empty initialization, same gate.
- `tests/spell_count_outputs.rs`: grouped autodiscovery scaffold, Retail125 plus Retail/PTR only; no additional Cargo test binary.
- `src/c_api/c_spell_counts.rs`: Retail125-only real producers. Cast conservatively validates public identifiers, copies explicit quantity/restriction without charge reads, then immediately pushes ordinary or typed host-secret NUM. Display authenticates all three original arguments before validation/model access, resolves the authenticated value, copies quantity/restriction under one immutable snapshot borrow, then immediately pushes ordinary or typed host-secret STR. Original wrappers remain rooted and untouched. No new state or acquisition.
- `src/c_api/mod.rs`: only the Retail125 module declaration.
- `src/c_api/c_spell.rs`: Retail125 entries in the existing `SPELL_QUERY_METHODS`, published once through the existing root namespace. `read_spell_identifier_at` delegates to extracted `read_spell_identifier_value`; the existing `alias_key_from_input`, alias-first body and numeric passthrough are unchanged. Authentication/strict validation belongs to the new display caller; other callers retain their policies.
- `src/c_api/c_action_bar_counts.rs`: only `format_display_count` visibility becomes `pub(crate)`; body and action-count decisions unchanged. Display reuses the exact Source66 pure formatter without duplicated formatting policy. Current Source66-scope Forever/control proof remains applicable to that unchanged body, not proof of these spell producers.
- `src/lua_api/workarounds/temporary/spell_metadata_defaults.rs`: count-only Lua defaults execute through a Rust inverse-Retail125 helper; earlier epochs/Forever deliberately retain numeric0 for both counts. Passive/Ranged/Press/PriorityAura defaults and the existing custom-provider guard remain unchanged. No second C_Spell publication or new global policy flag.

## Tests asserting this spec

`tests/spell_count_outputs.rs`: 30 unchanged behavioral cases for both scalar outputs, explicit identity/state, formatting, display authentication, conservative cast rejection and authentic secret outputs/lifetimes. Main-owned compiled RED is recorded below; no producer GREEN or gates run by this implementer. All requirements remain unchecked pending independent proof.

Existing `tests/c_spell_probes.rs::test_spell_count_shims_return_zero` and the `spell_metadata_defaults.rs` unit tuple still expect display numeric0. Expected Retail125 control mismatch: documented display result is STRING, so the new producer returns empty STR without a source. Both tests remain untouched; main must first demonstrate actual producer GREEN/control failure before deciding corrections. Earlier/Forever numeric0 defaults remain deliberate.

## First producer execution and confirmed fixture corrections — 2026-10-01

Saved `batch67-green-build` at `6269e88698ff8d9b74ee4866f392877c0691d60f` compiles the existing lib and grouped integration targets successfully in321.9952879860066s with zero diagnostics. First focused run is29PASS/1FAIL, not fully GREEN: known-name fixtures stored mixed-case keys while the unchanged shared resolver normalizes input to lowercase. Fixture keys are corrected to canonical lowercase; no new name/catalog fallback or resolver semantics added.

The31-test spell-control group is30PASS/1FAIL and the two shim-unit tests are1PASS/1FAIL. Both failures expect a numeric display result and error on the actual STRING result. Cached output contract requires STRING; only these observed obsolete assertions are updated under Retail125, retaining earlier/Forever numeric0 behavior. New count queries/default domains remain model-backed, not a test-only callback replacement. Full outputs/exit/time/hash references live in `/tmp/patch-12.0.5-batch67-initial-green-runs.json`.

Refreshed focused/control/unit execution and independent verification remain pending. These are fixture/state and outdated-contract corrections, not production fallback changes. Native known-name acquisition, cast secret-input permission and nominal-type limits remain explicit.

## Saved corrected parent GREEN — 2026-10-01

At `9615c668a5a53fbe6c3e920364411cc788e22155`, existing lib plus grouped integration compilation exits0 with zero diagnostics in268.4409900170285s. `/tmp/patch-12.0.5-batch67-green-fixed-build-result.json` and full compiler JSON/stderr retain executable provenance: integration SHA256 `4104988f850d6c138f6ea00b90aea116106f965215fa57bace876332818f5d03`, lib-test SHA256 `682363c3a625e5e55192a2f9bb190abc8a15b7befaba10e43da7f80f08ff8984`.

Six finite executions record **127 distinct PASS**:30 focused,31 spell controls,2 shim units,28 Action counts,14 spell/flyout and22 tooltip identifier controls; all exits0. Runtime81.26639102597255s separately from compilation. Startup returns `[]`, exit0,7.463416979997419s. Exact commands/hashes/full output paths: `batch67-green-fixed-runs.json` and `green-startup-run.json`.

Canonical lowercase fixture keys and Retail125 STRING-domain assertions now pass; no new catalog/name fallback, callback replacement or input-permission bypass. Cast secret identifiers remain conservatively rejected and do **not** satisfy native AllowedWhenTainted; only output-predicate simulator coverage is proposed. Display’s actual VM guards and numeric/string payload privacy execute. Independent528 source/security/wiring/readability/scoped Rust checks and earlier/Forever preservation remain pending;301/309 uncredited. Dirty/globalfmt/process/type/acquisition/native/global-privacy/UI limits retained. No valid runtime/build replay solely for docs/accounting. Applicable header date and observed host provenance timestamps are distinct.

## Known gaps (current cycle)

- [ ] Independently demonstrate producer GREEN after the main-owned compiled RED below. Earliest RED failures prevented downstream privacy/GC assertions from executing; authored coverage is not completed proof.
- [ ] Verify C API producers and Value-based display resolution without VM bypass, generic declassification, callback swaps, stack clearing, secret cast payload host reads, new rilua pin or publication.
- [ ] Run main-owned startup, check, security, readability, current-profile/control and acceptance gates; no library suites or gates executed by this implementer.
- [ ] Independently prove output deltas before any retained-row301/309 credit. Inventory remains requester checkpoint194pending/153bounded/14partial/1metadata,362IDs/72capabilities; rows301/309 remain pending, as do separately verified237/241 until parent acceptance. No Maw/base295/action237/241 credit from this batch.
- [ ] Keep cast AllowedWhenTainted permission gap separate and open; it does not block meaningful bounded output work or authorize expanded VM changes.
- [ ] Native probes deferred: real reagent/charge counts;9999/10000/equality; coercion/nil; opaque type, root copies and tainted cast permission versus display permission; width, priority, no-source, profile/UI/global privacy. These remain unknown, not native claims.

## Out of scope

No casting/consumption transitions, reagent inventory, acquisition, arithmetic production of counts, action-slot aggregation, alias reversal, hidden inventory or new datasets. No protected aura-duration access, vendor edits, other wiki/data/PLAN/spec changes, broader API rows, operations or push. Full secret support is an implementation gap above, not a new exclusion.

## Proof ledger

Main-reported pre-producer proof at `23ac1c89d`, integration artifact `a65bdce18f20fe0e23dfeedeb5939c1f672ae9a55b660151f66f6144437ab7db`:

- Compile exit0, 420.46007691998966s, zero diagnostics.
- `/tmp/patch-12.0.5-batch67-red-run` FULL: 30 tests, 0PASS/30FAIL, exit101, 6.516457385965623s.
- Earliest failures include old cast0 versus explicit7 and old display NUM0 versus required STR. Many authentication/GC cases stopped at the positive-provider baseline; these are not 30 genuine authentication/GC failures or downstream proof.

Producer scope is only source301/309 output deltas. Existing empty map remains the only cast quantity provider, including explicit valid numeric IDs absent from the catalog; this is host-model coverage, not live acquisition. Charge priority, default/no-source, threshold and strict-domain policies remain inferred from the bounded simulator contract, not native quantity limits. Cast secret argument1 is rejected before aliases/model even for secure callers: no native AllowedWhenTainted input-permission credit, NeverSecret metadata claim, or full input/output privacy parity.

Required GREEN/startup/check/security/readability/current-profile/gates/accounting remain unchecked and main-owned. No tests, build, check, lint, readability gate, library suite, operations or push run here. Owned Rust formatting uses `rustfmt --config skip_children=true`; no module traversal. Existing observed host Git/build timestamps remain distinct from the authored 2026-10-01 header; no RED timestamp is inferred or rewritten. No retained-row credit or native-parity claim.
