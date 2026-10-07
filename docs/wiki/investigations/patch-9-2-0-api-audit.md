# Patch 9.2.0 API page audit

Page 496546, revision 4788278 (June 6, 2022, 01:53:21 UTC), retrieved October 7, 2026. Current retail carries 12.1.0. Branch `p920-page` starts from master `572a77d84`.

## Source accounting

80 inventory occurrences: 55 Global API (51 added, four removed), four widgets, thirteen events, seven CVars and one command. Header counts match. Initial parser treated unbolded Commands as a CVar and omitted HTML CVar defaults. Behavioral fixture reproduces both errors; parser now retains defaults and command identity. Existing source/register inputs stay unchanged. Extract has fourteen nonblank rows; source/build caption remains separately accounted.

## Runtime findings

Initial cached sweep records 28 publication gaps. Two unused removed C_PvP members (`GetSpecialEventDetails`, `GetSpecialEventInfo`) were fabricated by namespace lazy lookup. Mark them retired using the existing retail-epoch module; no classic profile loads it. Qualified and bare-name cached Lua scans have zero consumers; whole src/tests scan has no pre-existing callers. Deprecated vendor wrappers untouched. Repeated lookup behavioral regression reproduces the failure.

26 publication gaps retained; added lookup-only stubs do not qualify as modeled producers. C_CharacterServices.GetCharacterServiceDisplayOrder stays a gap: no qualified consumer, but bare name is defined/called at Blizzard_GlueXML/Mainline/CharacterSelect.lua:1493/1542; conservatively retained under the task rule, not claimed as the same namespace API. Removed global GetBattlefieldFlagPosition is already absent; current qualified C_PvP consumer in BattlefieldFlagDataProvider.lua:43 is its successor, not a reason to remove the successor.

## Verification

Pending final targeted proof; initial 80-row discovery, parser RED/GREEN and retirement RED retained in the evidence directory. No completion or full behavior parity claimed.

## Sources

- [Source response](../../../data/patch-api/evidence/9.2.0-session-2026-10-07/p920-fetch.json) — original revision and wikitext.
- [Register](../../../data/patch-api/sources/9.2.0-wikitext-register.json) — every inventory ID.
- [Spec](../../specs/patch-9-2-0-publication-sweep.md) — proof requirements.
- [Qualified scan](../../../data/patch-api/evidence/9.2.0-session-2026-10-07/p920-retirement-qualified.txt) and [bare scan](../../../data/patch-api/evidence/9.2.0-session-2026-10-07/p920-retirement-bare.txt) — read-only cached references.

## See Also

- [[patch-9-2-7-api-audit]] — adjacent later audit conventions.
- [[patch-10-0-0-api-audit]] — full publication accounting boundary.
