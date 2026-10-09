# Historical Retail Patch 1.12.0 frozen redirect audit

SOURCE-only audit at base `9252c6cc93c087518f25e64987e095ed76312a51`, isolated branch `p1120-page`. Frozen source is a redirect, not a publication inventory. Empty inventory is not proof that historical 1.12 introduced no API changes.

## Frozen identity

Page 326189, revision 3145618, timestamp `2020-04-05T21:07:30Z`; captured `2026-10-09T08:51:36.402799+00:00`. Revision timestamp is not the patch release date. Exact 45 bytes, no trailing newline:

```text
#REDIRECT [[API change summaries/Historical]]
```

- Wikitext SHA256: `96679828d7b158a7117a5123bcd91a92be223cfa2c285d6ba84beba2606a48b2`.
- Response SHA256: `b0d15e5e41f2307c0460df8d3de0dfc9c614ac2d1986d26469dab4fcc8f60a37`.
- Manifest-linked registry SHA256: `e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c`, 101 entries ending at 1.0.0.

## Coverage matrix

| Boundary | Derived count | Proof |
|---|---:|---|
| Physical/nonblank/raw metadata rows | 1/1/1 | Exact literal source/response identity |
| Linked target / missing contract | 1/1 | Retained, unexpanded, UNPROVEN |
| Inventory/signatures/defaults/prose | 0/0/0/0 | No declarations on frozen redirect |
| Headers/count claims/templates | 0/0/0 | No declarations on frozen redirect |
| Meaningful model/runtime/native observations | 0/0/0 | No execution or parity credit |

Actual model review found no grounded behavior to test or repair: target revision/content and patch-specific historical delta are missing. API identities, arguments/returns, defaults, event payloads, model transitions and security contracts cannot be selected from a redirect. Current default Retail 12.1 proves only explicitly bounded current models, never native 1.12. No runtime or Rust changes/builds justified.

## Separate histories

Original historical Retail 1.12.0 is not Classic Era 1.13.x. Classic/TBC/Wrath/Forever successors and linked navigation import no behavior. Original Retail 2.0.1 and later historical Retail pages are actual separate successors: ledger derives 47 reference rows from the sealed registry, preserving registry order, with no applied supersession. These are references, not expanded source contracts. Queued 1.13.2 integration goes first for ordering only; main owns integration and native/final acceptance.

## Development epochs

Initial scaffold retained RED4+ERROR1; corrected omission precondition retained RED5/5 (no errors). Targeted SOURCE GREEN5/5 covers exact pin/registry, all three populated boundary omission controls plus count/invention rejection, raw/response mutations, foreign-history exclusion, and historical defaults. No broad/check/lint/readability/coverage/final gates.

Unchanged historical generator CLI defaults produce exact retained register bytes (`source.path` is `source.wikitext` in the portable tool layout). Unchanged extractor function `extract_text(raw)` with all defaults produces exact `#REDIRECT API change summaries/Historical\n` bytes. This is a function-default replay, not a claim of legacy extractor CLI coverage-ledger success. Canonical shared tools are untouched; no new flags or alternate paths.

Source epoch `361348ba1`; log-retention cleanup `a14e7e536` removed an accidentally staged Python cache and retained ignored logs. Archive epoch `3364a24c8`: 23 original seals, 24 archive members, 47,289 compressed bytes. Original map/archive are immutable; later receipts are separately sealed.

Portable RED3/3 retained before seal creation. Fresh copied historical validator passed; copied SOURCE GREEN5/5 and portable GREEN3/3 passed with `PATH=/nonexistent`, isolated Python, and no Git/target/current-tool files in the copy. Both serialized ledger omission and log fabrication rejected by original seals, then exact bytes and map restored; no resealing. Tests use only copied historical tools/standard library. [Portable receipts](../../../data/patch-api/evidence/1.12.0-session-2026-10-09/portable-proof.json) and [command context](../../../data/patch-api/evidence/1.12.0-session-2026-10-09/portable-context.json) retain hashes, tested revisions and bounded scope. Later documentation does not invalidate source proof or rewrite the original epoch.

No compiler warnings measured because no compilation; targeted Python GREEN logs have no warnings. No suppressions or vendor/cache edits. No parent/final/native acceptance.

## Sources

- [Frozen manifest](../../../data/patch-api/source-cache/legacy-2026-10-09/manifest.json)
- [Frozen response](../../../data/patch-api/source-cache/legacy-2026-10-09/1.12.0-response.json)
- [Derived ledger](../../../data/patch-api/evidence/1.12.0-session-2026-10-09/ledger.json)
- [Contract spec](../../specs/patch-1-12-0-source-accounting.md)
- [Handoff prompt](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/page-agent-prompt.md): explicit task restrictions override generic prompt supersession/build guidance.

## See Also

- [[patch-2-0-1-api-audit]] — separate original historical Retail successor.
- [[patch-1-13-3-api-audit]] — separate Classic Era history, not supersession here.
