# Crafted-quality passive source gate

Date: 2026-10-13. Repository: `<PRIVATE_HOME>/Projects/wow/wow-ui-sim`.
Current HEAD: `6c38a82739dbf27fc58c3c801c32a0f0fc98d21c`.

**PASS — bounded source/spec/manual Rust-readability gate only. PARTIAL feature acceptance.**
No compiler, build, test, formatter, linter or simulator execution occurred. No positive RED/GREEN, consumer-fixture validity, native parity or runtime passing claim is made.

## Scope and exact revision

Read `verify/SKILL.md` and `rust-readability/SKILL.md`; performed Mode B directly as the assigned verifier. No delegation. Audited changed portions of the two production Rust files and all three new test/fixture files; read the complete changed spec. Supporting sources were read only for wiring and semantics.

`git -C … rev-parse HEAD` returned the exact requested revision (exit 0). `git -C … show --format=fuller --stat --oneline HEAD` returned six files, 512 insertions, 3 deletions (exit 0). Each audited file was compared as text against its exact HEAD blob, with six matches; no source-tree hashes were computed. A scoped quiet diff bound the supporting reads to HEAD (exit 0). Immediately before artifact creation, HEAD and the scoped changed/supporting sources were revalidated (exit 0). Binding covers these paths, not an entire clean-worktree assertion.

| Changed file | Lines | Exist / substantive evidence |
| --- | ---: | --- |
| `docs/specs/crafted-item-quality-info.md` | 53 | Explicit passive stage, unchecked supplied-model policy, unexecuted positive tests and consumer limits |
| `src/lua_api/state.rs` | 1080 | Changed lines 309–310 add guarded `crafted_item_quality: Default::default()` |
| `src/lua_api/state/sim_state.rs` | 755 | Changed lines 154–157 add documented guarded map field |
| `tests/crafted_quality_owned.rs` | 271 | Five public-query tests; real host-owned record insertion and real Lua statement execution |
| `tests/crafted_quality_item_button.rs` | 193 | Separate whole-addon consumer candidate plus bounded loading/diagnostic helpers |
| `tests/crafted_quality_owned/assertions.lua` | 29 | Distinct expected values and exact field/type/count assertions |

## Mode B findings

[EXIST] PASS — six files read and six HEAD-text matches.

[SUBSTANTIVE] PASS for passive stage — `HashMap<u32, …::CraftingQualityInfo>` reuses the existing record (`src/c_api/c_trade_skill_quality.rs:9–23`) rather than introducing another record or recipe alias. Empty initialization is deliberate stage behavior, not a fabricated successful getter. Only the map declaration and its initializer change production code. No getter, registration, parser or serializer changes appear in this six-file commit.

[WIRED] PASS for passive inputs/test authorship — declaration and initializer share `cfg(feature = "retail-12-0-0")`; the record module has the same gate (`src/c_api/mod.rs:206–207`). HashMap is imported in `state.rs:11` and visible through `sim_state.rs:1`. Existing construction reaches `build_empty_sim_state!` at `state.rs:1005–1006`. Retail and PTR bundles transitively include `retail-12-0-0`; both also include `retail-12-1-0`, needed for genuine secret wrappers. The two new tests use `cfg(any(client-retail, client-ptr))` and are intentionally absent outside those client bundles.

`Cargo.toml` has `autotests = false`: wiring does NOT rely on Cargo autodiscovery or standalone crafted-quality targets. `tests/integration.rs:1` invokes `wow_ui_sim_test_harness::integration_tests!()`. The dev dependency points to `crates/test-harness`; its `src/lib.rs:7` includes the generated macro. `crates/test-harness/build.rs:29–33,57–69,441–448,462–475` discovers top-level `.rs` files and emits modules. Neither new filename is excluded. This is source reachability, not proof the macro was rebuilt or the tests were admitted at runtime. `crafted_quality_owned.rs:30` includes the Lua fixture relative to its source file. The passive map has no runtime-query reader yet, intentionally.

[ANTI-PATTERN] PASS — changed production additions and complete new files/spec have zero TODO/FIXME/HACK/XXX markers and zero new `#[allow(...)]` / `#[expect(...)]` suppressions. Manual inspection found no commented-out code, swallowed errors, query substitutes, source-sliced consumers, mocks or spies. Existing harness-generated allow attributes are unchanged supporting code, not new suppressions introduced by this stage.

## Authored public-query coverage — all unexecuted

| Case | Evidence | Proof level |
| --- | --- | --- |
| Complete projection / arity | Owned test at line 36; full snapshot plus `select('#', …) == 1` | Authored only |
| Four ordinary selectors | Line 54: numeric ID, decimal string, item string, hyperlink | Authored only; existing parser supports these forms at `c_item.rs:35–41`, not native grammar proof |
| Freshness / mutation / replacement / removal | Line 75: two results differ by `rawequal`; all returned keys mutated; replacement changes quality/inventory; retained old/new snapshots and other item preserved; removal expects one nil | Authored only |
| Environment / recipe independence | Line 136: two environments; second recipe-pair input does not supply crafted data; then differing crafted quality under same item ID; first remains unchanged | Authored only |
| Secret selectors / denial / recovery | Line 195: four present and four absent wrapped selectors, tainted denial, preserved wrapper/payload/taint, untainted recovery, secret boolean authentication-before-type, removal | Authored only |
| Original ItemButton consumer | ItemButton test at line 17; whole closure and real Button/Texture; public sibling baseline precedes map insertion | Candidate only; fixture unknown |

### Full thirteen-field fixture matrix

Record fixture: `crafted_quality_owned.rs:10–25`; Lua expectation/assertion: `assertions.lua:1–29`.

| Rust field | Lua field | Distinct fixture value |
| --- | --- | --- |
| quality | quality | 3 (numeric) |
| icon | icon | crafted-icon |
| icon_small | iconSmall | crafted-small |
| icon_inventory | iconInventory | crafted-inventory |
| icon_mixed | iconMixed | crafted-mixed |
| icon_appear | iconAppear | crafted-appear |
| icon_dissolve | iconDissolve | crafted-dissolve |
| bar_fill | barFill | crafted-fill |
| bar_background | barBackground | crafted-background |
| bar_background_cap | barBackgroundCap | crafted-cap |
| bar_highlight | barHighlight | crafted-highlight |
| icon_chat | iconChat | crafted-chat |
| icon_quest_objective | iconQuestObjective | crafted-quest |

The fixture asserts table type, exactly thirteen enumerated fields, no unexpected field, exact values/types and every expected field present. Distinct values expose field swaps. Replacement uses quality 4 and another inventory string; the second environment uses quality 5. These are supplied simulator records, not native catalog inputs.

## VM and original-consumer boundaries

`WowLuaEnv::exec` (`env.rs:123–125`) reaches `exec_rilua` (`env_rilua.rs:23–26`), which compiles the statement chunk and calls the real rilua VM. No expression-only helper is used. `secret_values.rs:31–55` registers genuine VM wrapping/unwrapping; `env_init/mod.rs:72–73` installs them after runtime bootstrap under `retail-12-1-0`. Tests call these public functions without redefining them. This proves intended source route, not successful secret execution or denial ordering.

The consumer uses the existing profile-cache helper and discovers the complete `Blizzard_ItemButton` closure with an empty override list (`crafted_quality_item_button.rs:128–137`), then calls normal `load_addon` per discovered TOC (line 140). Its Lua provenance check requires a Lua handler and prints source/line metadata. It does not compare or authenticate cached content against native originals; cache availability, load warnings/errors, original handler behavior and dependency sufficiency remain runtime unknowns.

Before map insertion, the first boundary must actually call `C_TradeSkillUI.GetItemReagentQualityInfo(900001)` and require nil, then call the crafted query on empty state, require nil, create a real Button and call the original global overlay handler. Only afterward does Rust insert the crafted record and reach positive effects. This ordering is present in source, NOT proven executed. A sibling non-nil/error, closure failure, missing original function, unexpected `ProfessionsFrame`, or initial overlay behavior mismatch is a fixture/baseline failure, not positive crafted-data RED.

`ProfessionsFrame` is explicitly a host-created real Frame, NOT the original professions panel. Positive assertions cover real overlay Texture, inventory atlas, anchor (-3, 2), OVERLAY/7 layer, flags, frame visibility, explicit always-show boolean, clear helpers and removal. No query override or `IsShown` spy appears. Whole original panel, native catalog and rendered/native acceptance are excluded. No cache/vendor/service mutation occurred.

## Manual Rust-readability result

No scoped violations. Read every changed Rust line and all new Rust files; no automated metrics, fmt/check or broad readability tool ran. Ordinary helper bodies are 15/4 lines (owned) and 9/25/8/13 lines (consumer); all below the 30-line helper limit. Five owned test bodies are 14/17/57/55/75 lines; consumer test is 108 lines, below the 200-line test limit. Extracting Lua snapshot assertions keeps `new_contract_env` to four body lines. Manual control-flow review found shallow Rust nesting and no high-complexity branches, hidden-neutral-name effects, parameter overload, state-accumulating return construction, duplicated production decisions or warning suppressions. Fixture literals and phase-by-phase mutation are explicit behavioral inputs, not unexplained production magic or unrelated bumpy-road blocks.

## Proof limits / stage honesty

New positive file stage comments correctly say unexecuted. The spec describes five supplied-model cases plus one separate consumer candidate as authored/unexecuted, with fixture validity unproved. The two original required-input tests have independently classified prior Retail/PTR first-call rejection RED per the task and spec (two per profile, exit 101); this gate did not inspect old runtime logs or rerun them. They establish neither positive-data RED nor downstream secret recovery. Their older unchanged file header still says unexecuted; it is not fresh evidence and not audited as a new-stage claim.

Remaining proof: execute valid positive and whole-consumer RED independently on Retail/PTR, classifying setup/admission/baseline failures separately; then implement and prove the getter/projection/security behavior. Compilation, profile build validity, complete query behavior and consumer acceptance remain unverified. No fixes or canonical writes were authorized or made.
