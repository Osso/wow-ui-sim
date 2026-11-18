# Spellbook R88m.4 follow-ups

Tracker `lt-39c0`, comment `1090`, supplies VERIFIED Retail **12.1.5.70688** contracts for eleven spellbook-related names. This tranche corrects observable arity, conversion and publication boundaries without inventing native link maps, pet snapshots, selection caches or cooldown acquisition. A no-data simulator seam is not evidence that a live client starts empty.

## Eleven-name disposition

| API | Bounded disposition |
|---|---|
| `C_SpellBook.FindSpellOverrideByID` | Parsed signed ID, one signed number if the final Spell record exists, otherwise one nil. Existing no-link adapter preserves input; eligible override chains and their writers remain unmodeled. |
| `C_SpellBook.FindBaseSpellByID` | Same one-number/one-nil record boundary for the existing no-link adapter. Selected-context/base-link traversal is not inferred from the separate `C_Spell` relationship fixture. |
| `C_SpellBook.FindFlyoutSlotBySpellID` | One numeric zero on the represented miss path. Native ordered candidates/marks and kind-4 player-book rows are absent; existing display flyouts do not supply that choice map. |
| `C_SpellBook.GetSpellBookItemAutoCast` | Missing/unsupported item, zero resolved ID or missing Spell record: zero values. Represented supported record with absent candidate/flag state: two false booleans. No pet-book order is inferred from the membership set or action-bar indices. |
| `C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo` | Retail item rejection or failed query: zero values. Supplied successful decoded query: one fresh five-field table. Sibling `C_Spell` absence behavior is not changed incidentally. |
| `GetCallPetSpellInfo` | **VERIFIED (Retail 12.1.5.70688):** zero values for missing membership/invalid index; `(index, pet-info name-or-nil)` for indices 0–4, preserving empty strings. Retail now reads explicit empty-default host inputs; same-build ID rows and runtime name writers remain UNKNOWN. |
| `IsSelectedSpellBookItem` | Numeric-slot versus name form, invalid-slot errors, checked conversion, and missing-slot/name arity. Existing represented player rows remain an explicitly unselected seam; native variant/effect/selection caches are absent. |
| `C_SpellBook.GetOverrideSpell` | UNKNOWN outside-census publication, no selected Retail consumer: retire both Rust publication and fallback on Retail only. |
| `_G.GetSpellInfo` | UNKNOWN outside-census publication; no identified Retail producer or selected caller, so no new alias or removal needed. Preserve existing non-Retail publishers. |
| `_G.GetSpellBookItemInfo` | UNKNOWN publication; retire Retail fallback after selected-consumer audit. Preserve other profiles' existing producers. |
| `_G.IsPassiveSpell` | UNKNOWN publication; retire Retail false fallback after selected-consumer audit. Preserve other profiles' existing producers. |

The four census-absent names are **not** classified as universally absent or CONTRADICTED merely by the registration census. Task-authorized retirement is narrower: no original runtime-selected Retail caller was found.

## Inputs, state and inference boundaries

The Find family accepts numeric strings and numbers in signed-32 range, checks range before truncation, and requires argument 1. The item APIs consume two positions, unsigned-32 slot and signed-range bank subsequently restricted to 0/1; missing bank is not player-bank shorthand. Invalid numeric bank diagnostics include the verified enum reason and Usage text. The evidence lists several item conversion reasons without establishing every ambiguous nonnumeric routing detail; do not expand assertions from that list alone.

Native secret modes, stack-capacity and integrity checks remain separate constraints. Existing VM authentication is not proof of every native mode's error wording or integrity machinery. No failed check is a normal nil result. R88m.4 explicitly describes the first-secret type predicates of `IsSelectedSpellBookItem`; second marked string/number handling remains unverified. The adapter preserves its raw second-argument path rather than inventing that missing rule.

`LossOfControlInfo` is an already-decoded supplied DTO: seconds, modulation and explicit activity/replacement flags. Serialization is distinct from the native upstream calculation (milliseconds conversion, nonzero-pair activity and comparison of cooldown ends). Do not rescale seconds or overwrite supplied flags to manufacture a native producer. Existing off-spec book rejection is preserved, not proof that `off_spec_id` equals the native rejected-item flag. Arbitrary native row kinds and ordered pet rows are not represented.

Pet membership, ten existing action slots and an indexed autocast toggle do not constitute the verified native snapshot handler's spell-keyed flag map or full writer/event sequence. Selection queries similarly must not invent effect predicates, reaction/command/category mappings, locale name variants, or runtime pet names. None of these getter corrections creates a public update event.

## Retail call-pet flyout

**VERIFIED (R88i.28/R88j.3, Retail 12.1.5.70688):** accepted indices 0–4 publish exactly `(index, name-or-nil)`; missing membership or an invalid index publishes zero values, and an empty name stays empty. **VERIFIED (simulator source):** Retail supplies spell/index membership and current-player names through explicit empty-default host inputs; this does not claim that a native fresh client has no call-pet data.

**VERIFIED (Retail runtime cache, ActionBar TOC and `Shared/SpellFlyout.lua:270-273`):** the original `SpellFlyoutMixin:Toggle` hides call-pet entries with nil or empty names only for the action bar. **VERIFIED (Retail simulator test):** named and unmapped inputs remain visible, empty/missing names are hidden, and a callable-wrong producer fails the same original-consumer assertion.

**UNKNOWN (R88i.28/R88j.3):** same-build spell-ID/index rows, the current-player pet-info writer, and event effects. The simulator does not seed export-derived candidate IDs or fixed pet names. See the [call-pet contract](../../specs/call-pet-spell-info.md).

## Original-source census and consumer proof

The pinned Retail UI cache (`Osso/wow-ui-sim-blizzard-ui@a1bb331b6`) was independently checked against all **4,026** manifest entries. The lexical audit scanned all **3,647** Lua/XML files, including nonselected flavors. It found zero exact `C_SpellBook.GetOverrideSpell` or global `GetSpellBookItemInfo` references; twelve global `GetSpellInfo` calls and one global `IsPassiveSpell` call belong only to excluded Classic Commentator/Cata/Mists TalentUI routes. Namespaced calls and method names are not those globals. Exact-name string literals were generated namespace documentation, not dynamic aliases. Arbitrary constructed names or external addons remain outside this bounded audit.

`Blizzard_Deprecated/11_0_0_SpellBookAPITransitionGuide.lua` contains commented migration examples, not executable alias definitions. The selected `Blizzard_DeprecatedSpellBook` publishes six other wrappers, including Find APIs, not the four retired names.

The two Find selectors each cover two distinct present records (6603 and 133) plus absent records. Both AutoCast selectors independently pair two distinct supported book records (different slots and spell IDs, two Booleans each) with missing items (zero results). Both LoC miss/supplied-query selectors independently pair failed queries (zero results) with two distinct supported records and different supplied DTO values (one fresh table each). These contrasting selectors reject constant/no-op producers in isolation. DTO serialization tests do not establish native acquisition or cooldown calculation. Other narrow error, secret-input, missing-state and alias-publication selectors are limited boundary or preservation evidence, not complete producer acceptance.

Original selected consumers are explicitly **PRESERVATION**, not proof that a constant/no-op producer is rejected:

- Actual PlayerSpells TOC → SpellBook item XML → `SpellBookItemMixin.UpdateAutoCast`, original paged items and open/close/reopen, without fabricated item data or callbacks. Hidden overlays with no candidate state also pass a false/false constant.
- Actual deprecated Find wrappers preserve their original forwarding behavior.
- Selected Mainline `Deprecated_12_0_1.lua` LoC wrapper forwards only one argument. Its required-bank rejection is preserved; it is **not** a successful LoC consumer proof. The current SpellBook item's cooldown owner calls the normal cooldown API, not the LoC API.

All four original-consumer baseline selectors passed after the authentic UI-cache completion marker was restored; that is preservation evidence, not a producer RED. Test files and compiled receipts, not authored assertions alone, determine observed results. Profile compilation and Retail clean-error receipts must be reported with the delivered revision; this page does not substitute for those receipts or claim other-client native parity.

## Sources

- Tracker `lt-39c0` description and comment `1090` — full R88m.4 evidence and bounded delivery rules.
- [C_SpellBook](../../../src/c_api/c_spell_book.rs), [legacy input seams](../../../src/lua_api/workarounds/temporary/retail_spellbook_legacy.rs), [legacy aliases](../../../src/lua_api/workarounds/temporary/legacy_spell_globals.rs), [bootstrap defaults](../../../src/lua_api/workarounds/temporary/spell_book_static_defaults.rs).
- [Item API tests](../../../tests/spellbook_r88m4_items.rs), [original item consumers](../../../tests/spellbook_r88m4_consumer_items.rs), [Find consumers](../../../tests/spellbook_r88m4_consumer_find.rs), [alias boundary tests](../../../tests/spellbook_r88m4_aliases.rs).
- [Earlier LoC output spec](../../specs/spell-book-loss-of-control-outputs.md) — prior supplied-DTO policy; R88m.4 supersedes Retail book one-nil absence claims, not sibling Spell policy.

- `pr46-49-decomp-answers.md` — R88i.28 and R88j.3.
- [Call-pet spell contract](../../specs/call-pet-spell-info.md) and [original ActionBar consumer test](../../../tests/blizzard_ui/blizzard_actionbar/behavior_spell_flyout.rs).

## See Also

- [[consumer-selection-and-red-proof]] — selected ownership and independent RED requirements.
- [[mainline-spellbook-lifecycle]] — real spellbook load/open lifecycle.
- [[lua-api]] — namespace and shared cooldown output architecture.
