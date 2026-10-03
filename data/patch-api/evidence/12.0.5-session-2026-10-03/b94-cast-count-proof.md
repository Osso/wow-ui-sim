# B94 cast-count independent verification

Pinned producer: `a881a1729`; inputs: `ffd50c5d1`, `96994beaf`.
Read-only repository inspection; report is the sole created file.

## Evidence ledger
- Supplied RED: `test result: FAILED. 0 passed; 5 failed; 0 ignored; 0 measured; 10088 filtered out; finished in 1.61s`.
- Supplied GREEN: `test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 10058 filtered out; finished in 9.88s`.
- Supplied startup stdout ends in `[]`; not independently repeated.
- Eligibility diff initially empty; source inspection pinned to producer.

Independent prebuilt command: `timeout 90 target/debug/deps/integration-8ea324359263a4d2 spell_book_cast_count:: --test-threads=1` (cwd repository).
> test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 10088 filtered out; finished in 1.46s

Independent prebuilt command: `timeout 90 target/debug/deps/integration-8ea324359263a4d2 spell_count_outputs:: --test-threads=1` (cwd repository).
> test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 10063 filtered out; finished in 9.45s

## 1. Source and declaration — PASS
- Pinned source `data/patch-api/sources/12.0.5-api-changes.txt:317–318`: `C_SpellBook.GetSpellBookItemCastCount`, `# SecretWhenSpellCooldownRestricted -> SecretWhenCooldownsRestricted`.
- Register JSON:2090–2099 records ID `global api-C_SpellBook-GetSpellBookItemCastCount-318`, kind `delta`, subject matching API, source line `318`, status `consolidated-delta`.
- Cached retail `Blizzard_APIDocumentationGenerated/SpellBookDocumentation.lua:160–175`: `SecretWhenCooldownsRestricted = true`, `SecretArguments = "AllowedWhenUntainted"`.
- Arguments: nonnil `spellBookItemSlotIndex: luaIndex`, nonnil `spellBookItemSpellBank: SpellBookSpellBank`. Return: one nonnil `castCount: number`.
- Documentation: "Returns number of times a SpellBookItem can be cast, typically based on availability of things like required reagent items; Always returns 0 if item is not found or is not a spell".
- `git show a881a1729` adds only producer and registration in `src/c_api/c_spell_book.rs`; parent has no named implementation/registration. This proves absent explicit producer, not necessarily Lua nil (RED failures are count assertions, not nil-call errors).

## 2. Producer and feature gates — PASS, qualified static proof
- `src/c_api/c_spell_book.rs:411–412`: `let slot = unwrap_secret(state, stack_val(state, 1))?;` then `let bank = unwrap_secret(state, stack_val(state, 2))?;`. Both authenticate before model borrow/resolution; first denial returns immediately.
- Lines 415–417: `resolve_player_spellbook_entry(slot, bank)` → spell-keyed lookup → `.unwrap_or(0)`.
- Lines 423–429: `if restricted { wrap_host_secret_number(state, count) } else { Val::Num(count) }`, then `state.push(result); Ok(1)`. Exactly one result on success; errors do not promise a result.
- `charge_state.rs:25–30`: `cfg!(all(feature = "retail-12-0-5", any(feature = "profile-retail", feature = "client-ptr"))) && sim.cooldowns_restricted`.
- Thus true only for 12.0.5+ retail-profile/PTR builds with explicit flag set; never from unit-stat restriction alone, nor automatically from combat.
- Function and registration: both `#[cfg(feature = "retail-12-0-5")]`. `SimState.spell_cast_counts: HashMap<u32, u32>` and initialization use the same gate (`sim_state.rs:258–260`, `state.rs:238–239`).
- Resolver: `#[cfg(feature = "retail-12-0-0")]`; Cargo:119 makes 12.0.5 imply 12.0.0. `charge_state` is ungated; restriction helper always exists. No added dangling reference or unused producer apparent.

| Feature selection | Static result for this change | Proof |
|---|---|---|
| client-retail / profile-retail + 12.0.5+ | Registered, flag can make result secret | Source; default binary 5/5 |
| client-ptr | Registered; cumulative 12.1.5 includes dependencies; secret-capable | Static only |
| profile-retail + 12.0.0, or no retail epoch | New producer/registration/map excluded | Static only |
| ordinary non-retail profile without 12.0.5 | New producer/registration/map excluded | Static only |
| non-retail profile + explicit 12.0.5 | Registered/map/resolver present, but restriction always false | Static only; tests excluded |

- No new feature-related compile failure or unused-function warning identified by inspection; no feature builds performed, so compilation/warning cleanliness across combinations is NOT certified.
- Sibling `c_spell_counts.rs:16–35` uses `quantity.unwrap_or(0)`, same restriction helper, same number wrapper, `Ok(1)`: zero/secrecy policies agree. Selector permission differs intentionally: sibling conservatively rejects secret identifiers.
- Test gate is 12.0.5 AND (retail-profile OR PTR): matches secret-capable behavior, not all registrations. Hybrid non-retail+epoch public behavior remains untested. Older epochs lack this new function.

## 3. Non-spell entries — PASS as bounded model; native clause UNMODELED
- `globals/spellbook_data.rs:10–15`: "A single spell entry in the spellbook." `pub struct SpellBookEntry { pub spell_id: u32, pub is_passive: bool, }`.
- Skill lines hold `pub spells: &'static [SpellBookEntry]`; `get_spell_at_slot` walks contiguous entries, returning their spell IDs. No item-kind field, empty-slot representation, flyout entry, or future-spell entry exists in this slot model.
- Resolver:445 returns `.map(|(_, entry, _)| entry.spell_id)` without checking `is_passive`. It therefore resolves passive spells too. `spellbook_item_info:841,845` emits `itemType = 1.0` and the passive flag.
- Flyout information exists separately; resolver does not access that model. Flyout/future-spell "not a spell" behavior cannot be exercised by these slots: UNMODELED, not proven honored or concretely violated.
- Passives are represented as spells, not non-spell items. A host count for a passive is returned; native passive castability/zero policy is not proven. Do not equate passive with non-spell without native evidence.

## 4. Tests and spec coverage — PASS, bounded
- Inputs: `ffd50c5d1` adds tests/spec; `96994beaf` changes fixture read from `u32` to `f64` then casts. Tests unchanged between corrected inputs and producer.
- All five run, not vacuous: actual assertions check `select('#', ...) == 1`, `issecretvalue(value) == restricted`, `secretunwrap(value) == expected`.
- All five fail against pre-producer behavior in supplied RED log. Four fail with `message: "count"`; tainted case fails with `message: "assertion failed!"`. Literal nil/unregistered would also fail all: positive paths call API; tainted test eventually requires public recovery. Denial alone is not its sole assertion.
- Decoy is meaningful: fixture asserts `assert_ne!(spell_id, SLOT as u32)` before `spell_cast_counts.insert(SLOT as u32, 88)`. Static data: slot 5 is Flash of Light, spell `19750`, not `5`; expected `9` then live `4` defeats slot-keyed lookup.

| Spec requirement | Concrete assertions | Limits |
|---|---|---|
| Spell-keyed live count, exactly one number | Test 1 checks 9 → 4; decoy 88; shared arity/payload checks | No direct public `type(value)` assertion; numeric payload comparison supplies practical evidence |
| Missing/invalid/bank zero | Test 2: missing count, slot 99999, Pet bank, slot 0, slot 5.5 | Negative/nonfinite/wrong-type/nil slots, other banks, and actual empty/non-spell entries unasserted |
| Cooldown-only secrecy, including zero, reversible | Test 3: unit-stats-only public 9; cooldown flag secret 9 and missing-slot 0; flag-off public 9 | Missing-count/Pet/invalid-slot zero secrecy not separately exercised |
| Secret selectors and taint unchanged | Test 4 each/both secret selectors accepted; test 5 tainted denial, invalid other selectors, equal denial, public recovery, unchanged taint and input secrecy | Number-secret selectors only; not every possible other argument/secret kind |

- Additional missing direct assertions: book API query immutability/environment isolation, restricted output arithmetic denial/GC, simultaneous restricted outputs with secret selectors. Sibling tests cover several shared runtime properties, not this API's exact composition.

## 5. Callers/regressions — PASS for inspected references
- Recursive literal-name search of cached retail AddOns, `Interface/AddOns`, `tests`, `docs/addons` in Lua/XML/Rust finds only cached declaration, a transition-guide comment, and new Rust tests. No addon Lua invocation or nil/absence-dependent branch found.
- `Blizzard_Deprecated/11_0_0_SpellBookAPITransitionGuide.lua:64`: `GetSpellCount(index, bookType) = C_SpellBook.GetSpellBookItemCastCount(index, spellBank)` is inside a `--[[` documentation comment, not executable Lua.
- Therefore no actual Blizzard caller performing arithmetic/comparison on this result was found in the supplied retail cache. Nothing observed to audit for secret arithmetic regression here; dynamic constructed-name references and external addons are not ruled out.

## 6. Spec honesty — PASS with qualifications
- Spec explicitly says "Contract context, not native execution evidence", host-input count, and excludes pet/non-spell/reagent models, native wrong-type errors, automatic activation, native parity. No broad native-parity overclaim.
- "Under explicit cooldowns_restricted" needs its build qualifier: true only with 12.0.5+ retail-profile/PTR. Producer exists more broadly if non-retail profiles manually enable that epoch.
- "Whatever the other argument" is established by producer authentication ordering but not exhaustively tested. "Slot is empty" cannot be directly modeled in the contiguous static catalog.
- "Non-spell entries: no model" is honest; passive entries do exist and resolve, so passive castability is an additional unproven limit.
- Checklist remains unchecked and known-gap paragraph says independent verification pending. This report supplies bounded verification, not permission to mark native completeness.

## Verdict — ACCEPT WITH QUALIFICATIONS
1. Accept bounded explicit-host count and cooldown secrecy implementation: independent prebuilt 5/5 and sibling 30/30 pass, exits 0, empty stderr.
2. Non-spell/future/flyout slot behavior and passive castability remain unproven/model-limited.
3. Feature matrix is static proof only; hybrid non-retail+epoch registration exceeds test scope and cannot become secret.
4. Invalid-selector/test-composition limits above remain; supplied startup `[]` is not independent startup verification.

Merge risk today: low for tested default-profile host-input behavior; compatibility risk if treated as complete native spellbook behavior or feature-matrix compilation proof. No demonstrated in-scope blocker.
Not verified: builds/formatters/compiler warnings, PTR/classic execution, native client, reagent/casting derivation, full suite, deployment/GUI, independent startup, binary rebuild provenance. Prebuilt evidence uses authorized eligibility check; no build performed.
Repository unchanged; no commits, agents/models, shell, cargo, or formatters used. A failed `rg` launch (executable unavailable) was replaced by Python reads; no source mutation.
