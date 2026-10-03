# Spell book item cast count — B94

`C_SpellBook.GetSpellBookItemCastCount(spellBookItemSlotIndex, spellBookItemSpellBank)` reports how many times a spell book entry can be cast. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 317–318 rename its output restriction: `# SecretWhenSpellCooldownRestricted -> SecretWhenCooldownsRestricted`. Source ID: `global api-C_SpellBook-GetSpellBookItemCastCount-318`. Before this work the simulator did not register the function.

Cached retail `SpellBookDocumentation.lua` lines 160–176 declare nonnil `spellBookItemSlotIndex: luaIndex` and `spellBookItemSpellBank: SpellBookSpellBank`, one nonnil `castCount: number`, `SecretWhenCooldownsRestricted`, `SecretArguments = "AllowedWhenUntainted"`, and the note "Always returns 0 if item is not found or is not a spell". Contract context, not native execution evidence.

## What it must do

- [x] Resolve the player-bank slot to its spell through the existing spell book data, then return exactly one number: the explicit `spell_cast_counts` entry for that spell, read live. The count is keyed by spell, never by slot number.
- [x] Return one zero when the spell has no count, the slot is empty or out of range, the slot is not a positive integer, or the bank is not the player bank.
- [x] Under explicit `cooldowns_restricted` the result, including zero, is a secret host number with the same payload; plain again when the flag clears. `unit_stats_restricted` has no effect.
- [x] Untainted callers may pass either selector as an authentic secret. Tainted callers passing a secret selector are denied before resolution, whatever the other argument; caller taint and input secrecy are unchanged and public arguments keep working.

## How it works

- [Spell and spellbook cooldown outputs](spell-book-cooldown-outputs.md) — the shared cooldown restriction flag.
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/c_spell_book.rs` — registration and producer under `retail-12-0-5`.
- `src/c_api/charge_state.rs` — `cooldowns_are_restricted`.

## Tests asserting this spec

- `tests/spell_book_cast_count.rs` — five cases: spell-keyed live count, zero paths, cooldown-only secrecy, untainted secret selectors, tainted denial ordering.

## Development proof and independent bounded acceptance — 2026-10-03

Inputs `ffd50c5d1` did not compile (the fixture read the spell ID as `u32`); corrected fixture `96994beaf` RED 0 PASS / 5 FAIL; producer `a881a1729` GREEN 35/35 across `spell_book_cast_count::` and `spell_count_outputs::`, cargo exit0, no warnings, `cargo fmt --check` exit0, startup `lua-errors` `[]`.

Main accepts an independent GPT-6.1-sol review: **ACCEPT WITH QUALIFICATIONS** (report SHA256 `b549a8d5f36a10ae8ff94da33458a912c76fbd7e2cbb4b8f24251b79a8bf46c2`, scratchpad-only). It reran both filters itself, 5/5 and 30/30, exit0; confirmed both selectors are authenticated before the model is read, the zero and restriction policy matches `C_Spell.GetSpellCastCount`, the decoy count keyed by the slot number is meaningful (slot 5 is spell 19750), and no cached Blizzard code calls the function outside a documentation comment.

Checked requirements are bounded simulator proof. [Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): row318 `bounded-coverage` under new capability `spell-book-cast-count`.

## Known gaps (current cycle)

- [ ] The spell book data has no empty slots, flyouts or future-spell entries, so "is not a spell" cannot be exercised; passive spells resolve like any other and return their host count.
- [ ] Secrecy is possible only in `retail-12-0-5` builds with the retail or PTR profile, where `cooldowns_are_restricted` can be true. Other feature combinations were inspected, not built.
- [ ] Not asserted: negative, non-finite or wrong-type slots, zero secrecy for a missing count, restricted output arithmetic denial, environment isolation.

## Out of scope

- Pet bank entries, non-spell entries and reagent-derived counts: no model; the count is a host input shared with `C_Spell.GetSpellCastCount`.
- Native handling of wrong-type selectors: this returns zero rather than erroring.
- Automatic restriction activation and native parity.
