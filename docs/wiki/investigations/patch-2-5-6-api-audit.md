# Patch 2.5.6 Classic/TBC SOURCE audit

Bounded SOURCE/contract slice of the frozen full registry through 1.0.0, base `864b4f7e46319704732ba82eef8ffc0757771610`, verified source identity 2026-10-09. Own branch `p256-source`, worktree `/home/osso/.worktrees/wow-ui-sim-p256-source`. No other page audit or runtime/native completion.

## Literal identity and client boundary

Frozen Warcraft Wiki page **685353**, revision **6778086**, timestamp **2026-07-22T05:42:00Z**. Response hash `4b5c835e8f549a1e578be9a57b31c995c455d1f35560dffb44d74989d3a711f0`; raw **307 bytes**, hash `9ea30f11808982a5c3f0a051b5a41bd8f7472ec7450c8e3576cdb515a4f9978a`. Identity, hashes and returned content validated before copying; original frozen cache untouched. [Pin](../../../data/patch-api/evidence/2.5.6-session-2026-10-09/source-pin.json), exact response, full manifest and full registry copied locally.

Literal `* TOC: <code>20506</code>` plus navigation `prev=2.5.5` and both 2.5.5-to-2.5.6 resource diffs define the **205xx Classic/TBC accounting line**. The raw page does not spell out “TBC”; `tbc-classic` labels this TOC-derived source boundary, not a version-only guess or native measurement. It is not historical retail 2.x. Source TOC 20506 is a native interface expectation asserted by the page, **not an observed native client**.

Only same-line successors may establish supersession. Frozen registry has 2.5.1–2.5.6: 2.5.5 is a predecessor even though its editorial revision is newer (August 20, 2026 vs July 22, 2026). No later same-line source contract established here; `later_registers=[]`. Retail 3.x, Cata 4.x, Wrath Classic 3.4.x and Era 1.x do not supersede this line. Literal Era transclusion is a reference, not a TBC equivalence/successor contract.

## Exact SOURCE coverage matrix

[Serialized ledger](../../../data/patch-api/sources/2.5.6-page-coverage.json) preserves every nonblank raw row. Blank rows 2/6 have no capability.

| Raw line / ID | Literal boundary | Status / proof |
|---|---|---|
| 1 / source-context-001 | `{{apichanges\|prev=2.5.5\|2.5.6}}` | Metadata navigation, no successor proof |
| 3 / source-context-003 | Resources heading | Metadata |
| 4 / source-context-004 | TOC 20506 | Metadata source expectation, not configured/native observation |
| 5 / source-context-005 | Two external diff links | Metadata; unexpanded, not member inventory |
| 7 / source-context-007 | Blue posts heading | Metadata, no local summary contract |
| 8 / prose-undated-008 | `{{:Patch_1.15.9/API_changes}}` | **UNPROVEN** unexpanded blue-post contract |

**Six nonblank rows: five metadata, one UNPROVEN contract. Zero explicit API occurrences, signatures, summary statements, inventory header counts or removals.** No publication register or model closure. Empty local inventory is not API/full-UI parity; unexpanded content is not empty content.

The transclusion supplies no local member identities, arguments, returns, event triggers/payloads, state transitions or security rules. Each unspecified field remains null/UNPROVEN in the contract ledger; no cheap runtime implementation justified. Native equivalence between the referenced Era page and TBC is also unspecified. External diffs and transclusion are not retrieved, reconstructed or fabricated.

## Actual configured code and manifest observations

[Historical observation](../../../data/patch-api/evidence/2.5.6-session-2026-10-09/profile-observation.json) derives from copied base Cargo feature graph, Rust profile/interface/cache arms and complete committed manifests—not names alone. Copied classifier/generator delimit tool support. No cache contents or TOC payloads loaded; manifest paths cannot supply TOC interface values or publisher/native measurements.

| Feature / cache scope | Configured interface | Manifest entries / TOCs / Mainline TOCs |
|---|---:|---:|
| client-retail / retail | 120100 | 4041 / 394 / 62 |
| client-ptr / ptr | 120105 | 4025 / 374 / 45 |
| client-wrath / wrath | 38001 | 3981 / 360 / 56 |
| client-mists / mists | 50504 | 3981 / 360 / 56 |
| client-era / era | 11507 | 3981 / 360 / 56 |
| client-anniversary / anniversary | 11507 | 3981 / 360 / 56 |
| client-wowforever / wowforever | 16001 | 4398 / 349 / 2 |

Wrath/Mists/Era/Anniversary manifest bytes are identical at this base; each contains `_Mainline.toc` paths. These are literal path inventories, not proof of a loaded UI or appropriate client source. Anniversary is **configured 11507**, not assumed TBC 20506 because of its name. No inspected configured interface equals source 20506; this is **not an unsupported-client/runtime diagnosis**. Native correspondence remains UNPROVEN. Cache/runtime/model/native/full-UI measurements: **not performed**.

Current generator choices: retail, mists-classic, classic-era. Current publication classifier includes those plus wrath-classic, but not this TBC line. No default-retail surrogate or shared classifier solely to probe a wrong profile. Local source inventory remains separate; existing section parser and extractor are copied byte-for-byte for historical use.

## Reproducible extraction and immutable proof

Stock extractor `--text-only --canonical-patch-navigation` rejects literal `{{:Patch_1.15.9/API_changes}}` with `unhandled template`. Own validator replaces exactly that marker with `[Transcluded source: Patch_1.15.9/API_changes; not expanded]` before calling the copied stock extractor. Exact transform and flags retained in ledger; this is a source-only local adapter, not a shared-tool fallback or linked expansion. Plaintext preserves positional 2.5.6 and all six rows.

Own validator checks frozen response/manifest/registry identity and hashes, exact raw/text reproduction, every source row and absent signatures/headers/local summaries, precise contract fields, profile observations rederived from copied configuration/manifests, and exact seals. Replay uses its relocated `__file__`, not Git or live repo/cache/tool state. [Targeted fixtures](../../../data/patch-api/evidence/2.5.6-session-2026-10-09/test_source_accounting.py) mutate serialized facts and reject fabricated proof. [Controls](../../../data/patch-api/evidence/2.5.6-session-2026-10-09/replay_controls.py) mutate ledger/log, reject exact seals, restore bytes/hashes, archive sealed inputs and run a fresh process with no Git/target/current tools/current runtime.

## Proof ledger and ownership

Targeted development RED retained: required derived accounting fails against temporary empty validator; exact stub/hash/log preserved. First coherent implementation `95a35546d`; packaging correction `c77aab464` removes generated bytecode and retains ignored RED log explicitly. At `c77aab464` (full tested revision in [proof ledger](../../../data/patch-api/evidence/2.5.6-session-2026-10-09/source-proof.json)), SOURCE GREEN **9/9**, own historical validator exit **0**, **27** sealed inputs. Derived six source rows/five metadata/one UNPROVEN, zero explicit APIs/signatures/local summaries/headers/removals; seven configured profiles and zero runtime/native/model observations. At `4db382e41`, serialized ledger and sealed GREEN-log tamper controls each exit **1** at their exact seal; original bytes/hashes restored. [Portable receipt](../../../data/patch-api/evidence/2.5.6-session-2026-10-09/portable-proof.json) records exact execution revision/argv/cwd and input seals; [control log](../../../data/patch-api/evidence/2.5.6-session-2026-10-09/portable-controls.log) records both rejection traces, original/tampered/restored hashes and every archive member hash. Fresh relocated replay exits **0**, derives the same SOURCE summary with **30** historical sealed inputs, no Git/target/current tools/current runtime, and PATH `/nonexistent`. Archive **228401 bytes**, SHA-256 `7a9d805360ac34619da76f13dc57e7f1efcdd5f24f87fa5259ae649d4e553ed1`. Its 31 members include the historical 30-input seal map; later outer seals include archive/receipts separately. Source/test/validator/control/copied tool/configuration bytes remain unchanged, so original proof remains applicable—not a fresh rerun or final acceptance. No broad/check/lint/profile/startup/full-suite/final gate. No Rust edits, delegate/model CLI, operational change, cache/vendor/Wowless edit, push, merge or deploy. Python kept manually formatted; no lint/formatter installation.

Outer map now records **33** sealed inputs; archive retains its original **30**-input map. Own capture script refuses to overwrite a sealed archive: extract the retained archive into a fresh directory and run its relocated `validate.py` for repeat historical replay. No original Git objects, target, live cache or current mutable tool/source/configuration files needed. Main retains integration/native/final-gate ownership. This bounded source slice cannot close runtime compatibility or broader registry accounting.

## Main integration — 2026-10-09

Integrated source slice at `93bcc6afe`; no shared parser or runtime changes. Original archived source, configured observations and 33 outer seals remain historical inputs, not current native measurements. Independent SOURCE replay gate pending. Unexpanded Era transclusion remains UNPROVEN; zero API/model/native credit. No surrogate Anniversary 11507 probe for source TOC 20506.

## Sources

- [Raw](../../../data/patch-api/sources/2.5.6-api-changes.wikitext), [plaintext](../../../data/patch-api/sources/2.5.6-api-changes.txt), [ledger](../../../data/patch-api/sources/2.5.6-page-coverage.json).
- [Own evidence](../../../data/patch-api/evidence/2.5.6-session-2026-10-09/), [spec](../../specs/patch-2-5-6-source-accounting.md).
- Canonical read-only source-accounting templates: [[patch-3-4-3-api-audit]], [[patch-3-4-1-api-audit]]. No Wrath contracts transferred.

## See Also

- [[client-profiles]] — configured architecture, not native proof.
- [[patch-3-4-3-api-audit]] — separate Classic line/source-only summary limits.
- [[patch-3-4-1-api-audit]] — source row/signature/provenance accounting.
