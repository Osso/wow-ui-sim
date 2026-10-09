# Historical Retail Patch 2.4.0 source audit

Verified 2026-10-09. Frozen page 73272 / revision 6471380 / timestamp 2025-09-13T09:55:32Z describes 2008 Retail, not TBC Classic 2.5.x. Response and raw hashes match the committed legacy manifest; response content equals raw source. Owned `p240-source` starts at `f95eed96e`; main owns integration/current-runtime/native gates.

## Coverage matrix

| Scope | Accounted | Proof / remaining limits |
|---|---|---|
| Literal source | 99 nonblank rows: 73 pending contracts, 26 metadata (24 headers, navigation, reference marker) | Exact source-line/literal reconstruction; linked guide/forum/UIOptionsPanels comments not expanded. |
| Inventory references | 50 occurrences: 45 globals, one widget method, two events, one CVar, one command example | Source only, not a current publication measurement. Contextual GetItemFamily/GetItemInfo and `/console` are not inferred additions. |
| Signatures | 46 boundaries: 35 explicit call fragments, 11 unspecified/contextual | Full source retained; no arity/output/validation/native credit from spelling. Event payload limitations separately retained. |
| Ledgers / gaps | 195 unique IDs: 169 pending, 26 metadata | Original and zero-closure ledgers/gaps separate; these are contract IDs, not 169 measured runtime failures. |
| Meaningful closures | Zero; no Rust/runtime changes | No aliases, shims, fallback, retirement or native parity. Current Retail publication remains unmeasured; main owns that decision/gate. |

Literal accounting preserves optional first coroutine for debugstack, PARTY_MEMBER_ENABLE/DISABLE losing which-member payload, SecureStateHeader attribute changes, unitHighlights 0/1 (not a declared default), inline texture path/width/optional height/offset grammar, selected `{star}` chat textures, historical bag-family bits including unknown 16/2048 and 4096 Vanity Pets, GetItemIcon despite nil GetItemInfo, random-loot true/false semantics, nine-field GetSpellInfo, totem slot order and shared unit/combat-log GUID identity. Each source/inventory/signature row retains literal text and an unproven-contract reason. Existing modern namespaces or constant defaults cannot establish these historical contracts.

## Parser boundary

`--retail-240-summary` is opt-in in both existing tools. The mixed prose/inventory full extract deliberately returns literal raw markup rather than discarding shared event lines, citation fields or `<path>` tokens. Default own extraction still rejects the original unsupported citation template; default own generator remains empty.

Archived parser regression compares base `f95eed96e` with changed tools on 156 inputs: all default bytes/errors equal (146 successful default registers, 125 successful default extracts). Across 79 recorded registers/extracts, 76 registers and 70 extracts reproduce; all pre-existing nonmatches/errors are base-equal. Register nonmatches: 10.0.0, 12.1.0, 5.0.4. Extract nonmatches: 10.0.0, 10.0.2, 10.1.0, 10.1.7, 10.2.5, 12.0.5, 12.0.7, 12.1.0, 9.2.5. Originals remain unchanged. Own opt-in register and full extract both reproduce exactly; no claim that inherited unsupported/default cases now succeed.

## Retail ordering

Archive contains 73 actual ordered Retail registers starting 3.2.0, 3.3.0, 3.3.3, 3.3.5, 4.0.1. These first five have zero own tuple overlap. Later removal overlaps include DeclineInvite (5.2.0), CanGrantLevel/GrantLevel (8.2.5), GameTooltip:SetTotem/GetContainerNumFreeSlots (10.0.2), item and summon globals (10.2.6), spell globals (11.0.0) and nine combat-log functions (12.0.0). These are literal source-overlap receipts, not behavioral closures or new retirements.

Five pending placeholders explicitly require main to add actual registers: 2.4.2 (lexical CombatLogGetNumEntries/CombatLogSetCurrentEntry), 3.0.2 (none), 3.0.3 (none), 3.0.8 (SetFriendNotes), 3.1.0 (GetTalentLink). Frozen queued source identities are retained; lexical coincidence alone is not supersession. Classic 2.5.x, Wrath 3.4.x and Era never supersede this page. The pinned older-page registry has 101 entries through 1.0.0; this task audits only 2.4.0.

## Development proof and seals

Source fixtures GREEN 2/2 at `00708e36af358933032623acce28ceae469ce3e2`; parser regression exit 0 at `c5ea90ccf07f7ea22f376eae067fe179dcda7ab9`. Fresh copied historical replay GREEN 3/3 at `44efecb5a3f164bf23d782a7486d50acbe63cbb9`: no Git on PATH, no target or current source dependency. Nine serialized ledger/gap/log/parser/register/raw/registry/closure/manifest tamper controls reject with exact paths; restored SHA-256 and clean replay output match. Serialized omitted bag-bit row also rejects.

Original manifest seals 432 files; separate zero-closure manifest seals three. Root SHA-256: `084ccd99697600fc9dda41478927e4fec56cdc9ab13e0a34a39872c4bb27b33a`. Original parser/code/source/log bytes stay frozen; subsequent development receipts have a separate seal manifest. Hash seals detect drift against the committed root, not cryptographic authorship or authenticity if every trust anchor is replaced.

No broad/check/lint/readability/profile/startup/full-suite/final gates, Rust tests, vendor/cache/Wowless edits, push, merge, deployment or delegation. No profile-only Rust test file was created, so no missing aggregate-module cfg guard. Source audit completion is not main/native acceptance.

## Sources

- [Spec](../../specs/patch-2-4-0-source-audit.md)
- [Frozen literal source](../../../data/patch-api/sources/2.4.0-api-changes.wikitext)
- [Original ledger](../../../data/patch-api/evidence/2.4.0-source-2026-10-09/original/ledger.json)
- [Proof ledger](../../../data/patch-api/evidence/2.4.0-source-2026-10-09/proof-ledger.json)
- [Replay and handoff](../../../data/patch-api/evidence/2.4.0-source-2026-10-09/handoff.md)

## See Also

- [[patch-3-2-0-api-audit]] — first actual retained later Retail register.
- [[patch-3-3-0-api-audit]] — historical Retail source/modern publication distinction.
