# Spell book item cast count — B94

`C_SpellBook.GetSpellBookItemCastCount(spellBookItemSlotIndex, spellBookItemSpellBank)` reports how many times a spell book entry can be cast. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 317–318 rename its output restriction: `# SecretWhenSpellCooldownRestricted -> SecretWhenCooldownsRestricted`. Source ID: `global api-C_SpellBook-GetSpellBookItemCastCount-318`. Before this work the simulator did not register the function.

Cached retail `SpellBookDocumentation.lua` lines 160–176 declare nonnil `spellBookItemSlotIndex: luaIndex` and `spellBookItemSpellBank: SpellBookSpellBank`, one nonnil `castCount: number`, `SecretWhenCooldownsRestricted`, `SecretArguments = "AllowedWhenUntainted"`, and the note "Always returns 0 if item is not found or is not a spell". Contract context, not native execution evidence.

## What it must do

- [ ] Resolve the player-bank slot to its spell through the existing spell book data, then return exactly one number: the explicit `spell_cast_counts` entry for that spell, read live. The count is keyed by spell, never by slot number.
- [ ] Return one zero when the spell has no count, the slot is empty or out of range, the slot is not a positive integer, or the bank is not the player bank.
- [ ] Under explicit `cooldowns_restricted` the result, including zero, is a secret host number with the same payload; plain again when the flag clears. `unit_stats_restricted` has no effect.
- [ ] Untainted callers may pass either selector as an authentic secret. Tainted callers passing a secret selector are denied before resolution, whatever the other argument; caller taint and input secrecy are unchanged and public arguments keep working.

## How it works

- [Spell and spellbook cooldown outputs](spell-book-cooldown-outputs.md) — the shared cooldown restriction flag.
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/c_spell_book.rs` — registration and producer under `retail-12-0-5`.
- `src/c_api/charge_state.rs` — `cooldowns_are_restricted`.

## Tests asserting this spec

- `tests/spell_book_cast_count.rs` — five cases: spell-keyed live count, zero paths, cooldown-only secrecy, untainted secret selectors, tainted denial ordering.

## Known gaps (current cycle)

- [ ] Development proof: inputs `ffd50c5d1` did not compile (test fixture read the spell ID as `u32`); fixed fixture `96994beaf` RED 0 PASS / 5 FAIL; producer GREEN 35/35 across `spell_book_cast_count::` and `spell_count_outputs::`, cargo exit0, no warnings, `cargo fmt --check` exit0. Independent verification and accounting pending.

## Out of scope

- Pet bank entries, non-spell entries and reagent-derived counts: no model; the count is a host input shared with `C_Spell.GetSpellCastCount`.
- Native handling of wrong-type selectors: this returns zero rather than erroring.
- Automatic restriction activation and native parity.
