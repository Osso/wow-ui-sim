# Patch 7.0.1 API audit

Fresh MediaWiki query for pageid **336026** on 2026-10-08 returned revision **3241525**, timestamp **2016-02-28T20:50:06Z**, containing only `#REDIRECT [[Patch 7.0.3/API changes]]`. The requested page is a redirect, not an API-change inventory. The 7.0.3 destination remains separately owned by the parallel `p703-page` audit; no target-page content is silently substituted.

## Coverage matrix

| Source scope | Accounted | Proof boundary |
|---|---|---|
| API inventory | Zero entries and header counts | Default generator reproduces empty register; no invented API rows |
| Redirect context | One metadata-only ID, `source-context-001` | Default extractor retains redirect identity; no runtime capabilities |
| Meaningful modeled behavior | No statements in pinned source | No model changes needed or credited |
| Problematic own contracts | None | Target-page contracts are not included in this claim |
| Runtime retirements | None | No candidate names exist; no retirement or caller scan can be attributed to this source |

## Source and discovery

[Fetch](../../../data/patch-api/evidence/7.0.1-session-2026-10-08/p701-fetch.json) pins the full response without following redirects. [Provenance](../../../data/patch-api/sources/7.0.1-api-changes.provenance.json) retains request, response headers, revision, timestamp and SHA-256. [Register](../../../data/patch-api/sources/7.0.1-wikitext-register.json), [extract](../../../data/patch-api/sources/7.0.1-api-changes.txt), and [ledger](../../../data/patch-api/sources/7.0.1-page-coverage.json) preserve the literal redirect boundary.

No tool changes or additional flags are necessary. The prefork sweep uses the existing shared classifier with an empty register and empty known-gap set. It has the required one-line 7.0.3 placeholder first in `later_registers`, followed by 7.1.0 and every merged newer register. A passing empty sweep means the harness executed with zero publication observations; it is not API parity proof for the redirect destination.

## Retirement accounting

[Decision manifest](../../../data/patch-api/evidence/7.0.1-session-2026-10-08/p701-retirement-scans.json) records zero members and zero scans, plus complete register snapshots at recorded master and unmerged p703-page revisions. The p703 register is present in that Git snapshot. For any actual member, `/usr/bin/grep` whole-word qualified and bare-name scans would cover the retail cache excluding `*Documentation*`, and untruncated src/tests callers including `pcall(Name, ...)` and `and Name then`. No per-member scans are applicable to an empty candidate set; no absence-by-scan claim is made.

No `src/`, runtime registration, retirement gate, Blizzard/vendor/Wowless/WowlessData or other-worktree file changed. Consequently there is no widely-used runtime path change requiring caller regressions or an addons-enabled master/branch `lua-errors` comparison. Such startup proof is not claimed.

## Verification and preservation

Evidence: [7.0.1-session-2026-10-08](../../../data/patch-api/evidence/7.0.1-session-2026-10-08/).

Initial recorded source reproduction passes **47/47 registers**, **44/47 extracts**. Three inherited extract failures remain unchanged: 12.0.5 and 12.0.7 byte mismatches; 12.1.0 unsupported-template error. All prior source files and extraction-mode outcomes are preserved byte-for-byte. Python generator/extractor/validator fixtures pass **30/34/8**. All **27 prior validators**, including integrated validators, pass.

Requested asynchronous discovery, all publication sweeps, format and Mists tests-check are running. No full integration suite or environment-failing CASC texture tests were requested or run. The source requires no separate area-specific behavior tests beyond its discovery sweep.

## Validator contract

[Read-only validator](../../../data/patch-api/evidence/7.0.1-session-2026-10-08/validate.py) derives inventory, context, register and sweep counts from retained files. Register and sweep sets use `historical_registers` / `historical_sweep_tests` at recorded revision `2c43df7ea`, never moving globs or receipt-derived subsets. Current source input provenance is protected; inherited extract outcomes remain exact. No absolute cwd or target-path equality checks. Wiki preservation uses a recorded documentation revision, not mutable future wiki text. Final proof context and portability evidence will seal verification results.

## Sources

- [Pinned source](../../../data/patch-api/sources/7.0.1-api-changes.wikitext).
- [Publication contract](../../specs/patch-7-0-1-publication-sweep.md).
- [Handoff integration procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[patch-audit-validator-portability]] — fixed historical proof scopes.
- [[patch-7-1-0-api-audit]] — per-page evidence template and current newer register.
