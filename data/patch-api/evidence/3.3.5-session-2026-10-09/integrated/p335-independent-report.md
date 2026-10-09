# Independent 3.3.5 source/tool proof

**PASS — bounded source/tool scope only. No native/current-runtime/integration acceptance.**

Read and followed `/home/osso/AgentConfig/skills/verify/SKILL.md` as verifier, and `rust-readability/SKILL.md`. Read-only checkout `/home/osso/.worktrees/wow-ui-sim-p335-source` at `81ee0471dee4d6235e0f19a9b6069cc7e7dfe847`; merge-base with actual master is `9b932d3f89d249d3c1e6a17b3ad0e1284bea84f5`. Applicability comparison: original `83e01f582500b432e2b10096da2641ca947671ba`. Initial git status clean. Only requested `/tmp` reports written; no tests/Cargo/build/check/full suites, native/runtime execution, repository/cache changes, delegation/model CLI, cwd switch or operational changes.

## Coverage matrix

| Scope | Evidence | Result / limits |
|---|---|---|
| Fresh own historical replay | `PYTHONDONTWRITEBYTECODE=1 python3 -B data/patch-api/evidence/3.3.5-session-2026-10-09/validate.py`, exit 0, empty stderr | 25 seals; 77 archived files; 68 **original** retail successors; 124 inventory rows; original 80 matches / 44 gaps; negative 45 gaps. Historical accounting only. |
| Complete historical accounting | Fresh replay | 324 IDs = 124 inventory + 130 extract + 70 signature/return rows. Statuses: 32 publication-only, 2 absence-only, 46 superseded-publication, 44 publication-gap, 129 metadata-only, 1 semantic-gap, 70 signature-gap; 5 named headers. |
| Original byte outputs after rebase | SHA-256 and size comparison of all 149 original register/extract files; original83e01f→HEAD diff empty for those paths and all sealed inputs | 149/149 byte identity. Not broad regeneration or all-publication proof. |
| Parser opt-in/default boundary | Executed archived and current generator main against frozen own real source, output to `/dev/stdout`, default and `--legacy-section-lists --client-line retail`; all four exit 0, empty stderr | Default outputs byte-equal: 295 bytes, zero entries, no added client label. Opt-in outputs byte-equal to frozen literal register: 124 entries, 69 explicit parenthetical signatures and 1 return-only fragment. No invented signature from linked pages. |
| Current retail successor inputs | Read every actual include input; checked client line, hashes, order, original/current expectation maps | 69 current successors = original 68 + actual 4.0.1. Added 4.0.1 has 419 rows; zero `(section,symbol)` **and bare-symbol** overlap with 124 own rows. All 124 complete expected objects unchanged. No Wrath Classic successor credit. |

## Retained targeted proof applicability

- Parser fixture remains unchanged against original83e01f; historical GREEN 1/1 retained. New 4.0.1 parser/extractor branches are opt-in, not default; fresh own generator serialization proves the rebased real-source boundary. Original `parse_legacy_section_lists` body unchanged.
- Validator/fixture hashes match retained GREEN receipt at `6c5c8fcf9dcaa80a57b329efcedab90ba1460629`. Historical relocated-root/no-Git/no-target proof, future mutable-closure independence and five exact tamper controls remain retained evidence. Fresh own replay passed; relocation/tamper fixture was **not rerun**.
- Historical own-publication GREEN 1/1 and fabricated-global negative 44→45 retain original provenance. Shared runner **has changed** since original83e01f: Wrath client enum/profile cases and extracted `run_sweep` alias-reader closure. Inspected retail route still invokes the existing cached alias reader and unchanged classifier/expectation algorithm. That preserves source-level applicability, not fresh current runtime acceptance.
- Existing chat probe source unchanged against original83e01f, with concrete 430×120 and BOTTOMLEFT/11/22 assertions. Historical GREEN 1/1 at89f37f0f6 applies only to temporary session-local state. No persistence/account-settings/native or current-runtime claim. Historical logs retain six unsuppressed inherited iced deprecations; no warning-clean claim.

## Rust readability

Manual changed-line audit of `tests/patch_3_3_5_publication_sweep.rs:20`: queued comment replaced by static `include_str!` actual 4.0.1 input. No readability violations: no added nesting, branching, state accumulation, suppression, parameter complexity or runtime side effect. Existing list format retained. No metric/check/build invocation.

## Ownership and proof boundary

This slice independently covers **parser opt-in/default serialization and portable historical validator behavior**, distinct from original literal page-only evidence. Original manifest/ledger/gap/bundle/command/log seals remain unchanged. Current publication async target and native/integration acceptance belong to main and are not established here.

One verifier-script failure is retained: the initial full-object expectation comparison omitted `direction` and `page_default` keys. Corrected script includes both keys and passes all 124 exact comparisons; no source correction or mutation occurred.

## Receipt

`/tmp/p335-independent-receipt.json` records exact argv, cwd, explicit environment overrides, revision, exits, **untruncated stdout/stderr**, SHA-256/byte sizes for seals/archive/149 outputs, actual successor files and full retained historical logs. Child commands inherit host environment except recorded overrides; secret-bearing ambient environment is not dumped. Receipt SHA-256: `0cf5e32f82807c7ffa14e6d5c378e4c360328977ecfc08728a12366e0cc2156b`.
