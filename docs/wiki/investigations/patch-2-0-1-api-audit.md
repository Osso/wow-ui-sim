# Historical retail Patch 2.0.1 SOURCE audit

Frozen page 324401/revision 3129557, timestamp 2019-06-15T12:04:30Z; describes historical 2006 retail, not Classic 2.5.x, Wrath 3.4.x, Era or current native behavior. Response SHA256 `5f702e6671d16de64f15bc48711ac2bc9ff19dde314e469273b334e98828ea67`, raw SHA256 `2c932b03d47af4668069840292c2b523e70deb52a1402a724a532ac71e40ad86`, 27,891 bytes. Exact response slot and frozen manifest checked before copying; manifest-pinned full registry has 101 pages ending at 1.0.0.

## Literal accounting / capability matrix

| Domain | Current source accounting | Proof / boundary |
|---|---|---|
| Full raw page and headers | 316 nonblank lines; 35 named headers; 280 substantive prose/example rows, 36 metadata rows | Every raw line literal, including XML, Lua upgrade/examples, macro conditions/reset/feedback, casting, security/protection, saved variables, performance and bug fixes. No numeric header count claims; headers carry `count_claim: null`. Row count is not a count of independently specified behaviors. |
| APIs and strings | 87 global call mentions, four bare replacement-name references, one TargetNearest* family, three string-method references | Argument/return declarations remain in `declaration_literal` and the whole source row. Names alone do not establish full callable signatures, defaults, endpoints or models. All UNPROVEN. |
| Widgets | Eight method calls; eight script-name mentions; 31 wrapper fragments (30 named handlers plus default wrapper); nine secure template names | XML inheritance/metatables, secure handler inheritance, combat protection, tooltip growth, mouseover, attributes and click ordering remain literal, not measured. No broad template/widget equivalence claim. |
| Events / CVars / commands | 21 event mentions, three unexpanded event-family mentions; zero CVars. 79 command mentions, five command families, one `/command` syntax placeholder, one `/alt` reset-option suffix | Event payloads/prose remain whole literal rows, including obsolete families, protection event ordering and CHAT_MSG_SPELL_SELF_BUFF container exclusion. Command mentions include disallowed/example uses, not 79 newly published commands. No wildcard expansion or guessed CVar/default. |
| Signatures / references / credit | 206 fragment records: 87 globals, eight methods, 31 wrappers, 78 macro/command fragments, two Lua examples. Twelve reference records. 793 UNPROVEN gap records | Literal fragments do not reconstruct unspecified contracts. All twelve links/transclusions UNPROVEN and unexpanded. Zero runtime/publication/native observations, model closures, aliases, shims, fallbacks or retirements. |

The current ledger contains 864 records across raw rows, headers, occurrences, signature subset and references, with 658 distinct IDs; signature records deliberately reference occurrence IDs. Preserve `ButtoName`, `charaters`, `targetting`, `Programatic`, `interchangably`, `/randomcast` and `/randomuse` exactly. `/randomcast` is not normalized into `/castrandom`. All optional parameters, return-name declarations, ranked/name/ID alternatives and event payload annotations remain literal. The malformed fixture retains an unterminated signature rather than supplying missing syntax.

Explicit raw qualifications: 33 rows under/marked 2.0.3, four under 2.0.4, five under 2.0.6, two under 2.0.1; 272 have no explicit row qualifier. These totals include headers. Later-patch annotations are not silently attributed to launch 2.0.1.

## Successor boundary

[Original successor receipt](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/successor-boundary.json) seals 74 actual later retail registers from this checkout, 41 same-symbol mentions and zero supersession credit. Name overlap is not tuple/behavior parity or proof of indirect changes. Classic histories excluded. Queued 2.1.0/2.2.0/2.3.0/2.4.0/2.4.2/3.0.2/3.0.3/3.0.8 remain main-owned, ordered and unused for supersession even where template artifacts are available.

No backing-state/runtime probe performed. Existing modern simulator state cannot by itself establish historical secure execution, command transitions, event timing, Lua GC/performance or persistence. No new runtime/model proposal made; main owns supported publication, native comparison and integration/final gates.

## Original versus current immutable evidence

[Original manifest](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/historical-inputs.json): 18 sealed files, 114 independently hashed snapshots in a 371,564-byte archive. Includes original ledger/gaps/source/response/logs, full frozen manifest and 101-page registry, exact revision-specific source fixtures/tools, shared tools, actual successor registers, queued raw inputs and canonical 3.0.2/3.0.3/3.0.8/3.4.0 templates. Original ledger retains 295 lexical occurrences, 11 outer references and 792 gap records. Original signatures/headers/source unchanged.

[Original validator](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/validate.py) reproduces six revision-specific SOURCE RED/GREEN command results from sealed code, preserving original physical logs. Timing/path normalization affects comparison only, never original logs. Four recorded register outputs reproduce byte-for-byte (default 2.0.1 plus opted-in 3.0.2/3.0.3/3.0.8); default own extract and three recorded template extracts reproduce. Shared parser/extractor untouched relative to `93bcc6afe`; no new shared opt-in required.

[Separate current manifest](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/current/inputs.json): nine sealed files, corrected `/alt` and `/command` classifications and separately retained nested forum reference; 12 references/793 gaps. [Current validator](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/current/validate.py) verifies original 18 seals plus both original outer hashes, replays corrected accounting and six frozen own source cases. It never rewrites or consumes mutable current source ledgers. Manifests anchored in validators; seals are local integrity controls, not externally trusted signatures.

## Exact bounded development proof

[Original command ledger](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/original-command-ledger.json), [current command ledger](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/current/command-ledger.json) and [completion receipt](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/completion-proof.json) retain argv, cwd, exact revisions, logs and scope validity.

| Proof | RED | GREEN / scope |
|---|---|---|
| Own SOURCE fixtures | `9276337ae`, three missing-tool failures; `c2f1531e1`, one boundary failure; `68c7d540e`, one identity failure | `19b90446e` 3/3; `24f756e3e` 4/4; `615a5cf74` 5/5. Earlier subset proofs retained historically, not final expanded scope. |
| Original copied historical replay | `edb2fca5b`, two missing-validator failures | `717009936` 2/2; 1,655 decoded omission controls plus four spelling/signature/history/credit corruptions; all 18 sealed files plus manifest reject serialized tampering and restore exact bytes (19 controls). Synthetic mutable current ledgers do not change stdout. |
| Current literal refinement | `7e3d66d3a`, one expected boundary failure/five passes | `5075cf42b` 6/6; all final source fixtures also execute from frozen copied files in current replay. |
| Separate current copied replay | `7afa9df38`, one missing-validator failure | `939d8b73b` 1/1; 1,657 decoded omission controls and all nine current sealed files plus manifest reject/restore (10 controls). Original 18 seals remain unchanged. |

Copied fresh processes use isolated Python with unusable Git/tool PATH and empty PYTHONPATH; copied roots lack Git, target, src and mutable current tools/sources. Temporary tool trees contain only archived inputs. Original/current fixtures are distinct scopes, not a claim that the whole current test module ran in one command. Largest sealed ledger is below 560 KB; every retained file below 5 MB. No formatter installed; Python manually formatted before commits. PLAN.md checkboxes updated locally but excluded by explicit plan-md skill policy (`NEVER track PLAN.md`). No check/lint/readability/coverage/broad/profile/startup/final gates, network, delegation/model CLIs, canonical/vendor/cache/Wowless writes, push/merge/rebase/deploy.

## Sources

- [Spec](../../specs/patch-2-0-1-source-accounting.md).
- [Literal source](../../../data/patch-api/sources/2.0.1-api-changes.wikitext).
- [Current source ledger](../../../data/patch-api/sources/2.0.1-page-coverage.json).
- [Source pin](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/source-pin.json).

## See Also

- [[patch-3-0-2-api-audit]], [[patch-3-0-3-api-audit]], [[patch-3-0-8-api-audit]] — read-only source/replay templates; no inherited behavior credit.
- [[patch-3-4-0-api-audit]] — separate Classic history, not a retail successor.
