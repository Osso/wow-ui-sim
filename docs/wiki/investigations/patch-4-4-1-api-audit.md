# Cataclysm Classic 4.4.1 source audit

Source accounting only, verified source identity 2026-10-08. Warcraft Wiki page **608915**, revision **6234252**, timestamp `2025-02-08T13:53:50Z`, literal TOC **40401**. Source commit `5960d3b73c48895dc216c4ecf1afccc8b51aa074`. This is Cataclysm Classic, not historical retail Cataclysm or Mists Classic. No network retrieval or runtime measurement is claimed here.

## Exact ID/proof matrix

[Ledger](../../../data/patch-api/sources/4.4.1-page-coverage.json): every nonblank wikitext line, including table markup. Blank lines 2 and 6 remain in the pinned source. Metadata proof is exact source identity only; both occurrence runtime claims are UNPROVEN.

| Source ID | Wikitext line | Literal metadata or API identity | Proof classification |
|---|---:|---|---|
| `source-context-001` | 1 | {{apichanges&#124;4.4.1&#124;prev=4.4.0&#124;next=4.4.2}} | metadata-only |
| `source-context-003` | 3 | ==Resources== | metadata-only |
| `source-context-004` | 4 | * TOC: <code>40401</code> | metadata-only |
| `source-context-005` | 5 | * Diffs: [https://github.com/Gethe/wow-ui-source/compare/4.4.0..4.4.1 wow-ui-source], [https://github.com/Ketho/BlizzardInterfaceResources/compare/4.4.0..4.4.1 BlizzardInterfaceResources] | metadata-only |
| `source-context-007` | 7 | ==Consolidated diffs== | metadata-only |
| `source-context-008` | 8 | ===Global API=== | metadata-only |
| `source-context-009` | 9 | {&#124; class="wikitable" style="min-width: 600px" | metadata-only |
| `source-context-010` | 10 | &#124;+ 4.4.0 (x) &rarr; 4.4.1 (x) xxx xx 2024 | metadata-only |
| `source-context-011` | 11 | &#124;- class="mw-customtoggle-global-c" | metadata-only |
| `source-context-012` | 12 | ! style="width: 50%" &#124; <font color="lightgreen">Added</font> <small>(x)</small> | metadata-only |
| `source-context-013` | 13 | ! style="width: 50%" &#124; <font color="pink">Removed</font> <small>(x)</small> | metadata-only |
| `source-context-014` | 14 | &#124;- class="mw-collapsible" id="mw-customcollapsible-global-c" | metadata-only |
| `source-context-015` | 15 | &#124; valign="top" &#124; <div style="margin-left:-1.5em"> | metadata-only |
| `global-api-added-016` | 16 | C_SpecializationInfo.GetNumSpecializationsForClassID | UNPROVEN |
| `source-context-017` | 17 | </div> | metadata-only |
| `source-context-018` | 18 | &#124; valign="top" &#124; <div style="margin-left:-1.5em"> | metadata-only |
| `global-api-removed-019` | 19 | GetNumSpecializationsForClassID | UNPROVEN |
| `source-context-020` | 20 | </div> | metadata-only |
| `source-context-021` | 21 | &#124;} | metadata-only |

Totals: **19 rows, 17 metadata/markup rows, two API occurrences**. `global-api-added-016` records added `C_SpecializationInfo.GetNumSpecializationsForClassID`; `global-api-removed-019` records removed `GetNumSpecializationsForClassID`. Both directions describe **4.4.0 → 4.4.1**, not 4.4.1 → 4.4.2. This does not retire the live retail global.

## Literal metadata boundaries

Navigation says prev=4.4.0 and next=4.4.2. Both external resource diffs compare 4.4.0..4.4.1; links are retained, not expanded. Caption `4.4.0 (x) → 4.4.1 (x) xxx xx 2024` and Added/Removed headers `(x)` contain **four literal placeholder counts**, not numeric totals. The observed one-added/one-removed occurrence counts are counted rows, not replacements for those placeholders. `xxx xx 2024` is a literal incomplete date; neither revision timestamp nor a guessed release date replaces it.

## Source proof versus runtime proof

Returned response: 1206 bytes, SHA-256 `6f022bc0d02bb1c67f4f531ae52d18309682ae9bb96f58f42f4f4641b1205579`. Main-slot content equals the 883-byte wikitext exactly, SHA-256 `076080f53193f5c8808f255e5630b9e83db8e9e284c53eff7a7ed7b84623284a`. Non-inventory plaintext: 131 bytes, SHA-256 `c27d366a643a97442af4cf11344153de401492753bca984e49f7b7f83a3b7400`. It omits the inventory table; the ledger retains its metadata and both occurrences separately. Serialized ledger SHA-256 `4469d9bc2d8583cc145709bdaad8d60bc582b5295e329ca876b3272f1ce55dd5`.

[Validator](../../../data/patch-api/evidence/4.4.1-session-2026-10-08/validate.py) checks committed response identity, source/text bytes, exact metadata/occurrence matrix, direction, placeholders, serialized ledger hash and no runtime credit. Four tests include 19 negative cases: response revision/bytes, source/text bytes, pin revision, omitted metadata/each occurrence, added/removed credit, capability credit, reversed direction, invented count/date, retail profile, register, supersession, shared pin and equivalent ledger reformatting. No live shared-file hashes or HEAD-dependent assertions: extractor provenance is Git revision `5960d3b73c48895dc216c4ecf1afccc8b51aa074`, blob `6fcc1724827c7588b91d23e3c815324d86a33b97`. Historical validator uses only local external artifacts and fixed contracts, so unrelated later source/tool/wiki/register edits do not invalidate it.

## Unsupported runtime limits

[Client profiles](../systems/client-profiles.md) provide no supported Cataclysm Classic runtime profile. Addition publication and removal absence in Cata are both UNPROVEN. Signatures, outputs, security, behavior, cached Cata UI loading and native parity remain unmeasured. No Mists/retail stand-in, runtime build/test, shared classifier change, publication register or retail supersession register. Numeric ordering and navigation are not runtime supersession proof. Follow the [4.4.2 source-only precedent](patch-4-4-2-api-audit.md), not its namespace inventory: this page has two explicit occurrences.

## Sources

- [Pinned wikitext](../../../data/patch-api/sources/4.4.1-api-changes.wikitext), [response](../../../data/patch-api/evidence/4.4.1-session-2026-10-08/source-response.json), [pin](../../../data/patch-api/evidence/4.4.1-session-2026-10-08/source-pin.json).
- [Plaintext](../../../data/patch-api/sources/4.4.1-api-changes.txt), [ledger](../../../data/patch-api/sources/4.4.1-page-coverage.json), [spec](../../specs/patch-4-4-1-source-accounting.md).

## See Also

- [[patch-4-4-2-api-audit]] — merged source-only accounting precedent.
- [[client-profiles]] — runtime support boundary.
