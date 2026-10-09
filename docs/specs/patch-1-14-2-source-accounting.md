# Patch 1.14.2 SOURCE accounting

Frozen page accounting lives in `data/patch-api/evidence/1.14.2-session-2026-10-09/`; [audit wiki](../wiki/investigations/patch-1-14-2-api-audit.md) describes provenance and limits.

## What it must do

- [x] Preserve page236101/revision2290155/timestamp2022-06-05T00:14:08Z, response/raw hashes, and registry/manifest identity.
- [x] Retain all 36 nonblank rows, seven inventory entries, three section headers, four numeric inventory headers, caption, navigation and seven unexpanded links.
- [x] Keep four callable signatures unspecified; preserve three hidden CVar defaults/descriptions literally, including `Blizzard.Telemetry.Wow_Mainline`.
- [x] Distinguish configured Era/Anniversary11507 from source11402; retain successor boundaries without applying them or granting model/native credit.
- [x] Reject omissions, fabricated credit and source tampering.
- [x] Reject serialized ledger/log tampering and reproduce original default bytes in a fresh copied no-Git/no-target environment.
- [x] Separately observe current Era getters: two gamepad defaults/current values match literal `1`; telemetry and unknown control nil/nil. No setter/physical/native claims.

## How it works

- [SOURCE audit](../wiki/investigations/patch-1-14-2-api-audit.md)

## Implementation inventory

- `data/patch-api/evidence/1.14.2-session-2026-10-09/audit.py` — frozen SOURCE ledger and seals.
- `data/patch-api/evidence/1.14.2-session-2026-10-09/ledger.json` — literal serialized accounting.

## Tests asserting this spec

- `data/patch-api/evidence/1.14.2-session-2026-10-09/test_source_accounting.py` — eight own SOURCE tests.
- `data/patch-api/evidence/1.14.2-session-2026-10-09/test_portable.py` — copied SOURCE/default replay and serialized seal controls.
- `patch-tests/patch_1_14_2_cvars.rs` — current Era state getters; no registration/setters.

## Known gaps (current cycle)

- [ ] Missing telemetry default versus frozen literal package remains unmodeled.
- [ ] LED methods have no backing LED state; legacy makeable function is a no-op. Native arguments/returns unspecified.

## Out of scope

Native1.14.2 parity, physical LED/vibration/telemetry effects, link expansion, invented native signatures/default colors, runtime patches and main integration/final gates.
