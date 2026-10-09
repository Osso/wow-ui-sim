# Retail Patch 3.3.0 API audit

Frozen historical Retail source; verified 2026-10-09. Page 522376, revision 6055853, timestamp 2024-06-04T04:47:16Z; manifest response and wikitext hashes match and response content equals wikitext.

## Scope
Eleven explicit publication identities including the quest response event, GetObjectType successor and MouseIsOver replacement reference. Seven explicit NEW call signatures are separate from publication. Retain complete page prose including XML attributes, namespace, GUID and macro semantics. Linked Special:Diff/4997637/4997649 remains an unexpanded source boundary.

Retail 3.3.3, 3.3.5 and 4.0.1 are ordered queued placeholders; actual Retail 4.1+ registers follow. Wrath Classic 3.4.x is separate history and never supersedes this page. No runtime removal or model closure yet. Main owns integration and final acceptance.

## Proof
Parser RED records missing opt-in summary support. New generator `--wrath-retail-summary` and extractor `--wrath-summary-markup` preserve defaults. Targeted GREEN/accounting proof pending.

## Sources
- [Frozen source](../../../data/patch-api/sources/3.3.0-api-changes.wikitext)
- [Source pin](../../../data/patch-api/evidence/3.3.0-session-2026-10-09/source-pin.json)
- [Spec](../../specs/patch-3-3-0-publication-sweep.md)

## See Also
- [[patch-4-1-0-api-audit]] — actual later Retail source.

## Established model closure
XML `motionScriptsWhileDisabled` was silently discarded by FrameXml deserialization. Add the bool attribute and apply it to the existing Frame motion flag in both XML loading and Lua CreateFrame template application before scripts. Explicit instance/derived false overrides inherited true. No new backing model, API shim or vendor modification. Behavioral RED: real XML addon loads but inherited getter returns false. GREEN pending. `registerForClicks` remains deferred: source does not establish its token/delimiter grammar. Texture source-file dimensions remain deferred; widget/atlas dimensions are not interchangeable.
