# Patch 1.13.7 literal SOURCE audit

Frozen Classic Era audit scope: page46829/revision458410, revision timestamp `2021-09-04T08:55:38Z`, retrieved `2026-10-09T08:51:36.399298+00:00`. Raw1,179 bytes; source contains no literal client name. Client-history association follows this audit's explicit Era scope, not invented source metadata. Base `5b12dac256abc0ffaf541df2bc445e8381612d2d`.

## Coverage matrix

| Literal feature | Accounting | Proof |
|---|---|---|
| Raw rows/default rendered extract |25 nonblank raw/5 extracted;19 metadata/6 UNPROVEN raw|SOURCE only; raw inventory retained despite default extract dropping tables|
| Added global |GetDefaultScale, one occurrence|UNPROVEN; no arguments, returns, state or security declaration|
| Added CVars |heardChoiceSFX, lastCharacterGuid, seenTBCInfoPane, showUnactivatedCharacters|Four occurrences; no literal defaults, scopes, descriptions or transitions|
| Headers |Summary, Global API, CVars; two Added table headers|No numeric counts/caption/build/date; no inferred zero removal counts|
| References |Two diff links, navigation template and five API templates|Unexpanded; seven substantive contracts, no prose/examples/defaults|

Literal navigation1.13.6→1.13.7→1.14.0; TOC11307. Manifest-linked registry101 ends at1.0.0. Exact response/body equality, registry membership and hash validation precede derivation. Raw SHA256 `b9f8be9d2db166e6b85ec868fc077bc5591071d55c05bb5f11267eb5ba11e996`; response SHA256 `b0d750e17ee76815e6c4917f9d98f81780ef3b5bb26466acbcc9e1a8899099c4`.

## Existing state assessment

Copied `state-inputs/src/lua_api/workarounds/temporary/display_scale_defaults.rs` explicitly says the display/render-scale model is missing and supplies return1. This is a temporary compatibility default, not meaningful modeled display state. The page does not specify signature/output/state transitions, so calling it would measure current behavior without testing a historical contract. No fabricated zero-argument signature/default/native equivalence or production change.

Source scan of five literal names finds GetDefaultScale in that temporary shim and lastCharacterGuid in a retail12.0.7 gated removal list (`state-inputs/src/cvars.rs`). Neither source availability nor that foreign retail history establishes Era behavior/removal. Four source CVar defaults/descriptions absent; no getter value can count as source-default equality. No modeled behavior closed; no factory/runtime measurements or native proof claimed. Canonical1.14.3/2 audit templates show why publication and getter receipts must stay separate from SOURCE/model/native evidence.

## Client and successor boundaries

Copied current configuration selects Era/Anniversary11507, distinct from source11307. No build/date/native correspondence invented. Same-history successor inputs preserve frozen pin/response/body identity:1.14.0/1 completed-queued,1.14.2–1.15.9 integrated-not-applied. No later registers applied. TBC wording inside the `seenTBCInfoPane` name and diff target contents does not authorize linked expansion, TBC identity or retail/TBC supersession. Main owns ordered integration and native/final gates.

## Own proof

SOURCE RED eight assertion failures retained; implementation committed before GREEN. GREEN and portable receipts pending. Historical generator/extractor defaults unchanged, flags `[]`; no shared tool edits. Later observations/receipts remain separate from original source seals. No final gates, broad suites, Cargo/dependency/toolchain/cache updates or production operations.

## Sources

- [Spec](../../specs/patch-1-13-7-source-accounting.md).
- [Frozen source pin](../../../data/patch-api/evidence/1.13.7-session-2026-10-09/source-pin.json), [ledger](../../../data/patch-api/evidence/1.13.7-session-2026-10-09/ledger.json), own SOURCE tests and historical tools in the same directory.
- [1.14.2 audit](patch-1-14-2-api-audit.md), [1.14.3 audit](patch-1-14-3-api-audit.md): read-only accounting/factory templates; not imported runtime or model credit.

## See Also

[[patch-1-14-2-api-audit]], [[patch-1-14-3-api-audit]] — same-Era successors with distinct literal contracts and configured-state observations.
