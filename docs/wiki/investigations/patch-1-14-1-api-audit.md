# Patch 1.14.1 frozen Era API audit

Frozen SOURCE accounting only. Exact Warcraft Wiki page102588/revision1010974/timestamp2022-03-02T05:21:47Z, retrieved2026-10-09T08:51:36.399083+00:00. No links or templates expanded. [Spec](../../specs/patch-1-14-1-source-accounting.md).

## Source identity and coverage

Raw6207bytes/SHA256 `41c615541135cabceb8ff85bb0ff633ffa8d4eafe861e724d6fdf60e739d8559`; responseSHA256 `aac93df0da292053fee1aee1534d2de00101cf01c46dcb51ab1be858a88c30f8`. Frozen manifest and response agree; registry101 ends1.0.0. TOC11401; literal caption1.14.0/build39958 →1.14.1/build41030/Nov10,2021 is distinct from revision time/configuration.

| Source contract | Count | Proof/remaining boundary |
|---|---:|---|
| Physical/nonblank rows |120/114|Lossless SOURCE;64 substantive rows UNPROVEN,50 metadata|
| Global callable additions/removals |14/3|Names only; no signatures, security, aliases or semantic retirement proof|
| Widget additions/removals |2/0|Exact `FontString:GetTextScale`, `FontStringSetTextScale`; latter not normalized to a method|
| Event additions/removals |31/1|Names only; no payload/order/delivery model|
| CVar additions/removals |7/1|Four lexical defaults and descriptions; Account scope on seenRegionalChatDisabled; four bare-link defaults unknown|
| Headings/numerical headers |6/8|Observed counts equal14/3,2/0,31/1,7/1; zero-removal widget column preserved|
| Unspecified signatures |59|Every inventory identity retains unknown arguments/returns/payload; zero declared signatures|
| Summary/template inheritance claims |3/2|C_Seasons purpose; both tooltip templates no longer inherit BackdropTemplate; event trace hidden logging supported, disabled by default|
| Links/template occurrences |14/57|Exact boundaries, no linked member/body/native credit|

All original literal wording remains in `source.wikitext` and `ledger.json`, including HTML hidden CVar descriptions, dynamic-scaling BETA/hitching/vsync caveats, texture-heap fallback wording, UK AADC account alert, Wow-inline markers, navigation and commented WikitextDiff reference.

## Meaningful-model review

Existing FontString getter reads `frame.text_scale`; setter writes per-frame visual state (`src/lua_api/frame/methods/text_attribute_event/text/style.rs:457–475`). Direct positive-value roundtrip is grounded as existing simulator behavior, not a native/default/signature inference. Standalone offline Era probe GREEN1/1 at `a6d38f864`: first FontString1.5→2.25, second remains0.75. Proves isolated existing getter state, not native/default/signature semantics. Actual style/test/Cargo inputs retained separately in current receipts. Typo-shaped source setter cannot justify adding an alias.

C_Seasons: namespace purpose and member names omit IDs/values/activation transitions. No season model found in retained exact-name scan; fake defaults or no-op activation not justified. Tooltip claims require actual11401 XML; current11507 profile configuration does not prove inheritance. Hidden event-trace setting belongs to Blizzard tool UI, not generic frame event registration; setting identity/tool implementation unretained.

Community/social/commentator/messaging changes omit state transitions, arguments, returns, event payloads and security. Existing placeholders/permissive Classic event registration are not community transport or UK AADC models. Secure-call paths already exist but page gives no taint/error/iteration semantics; no security change justified. Render/cache CVar registry roundtrips cannot prove dynamic-GPU scaling, heap strategy, chat alert history or seasonal notifications. Bag-bank index has no source numeric boundary. All precise limits retained in `model_review` and exact code scan/snapshots.

## History/configuration boundary

Era/Anniversary currently configured11507; source11401 not native parity. Same-Era1.14.2/3 queued for main integration. Actual canonical1.14.4 and1.15.0–9 ledgers copied/hash-recorded as inputs, not applied supersession. No Retail, Wrath, Mists, TBC or Forever histories imported. Main owns integration/native/final gates.

## Targeted evidence

Own SOURCE RED8/8; development GREEN8/8 with353 omission controls and fabricated numeric/alias/foreign-credit rejection. Historical register defaults captured unchanged. Extractor CLI requires repo paths rather than positional raw input; failed invocation retained, then own import adapter used unchanged `extract_text` defaults. No shared-tool changes. Portable RED3 retained; GREEN3/3 at `a6d38f864` includes fresh copied SOURCE8/default-register/default-extract byte replay, PATH empty, no Git/target/current tools, both ledger/log disk tampers rejected and exact originals restored.160 original seals unchanged,161-member402872-byte archive; later receipts separately sealed. Ignored log files explicitly force-tracked, so direct replay survives ordinary checkout as well as archived copy. Offline Era target GREEN1/1 at the same revision, no dependency network/cache copying/toolchain changes. Six existing library warnings/one existing binary warning and six vendor manifest deprecations retained; no suppression or warning-free claim. No broad/final gates.

## Sources

- [Frozen manifest](../../../data/patch-api/source-cache/legacy-2026-10-09/manifest.json).
- [Own evidence](../../../data/patch-api/evidence/1.14.1-session-2026-10-09/ledger.json) — frozen identity, literal inventory/limits and actual successor boundaries.
- [State scan](../../../data/patch-api/evidence/1.14.1-session-2026-10-09/state-scan.json) — historical code locations, not native proof.

## See Also

- [[patch-1-15-0-api-audit]] — separate seasonal namespace/link boundaries.
- [[patch-1-15-1-api-audit]] — independent existing-state enum alias proof, not applicable supersession here.
