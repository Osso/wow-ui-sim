# Cataclysm Classic 4.4.0 source accounting

Account only for pinned CLASSIC4.4.0 page `580953`, revision `6163701`, literal TOC `40400`. The [source ledger](../../data/patch-api/sources/4.4.0-page-coverage.json) is not a publication register. [Audit](../wiki/investigations/patch-4-4-0-api-audit.md) separates source proof from native contracts.

## What it must do

- [x] Preserve response/provenance identity, exact bytes/hashes, reproducible plaintext and all twelve nonblank literal occurrences: six metadata and six UNPROVEN prose rows, including both messaging contracts on line 6.
- [x] Reject fabricated revision/coverage, each omitted row, modified rationale, source/plaintext/provenance/response bytes and receipt tampering. Seal only owned inputs; retain context as compact Git tree pins.
- [ ] Prove native Cata `_Cata` selection and comma-delimited Interface acceptance. Existing comma parser implementation is not native proof.
- [ ] Prove native Cata per-prefix `C_ChatInfo.SendAddonMessage` throttling and enum-versus-boolean queue-failure output. No limits or enum members inferred.
- [ ] Prove the `_Classic` behavior transition, all-Classic versus prior Era-only semantics and `_Vanilla` Era-only successor selection/exclusion. Supported-profile implementation does not establish native Cata semantics.

## How it works

- [Exact source/proof matrix](../wiki/investigations/patch-4-4-0-api-audit.md).
- [Merged 4.4.2 source-only boundary](../wiki/investigations/patch-4-4-2-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/4.4.0-api-changes.wikitext`: committed returned source.
- `data/patch-api/sources/4.4.0-api-changes.txt`: reproducible plaintext.
- `data/patch-api/sources/4.4.0-page-coverage.json`: exact rows and individual unproven rationales.
- `data/patch-api/evidence/4.4.0-session-2026-10-08/`: response, provenance, source receipt and portable source validator.

## Tests asserting this spec

`python3 -B data/patch-api/evidence/4.4.0-session-2026-10-08/validate.py`: eleven source-proof/receipt tests, including all twelve omission controls, plaintext reproduction and sealed acceptance-log tampering. `python3 -B tools/extract_patch_non_inventory.py --patch 4.4.0 --text-only --canonical-patch-navigation --check`: exact plaintext check. Neither tests runtime compatibility. Committed source-revision gate results live in the audit's acceptance receipt; final current-HEAD replay is reported separately.

## Known gaps (current cycle)

- [ ] All seven contracts on six prose rows remain UNPROVEN: unsupported Cataclysm Classic profile, no native observations. Source accounting completion cannot close them.

## Out of scope

Runtime, parser or profile redesign; new client-line classifier/register; Cargo/build/runtime probes; retail/Mists substitutes; external links/diffs expansion; unlisted API or enum members; cross-line supersession. Coordinator integration follows 4.4.1; source accounting does not establish cross-line runtime supersession.
