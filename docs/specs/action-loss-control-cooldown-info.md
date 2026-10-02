# Action loss-of-control cooldown info

Retail 12.0.5 exact plaintext row **239**, `api-C_ActionBar-GetActionLossOfControlCooldownInfo-239`, requires a meaningful action-slot snapshot before its output restriction can receive credit. The epoch125 producer now resolves existing typed slot/spell inputs, authenticates argument1 and restricts three numeric snapshot fields. Earlier epochs retain the inactive legacy callback. See [audit SSOT](../wiki/investigations/patch-12-0-5-api-audit.md). Corrected pre-producer RED is recorded below; producer GREEN and acceptance remain main-owned and unexecuted here.

## What it must do

### Modeled snapshot

- [x] Resolve existing `action_bars` slot → spell ID → `spell_loss_of_control` typed record. Slot17 →19750 returns `(312,237,1.25,true,true)`; slot19 →642 independently returns `(11,27,0.5,true,false)`. These are data fixtures, not native LoC observations.
- [x] Return exactly one fresh ordinary public table with exactly `startTime`, `duration`, `modRate`, `isActive`, `shouldReplaceNormalCooldown`. Preserve f64 timing and convert f32 rate to f64; fixture rates are exactly representable, not evidence of arbitrary native precision.
- [x] **Inferred absence policy:** valid unassigned/unknown positive slot and assigned spell without a record return `(0,0,1,false,false)`, preserving current action shape; do not borrow spell getter's nil-on-miss policy.
- [x] **Inferred snapshot/flag-copy policy:** read host interval/flag replacements, mapping reassignment and clear immediately. Copy flags verbatim, including zero intervals with active/replacement flags. Do not calculate expiry, compare normal cooldowns, or derive active status.
- [x] Reads and DTO mutation/replacement leave host maps and other DTOs unchanged; environments remain isolated. For an assigned unrestricted spell, existing real `C_Spell.GetSpellLossOfControlCooldownInfo` matches all five concrete action fields without changing spell API secrecy.

### Output security

- [x] Apply existing `charge_state::cooldowns_are_restricted` predicate to three actual host-secret numeric values, including default zero timing/rate1. When false, same numbers are public. Do not substitute combat/stat policy or fake secrecy metadata.
- [x] **Inferred table-versus-field/default restriction policy:** table stays public/accessibly ordinary; two copied flags remain ordinary booleans under restriction. Do not make booleans secret or declassify numeric wrappers.
- [x] Secure and public-tainted callers preserve context. Tainted arithmetic on each restricted numeric field fails through existing VM authorization with nonempty errors that do not disclose payloads.
- [x] Tainted copies retain actual numeric wrappers; host-only identity/allocation observations survive failures, secure recovery and forced GC. Turning restriction off produces a fresh public DTO while old secret fields remain opaque to addons.

### Slot authentication and validation

- [x] Authenticate the documented argument position through native VM `unwrap_secret` **before** parsing or lookup. Public tainted assigned slots remain allowed; secure authentic secret NUM17/19 select the concrete records. Tainted secrets fail before known, unknown or missing-record lookup and preserve caller context; secure recovery remains usable.
- [x] **Inferred strict domain:** accept finite integral positive u32 slots, including unknown positive slots; reject missing/nil, BOOL, STRING, table/frame, nonfinite/fractional, zero/negative and out-of-range arguments. Secure authenticated wrong types must fail, not select a default. Native `RequiresValidActionSlot` enforcement is unknown; the previous stub accepted anything.
- [x] Invalid public/secure-secret numeric inputs yield nonempty contextual errors naming public API and argument1; error text does not expose private string/secret payloads. Input authentication and numeric output restriction are separate tests, not extra annotation-row credit.
- [x] Publish one epoch125 retail/PTR C API handler in the real namespace; inverse-gate existing legacy handler for Forever/earlier profiles. Do not swap callbacks in tests, add generic declassification, or invent activation/catalog inputs.

## How it works

- [Lua API architecture](../lua-api.md)
- [Client profile system](../wiki/systems/client-profiles.md)
- [Audit classification/accounting](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/lua_api/state/support_types.rs`: existing `LossOfControlInfo` fields `start_time:f64`, `duration:f64`, `mod_rate:f32`, `is_active:bool`, `should_replace_normal_cooldown:bool`; no new host type required.
- Existing `SimState.action_bars` and `spell_loss_of_control`: assigned-slot and spell-keyed explicit inputs; no new state fields planned.
- `src/c_api/c_action_bar_loss_of_control.rs`: epoch125 first-class callback; authenticates original rooted arg1 via VM `unwrap_secret`, parses authenticated `Val::Num` with a positive-u32 cast-round-trip predicate, then `read_snapshot(state, slot)` snapshots the cloned record and restriction predicate in one immutable borrow. Missing records use the inferred inactive shape. Borrow ends before result allocation; ordinary five-field table is immediately stack-rooted before wrapper/key allocations, returned once. All five fields are copied; no expiry or flag recomputation.
- `src/c_api/mod.rs`: `retail-12-0-5` module gate.
- `src/lua_api/globals/action_bar_api.rs`: epoch125 import supplies the new callback; inverse gate retains old constant/stub and its capacity-helper import for earlier epochs.
- `src/lua_api/globals/action_bar_api/registration.rs`: unchanged `COOLDOWN_SLOT_METHODS` entry binds the imported callback through the existing rooted namespace once; no parallel registration or fallback.
- `src/c_api/c_spell.rs`: existing real spell-level five-field DTO and nil-on-miss behavior, unchanged.
- `src/c_api/charge_state.rs`: existing restriction predicate and authentic VM secret-number construction; no charge entry/selection work.
- VM typed-host-secret numbers retain opaque userdata representation; producer does not override nominal Lua types, clear taint, swap callbacks, or generically declassify.

## Tests asserting this spec

- `tests/action_loss_control_cooldown_info.rs`: 22 focused behavioral cases, grouped autodiscovery; cfg `all(retail-12-0-5, any(profile-retail,client-ptr))`. No new Cargo target, private fixture imports, fake API producer or callback replacement.
- `tests/c_spell_flyout_probes.rs`: existing lower-level spell DTO controls, unchanged; not action-path proof.
- `tests/action_cooldown_output_restriction.rs`: existing assertion style only, unchanged; row233 cannot supply row239 credit.

## Fixture correction — 2026-10-02

Shared input compilation reported unused `Result` at `secret_slot` publication. Fixture now handles `set_global_val` failure explicitly with `expect`; no warning suppression. Initial compiled execution at `fad6e780f` recorded1PASS/21FAIL; corrected input `add2d0a84` compiled and ran before this producer. Tests remain unchanged by producer work.

Pinned VM typed-secret numbers are opaque userdata with authenticated `Val::Num` payloads. Numeric Lua types are asserted only when unrestricted; restricted fields retain actual secret metadata, exact trusted-host numeric payload and Lua-side denial/copy/GC checks. No native nominal-type parity or VM override is claimed. This corrects an unsupported fixture expectation also found in row233, without changing selected-state/payload predicates.

## Corrected pre-producer RED — 2026-10-02

- Revision: `add2d0a84fe834c7bab8e3023bb47b12104e4bdf`. Main reports corrected compile exit0, **80.86110783007462s**, zero diagnostics. Initial unused-Result diagnostic was corrected, not suppressed.
- Saved `/tmp/patch-12.0.5-batch65-red-fixed-run.json`, `.stdout`, `.stderr`: integration binary SHA256 `98dec8a9581fffee6ad2e93adc7f43a6a7339821cdc6b6388c464a7b4e7fb58a`; timeout90, one test thread; **22 tests,1PASS/21FAIL**, exit101, **3.9129977279808372s** wall time (harness3.56s). Only `unassigned_positive_slots_return_exact_inactive_default` passed.
- Concrete assigned payloads fail against stub zeros/rate1; restricted fields fail secret metadata; secure-secret selection, strict domain and tainted-secret denial fail at their first assertions. GC/copy/recovery tests stop at earlier field/payload failures: downstream GC/lifetime behavior is not established by this RED.
- Proof ledger: saved compile/run cover corrected tests against the old producer only. New producer is formatted, not compiled or executed; no GREEN, startup, check, security/readability gate or independent acceptance evidence is inferred.

## Snapshot extraction — 2026-10-02

Extracted the existing immutable slot → spell → LoC/default and restriction read into `read_snapshot`; callback parsing, rooting order, three numeric fields, two booleans and single return remain unchanged by source inspection. Only owned Rust is formatted with children skipped; compiled/executed equivalence was pending at this checkpoint and is supplied by subsequent combined GREEN/independent acceptance below. No new behavior or policy.

## Saved combined parent GREEN — 2026-10-02

Default integration compiled `cec856187c1c5bb2fc278d2aa0e268e6a20eb755` successfully in186.0260727679124s with zero diagnostics; source-equivalent snapshot extraction precedes this compilation. `/tmp/patch-12.0.5-batch64-65-final-green-build-result.json` and full compiler JSON/stderr bind integration SHA256 `7315587398811b5976e50819f5961983276070f4908b274685d7dc16c9e2668d`.

Combined `green-runs.json` records117 distinct PASS:22 row239 focused,18 row233 focused and77 controls across four groups; six exits0/no duplicate names. Execution24.42142666503787s separately from compilation is bounded development below60s target, not padded whole-goal proof. Startup returns `[]`, exit0,9.541190293966793s; exact hash/full output references in combined `green-startup-run.json`. Authenticated slots, both meaningful typed snapshots, default misses, private numeric payloads, public flags, live/read-only/caller/copy/GC assertions now execute; RED never proved those downstream paths.

Native nominal-type/miss/domain/activation/acquisition/profile/UI limits remain, with opaque VM values and no type override. Independent515 gates and exact239 accounting were pending at this checkpoint; subsequent bounded acceptance below supersedes that status. Dirty-combined/globalfmt limits explicit. Do not rerun valid build/runtime solely for docs/accounting.

## Independent bounded acceptance — 2026-10-02

Parent accepts independent515's combined proof. [Combined evidence/accounting SSOT](action-cooldown-output-restriction.md#independent-bounded-acceptance--2026-10-02) owns117 Retail PASS, startup, fresh formatting/check and36 separate existing Forever controls, costs, hashes and retained process/global-format failures. Exact239 only receives this capability;237 and partial231 remain unchanged.

Typed slot/spell snapshots and five copied fields, real authenticated slot boundary, private numeric payloads/public BOOLs, root/copy/GC/caller/read-only/live behavior and source-equivalent snapshot extraction are independently accepted. Inferred inactive defaults, strict slot domain and verbatim flags remain inferences; authenticated nonnumeric secret rejection is source-proven rather than separately fixture-executed. No activation/expiry/GCD/catalog or native acquisition/primitive-type/UI parity claim. Broader goal remains open.

## Known gaps (current cycle)

- [x] Meaningful snapshots, focused/control GREEN, startup, independent security/wiring/readability/Rust proof and exact239 accounting accepted above.
- [x] Source identity corrected to239;237 is display count and receives no credit here.
- [ ] Native nominal numeric type, LoC acquisition/conditions/expiry, spell/action mapping, secrecy behavior, valid-slot domain, GUI/profile behavior and automatic signals remain unknown. Absence, strict input domain, table/field restriction, restricted defaults and verbatim flag-copy are chosen inferences, not native exception facts.
- [ ] Deferred non-gating native probe: actual LoC spell/action slot; capture three numeric/two BOOL fields across empty/inactive/active/replacement states; secure/tainted read-copy and secret observations under cooldown restriction. No native probe required for this bounded cycle.

### Source grounding

Cached profile files under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`:

- `ActionBarFrameDocumentation.lua`, full `GetActionLossOfControlCooldownInfo` function: `RequiresValidActionSlot=true`, `SecretWhenCooldownsRestricted=true`, `SecretArguments="AllowedWhenUntainted"`; one required `actionID:luaIndex`; non-nil `SpellLossOfControlInfo` return.
- `SpellSharedDocumentation.lua:33–44`: three non-nil NUM fields without `NeverSecret`, two non-nil BOOL fields marked `NeverSecret=true`. Descriptions associate activity with timing and replacement with normal cooldowns; explicit host-fixture flags deliberately remain snapshots, not native-derived conditions.
- Exact239 registration delta: `SecretWhenActionCooldownRestricted` → `SecretWhenCooldownsRestricted`. Metadata is not proof of modeled LoC selection or exception behavior.

## Out of scope

Rows237/231/233/295; duration APIs, legacy LoC pair, `GetActionCharges`, generic cooldown/GCD/charge selection; new host state/catalog/activation/expiry/events; full-profile/UI compatibility; native parity or native gate. Protected `aura_duration.rs` body inspection and edits are forbidden in this task. No vendor/tests/Cargo/new model state, other-row changes, wiki/coverage/PLAN updates or spell-secrecy credit.
