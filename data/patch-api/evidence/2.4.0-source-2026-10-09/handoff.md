# Independent historical Retail 2.4.0 source handoff

Owned checkout `/home/osso/.worktrees/wow-ui-sim-p240-source`, branch `p240-source`, base `f95eed96e`. Scope complete as a bounded source audit, not current-runtime/native/final acceptance. Main owns integration and native gates. No operations, push, merge, deployment, delegation, model CLI, Bash or cwd switch.

## Identity and accounting

Page 73272, revision 6471380, timestamp 2025-09-13T09:55:32Z. Historical 2008 Retail, never TBC Classic 2.5.x.

- Response SHA-256: `cef58e2e702a5dbc9b6291d719b9534199766c3e2f6e5d83bb42f57eec92c053`.
- Raw SHA-256: `e2269b1705daa604bd16aff912e92b5cbd7ee2e11681137b4543d74150ce1909`.
- Registry SHA-256: `e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c`: 101 retained older-page identities through 1.0.0, not 101 audited pages.
- Inventory: 50 occurrences (45 globals, one widget method, two events, one CVar, one console example). Source: 99 nonblank rows (73 unproven, 26 metadata including 24 headers). Signatures: 46 boundaries (35 explicit call fragments, 11 unspecified/contextual). Unique ledger IDs: 195, pending 169, metadata 26. All contract limitations are occurrence-linked; no skipped literal row.
- Meaningful closures: zero. Runtime publication not measured. Pending contract IDs are not a measured runtime gap count. No historical output/security/behavior/native credit, aliases, shims, fallbacks or new runtime model. No Rust files/test modules changed; file-level retail cfg requirement therefore has no new instance.

## Exact development proof ledger

Commands run from the owned worktree with explicit Pyrun argv/cwd. Full commands, revisions, exits, scopes and failures in `proof-ledger.json`.

| Exact revision | Command | Exit / proof |
|---|---|---|
| `33039957951bbdfc751b83b1333fe9c5d901dcb6` | `python3 -B tools/test_patch_2_4_source.py` | 1: two intended missing opt-in feature assertions. |
| `a0d60be60920ec7e512ebe455bcd44d3c88d4a8f` | same | 1: actual count 50, not guessed 49; unchanged default extract rejects Ref web. Test expectations corrected, not suppressed. |
| `00708e36af358933032623acce28ceae469ce3e2` | same | 0: 2/2, literal references/full extract and own default preservation. |
| `5f59bfb2fc96ea3b6aa79ecefe54c6ec268953e4` | `python3 -B tools/test_patch_2_4_replay.py` | 1: three intended missing historical validator/original-ledger assertions. |
| `c5ea90ccf07f7ea22f376eae067fe179dcda7ab9` | `python3 -B data/patch-api/evidence/2.4.0-source-2026-10-09/replay_parsers.py` | 0: 156 defaults byte/error-equal to base; 79 recorded routes compared to base. 76 register / 70 extract exact matches; inherited nonmatches unchanged. |
| `44efecb5a3f164bf23d782a7486d50acbe63cbb9` | `python3 -B tools/test_patch_2_4_replay.py` | 0: 3/3, fresh copied no-Git/no-target/no-current-source replay, nine serialized tamper/reject/restore controls and omitted literal row rejection. |

No broad/check/lint/readability/profile/startup/full-suite/final gates. Later changes only retain generated receipts/logs and tracked docs; parser/source proof scope unchanged. Logs ignored by repo defaults are explicitly committed as audit evidence, not regenerated.

## Portable originals / zero closures

Copy this entire evidence directory to a fresh location. Run `python3 -B validate.py` there, with Git unavailable. It imports only the sealed archived parser; never discovers current checkout files or target. It reconstructs all source rows, occurrences and signature boundaries, verifies frozen response/source/registry identity and derives gap/successor counts. Parser-regression receipts are retained rather than wastefully rerun on every seal check. `replay_parsers.py` independently reproduces archived defaults and recorded routes when explicitly needed.

Original ledgers/gaps, code/parser/input/output archives and logs have 432 seals. Independent empty-closure ledger/gaps/archive declaration have three seals; no fabricated closure code/logs. Root SHA-256 `084ccd99697600fc9dda41478927e4fec56cdc9ab13e0a34a39872c4bb27b33a`. Serialized controls reject original ledger, gaps, parser log, archived parser, register, raw source, registry, closure ledger and seal manifest; each restoration matches exact original SHA-256 and clean replay output. Later development logs/proof ledger/handoff have separate `development-seals.json`; originals remain unchanged. These are integrity hashes anchored by the committed evidence, not signatures against an attacker who can replace every anchor.

Recorded register nonmatches: 10.0.0, 12.1.0, 5.0.4. Extract nonmatches: 10.0.0, 10.0.2, 10.1.0, 10.1.7, 10.2.5, 12.0.5, 12.0.7, 12.1.0, 9.2.5. All compare equal to base behavior; no adjacent repairs or existing-output modifications.

## Main integration ownership

73 actual later Retail registers are pinned, starting 3.2.0/3.3.0/3.3.3/3.3.5/4.0.1; these five have zero tuple overlap. Full later literal overlaps and directions in `successors.json`; they confer no historical behavior closure. Classic 2.5.x, Wrath 3.4.x and Era never supersede this source.

Main must replace five queued placeholders with real registers before any integrated current publication measurement:

| Queued patch | Frozen lexical overlap, not supersession proof |
|---|---|
| 2.4.2 | CombatLogGetNumEntries, CombatLogSetCurrentEntry |
| 3.0.2 | None |
| 3.0.3 | None |
| 3.0.8 | SetFriendNotes |
| 3.1.0 | GetTalentLink |

No source-only GREEN statement implies current simulator publication or native 2008 contract parity. Do not restore APIs already removed by actual later Retail pages merely to satisfy historical spellings.
