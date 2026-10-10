# Independent private aura helper gate audit

OVERALL: PASS — source-only private-helper claim and supplied compiler receipts. No public runtime/native-parity claim.

Repository: `/home/osso/Projects/wow/wow-ui-sim`
Revision: `72971773c5014774471a28ea065aeff2ced099d6` (current HEAD).
Skills: verify (artifact/receipt mode) and rust-readability read and followed. No checks, builds, tests, operations, or delegation executed by this auditor. Receipt wait: 123 seconds total.

## Source evidence

- EXIST/SUBSTANTIVE: commit changes exactly one attribute in `src/c_api/c_secrets.rs:109`, from `aura-containers` to `retail-12-1-0`. Helper body, signature, and visibility unchanged. Working source and Cargo.toml match audited commit.
- WIRED: repository-wide tracked-symbol search plus all Rust files under src/tests/patch-tests/examples (including untracked source) finds exactly one caller of the c_secrets helper: `src/c_api/unit_aura_access.rs:111`. Its wrapper at lines 109–112 has the identical `retail-12-1-0` gate. The name at line 106 calls this local wrapper; lines 114–117 define its mutually exclusive non-12.1 wrapper, not another caller of c_secrets. Both modules are unconditional in `src/c_api/mod.rs:160,219`; no parent gate mismatch.
- Non-12.1 wrapper remains unreachable by the existing short-circuit: `auras_restricted` at unit_aura_access.rs:23–24 returns false without the epoch, and line 106 only evaluates the wrapper when restriction is true. No replacement/fallback introduced by this change.
- Public publication unchanged: c_secrets.rs:34–41 still registers `GetSpellAuraSecrecy` under `aura-containers`; implementation at :97 and classifier at :117 retain that gate. Attributes and secrecy constants also retain `aura-containers`.
- ANTI-PATTERN/readability: no changed-line violations. One explicit feature attribute; no added suppression, TODO/FIXME/HACK/XXX, branching, side effects, complexity, duplication, or parameter changes. Manual audit; no metric/check command executed.

## Full feature graph audit

Cargo.toml feature closure computed without invoking Cargo. All roots whose closure contains `retail-12-1-0`: `default`, `retail-12-1-0`, `retail-12-1-5`, `client-retail`, `client-ptr`. That epoch always implies `aura-containers`, hence all helper dependencies exist whenever its caller exists. Every other feature root omits both private helper and its caller.

| Configuration | Private helper/caller | Public classifier |
|---|---|---|
| default / client-retail | enabled | enabled |
| client-ptr / retail-12-1-5 | enabled | enabled |
| client-wowforever | disabled | enabled |
| profile-retail,retail-12-0-0 (exact1200) | disabled | disabled |
| aura-containers alone | disabled | enabled |

## Compiler receipts (main-produced; independently inspected)

Read complete stdout/stderr and results.json/outcome.json. All stdout files empty. Each result records this exact revision, exit 0, source_equal=true, helper_dead_warning_present=false. Before/after manifests identical; all 3,851 recorded files also match current source hashes.

| Receipt / command | Result | Warnings |
|---|---|---|
| fmt: `/usr/bin/cargo fmt --check` | PASS, exit 0 | 0 |
| default-check: `/usr/bin/cargo check --offline --locked -j 12` | PASS, exit 0 | 6 distinct vendor manifest warnings, 0 simulator warnings |
| forever-check: `/usr/bin/cargo check --offline --locked --no-default-features --features sound,gui,casc,client-wowforever -j 12` | PASS, exit 0 | 6 distinct vendor manifest warnings, 0 simulator warnings |
| exact1200-check: `/usr/bin/cargo check --offline --locked --no-default-features --features profile-retail,retail-12-0-0 -j 12` | PASS, exit 0 | 6 vendor + 10 simulator diagnostics (9 lib, 1 bin) |

Forever after: exactly six distinct iced-wgpu-patched manifest deprecations: large-enum-variant, map-entry, match-wildcard-for-single-variants, redundant-closure-for-method-calls, trivially-copy-pass-by-ref, type-complexity. The aggregate `generated 6 warnings` line is not a seventh distinct diagnostic. No private-helper dead-code warning. User-supplied before count was seven including helper; that prior current compiler receipt was not independently supplied/read, so the numerical before count remains attributed to the request, not independently proven. After count and removal are proven by the fresh Forever receipt.

Exact1200 warnings remain visible, not suppressed: item_spell/mod.rs:20 unused import; blizzard_ui_sync.rs:28,180,663 unused constant/functions; c_transmog_collection.rs:246 unused function; casc_asset_fallback.rs:93 unused function; render/font.rs:44 unread field; globals/auras.rs:251 unused function; widget/frame.rs:601,610 unused methods (one diagnostic); bin/wow_sim/main.rs:20 unused import. These are outside this one-attribute change. No warning-free whole-repository claim.

No source edits or docs required/performed. Only this requested independent report written.
