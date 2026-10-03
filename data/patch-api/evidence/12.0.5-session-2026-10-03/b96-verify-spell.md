# B96 spell verification — a0e23199d

Read verify skill; operating as verifier, no delegation. Read-only repository inspection. HEAD is a0e23199dc60d4056add5f5a15b68f23dcac38cc. `git diff a0e23199d HEAD -- src tests` and unstaged `git diff -- src tests` both empty, exit 0. Prebuilt targeted test execution authorized.

Initial evidence: both producers read explicit state maps, not wrapped constants. Max uses only unit_auras_restricted for output secrecy. Specs explicitly exclude native AllowedWhenTainted secret-input parity; stale integration/implementation gap statements require reconciliation. Targeted execution complete: maxima 8 passed / 0 failed; Maw 10 passed / 0 failed; both exit 0. Wiring and cfg gates confirmed. Cached policies are AllowedWhenTainted, not AllowedWhenUntainted. Both slices qualify for bounded state-backed credit, not native parity. Final findings below.

## Final report

### 1. Maximum cumulative aura applications — ACCEPT WITH QUALIFICATIONS

Source rows 315/316 match public SpellIdentifier resolution and unit-aura output restriction. Producer `src/c_api/c_spell_counts.rs:41–58` reads a live spell-keyed map and separately wraps its payload under `unit_auras_restricted`; this is not a wrapped constant. Registration: `src/c_api/c_spell.rs:89–93`. State: `src/lua_api/state/sim_state.rs:276–280`; empty-map/false defaults: `src/lua_api/state.rs:248–250`.

Earned requirement checkboxes in `docs/specs/spell-max-cumulative-aura-applications.md`: lines 11–14 and 18–19, for the stated bounded simulator contract. Tests prove distinct values 7/13, normalized names, numeric-alias precedence, independence from active stack count 2, live replacement to 9/removal, isolated defaults, exact return arity, malformed-input rejection, independent restriction flags, persistence of earlier secrets, and tainted caller opacity. Nonfinite rejection is exercised with NaN; ±infinity and all wrong-type variants are not individually exercised in this slice, although the reader explicitly rejects them.

Concrete documentation defect: spec line 38 incorrectly says the function is unregistered and state inputs do not exist. Runtime and source evidence contradict it. No bounded producer defect found.

### 2. Maw powers — ACCEPT WITH QUALIFICATIONS

Source rows 297/299 change both argument types to SpellIdentifier. `src/c_api/c_spell_maw_powers.rs:14–17,36–75` reads two independent live maps; atlas miss yields one nil, link miss zero results. Exact supplied strings are returned without catalog synthesis. Wiring: `src/c_api/mod.rs:132–133`, `src/lua_api/globals/register.rs:88–89`, state field `src/lua_api/state/sim_state.rs:191–193`, initialization `src/lua_api/state.rs:185–186`.

Earned requirement checkboxes in `docs/specs/spell-maw-powers.md`: lines 47–51, 55–57 and 61–62, only as explicitly inferred/bounded policies. Tests prove empty defaults, distinct hits, names and full colored-link aliases resolving to a different embedded ID, alias precedence/removal, absent alias targets, map independence/replacement/removal, exact miss arities, empty-string hits, u32 endpoints, malformed numbers/types/UTF-8, read-only state/caller inputs, environment isolation, public taint preservation, and authentic secret rejection/identity through GC.

Concrete defects: spec line 73 says integration is pending although wiring exists; line 82 still describes all requirements as unchecked. Shim retirement remains genuinely unfinished: `src/lua_api/workarounds/temporary/spell_static_defaults.rs:23–27` retains the nil border fallback, with shim-specific expectations at lines 48 and 70–71. It remains reachable through `src/lua_api/workarounds/mod.rs:367`. Thus do not credit line 82's complete integration/retirement gate or claim the out-of-scope statement at spec line 89 ('no replacement shim or fallback is retained'). The guarded shim does not override an already registered provider.

### Security, callers and profiles

Cached `Blizzard_APIDocumentationGenerated/SpellDocumentation.lua:461–475` declares maxima `SecretWhenUnitAuraRestricted` and `SecretArguments = "AllowedWhenTainted"`; lines 132–146 declare Maw link `AllowedWhenTainted`, SpellIdentifier, MayReturnNothing, nonnil cstring. No generated border declaration was found. Neither reviewed API has an AllowedWhenUntainted declaration, so the requested unwrap-before-validation authentication rule is not applicable here. Shared reader `src/c_api/c_spell.rs:670–692` rejects secrets before representation validation, never calls unwrap_secret, and deliberately does not implement native AllowedWhenTainted acceptance. Tests verify conservative rejection, not native permissions.

Cached `Blizzard_Deprecated/Mainline/Deprecated_12_0_7.lua:24–27` assigns the border API to a rarity wrapper. If that addon loads after host registration, the assignment replaces this producer; these isolated tests do not prove full-Blizzard-load reachability. Cached MawBuffs uses rarity API at lines 253–261 and legacy global link at 293–298, neither of which this slice models. No rendering or existing full-load consumer credit earned.

New modules, registrations, map fields and tests use retail-12-0-5 gates. Cargo feature inheritance enables them for current retail/PTR, not stock older profiles. `unit_auras_restricted` is ungated but defaults false; the shared public reader lost its cfg gate without a body change. No older-profile runtime regression found by inspection; older-profile builds were not run. Retained guarded border shim preserves prior absence behavior there. Stricter malformed-input errors replace the retail nil shim's permissiveness as explicitly specified; full-load caller behavior remains untested.

### Behavioral test quality and proof ledger

All 18 tests assert observable behavior; no query replacements or source-shape assertions. Not every individual test kills every constant implementation: a miss-only Maw test can pass a constant nil/zero-result provider, and a single-key/security test can pass a matching constant plus validation. The suites as a whole cannot pass constant producers: distinct keyed hits, live mutations, alias changes and secret/public transitions defeat them. No mutation run or independent RED run performed.

- `target/debug/deps/integration-8ea324359263a4d2 spell_max_cumulative_aura_applications:: --test-threads=1`: `8 passed; 0 failed; 0 ignored; 0 measured; 10153 filtered out`; exit 0; 0.79s.
- `target/debug/deps/integration-8ea324359263a4d2 spell_maw_powers:: --test-threads=1`: `10 passed; 0 failed; 0 ignored; 0 measured; 10151 filtered out`; exit 0; 1.08s.
- HEAD/target source-test diff, unstaged source-test diff and staged source-test diff empty, exit 0. Proof uses caller-supplied prebuilt binary; binary build provenance was not independently established. No cargo, compilation, profile matrix or native-client execution.
- EXIST/SUBSTANTIVE/WIRED pass for both slices. Producer/test files contain zero TODO/FIXME/HACK/XXX markers. Repository files unchanged; only this authorized report written.

Merge risk: bounded host-state APIs have passing targeted behavioral evidence. Do not mark native secret permissions, full-load consumer compatibility, or border contract parity complete. Reconcile stale specs and retained fallback before claiming the full documented retirement gate.

