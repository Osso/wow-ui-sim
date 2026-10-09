# Patch 2.5.3 TBC Classic SOURCE/contract audit

Frozen page 53568, revision 523812, timestamp `2022-06-04T23:59:56Z`; literal TOC `20503`. Separate `tbc-classic` source history, no runtime profile selection. Source-only development; main owns successor integration and native/final gates. [Spec](../../specs/patch-2-5-3-source-accounting.md).

## Provenance

[Source pin](../../../data/patch-api/evidence/2.5.3-session-2026-10-09/source-pin.json) preserves exact request/retrieval metadata from committed `legacy-2026-10-09`. Response SHA256 `0feb1ca97798ce6b2169e97207cc44f7e4847165d28e95fa9fbd8dbb6d71283e`; 8,310-byte wikitext SHA256 `f922ba1f9ee5e22f6a6c5a70bf8b5fe614b76a2a993ff5ee68149250198a2ca1`. Identity, timestamps, response content and both hashes validated before copying own and three successor inputs. Frozen full registry has 101 pages through 1.0.0; retained manifest has 65 snapshots, not 65 completed audits. Warcraft Wiki source attribution and CC BY-SA 4.0 metadata retained in the original cache README.

Base `acf7fbfe9b07dc342b8078cb1c54b702151a23d6`; branch `p253-source`, isolated `/home/osso/.worktrees/wow-ui-sim-p253-source`. Canonical 3.4.3 audit and read-only 2.5.6 sibling informed the bounded replay structure; all actual input bytes come from this committed base/cache. Replay never needs the sibling.

## Coverage matrix

| Literal source | Accounted | Proof level / remaining contract |
|---|---:|---|
| Every nonblank raw row | 103 | Exact literal/line/kind/ID/status/limit; no skipped HTML/table/comment/navigation rows |
| Global API | 20 added / 3 removed | 23 source claims; all callable signatures/state/security/native semantics UNPROVEN |
| Widgets | 2 added / 0 removed | FontString:GetTextScale and SetTextScale; arguments/returns/scale/rendering/native semantics UNPROVEN |
| Events | 2 added / 1 removed | NOTIFY_CHAT_SUPPRESSED, UNIT_HEAL_PREDICTION, SHOW_AADC_ALERT; payloads, dispatch/lifecycle/security/native contracts UNPROVEN |
| CVars | 18 added / 3 removed | 21 source claims; 15 literal defaults/descriptions and one Account scope preserved; six default omissions remain unspecified |
| Numerical headers | 8 | Exact 20/3, 2/0, 2/1, 18/3 reconciliation; source accounting, not publication parity |
| Summary | 1 | “SharedTooltipTemplate and GameTooltipTemplate no longer inherit BackdropTemplate.” Actual ancestry/method/backdrop/native behavior UNPROVEN |
| Signatures / content transclusions | 0 explicit / 0 | 25 callable signature absences recorded. API/navigation template references not expanded into linked documentation |
| External resource links | 4 | Both commit diffs, mutable classic Deprecated_2_5_3.lua URL and community notes retained but not fetched; linked contracts UNPROVEN |

49 inventory occurrences total: 42 additions, seven removals. Fifty substantive source rows/contracts remain UNPROVEN; 53 rows are metadata-only. Local register retains identity/direction and inline CVar defaults; never a runtime publication observation. Raw ledger includes CVar scope and exact descriptions. Stock plaintext strips inventories by design; raw-row ledger/register prevent loss. No shared tool changes, inferred rename behavioral equivalence, linked-page reconstruction or retirement.

The build caption literally says `2.5.2 (40011) → 2.5.3 (41812) Jan 7 2022`; these builds are not configured TOCs. The navigation is literally `prev=2.5.2|next=2.5.4`. Source generator comment is retained as metadata, not an API/security claim. Only content transclusions count toward the zero-content-transclusion observation; navigation/API templates are not proof of their expansion.

## Configured interfaces, not runtime proof

[Observation](../../../data/patch-api/evidence/2.5.3-session-2026-10-09/profile-observation.json) is independently derived from copied Cargo feature graph, Rust interface/cache arms and seven committed manifest path lists at the base revision. Retail 120100; PTR 120105; Wrath 38001; Mists 50504; Era and Anniversary 11507; WowForever 16001. Manifests contain paths, not TOC contents or measured interface versions. No configured 205xx arm observed; native-profile correspondence remains UNPROVEN. This is not a blanket unsupported-API diagnosis. No cache inspection/sync, loaded runtime/model/native/full-UI measurement performed.

## Successor isolation

Only same TBC Classic history 2.5.4/2.5.5/2.5.6 is referenced, all `pending-main-integration` with zero positive supersession credit. Their exact pins/responses/wikitext are copied under `pending-successors/` after pre-copy hash/identity validation. Main must integrate actual successors and assess literal same-history contracts separately; no retail 3.x, Cata 4.x, Wrath 3.4.x or Era 1.x supersession. The 2.5.6 source itself has an unexpanded Era link; frozen bytes are provenance, not a license to import that client's contracts into this audit.

## Development proof and replay

[Proof ledger](../../../data/patch-api/evidence/2.5.3-session-2026-10-09/source-proof.json) owns exact commands/revisions/results/invalidation boundaries. RED at `d0725eb00`: seven expected assertion failures for absent own source ledger, not a missing module/import. GREEN, seal/replay and serialized tamper receipts are recorded there after execution; no final acceptance claim.

`validate.py` reads only own frozen configuration/source/tool/registry/manifest inputs and serialized artifacts. `replay_controls.py` mutates serialized ledger/native-equivalence and GREEN log independently; expects exact seal rejection and restores original bytes/hash in finally. Archive contains original sealed inputs and its original seal map; fresh-process replay uses missing PATH and fresh HOME with no `.git`, `target`, `tools`, current `src` or configuration source tree. Archive membership/hashes are checked before replay; later append-only receipts do not rewrite original archive bytes.

## Sources

- [Ledger](../../../data/patch-api/sources/2.5.3-page-coverage.json) — every literal row and precise UNPROVEN contracts.
- [Local inventory](../../../data/patch-api/sources/2.5.3-wikitext-register.json) — exact source occurrences and eight headers.
- [Frozen source cache](../../../data/patch-api/source-cache/legacy-2026-10-09/README.md) — capture/attribution boundary.
- [Historical handoff](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md) — integration background; current explicit bounded request overrides operations/delegation/broad gates.

## See Also

- [[patch-3-4-3-api-audit]] — distinct Wrath history and empty-inventory proof limits.
- [[client-profiles]] — runtime architecture, not TBC native proof.
