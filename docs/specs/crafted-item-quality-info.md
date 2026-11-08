# Crafted item quality information

`C_TradeSkillUI.GetItemCraftedQualityInfo` queries crafted-item quality on Retail/PTR/Forever. Current cached declarations require `itemInfo: ItemInfo`, allow secret arguments only when untainted, and return nullable `CraftingQualityInfo`. See [Lua API architecture](../lua-api.md). An empty environment-owned per-item map supplies the existing thirteen-field record. The native getter projects those inputs after executed positive-query and whole-original-consumer RED on Retail/PTR. Independent runtime verification passes eight feature cases plus four preservation cases per profile; 24 GREEN outcomes and two separate startup passes are retained.

## What it must do

### Required inputs and security
- [x] Reject omitted and explicit nil `itemInfo` before lookup. Simulator diagnostics identify the method and selector; native exact wording is unproved.
- [x] Authenticate the original VM secret before validation: untainted secret nil reaches required-input rejection; tainted secret nil is denied by the existing VM access guard. Preserve the wrapper, caller taint and restored untainted access after errors. Ordering is the simulator's interpretation of `AllowedWhenUntainted`, not a native observation.

### Declared result
- [x] A present result supplies all thirteen non-nil declared fields: numeric `quality`; texture-atlas strings `icon`, `iconSmall`, `iconInventory`, `iconMixed`, `iconAppear`, `iconDissolve`, `barFill`, `barBackground`, `barBackgroundCap`, `barHighlight`, `iconChat`, `iconQuestObjective`.

### Verified supplied-model policy
- [x] Query explicit `crafted_item_quality` records independently of recipe-pair inputs. Reuse the existing numeric/decimal-string/item-string/hyperlink parser; no name catalog or native validation claim.
- [x] Return one nil for accepted selectors without supplied data; project present records into fresh detached tables with all thirteen fields.
- [x] Preserve retained snapshots through returned-table mutation, replacement/removal and independent environments.
- [x] Authenticate present/absent VM-secret selectors before type/lookup and preserve wrapper/taint recovery.
- [x] Let the whole original cached ItemButton handler consume supplied `iconInventory`, flags, anchors and visibility. Its visibility input is a real host-created Frame, not the original professions panel; native/rendered panel acceptance remains separate.

## How it works

- [C API and Lua surface inventory](../wiki/systems/lua-api.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_trade_skill_quality.rs`: item-keyed native getter, original-secret validation, Retail/PTR/Forever registration and shared fresh snapshot projection; recipe queries retain their distinct input map.
- `src/lua_api/env_init/runtime_surface_bootstrap.lua`: missing namespace functions may become nil-returning compatibility callables; callability is not modeled behavior.
- `src/lua_api/state/sim_state.rs` and `state.rs`: separate per-item input map, empty by default.
- `tests/crafted_quality_api.rs`: public required-input and authentic VM-secret-nil boundary tests.
- `tests/crafted_quality_owned.rs` and `crafted_quality_owned/assertions.lua`: positive projection, selector, lifecycle, isolation and secret-selector tests.
- `tests/crafted_quality_item_button.rs`: whole original cached consumer with real Button/Texture; Retail/PTR closure and empty-input baselines passed before positive branch RED. Later overlay assertions pass on Retail/PTR at the recorded runtime scopes; original-panel/rendered behavior remains separate.
- `Cargo.toml`: pins rilua; `rilua::table_security::unwrap_secret` supplies the existing secret-access guard.

## Tests asserting this spec

`tests/crafted_quality_api.rs` authors two cases: omitted/nil contextual rejection and secret-nil ordering/taint recovery. Both executed on Retail and PTR: each profile has two first-call rejection failures (actual exit 101). Explicit-nil/context and later tainted-denial/recovery assertions were unreached; return payloads were not printed. Five supplied-model cases and the whole consumer also executed on both profiles: five first-positive table-check failures plus one original-consumer positive-flag failure per profile. Exact field/alias/lifecycle/security/overlay checks were unreached in those RED runs; subsequent GREEN outcomes are recorded below. Fixtures execute statement chunks without query overrides or synthetic secrets.

### Initial crafted API RED checkpoint

[Initial whole-text evidence](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/crafted-api-red-20261013/README.md) owns the Retail/PTR required-input failure matrix and reached/unreached boundaries. Both profiles have executed first-call rejection failures; both profiles subsequently reached five positive-query failures and one genuine original ItemButton consumer failure, after successful closure/empty-input baselines. Downstream field/security/visibility/clear assertions remained unreached. Initial public-copy and retained-gate copy checks pass at their recorded scopes; later producer GREEN passes at its recorded runtime scopes, without rewriting RED.

### Supplied-record RED checkpoint

[Whole supplied-record RED](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/crafted-host-20261013/red-current/README.md) retains five query and one original-consumer failure per profile, with successful closure/baseline and unreached downstream assertions distinguished. Public-copy acceptance passes at its recorded scope; later GREEN does not rewrite these failures.

### Retained GREEN and publication acceptance

[Whole host-record evidence](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/crafted-host-20261013/README.md) retains 24 GREEN outcomes (12 unique cases per profile), two separate startup passes, default check/format and source gates. Closing public-copy verification at `614ed7efb` passes for 211 selected texts across 217 files; prior accepted payload proofs are reused through unchanged Git metadata and current stat, not new runtime execution. Historical SOURCE/readability PASS / SPEC FAIL and additive documentation PASS remain distinct.

## Known gaps (current cycle)

- [ ] Native ItemInfo name lookup/validation, item catalogs, callback event delivery and original professions-panel/rendered acceptance remain unproved; passing host-model queries do not establish them. Earlier-profile coverage and full-parent acceptance also remain unproved.

## Out of scope

Native item-catalog/classification and exact-error parity remain unproved. Recipe-quality inputs do not establish crafted-item quality; no recipe, equipment or catalog inference is authorized by this contract. Rendered/native acceptance is separate from public Lua behavior.

## Forever publication

Forever 70707 `Blizzard_APIDocumentationGenerated/TradeSkillUIDocumentation.lua:349-361`
declares the same nullable ItemInfo query. The original
`Blizzard_ItemButton/Mainline/ItemButtonTemplate.lua:339` consumes it when the
reagent query returns nil. Forever publishes the existing supplied-state getter;
this adds no crafted-item catalog or inferred records. Original character-slot
hover tests exercise that consumer through the Camelot character panel.

## Declaration evidence

Profile-scoped cache sources for both Retail and PTR: `Blizzard_APIDocumentationGenerated/TradeSkillUIDocumentation.lua` and `TradeSkillUITypesDocumentation.lua`. Required signature and thirteen-field result are local declaration observations, not authenticated native captures. Numeric IDs, decimal strings, item strings and hyperlinks are the selected existing-parser model forms. Runtime alias behavior passes for these four model forms on Retail/PTR; native validation and names remain unproved.
