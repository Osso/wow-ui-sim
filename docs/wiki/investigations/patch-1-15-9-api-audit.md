# Patch 1.15.9 literal SOURCE audit

Frozen page 685352 / revision 6780591 / timestamp `2026-07-24T15:29:35Z`, audited offline at base `1174d7a8c9adb3a9933729fc256f3cc3614215f8`. Literal TOC **11509** and blue-post wording **Classic Era, Season of Discovery, Hardcore** establish the Classic Era source history—not a version-number-only guess. No runtime edits or native measurements.

## Source identity

[Own frozen inputs and ledger](../../../data/patch-api/evidence/1.15.9-session-2026-10-09/ledger.json) retain response, source, manifest and 101-page registry ending at 1.0.0. Raw source: 4,085 bytes / SHA256 `beea5ee3ad4a1a10c031cd02777a3d5bc1b5198e4637c713a088c4c9f94423db`; response SHA256 `2691663c5ffc7efbd0b862a8a94dd8632141c55eb7393f2aca8003d3448e6baf`. Page/revision/title/timestamp/content/manifest/registry identities are independently checked offline. Raw rows, duplicate C_CVar occurrences and exact spelling stay literal; no alias/default/signature reconstruction.

## Coverage matrix

| Literal scope | Accounting | Proof level / limit |
|---|---|---|
| Entire source | 52 physical lines; 37 nonblank rows: 21 metadata, 16 UNPROVEN | Every literal row retained; no behavior credit |
| Headers | Resources, Notes, Blue posts; UI Using Shared Code, Modifications Using Chat Commands, Upcoming WoW Classic Patch 1.15.9 – Week of July 19 | Six source headers; zero numerical inventory counts |
| API/return note | UnitAura; `shouldConsolidate` removed | One partial return contract; return position, remaining outputs and arity unspecified |
| Calls/CVars/commands | Three C_CVar.SetCVar calls, three /script occurrences; nameplateShowClassColor=1, nameplateShowFriendlyClassColor=1, raidFramesDispelIndicatorOverlay=0 | Three partial call contracts in quoted **TBC 2.5.6** context; values are not defaults or measured Era behavior |
| Prose | Eight contracts at raw lines 18/21/23/26/43/46/48/50 | Shared-code benefits/history, customization, planned Options, Era Edit Mode/nameplates/raid frames, compatibility advice and future refinement all UNPROVEN |
| Linked diffs | wow-ui-source and BlizzardInterfaceResources 1.15.8→1.15.9 | Two unexpanded linked contracts; identities/signatures/payloads/state/security/native equivalence UNPROVEN |
| Events/widgets/signatures | Zero explicit event or widget-method occurrences; zero complete signature declarations; zero consolidated inventory entries | UI prose is not a widget API enumeration; absence is not positive parity proof |

Total: **14 contracts**, four API-reference occurrences / two distinct API names, three partial calls and one partial return-removal note. Header markup and Bluepost attribution/body/wrapper rows remain accounted. Week-of-July-19 and next-week language is preserved as the July 10 quotation's prospective wording, not a verified deployment date.

## Client/model credit limits

Copied base code/Cargo/manifests record **Era and Anniversary configured 11507**, with feature dependencies and manifest hashes. This is static configuration, not native 1.15.9 observation, support diagnosis or publication measurement. No state/model tests run: source does not specify the removed return's position or full remaining signature; TBC chat examples do not justify an Era-default model. Zero runtime/model/native/full-UI credit.

The [2.5.6 audit](patch-2-5-6-api-audit.md) retains an unexpanded reference to this page. This independent audit neither backfills its sealed receipts nor establishes TBC equality. “Very closely matches” and addon-development advice do not prove exact API equivalence. No Retail 2.x, TBC 2.5, Wrath 3.4 or WowForever 1.60 supersession. No later registers applied; actual successors/current-native/integration belong to main.

## Development proof

Own serialized SOURCE fixtures: RED eight assertion failures against empty accounting; GREEN **8/8**, including every row/contract/header/API omission, foreign history, fabricated credit, source/revision tampering and configured-profile limits. Python AST formatting only; no lint/check/readability/coverage/broad/startup/final gates. Shared generator/extractor unchanged, frozen copies retained; own accounting uses raw literal text instead of claiming shared extractor expansion. Historical seals/replay and serialized ledger/log controls pending the next coherent step.

## Sources

- [Spec](../../specs/patch-1-15-9-source-accounting.md).
- [Frozen source](../../../data/patch-api/evidence/1.15.9-session-2026-10-09/source.wikitext).
- [Own tests](../../../data/patch-api/evidence/1.15.9-session-2026-10-09/test_source_accounting.py).

## See Also

- [Classic/TBC 2.5.6 source boundary](patch-2-5-6-api-audit.md) — unchanged historical receipt boundary.
