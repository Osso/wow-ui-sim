# Spell count outputs

Authored 2026-10-01. Batch67 covers only Retail12.0.5 exact source301 `C_Spell.GetSpellCastCount` and source309 `C_Spell.GetSpellDisplayCount` output-predicate deltas. Public C_* backing inputs are a C API model, not miscellaneous Lua glue; the literal host input is `SimState.spell_cast_counts: HashMap<u32,u32>`. Future producers belong in `src/c_api/c_spell_counts.rs`. No wrapper type is needed. Architecture: [Lua API](../lua-api.md).

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
- `src/c_api/c_spell_counts.rs`: future C API producer destination, **not implemented by this scaffold**.
- `src/c_api/c_spell.rs`: existing alias/public identifier helpers and future publication point, unchanged before compiled RED.
- `src/lua_api/workarounds/temporary/spell_metadata_defaults.rs`: existing numeric0 defaults, unchanged before compiled RED.

## Tests asserting this spec

`tests/spell_count_outputs.rs`: 30 authored behavioral cases for both scalar outputs, explicit identity/state, formatting, display authentication, conservative cast rejection and authentic secret outputs/lifetimes. No tests executed or compiled in this scaffold. All requirements remain unchecked.

Existing `tests/c_spell_probes.rs::test_spell_count_shims_return_zero` still expects `(numeric0,numeric0)`. Do not change until observed producer GREEN establishes its display assertion obsolete against the documented string result. Parent owns compiled RED, producer GREEN and that decision.

## Known gaps (current cycle)

- [ ] Compile and observe main-owned RED before changing any getter, resolver, registration or shim callback. Authored tests are not proof; earliest failure may prevent downstream privacy/GC assertions from executing.
- [ ] Implement C API producers and Value-based display resolution without VM bypass, generic declassification, callback swaps, stack clearing, secret cast payload host reads, new rilua pin or publication.
- [ ] Independently prove output deltas before any retained-row301/309 credit. Inventory remains requester checkpoint194pending/153bounded/14partial/1metadata,362IDs/72capabilities; rows301/309 remain pending, as do separately verified237/241 until parent acceptance. No Maw/base295/action237/241 credit from this batch.
- [ ] Keep cast AllowedWhenTainted permission gap separate and open; it does not block meaningful bounded output work or authorize expanded VM changes.
- [ ] Native probes deferred: real reagent/charge counts;9999/10000/equality; coercion/nil; opaque type, root copies and tainted cast permission versus display permission; width, priority, no-source, profile/UI/global privacy. These remain unknown, not native claims.

## Out of scope

No casting/consumption transitions, reagent inventory, acquisition, arithmetic production of counts, action-slot aggregation, alias reversal, hidden inventory or new datasets. No protected aura-duration access, vendor edits, other wiki/data/PLAN/spec changes, broader API rows, operations or push. Full secret support is an implementation gap above, not a new exclusion.

## Proof ledger

Scaffold-only scope: authored state inputs, 30 tests and this spec. Required compiled RED/GREEN/checks remain parent-owned and unexecuted here by explicit instruction. Owned Rust formatting uses `rustfmt --config skip_children=true`; no module traversal. No coverage or native-parity credit.
