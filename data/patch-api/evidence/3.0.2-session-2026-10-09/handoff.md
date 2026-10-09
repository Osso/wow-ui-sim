# Retail 3.0.2 bounded source handoff

Owned cwd: `/home/osso/.worktrees/wow-ui-sim-p302-source`, branch `p302-source`, canonical master base `864b4f7e4`. Initial worktree add ran from `/home/osso/Projects/wow/wow-ui-sim`; subsequent argv commands explicitly use owned cwd. No cwd switch, Bash, push/merge/deploy/delegation/model CLI or broad/final gates.

## Goal and result scope

Complete literal frozen-page accounting, opt-in default-preserving parser, precise model criteria/UNPROVEN limits and sealed historical replay. Native/current runtime publication is NOT MEASURED. Zero runtime changes, retirements, meaningful closures or native claims. [Spec](../../../../docs/specs/patch-3-0-2-publication-sweep.md), [audit](../../../../docs/wiki/investigations/patch-3-0-2-api-audit.md).

431 nonblank rows = 42 metadata + 389 UNPROVEN substantive. Separate dimensions: 373 inventory (332 globals, 33 widget methods, 2 FrameXML, 3 events, one widget handler/CVar/click modifier each), 367 signatures (294 literal call texts/73 missing call syntax), 80 prose/qualifications, 41 headings. Zero literal console commands. Raw 25,613 bytes; frozen page 482353/revision 4638841/time 2020-02-23T21:55:29Z. Registry snapshot ends 1.0.0, not an assertion that all older pages were audited.

## Archives and seals

`original/` archives response/raw/manifest/registry, original source ledger/gaps/register/extract, static code and actual/queued inputs. Historical generator/extractor, deterministic `accounting.py`, validator and RED/GREEN logs are owned originals. SHA256 `seals.json` names exact relative artifacts; immutable logs cannot be edited into PASS. `closures/claims.json` and `closure-seals.json` remain distinct, explicitly empty. Validation reads only local archived paths, no current mutable ledger/registry/source files or target/Git. Seals are commit-pinned drift detection, not signed native attestation.

## Main-owned integration

Actual archives: 3.3.0, 3.3.3, 3.3.5, 4.0.1. First three have no tuple overlap; 4.0.1 has seven literal global removals listed in `original/overlaps.json`, no behavioral closure. Queued 3.0.3/3.0.8/3.1.0/3.2.0 raw inputs are ordered placeholders, not applied registers. Literal shared-name counts: 0/0/14/1; exact names in `original/queued-literal-overlaps.json`. Main replaces placeholders in ascending retail order, reconciles current observations and owns native/integration acceptance. Never use Classic 3.4.x/TBC/Era supersession.

## Command proof ledger

Every command below uses explicit owned cwd and argv via Pyrun; tests scoped to this page only.

| Command argv | Revision/scope | Result | Later invalidation |
|---|---|---|---|
| `python3 -B tools/test_patch_3_0_2_source.py` | ab37ab930 test-first, missing parser/flag | RED 4 expected failures; red.log | Historical RED retained |
| `python3 -B tools/test_patch_3_0_2_source.py` | 74684cc53 launch parser | GREEN 4/4; green.log | Expanded assertions require their own later run; implementation proof retained |
| `python3 -B data/patch-api/evidence/3.0.2-session-2026-10-09/test_accounting.py` | d18cf30fc test-first | RED 4 expected missing-validator failures; accounting-red.log | Historical RED retained |
| Generator argv in provenance, own frozen raw/output and opt-in flags | staged source-accounting implementation | exit 0; generator.log | No runtime proof |

| `python3 -B tools/test_patch_3_0_2_source.py` | 6027d1349 expanded concrete frozen assertions | GREEN 4/4; expanded-source-green.log | No later source change |
| `python3 -B data/patch-api/evidence/3.0.2-session-2026-10-09/test_accounting.py` | 6027d1349 complete frozen replay | GREEN 4/4; accounting-green.log; 1,292 omission controls, false-credit/foreign-successor rejection | No later code/input changes |

Relocated fresh process has only copied evidence, an empty PATH, no Git/target/current mutable files. Initial replay succeeds; serialized ledger, original green log, raw source and frozen generator each reject at their exact seal. Each restoration asserts byte equality and successful fresh-process replay. All tampering occurs in disposable copied evidence, not owned originals. Test assertions provide source-accounting proof, not native/publication observations.

Original 36-input seals and one closure seal stay byte-identical to 6027d1349. `receipt-seals.json` separately seals the two later GREEN logs; no historical RED/GREEN artifact was edited. Python formatter executables were absent; new Python was manually formatted before commit. No Rust code/test files added, so no profile-only file guard was needed.

No historical-native profile was invented. Main must not present this source result as native/API compatibility completion. Bounded source accounting delivered; main integration and native/current behavior acceptance remain open.
