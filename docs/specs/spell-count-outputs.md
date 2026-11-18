# Spell count outputs

Authored 2026-10-01; updated 2026-11-17. This spec covers `C_Spell.GetSpellCastCount` and `C_Spell.GetSpellDisplayCount`. The older Retail 12.0.0/12.0.5 simulator contracts below remain historical. **VERIFIED (Retail 12.1.5.70688):** cast count uses the client Spell store, a shared context, and reagent limits; display mode uses SpellReagents before charge state. The simulator supplies reagent pairs, inventory, metadata/containment and empty-default property-candidate inputs through `SimState`; native property identity/decoding, detailed eligibility and active-record limits remain unmodeled. C_* backing stays in `src/c_api/`. See [Lua API](../lua-api.md) for architecture.

The pinned API documentation declares cast argument secrecy as `AllowedWhenTainted`, display argument secrecy as `AllowedWhenUntainted`, and both outputs as `SecretWhenCooldownsRestricted`. It declares a default display maximum of 9999 and replacement string `*`. **VERIFIED (Retail 12.1.5.70688):** the native routes and calculation order described below add behavior that the signature documentation alone does not establish.

## Exact1200 model preparation — 2026-10-10, after `3c36b2454`

Source inspection only: `SimState.spell_cast_counts: HashMap<u32,u32>` and its empty initializer are now gated by `retail-12-0-0`, not `retail-12-0-5`. This explicit map is internal simulator input, not native backing or count acquisition. The 12.0.0 retirement row establishes cast-successor presence only; it does not establish a meaningful quantity producer.

`tests/patch_12_0_0_prose.rs::prose_spell_cast_count_reads_live_explicit_inputs` is gated by `retail-12-0-0 && !retail-12-0-5`. It requires ordinary single-number cast results for19750→7/642→2, charge independence, live replacement/removal/clear, missing0 and environment isolation. Its assertions describe required behavior, not executed proof. Authentic exact1200 RED is retained in the [proof SSOT](patch-12-0-0-prose-models.md#authentic-count-red-and-source-present-repair--2026-10-10): actual1 FAIL, expected7 got0. After `84dcf2723`, cast producer/module/registration are present from Retail1200 onward; post-repair bounded GREEN is actual5/5 exact1200; see the [receipt SSOT](patch-12-0-0-prose-models.md#bounded-count-receipt-pass--2026-10-10).

Display formatting/maximum and secret-output behavior remain Retail125-scoped. Ordinary cast output before125 is simulator policy, not native proof; older API presence establishes neither internal native backing nor an acquisition requirement. The historical125 acceptance below does not establish exact1200 display, threshold, output restriction or secret-input rules and must not be applied retroactively.

## Current bounded receipts — 2026-10-10

[Proof SSOT](patch-12-0-0-prose-models.md#bounded-count-receipt-pass--2026-10-10) owns actual5/5 earlier +40/40 current-default count/control PASS, code84dcf2723/submission095081101, compile/check/fmt exits0 and retained sanitized evidence. Exact headless scope has10 preexisting project warnings; six iced manifest deprecations remain: not warning-free. BARE21 proves absence only; historical cached21 successor PASS atc6bc line13804 is separate, not current84 acceptance. Later AutoHide test963 is outside this proof. Pre125 ordinary output remains simulator policy; native secrecy/input permission and inferred display policies remain unproved.

## What it must do

### Retail 12.1.5 count sources and gates

- [x] **VERIFIED (Retail 12.1.5.70688):** on the checked normal-return route, `GetSpellCastCount` publishes one number. A Spell-store miss, null context, or absent limit returns numeric 0. A valid reagent with insufficient inventory can also set the limit to 0.
- [x] **VERIFIED (Retail 12.1.5.70688):** the ordinary reagent path checks at most eight identifier/quantity pairs. It skips IDs 0 and -2 and signed quantities less than or equal to 0. For each remaining pair, it divides inventory count by quantity using integer division and selects the smallest limit.
- [ ] The simulator uses the existing profile-aware `spells::get_spell` as its Spell-store membership input (Forever runtime rows first, then shared generated catalog). `spell_reagent_pairs` is a host-supplied seam with an empty default because the simulator has no SpellReagents catalog rows.
- [x] **VERIFIED (R88p.p3):** ordinary traversal includes carried roots, backpack and carried bags; mask32 additionally admits character-bank and account-bank tabs. Child containers recurse. The simulator reads bag stacks, equipped roots and deduplicated bag-container roots. Host-supplied parent coordinates represent nested containers. Detailed native eligibility remains UNKNOWN.
- [ ] **VERIFIED (Retail 12.1.5.70688):** count-policy statuses affect restricted-result handling only, not the calculated number. The simulator's explicit `cooldowns_restricted` input does not model the complete native status/runtime-policy mapping.
- [x] **VERIFIED (Retail 12.1.5.70688, R88p.p3; Retail 12.1.5.70783 original-consumer tests):** mask2 enables a property scan; only a candidate with signed magnitude greater than one selects per-item division followed by summation, with no second division. Mask2 alone does not select that arithmetic. The original `CooldownViewerCooldownItemMixin.CacheChargeValues` test distinguishes two backed results and verifies sum-then-divide for empty and ±1 candidates. The Retail-only simulator input holds only a signed candidate value and its available amount; property identity, native schema and decoding remain UNKNOWN. PTR/Forever applicability remains unclaimed. Metadata field identities and active-record inputs also remain UNKNOWN.
- [ ] **VERIFIED (Retail 12.1.5.70688):** a separate active-record limit can lower a reagent limit. That backing state is not modeled; the active-record count byte and linked schema remain UNKNOWN.
- [ ] Older profiles retain the explicit `spell_cast_counts` simulator input. Retail spellbook queries may also read that input; Retail 12.1.5 `C_Spell.GetSpellCastCount` does not.
- [ ] Retail 12.1.5 cast counts do not come from charge state or action-slot use counts.
- [x] **VERIFIED (Retail 12.1.5.70688; optional API test):** cast and display selectors accept ordinary numbers and numeric-convertible strings in the signed 32-bit range, then truncate toward zero. R88r.r2 confirms numeric, numeric-convertible string and recognized spell-text paths bypass name replacement. The name-map semantic interpretation remains INFERRED and its host seam empty by default; full secret-mode metadata remains UNKNOWN.
- [ ] **VERIFIED (Retail 12.1.5.70688):** missing or readable nil required spell selectors take an argument-error route; their exact secret-nil and guard behavior remains separate.
- [x] **INFERRED:** the shared Retail context resolves to the active player by default. `spell_cast_context_is_null = Some(true)` is a host-supplied null-context scenario; no login or lifecycle claim follows.

### Retail 12.1.5 display mode and formatting

- [x] **VERIFIED (Retail 12.1.5.70688):** a nonzero reagent ID with positive quantity in any of the first eight pairs selects cast-count display mode, including ID -2. If no pair qualifies or the row is missing, display mode uses charges.
- [ ] **VERIFIED (Retail 12.1.5.70688):** cast mode displays the cast count as decimal text, replacing it only when count exceeds the signed maximum. Defaults are 9999 and `*`; equality keeps digits. It returns one string, including for an empty result.
- [ ] **VERIFIED (Retail 12.1.5.70688):** charge mode displays current charges only when maximum charges exceed 1. Otherwise it returns an empty string. Charge mode ignores the display maximum and replacement string. A missing Spell record leaves charge output empty.
- [x] The Retail `SpellFlyoutPopupButtonMixin:UpdateCount` consumer accepts an ordinary numeric-string `maxDisplayCount` and passes a string to `SetText`; see `tests/blizzard_ui/blizzard_actionbar/behavior_spell_flyout.rs::spell_flyout_count_accepts_numeric_string_display_limit`.
- [ ] Secret-access details, exact guard-error decoration, and opaque VM type behavior remain bounded separately in this spec; they are not inferred from the count formula.

### Authentication and partial input parity

- [ ] **VERIFIED (pr46-49-decomp-answers.md §R88r.r1):** for an accepted secret numeric spell selector, a nonzero native metadata source rejects values at or below 1 through the required-argument Usage path; values above 1 continue to signed-32 conversion. The native metadata-source value/type is UNKNOWN. **INFERRED (simulator mapping):** Retail has `spell_identifier_metadata_source: Option<i32>`; `None` is the empty default, `Some(0)` skips the gate, and nonzero values enable it.
- [ ] **VERIFIED (pr46-49-decomp-answers.md §R88r.r1):** `GetSpellDisplayCount` converts an accepted secret `maxDisplayCount` within the inclusive signed-32 range by truncating toward zero; 1, 0 and -1 are accepted. A value outside that range takes the signed-range argument error. The identifier's <=1 gate does not apply to this maximum.
- [ ] Retail 12.1.5 cast keeps the bounded [numeric-secret selector subset](secret-selector-count-usability.md); other secret kinds remain conservatively rejected. Exact native `AllowedWhenTainted` access modes and all guard states are UNKNOWN.
- [ ] Retail 12.1.5 display authenticates all three original VM arguments, in argument order, before identifier conversion, maximum parsing, or model lookup. Secure authenticated identifiers resolve without stack mutation; tainted secret inputs remain rejected.
- [ ] Older-profile count APIs retain the existing strict public reader. Output-restriction policy is separate from selector permission.
- [ ] Tainted authentic secret arguments of all six kinds (NUM, STR, NIL, BOOL, ordinary table, frame-backed table) deny at their positions before invalid public identifiers, unresolved identifiers or missing sources. No blanket ban on public tainted calls, no nil-callback guard.
- [ ] Secure wrong-type authenticated display inputs report public API/position without payload. Denial preserves caller trust, frame/table state, secrecy metadata, roots, copies and allocation identity through GC; secure reads recover afterward.

### Output restriction

- [ ] **VERIFIED (Retail 12.1.5.70688):** count calculation completes before count-policy status handling. Status 0 does not select restricted handling; status 1 does when result-access bookkeeping is enabled; status 2 requires that bookkeeping and at least one of four runtime policy values equal 2.
- [ ] The simulator's current Retail output wrapper uses the explicit `cooldowns_restricted` input. This affects only the published scalar, not the calculated count, and does not claim the complete native status/runtime-policy mapping.
- [ ] Unrestricted cast/display are ordinary Lua number/string. Restricted positive cast and display, charges, missing0/empty, supplied0, Unicode/empty replacement are genuine typed host-secret NUM/STR values, not nil or fake callbacks.
- [ ] Meaningful positive cast and display provider assertions precede secrecy assertions. Tests inspect actual `issecretvalue` metadata and trusted-host `unwrap_secret` payloads as `Val::Num`/`Val::Str` UTF-8; opaque private userdata makes no nominal Lua scalar type claim.
- [ ] Tainted public callers can obtain private results but cannot add, concatenate, compute string length or expose private payload through opaque observations/errors. No secret BOOL equality under taint.
- [ ] Tainted copies preserve secret roots/allocation identity through GC. Live flag-off gives new ordinary results; old wrappers remain private.

### Related Retail visibility query

- [x] **VERIFIED (Retail 12.1.5.70688):** `GetVisibilityInfo` publishes zero values when its shared context query fails and three Booleans on success. A successful table miss publishes false, false, false.
- [ ] The simulator uses an explicit null-context override and host-supplied visibility rows. The native visibility tables and specialization-context selector remain UNKNOWN.

### Related Retail ranged-auto-attack identifier

- [x] **VERIFIED (Retail 12.1.5.70688):** `C_Spell.IsRangedAutoAttackSpell` compares its resolved identifier with one shared ranged-ID field. The field starts at zero in the initial image; zero therefore compares true before runtime writes.
- [x] **VERIFIED (Retail 12.1.5.70688):** the update writer stores an ID when its Spell record has `attributes_9 & 16_777_216 != 0`. Removal clears only when the removed ID equals the stored ID.
- [ ] The simulator reads the existing catalog `attributes_9` bit and exposes `ranged_auto_attack_spell_id` as a simulator state input. The exact DB2 column identity remains UNKNOWN.
- [ ] **INFERRED:** `A_Admin.LearnSpell` and `A_Admin.UnlearnSpell` supply simulator writer inputs. They are not names for the unknown native dispatcher cases.
- [ ] **UNKNOWN:** the dispatcher case meanings, additional spell-list update trigger, and update order at login remain unresolved. No native event name is assigned.

## How it works

- [Lua API architecture](../lua-api.md)
- [Existing charge state contract](spell-charge-state.md)
- [Independent action count contract](action-count-outputs.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs`: older explicit count input, INFERRED active-player context with a host null override, empty-default host reagent pairs, the INFERRED empty-default Retail metadata-source input, and host visibility rows.
- `src/c_api/c_spell_counts.rs`: profile-specific cast/display producers. Retail 12.1.5 uses the Spell catalog gate, shared context predicate, first-eight signed16 reagent limits, inventory traversal and verified reagent-before-charge display mode. `src/c_api/spell_reagent_inventory.rs` owns metadata/containment inputs, storage selection and empty-default property candidates; native property identity/decoding, detailed eligibility, active records and full policy secrecy remain unmodeled.
- `src/c_api/c_spell.rs`: Retail 12.1.5 spell-identifier conversion; registration for `IsRangedAutoAttackSpell` and `GetVisibilityInfo`; shared ranged-ID comparison, metadata-mask writer, equal-ID removal, and visibility output publication.
- `src/lua_api/globals/admin.rs`: INFERRED Retail host wiring from `A_Admin.LearnSpell`/`UnlearnSpell` into the verified ranged-ID writer and remover; no native dispatcher labels.
- `src/lua_api/workarounds/temporary/spell_metadata_defaults.rs` and `spell_static_defaults.rs`: the replaced Retail defaults are no longer published.
- `src/c_api/c_action_bar_counts.rs`: shared count formatter and replacement/maximum parsers.
- `tests/spell_native_verified.rs`: optional Retail 12.1.5 API and original-consumer tests, including explicitly labeled INFERRED host mappings.
- `tests/spell_count_outputs.rs`: legacy explicit-input controls remain scoped to older profiles; Retail 12.1.5 fixtures use supplied reagent and bag state.

## Tests asserting this spec

`tests/spell_native_verified.rs` covers the Retail 12.1.5 `CooldownViewer` cast consumer, including mask-2 candidate division and the no-qualifying-property sum-then-divide path, plus the `SpellFlyout` display consumer, `SpellSearch` ranged-ID consumer, `AuraUtil` visibility consumer and direct API results. Its `r88r_secret_selector_and_display_maximum_readers` test covers the secret readers at the API boundary while original selector secrecy remains UNKNOWN; `cast_count_consumers_preserve_ordinary_and_absent_inputs` checks the ordinary CooldownViewer path. `tests/spell_count_outputs.rs` keeps older-profile explicit-input controls and Retail formula/format controls. The Retail numeric-string maximum consumer case is in `tests/blizzard_ui/blizzard_actionbar/behavior_spell_flyout.rs`. Historical receipts below remain revision-scoped, not current blanket acceptance.

At the original producer checkpoint, existing `tests/c_spell_probes.rs::test_spell_count_shims_return_zero` and the `spell_metadata_defaults.rs` unit tuple still expected display numeric0. Expected Retail125 control mismatch: documented display result is STRING, so the new producer returns empty STR without a source. Both tests were then untouched; the observed control failures and subsequent corrections are recorded below. Earlier/Forever numeric0 defaults remain deliberate.

## First producer execution and confirmed fixture corrections — 2026-10-01

Saved `batch67-green-build` at `6269e88698ff8d9b74ee4866f392877c0691d60f` compiles the existing lib and grouped integration targets successfully in321.9952879860066s with zero diagnostics. First focused run is29PASS/1FAIL, not fully GREEN: known-name fixtures stored mixed-case keys while the unchanged shared resolver normalizes input to lowercase. Fixture keys are corrected to canonical lowercase; no new name/catalog fallback or resolver semantics added.

The31-test spell-control group is30PASS/1FAIL and the two shim-unit tests are1PASS/1FAIL. Both failures expect a numeric display result and error on the actual STRING result. Cached output contract requires STRING; only these observed obsolete assertions are updated under Retail125, retaining earlier/Forever numeric0 behavior. New count queries/default domains remain model-backed, not a test-only callback replacement. Full outputs/exit/time/hash references live in `/tmp/patch-12.0.5-batch67-initial-green-runs.json`.

Refreshed focused/control/unit execution and independent verification remain pending. These are fixture/state and outdated-contract corrections, not production fallback changes. Native known-name acquisition, cast secret-input permission and nominal-type limits remain explicit.

## Saved corrected parent GREEN — 2026-10-01

At `9615c668a5a53fbe6c3e920364411cc788e22155`, existing lib plus grouped integration compilation exits0 with zero diagnostics in268.4409900170285s. `/tmp/patch-12.0.5-batch67-green-fixed-build-result.json` and full compiler JSON/stderr retain executable provenance: integration SHA256 `4104988f850d6c138f6ea00b90aea116106f965215fa57bace876332818f5d03`, lib-test SHA256 `682363c3a625e5e55192a2f9bb190abc8a15b7befaba10e43da7f80f08ff8984`.

Six finite executions record **127 distinct PASS**:30 focused,31 spell controls,2 shim units,28 Action counts,14 spell/flyout and22 tooltip identifier controls; all exits0. Runtime81.26639102597255s separately from compilation. Startup returns `[]`, exit0,7.463416979997419s. Exact commands/hashes/full output paths: `batch67-green-fixed-runs.json` and `green-startup-run.json`.

Canonical lowercase fixture keys and Retail125 STRING-domain assertions now pass; no new catalog/name fallback, callback replacement or input-permission bypass. Cast secret identifiers remain conservatively rejected and do **not** satisfy native AllowedWhenTainted; only output-predicate simulator coverage is proposed. Display’s actual VM guards and numeric/string payload privacy execute. Independent528 source/security/wiring/readability/scoped Rust checks and earlier/Forever preservation remain pending;301/309 uncredited. Dirty/globalfmt/process/type/acquisition/native/global-privacy/UI limits retained. No valid runtime/build replay solely for docs/accounting. Applicable header date and observed host provenance timestamps are distinct.

## Bounded readability implementation — 2026-10-02

Independent report `/tmp/patch-12.0.5-spell-count-independent-proof.md` identified duplicated display validators and dense embedded Lua. Spell now imports the existing Action validators with explicit API context; no new parsing framework, input policy, stack mutation or model change. The formatter is unchanged. All30 focused tests retain their names, inputs, assertions and execution order, including canonical lowercase aliases. Lua statements/literals are expanded; long fixture helpers are split into named slot-input, scalar-wrapper, table-wrapper and metadata-publication steps with unchanged root/GC sequencing.

No build/test/check/lint/metrics/readability gate or native acquisition runs in this implementer. Main must refresh the focused30 Spell and28 Action tests after validator changes; later changed tests need only intersecting proof. Historical127 Retail and47 Forever results above remain historical evidence, not current validator execution proof. Unchanged control scopes retain their previous evidence; no broad rerun is implied. Source301/309 remain uncredited; requester checkpoint192 pending/155 bounded/14 partial/1 metadata,362IDs/73 capabilities is unchanged. Cast input-permission and opaque nominal-type gaps remain open; eight Forever warnings remain out of scope. Authored2026-10-01 and historical host observations remain untouched.

## Independent bounded acceptance — 2026-10-02

Independent528 and follow-up533 accept **exact301/309 output behavior only**. Proof SSOT: `/tmp/patch-12.0.5-spell-count-final-followup.md` and its JSON ledger. At `e5d11e497ca3c7eee51b3572866ca10185fe9199`, 30 refreshed Spell and28 refreshed Action cases pass;69 unchanged historical controls yield **127 distinct Retail PASS**, not185 fresh. Startup `[]` is inherited. Scoped rustfmt exits0; fresh default Cargo check exits0 with zero diagnostics in38.656696s. Refreshed compile511.677244s and runtime5.695362s/6.788062s remain separate costs.

Shared validators preserve domains, defaults, API-specific errors and all-original-argument authentication order. Full focused test/embedded Lua readability and literal equivalence reviewed. Historical Forever47 controls exclude the shared-validator refresh; eight historical warnings remain explicit. No full-profile or warning-free claim. Evidence is dirty-combined, not clean revision proof; globalfmt failure and historical502 protected-scope process failure remain unresolved.

Accounting promotes only301/309: **190 pending/157 bounded/14 partial/1 metadata,362 ordered IDs/74 capabilities**. Other rows, prior capabilities and source hashes remain unchanged. Cast AllowedWhenTainted secret-input parity remains **UNMODELED**, not NeverSecret credit or full API acceptance. Trusted-host private NUM/STR payloads and bounded Lua opacity, copy/root identity and GC behavior are proved; explorer540's contrary unproved assertion is rejected against the accepted proof SSOT. Native acquisition, native parity of inferred policies, nominal Lua scalar type, global privacy and UI/full-profile parity remain unproved. Earlier pending statements above are historical development checkpoints, superseded only within this bounded gate.

## Known gaps (current cycle)

- [ ] Exact SpellReagents field-to-pair mapping and catalog-backed reagent data remain unavailable; tests use host-supplied pairs.
- [ ] Metadata mask32/2 field identities, native property identity/decoding and detailed item eligibility remain UNKNOWN. The simulator's empty-default candidate input does not supply a decoder or writer. Character/account-bank admission is proved; reagent-bank inclusion and remote fetch are not.
- [ ] The verified active-record count limit is not implemented because the simulator has no corresponding active-record state.
- [ ] The shared context's native class, lifecycle, login availability, and complete writer history remain UNKNOWN.
- [ ] The visibility data tables and specialization-context selector are not modeled; results are host-supplied.
- [ ] Ranged-ID dispatcher meanings and list-update trigger remain UNKNOWN; no event names are inferred.
- [ ] Full native override/link/name resolution, all secret-access modes, exact guard-error decoration, and all guard states remain unmodeled.
- [ ] The simulator's bool output-restriction input does not reproduce the native count-policy status/runtime-value matrix.

## Out of scope

No cast consumption, native SpellReagents data generation, native property schema/decoder, or active-record schema is added. The alternate helper consumes only explicit empty-default candidate inputs. Do not infer dispatcher names, metadata column identities, remote/reagent-bank inclusion, property identity, or native visibility table fields. No Blizzard/vendor edits, VM changes or unrelated API rows.

## Proof ledger

Main-reported pre-producer proof at `23ac1c89d`, integration artifact `a65bdce18f20fe0e23dfeedeb5939c1f672ae9a55b660151f66f6144437ab7db`:

- Compile exit0, 420.46007691998966s, zero diagnostics.
- `/tmp/patch-12.0.5-batch67-red-run` FULL: 30 tests, 0PASS/30FAIL, exit101, 6.516457385965623s.
- Earliest failures include old cast0 versus explicit7 and old display NUM0 versus required STR. Many authentication/GC cases stopped at the positive-provider baseline; these are not 30 genuine authentication/GC failures or downstream proof.

At its 2026-10-02 checkpoint, the producer covered only source301/309 output deltas and used the empty-default explicit count map. That model is superseded for Retail 12.1.5 by the current R88k/R88m contract above; its bounded privacy and output evidence remains historical and does not prove reagent calculation, mode selection, active-record limits, or current-profile acceptance. Cast secret argument1 was rejected before aliases/model even for secure callers: no native AllowedWhenTainted input-permission credit, NeverSecret metadata claim, or full input/output privacy parity.

This RED ledger records the historical pre-producer boundary, not the current acceptance status. Current scoped GREEN/check/security/readability and retained-row credit are recorded above; native parity remains unproved. Owned formatting uses `rustfmt --config skip_children=true`; no module traversal. Observed host Git/build timestamps remain distinct from the authored 2026-10-01 header; no RED timestamp is inferred or rewritten. No operations or push authorized.

## R88p width and inventory correction (lt-67d2)

Native source: R88p.p2/p3, Retail 12.1.5.70688, verbatim in lt-67d2 #1207; R88k.5 line11622 in lt-c3ed #1056 and lt-67d2 #1226.

- **VERIFIED:** quantities narrow to signed16 before eligibility/division; only the first eight pairs participate. Cast mode skips IDs0/-2 and nonpositive quantities; display mode retains the separate nonzero-ID predicate.
- **VERIFIED:** display maximum uses signed32 range validation before truncation, accepts numeric strings, and defaults to9999 for absent/readable nil. Negative maxima are valid. Ordinary numeric replacement values become VM-formatted text.
- **VERIFIED:** modeled untainted-only secret access denies before ordinary conversion with the native Usage-based mode1 message. Other modes remain unsupported.
- **UNKNOWN (lt-67d2 #1231):** floating-point exception configuration for exceptional/out-of-range identifiers. The count endpoints supply an unresolved identity outside the represented signed range, without a maximum-style range error, wrap or clamp; tests do not assert that neutral result.
- **SUPERSEDED IN PART by R88r.r2 (#1239):** numeric-path replacement bypass is now VERIFIED. The earlier assertion removals remain historical commits; R88r.r1 now verifies accepted secret-number identifier and maximum truncation. The native metadata-source conditional identifier guard remains UNKNOWN in the simulator; no <=1 identifier outcome is asserted.
- Metadata, container-parent and property-candidate maps default empty. Existing inventory provides ordinary count backing; candidate available amounts are explicit host inputs. Bank data is never fetched, no events are added, and native property identity/decoding is not modeled.
- Root representation is deduplicated across equipment/BagInfo; parent-linked containers are items, not additional roots. Missing parents and cycles cannot provide reachable contents (simulator robustness, not native malformed-inventory claims).

Four independent base-RED selectors cover signed16/eight-pair limits, default bank rejection, carried roots, and numeric replacements. Each includes two distinct backed results and absence. Later mask32/recursion/dedup/mask2 cases are additional coverage, not retroactive base-RED claims. Exact build/runtime receipts and historical bisect transcripts are supplied on the tracker, not committed.

## PTR applicability (INFERRED, isolated commit)

The explicit `ClientProfile::Ptr` branch reuses Retail R88p/R88k readers, reagent traversal and display-mode selection. This is inferred applicability, not PTR native proof. Its fixtures use actual reagent/inventory backing. The old unsigned maximum test now rejects values outside signed32 while retaining valid decimal/count controls; public identifiers use the represented signed/truncating subset rather than the contradicted strict-u32 subset. The original unknown-type/secret seams remain separate.

## R88r.r2 numeric/name-path separation

VERIFIED Retail 12.1.5.70688, lt-67d2 #1239 (native answers lines13322–13340): numeric identifiers, numeric-convertible strings and recognized `spell:` identifier strings return before the name replacement resolver. Count-local parsing preserves this distinction without changing shared C_Spell alias resolution. Existing `spell_id_aliases` remains an empty-default host name-resolution/override seam; the semantic name “override map” is INFERRED and no overridden name-path result is asserted. The selector supplies conflicting numeric/link aliases, then observes two distinct backed IDs and a missing ID through both APIs. PTR/Forever reuse is inferred, not separate native proof.

## Accepted secret numbers (R88r.r1)

VERIFIED Retail 12.1.5.70688, lt-67d2 #1239, lines13322–13340: after successful access, spell identifiers use signed conversion toward zero, including 6.5→6. The native metadata-source conditional guard at <=1 is not modeled or asserted. This changes only the count-specific cast reader, not MaxCumulativeAuraApplications. Existing access policies remain bounded preservation rather than proof of all native modes. Authentic fractional selectors have two distinct reagent-backed results plus absence; a host reagent row at ID6 distinguishes display-mode routing without claiming a native Spell6 catalog row.

R88r.r1 also verifies accepted secret maxima: source-range checking precedes truncation; 6.5 becomes6, and1/0/-1 are valid. Out-of-range sources take the signed-range argument error after access succeeds. The existing signed reader already implements these facts, so the added secret-maximum selectors are PRESERVATION, not new RED claims. Charge mode still ignores maximum/replacement for output text; argument validation and access checks remain prior steps.


## Forever applicability (INFERRED, isolated commit)

Forever70707 API declarations for both count APIs are byte-identical to PTR, per main #1230. The modeled count readers, reagent inputs and inventory traversal reuse VERIFIED Retail R88p/R88k/R88r contracts through explicit `ClientProfile::WowForever` routing. No broad Retail epoch feature or Action API is enabled. The existing Forever `Enum.BagIndex` supplies its bank-tab endpoint23; mask32 routing is inferred for that profile. Count-output restriction is local to these two APIs; shared charge policy and the Retail-only legacy count map remain unchanged.

Exactly two temporary functions are retired for Forever: `SPELL_CAST_COUNT_DEFAULT_LUA` installing `C_Spell.GetSpellCastCount` as numeric zero, and `SPELL_DISPLAY_COUNT_DEFAULT_LUA` installing `C_Spell.GetSpellDisplayCount` as numeric zero. Other metadata defaults and MaxCumulativeAuraApplications are unchanged.

`forever_spell_count_consumer::forever_original_cooldown_viewer_observes_distinct_reagent_counts_and_absence` runs original `CooldownViewerCooldownItemMixin.CacheChargeValues`, verifies Game TOC selection, loaded-addon status and original source identity, then observes distinct backed counts7 and2 plus absence0. Positive IDs131 Water Breathing and467 Thorns exist in the owner-provided **70685** Spell/SpellName/Misc/Effect0 cache. The UI files are **70707**; these versions are not conflated. The RED used only an optional-tests host-input field gate and fixture, leaving compatibility producers unchanged; it failed0vs7. No new SpellReagents catalog is claimed or acquired.
