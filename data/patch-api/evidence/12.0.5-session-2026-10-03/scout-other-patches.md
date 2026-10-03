# Other selected patches — read-only scouting

Snapshot: 2026-10-03. Repository unchanged; no builds/tests or model invocations.

## Accounting snapshot

Counts computed from parsed JSON, not wiki prose: `Counter(row["status"] for row in json.load(open(path))["rows"])`. Open = evidence-required + untriaged + exception-requested; best-effort is bounded/qualified, not full conformance.

| Manifest | Total | Status counts | Open |
|---|---:|---|---:|
| `12.0.0.json` | 3410 | {'evidence-required': 1057, 'best-effort': 2351, 'exception-requested': 2} | 1059 |
| `12.0.7.json` | 131 | {'implemented': 29, 'best-effort': 101, 'exception-requested': 1} | 1 |
| `12.1-behaviors.json` | 54 | {'evidence-required': 12, 'best-effort': 42} | 12 |
| `12.1-framexml.json` | 432 | {'best-effort': 431, 'implemented': 1} | 0 |
| `12.1.5.json` | 449 | {'best-effort': 439, 'evidence-required': 10} | 10 |

All five manifests use `framexml-patch-audit/v2`, occurrence rows with id, symbol, change, status, resolution, owner, evidence, tests, assertions, commit, approval_id and notes. No non-12.0.5 page-coverage JSON exists. These manifests do not account for every sentence of the Wiki pages. 12.1 combines two manifests (486 total, 12 open); its cached manifest is a file list, not audit dispositions.

## Selection and page-vs-occurrence distinction

`PLAN.md:3` records the goal verbatim: “Audit all selected patches, Retail 12.0.5 first, one page at a time.” `PLAN.md:33` queues the other selected patches but does not enumerate them. The explicit set in this request is 12.0.0, 12.0.5, 12.0.7, 12.1.0/12.1 and 12.1.5. Persisted occurrence scopes are the five top-level manifests listed above, plus the separate 12.0.5 probe manifest. There is **no explicit selected boolean/list** in `patch-page-index.json`: it is a discovery catalog of 138 pages, 98 retail-history candidates, not a selection/completion ledger. It also discovers 12.0.1; that page is outside this scouting request. The user-selected scope must not be inferred from every candidate title.

- `Patch 12.0.0/API changes`: pageid 649552, scope `retail-history-candidate`, audit_status `source-discovered`.
- `Patch 12.0.5/API changes`: pageid 670710, scope `retail-history-candidate`, audit_status `in-progress`.
- `Patch 12.0.7/API changes`: pageid 675928, scope `retail-history-candidate`, audit_status `source-discovered`.
- `Patch 12.1.0/API changes`: pageid 679840, scope `retail-history-candidate`, audit_status `source-discovered`.
- `Patch 12.1.5/API changes`: pageid 705933, scope `retail-history-candidate`, audit_status `source-discovered`.

Only 12.0.5 has `patch-page-coverage/v1`: source_register + SHA-256, audit_status, proof_policy, capabilities (scope/spec/tests/implementation/proof/ledger), and source_rows (source_id/capabilities/status/note). Its vocabulary is **audit-pending, bounded-coverage, partial-development-green, metadata-only**. These are not the manifest statuses. Source-register `status` values describe chronology (e.g. consolidated-delta), not runtime proof.

Occurrence-manifest status vocabulary is `implemented`, `best-effort`, `evidence-required`, `exception-requested`; untriaged is **resolution=untriaged with status=null**, not a fifth serialized status (`patch_manifest.rs:499–518`). All inspected manifests have zero null-status/untriaged rows. `unsafe` is a resolution, not a machine status. Exception-requested no-3D rows are scope exceptions, not authorization to implement 3D.

Source accounting differs by patch:
- **12.0.0:** `sources/12.0.0-register.json`, patch-api-source-register/v1, 3,410 occurrences (2,554 added/313 changed/543 removed); wowless semantic endpoint diff with intermediate transient lifecycle accounting, **not the Wiki page**. Status lives in `12.0.0.json`. Historical endpoints must be compared before reusing newer cache declarations.
- **12.0.7:** same source-register schema; 131 named occurrences (79 added/29 changed/23 removed) extracted from checked-in `12.0.7-api-changes.txt`. Blue-post prose and unnamed CVar claims have no one-to-one page accounting; `crawler_claims` preserves omitted names instead of inventing symbols.
- **12.1.0/12.1:** `sources/12.1-framexml.json` is only addedCount/removedCount plus two symbol arrays (320/112; 430 distinct symbols, two appearing in both directions). `sources/12.1-behaviors.json` is a 54-occurrence source register with frozen original candidate totals 33 best-effort/21 unsafe; current machine dispositions are **42/12**, not those old candidate totals. `sources/12.1.0-ptr-cache-manifest.txt` lists source files, not dispositions or pages. Neither artifact covers the entire Wiki page.
- **12.1.5:** source-register/v1, 449 generated-documentation occurrences (262 added/185 changed/2 removed), Gethe endpoints 12.1.0.69587 → 12.1.5.69594. Declaration diff excludes prose, FrameXML-only helpers, CVars, GlobalStrings and intermediate changes. Machine status lives in `12.1.5.json`.

**Unstarted page accounting is not zero pending page rows.** For all four other pages, the number of unaudited *page sentences/atomic annotations* is unknown until a full page extraction/register is built. Existing manifestation counts cannot supply it.

## Reproducible count calculation (executed read-only)

```python
import json
from collections import Counter
for name in ("12.0.0", "12.0.7", "12.1-behaviors", "12.1-framexml", "12.1.5"):
    d = json.load(open(f"data/patch-api/{name}.json"))
    rows = d["rows"]
    print(name, len(rows), Counter(r["status"] for r in rows))
    print("untriaged", sum(r["resolution"] == "untriaged" for r in rows))
```

The wiki index is stale: its 12.1 summaries still say 33 best-effort/21 evidence-required; JSON and the current behavior inventory say 42/12. The 12.0.0 occurrence inventory has older prefixed checkpoints (2246/1162, etc.) and index has still older numbers. Use parsed current JSON, not the first prose total encountered. No documentation edits were made.

## Tooling: start an actual page audit

1. Preserve the full Wiki plaintext extract with page ID, retrieval date, source URL, method and SHA-256. 12.0.5 provenance records **MediaWiki query prop=extracts, explaintext=1**, entire plaintext including dated blue posts and consolidated blocks. Do not mistake a generated API endpoint diff for this step.
2. Inventory every content-bearing source line, with stable source ID and one-based source_lines, exact change text, subject, chronology and section. Split consolidated annotations atomically; preserve historical proposals separately and mark supersession without assuming shipment.
3. Create a coverage/v1 file bound to the exact register hash, one ordered source_row per register ID initially audit-pending; attach bounded capabilities only after matching literal delta, profile epoch and evidence. Keep metadata-only and partial records explicit. Update only the page being worked when later authorized.
4. The tracked provenance shows the 12.0.5 source/register were introduced by `d3d9d8fee` on 2026-09-30; page-coverage was introduced by `7a92a1cb5` the same day with **362 source rows and eight capabilities**. No reusable tracked page-extract/register/page-coverage generator was found in tools/scripts/src/xtask. The artifacts were committed directly; an untracked historical creation script cannot be established from this repository.
5. Existing tooling has a different scope: `tools/gen_patch_12_0_0_register.py` builds the wowless source register (`--wowless-dir`, `--output`, `--check`); `src/bin/wow_cli/gen_patch_api_docs/` implements `wow-cli generate patch-api-docs --repository ... --base-commit ... --target-commit ... --patch ... --output ...`, writing generated-documentation semantic registers. `wow-cli audit-api --patch-manifest PATH --format plan` validates artifact hashes/source rows and renders an occurrence checklist; `--complete --observations PATH` adds runtime gates. None generates page coverage. Commands are identified from source, **not executed**.

For 12.0.7, reuse the checked-in text as a starting source but capture a full current extract/provenance before claiming exhaustive page coverage. For 12.0.0, 12.1.0 and 12.1.5, their registers are not page extracts: perform step 1 first. Current caches are convenient declaration locators, not authenticated historical build proof.

## Recommended order and ranking policy

**12.0.0 → 12.0.7 → 12.1.0/12.1 → 12.1.5**, after closing the current 12.0.5 page checkpoint. 12.0.0 has the largest unresolved register but a promising low-code next slice is an epoch/contract re-audit against already-modeled queries from October 12.0.5 work; this also establishes the historical base before later deltas. If optimizing for shortest *page*, 12.0.7 (131 named rows) is the alternative, but it still lacks prose/page accounting.

“Smallest” below is a reasoned ranking, not measured effort: prefer existing simple state-backed getter/setter registration and reusable bounded proof; keep shared producer rows together; defer native/security unknowns. Where a patch has fewer than 15 open occurrences, fill the list with clearly marked best-effort **residual re-audits**, not invented pending rows. Source snippets below are verbatim retained detail/annotation text or canonical JSON of the retained declaration; no source statement is inferred from a simulator stub.

## 12.0.0 — first 15 concrete rows

All 15 remain **evidence-required** in the current manifest. Ranking estimates low producer-change cost only where the historical epoch supports reuse; it does not establish historical availability or native/security proof. Cached declaration paths below expand to `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`; these are the **current cache**, not pinned 12.0.0 source. Static registration anchors are inspected source, not runtime execution.

### 1. `added:C_NamePlateManager.GetNamePlateHitTestInsets` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:7361`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"type","type":{"enum":"NamePlateType"}}],"outputs":[{"name":"left","type":"number"},{"name":"right","type":"number"},{"name":"top","type":"number"},{"name":"bottom","type":"number"}]},"metadata":{}}`
- Cached declaration: `NamePlateManagerDocumentation.lua:11`.
- Simulator registration: `src/c_api/c_nameplate_manager.rs:22`.
- Model suggestion: Reuse per-NamePlateType inset state; re-audit four-result historical publication before metadata credit.
- Existing manifest gap: Source-register signature evidence establishes only the occurrence and return shape. Current nameplate behavior has no modeled 2D size/hit-test/state subsystem and exact 12.0.0 semantics remain unproven; no approval can close this row.

### 2. `added:C_NamePlateManager.SetNamePlateHitTestInsets` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:7448`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"type","type":{"enum":"NamePlateType"}},{"name":"left","type":"number"},{"name":"right","type":"number"},{"name":"top","type":"number"},{"name":"bottom","type":"number"}],"outputs":null},"metadata":{}}`
- Cached declaration: `NamePlateManagerDocumentation.lua:46`.
- Simulator registration: `src/c_api/c_nameplate_manager.rs:28`.
- Model suggestion: Reuse setter/getter round-trip; retain explicit four-inset state and isolate each type.
- Existing manifest gap: Source-register signature evidence establishes only the occurrence and return shape. Current nameplate behavior has no modeled 2D size/hit-test/state subsystem and exact 12.0.0 semantics remain unproven; no approval can close this row.

### 3. `added:C_ActionBar.GetActionText` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:1219`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"actionID","type":"number"}],"mayreturnnothing":true,"outputs":[{"name":"text","type":"string"}]},"metadata":{}}`
- Cached declaration: `ActionBarFrameDocumentation.lua:244`.
- Simulator registration: `src/c_api/action_macros.rs:100`.
- Model suggestion: Reuse current macro association/name producer; distinguish historical mayreturnnothing from newer secret annotation.
- Existing manifest gap: Current implementation returns constant nil for every input (no-op/absent-value behavior). Authoritative semantics or a correct model/test are required, and no approval can close the row.

### 4. `added:C_ActionBar.GetActionDisplayCount` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:1131`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"actionID","type":"number"},{"default":9999,"name":"maxDisplayCount","type":"number"},{"default":"*","name":"replacementString","type":"string"}],"outputs":[{"name":"displayCount","type":"string"}]},"metadata":{}}`
- Cached declaration: `ActionBarFrameDocumentation.lua:190`.
- Simulator registration: `src/lua_api/globals/action_bar_api/registration.rs:30`.
- Model suggestion: Reuse explicit per-action counts and threshold/replacement formatting; check 12.0.0 defaults 9999 and *.
- Existing manifest gap: Current implementation returns constant nil for every input (no-op/absent-value behavior). Authoritative semantics or a correct model/test are required, and no approval can close the row.

### 5. `added:C_ActionBar.GetActionUseCount` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:1268`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"actionID","type":"number"}],"outputs":[{"name":"count","type":"number"}]},"metadata":{}}`
- Cached declaration: `ActionBarFrameDocumentation.lua:277`.
- Simulator registration: `src/lua_api/globals/action_bar_api/registration.rs:31`.
- Model suggestion: Reuse explicit count producer; check historical return arity and absent-slot policy independently.
- Existing manifest gap: Current implementation returns constant 0 for every input (default behavior). Authoritative semantics or a correct model/test are required, and no approval can close the row.

### 6. `added:C_ActionBar.GetActionCharges` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:1053`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"actionID","type":"number"}],"outputs":[{"name":"chargeInfo","type":{"structure":"C_ActionBar.ActionBarChargeInfo"}}]},"metadata":{}}`
- Cached declaration: `ActionBarFrameDocumentation.lua:138`.
- Simulator registration: `src/lua_api/globals/action_bar_api/registration.rs:167`.
- Model suggestion: Re-audit current charge-state DTO; bind charge fields to slot-associated spell state, not zero/default tables.
- Existing manifest gap: Current implementation returns a partial zero/default ActionBarChargeInfo-shaped table: current/max charges and cooldown fields are zero, with modRate 1. Authoritative semantics or a correct model/test are required, and no approval can close the row.

### 7. `added:C_ActionBar.GetActionChargeDuration` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:1027`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"actionID","type":"number"}],"outputs":[{"name":"duration","type":{"luaobject":"LuaDurationObject"}}]},"metadata":{}}`
- Cached declaration: `ActionBarFrameDocumentation.lua:121`.
- Simulator registration: `src/lua_api/globals/action_bar_api/registration.rs:56`.
- Model suggestion: Reuse slot charge-duration object producer; keep historical publication and clock/progression claims separate.
- Existing manifest gap: Current implementation returns a fresh default LuaDurationObject without reading the action slot (default duration behavior). Authoritative semantics or a correct model/test are required, and no approval can close the row.

### 8. `added:C_Spell.GetSpellDisplayCount` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:9963`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"spellIdentifier","type":"string"},{"default":9999,"name":"maxDisplayCount","type":"number"},{"default":"*","name":"replacementString","type":"string"}],"outputs":[{"name":"displayCount","type":"string"}]},"metadata":{}}`
- Cached declaration: `SpellDocumentation.lua:339`.
- Simulator registration: `src/c_api/c_spell.rs:86`.
- Model suggestion: Reuse explicit spell count model/formatter; resolve historical identifier/default signature before credit.
- Existing manifest gap: The 12.0.0 display-count contract is not currently registered/implemented with focused proof; spell metadata/display semantics require authoritative evidence or a correct model, and no approval can close the row.

### 9. `added:C_Spell.GetSpellMaxCumulativeAuraApplications` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:10024`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"spellID","type":"number"}],"outputs":[{"name":"cumulativeAura","type":"number"}]},"metadata":{}}`
- Cached declaration: `SpellDocumentation.lua:461`.
- Simulator registration: `src/c_api/c_spell.rs:91`.
- Model suggestion: Reuse explicit spell-metadata lookup; historical addition still needs its own publication/result proof.
- Existing manifest gap: The 12.0.0 spell-metadata contract is not currently registered/implemented with focused proof; spell metadata semantics require authoritative evidence or a correct model, and no approval can close the row.

### 10. `added:C_UnitAuras.AuraIsBigDefensive` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:14326`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"spellID","type":"number"}],"outputs":[{"name":"isBigDefensive","type":"boolean"}]},"metadata":{}}`
- Cached declaration: `UnitAuraDocumentation.lua:54`.
- Simulator registration: `src/c_api/c_unit_aura_classification.rs:28`.
- Model suggestion: Reuse explicit spell classification lookup; do not infer class flags or native gameplay metadata.
- Existing manifest gap: Current C_UnitAuras behavior covers adjacent seeded aura lookup/table behavior, not authoritative 12.0.0 behavior for these rows. Defensive classification, expiration/display formatting, duration objects, refresh calculations, color curves, sorted instance IDs, private callback dispatch, and GetUnitAuras sort semantics remain unproven. Existing tests are intentionally not attached; no approval can close this row.

### 11. `added:C_UnitAuras.DoesAuraHaveExpirationTime` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:14350`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"auraInstanceUnit","type":"unit"},{"name":"auraInstanceID","type":"number"}],"outputs":[{"name":"hasExpirationTime","type":"boolean"}]},"metadata":{}}`
- Cached declaration: `UnitAuraDocumentation.lua:107`.
- Simulator registration: `src/c_api/aura_duration.rs:53`.
- Model suggestion: Reuse stored aura expiration producer; separate the 12.0.0 added query from 12.0.5 authentication delta.
- Existing manifest gap: Current C_UnitAuras behavior covers adjacent seeded aura lookup/table behavior, not authoritative 12.0.0 behavior for these rows. Defensive classification, expiration/display formatting, duration objects, refresh calculations, color curves, sorted instance IDs, private callback dispatch, and GetUnitAuras sort semantics remain unproven. Existing tests are intentionally not attached; no approval can close this row.

### 12. `added:C_UnitAuras.GetAuraApplicationDisplayCount` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:14378`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"auraInstanceUnit","type":"unit"},{"name":"auraInstanceID","type":"number"},{"default":2,"name":"minDisplayCount","type":"number"},{"name":"maxDisplayCount","nilable":true,"type":"number"}],"outputs":[{"name":"count","type":"string"}]},"metadata":{}}`
- Cached declaration: `UnitAuraDocumentation.lua:127`.
- Simulator registration: `src/c_api/c_unit_aura_display_count.rs:18`.
- Model suggestion: Reuse live aura applications and threshold formatting; keep historical contract distinct from AllowedWhenUntainted.
- Existing manifest gap: Current C_UnitAuras behavior covers adjacent seeded aura lookup/table behavior, not authoritative 12.0.0 behavior for these rows. Defensive classification, expiration/display formatting, duration objects, refresh calculations, color curves, sorted instance IDs, private callback dispatch, and GetUnitAuras sort semantics remain unproven. Existing tests are intentionally not attached; no approval can close this row.

### 13. `added:C_UnitAuras.GetAuraBaseDuration` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:14416`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"auraInstanceUnit","type":"unit"},{"name":"auraInstanceID","type":"number"},{"name":"spellID","nilable":true,"type":"number"}],"outputs":[{"name":"newDuration","nilable":true,"type":"number"}]},"metadata":{}}`
- Cached declaration: `UnitAuraDocumentation.lua:149`.
- Simulator registration: `src/c_api/aura_duration.rs:47`.
- Model suggestion: Reuse explicit base-duration metadata; unknown production metadata must remain unknown, not fabricated.
- Existing manifest gap: Current C_UnitAuras behavior covers adjacent seeded aura lookup/table behavior, not authoritative 12.0.0 behavior for these rows. Defensive classification, expiration/display formatting, duration objects, refresh calculations, color curves, sorted instance IDs, private callback dispatch, and GetUnitAuras sort semantics remain unproven. Existing tests are intentionally not attached; no approval can close this row.

### 14. `added:C_UnitAuras.GetAuraDuration` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:14486`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"auraInstanceUnit","type":"unit"},{"name":"auraInstanceID","type":"number"}],"outputs":[{"name":"duration","type":{"luaobject":"LuaDurationObject"}}]},"metadata":{}}`
- Cached declaration: `UnitAuraDocumentation.lua:266`.
- Simulator registration: `src/c_api/aura_duration.rs:56`.
- Model suggestion: Current October producer is newly registered; re-audit duration snapshot/object and historical signature, no whole-row upgrade.
- Existing manifest gap: Current C_UnitAuras behavior covers adjacent seeded aura lookup/table behavior, not authoritative 12.0.0 behavior for these rows. Defensive classification, expiration/display formatting, duration objects, refresh calculations, color curves, sorted instance IDs, private callback dispatch, and GetUnitAuras sort semantics remain unproven. Existing tests are intentionally not attached; no approval can close this row.

### 15. `added:C_UnitAuras.GetRefreshExtendedDuration` — evidence-required
- Source: `data/patch-api/sources/12.0.0-register.json:14516`: `api added in 12.0.0. {"category":"api","value":{"inputs":[{"name":"auraInstanceUnit","type":"unit"},{"name":"auraInstanceID","type":"number"},{"name":"spellID","nilable":true,"type":"number"}],"outputs":[{"name":"newDuration","nilable":true,"type":"number"}]},"metadata":{}}`
- Cached declaration: `UnitAuraDocumentation.lua:392`.
- Simulator registration: `src/c_api/aura_duration.rs:60`.
- Model suggestion: Reuse explicit base-duration/refresh producer; historic numeric spell input differs from later SpellIdentifier delta.
- Existing manifest gap: Current C_UnitAuras behavior covers adjacent seeded aura lookup/table behavior, not authoritative 12.0.0 behavior for these rows. Defensive classification, expiration/display formatting, duration objects, refresh calculations, color curves, sorted instance IDs, private callback dispatch, and GetUnitAuras sort semantics remain unproven. Existing tests are intentionally not attached; no approval can close this row.

## 12.0.7 — 15 smallest residual audits

**No evidence-required/null-untriaged rows**; the one exception-requested row is `ModelSceneActorBase.GetModelUnitGUID`, intentionally absent under the no-3D rule. Do not manufacture 14 more open occurrences or implement that scope exception. These 15 are **best-effort residuals** whose exact changed contracts remain unproven, or a missing state model. Current retail cache prefix is the same as above.

### 1. `changed:Button.GetButtonState` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:129`: `Button:GetButtonState + SecretReturnsForAspect`
- Cached declaration: `SimpleButtonAPIDocumentation.lua:72`.
- Simulator registration: `src/lua_api/frame/methods/button_anchor_hierarchy/mod.rs:93`.
- Model suggestion: Apply documented aspect-dependent result wrapping to existing button state; native aspect boundaries need probes.

### 2. `changed:Button.IsEnabled` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:130`: `Button:IsEnabled + SecretReturnsForAspect`
- Cached declaration: `SimpleButtonAPIDocumentation.lua:257`.
- Simulator registration: `src/lua_api/frame/methods/button_anchor_hierarchy/mod.rs:78`.
- Model suggestion: Reuse enabled-state producer and prove exact SecretReturnsForAspect wrapping, not only method presence.

### 3. `changed:Button.SetButtonState` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:131`: `Button:SetButtonState + SecretArgumentsAddAspect`
- Cached declaration: `SimpleButtonAPIDocumentation.lua:293`.
- Simulator registration: `src/lua_api/frame/methods/button_anchor_hierarchy/mod.rs:92`.
- Model suggestion: Authenticate input before mutation, add only the documented aspect, and prove rejected calls are atomic.

### 4. `changed:Button.SetEnabled` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:132`: `Button:SetEnabled + SecretArgumentsAddAspect, SecretArguments NotAllowed -> AllowedWhenUntainted`
- Cached declaration: `SimpleButtonAPIDocumentation.lua:336`.
- Simulator registration: `src/lua_api/frame/methods/button_anchor_hierarchy/mod.rs:79`.
- Model suggestion: Separate AllowedWhenUntainted authentication from SecretArgumentsAddAspect; preserve callback ordering.

### 5. `changed:EditBox.SetFont` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:133`: `EditBox:SetFont + RequiresValidFontHeight + RequiresValidFontAsset`
- Cached declaration: `SimpleEditBoxAPIDocumentation.lua:677`.
- Simulator registration: `src/lua_api/frame/methods/text_attribute_event/mod.rs:212`.
- Model suggestion: Validate font asset/height in shared font primitive before assignment; resolve native valid-height domain.

### 6. `changed:Font.SetFont` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:134`: `Font:SetFont + RequiresValidFontHeight + RequiresValidFontAsset`
- Cached declaration: `SimpleFontAPIDocumentation.lua:199`.
- Simulator registration: `src/lua_api/globals/font_strings_collection/fonts.rs:85`.
- Model suggestion: Reuse font-object assignment but validate actual asset/height; do not duplicate only presence tests.

### 7. `changed:FontString.SetFont` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:135`: `FontString:SetFont + RequiresValidFontHeight + RequiresValidFontAsset, arg2.Type number -> uiFontHeight`
- Cached declaration: `SimpleFontStringAPIDocumentation.lua:500`.
- Simulator registration: `src/lua_api/frame/methods/text_attribute_event/mod.rs:212`.
- Model suggestion: Implement/prove the later uiFontHeight constraint separately from 12.0.5 plain-number/optional-flags credit.

### 8. `changed:MessageFrame.SetFont` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:136`: `MessageFrame:SetFont + RequiresValidFontHeight + RequiresValidFontAsset`
- Cached declaration: `SimpleMessageFrameAPIDocumentation.lua:295`.
- Simulator registration: `src/lua_api/frame/methods/text_attribute_event/mod.rs:212`.
- Model suggestion: Reuse shared font validation while preserving MessageFrame output/state semantics.

### 9. `changed:SimpleHTML.SetFont` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:142`: `SimpleHTML:SetFont + RequiresValidFontHeight + RequiresValidFontAsset`
- Cached declaration: `SimpleHTMLAPIDocumentation.lua:203`.
- Simulator registration: `src/lua_api/frame/methods/text_attribute_event/mod.rs:212`.
- Model suggestion: Check tag-specific font signature before reusing the generic font setter; validation precedes mutation.

### 10. `changed:ScrollFrame.GetHorizontalScroll` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:138`: `ScrollFrame:GetHorizontalScroll + SecretReturnsForAspect`
- Cached declaration: `SimpleScrollFrameAPIDocumentation.lua:10`.
- Simulator registration: `src/lua_api/frame/methods/widgets/slider.rs:714`.
- Model suggestion: Wrap existing horizontal offset according to documented aspect; do not change offset layout behavior.

### 11. `changed:ScrollFrame.GetVerticalScroll` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:139`: `ScrollFrame:GetVerticalScroll + SecretReturnsForAspect`
- Cached declaration: `SimpleScrollFrameAPIDocumentation.lua:51`.
- Simulator registration: `src/lua_api/frame/methods/widgets/slider.rs:717`.
- Model suggestion: Mirror vertical offset aspect-dependent wrapping; probe native secret/public caller behavior.

### 12. `changed:ScrollFrame.SetHorizontalScroll` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:140`: `ScrollFrame:SetHorizontalScroll SecretArguments NotAllowed -> AllowedWhenUntainted + SecretArgumentsAddAspect`
- Cached declaration: `SimpleScrollFrameAPIDocumentation.lua:79`.
- Simulator registration: `src/lua_api/frame/methods/widgets/slider.rs:715`.
- Model suggestion: Authenticate authored offset and add documented aspect before layout/callback changes.

### 13. `changed:ScrollFrame.SetVerticalScroll` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:141`: `ScrollFrame:SetVerticalScroll SecretArguments NotAllowed -> AllowedWhenUntainted + SecretArgumentsAddAspect`
- Cached declaration: `SimpleScrollFrameAPIDocumentation.lua:103`.
- Simulator registration: `src/lua_api/frame/methods/widgets/slider.rs:718`.
- Model suggestion: Share the authored-input guard with horizontal setter; prove rejected writes leave offsets unchanged.

### 14. `added:C_UIFileAsset.IsLooseFile` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:51`: `C_UIFileAsset.IsLooseFile`
- Cached declaration: `UIFileAssetAPIDocumentation.lua:43`.
- Simulator registration: `src/c_api/c_ui_file_asset.rs:26`.
- Model suggestion: Replace constant-false classification only once a real loose-file state/source is established; current true case unmodeled.

### 15. `changed:CHAT_MSG_MONEY` — best-effort residual
- Source: `data/patch-api/sources/12.0.7-api-changes.txt:156`: `CHAT_MSG_MONEY - SecretInChatMessagingLockdown`
- Cached declaration: `ChatInfoDocumentation.lua:1816`.
- Simulator registration: `src/event/valid_events_a.rs:348`.
- Model suggestion: Audit removed lockdown secrecy annotation at event-payload producer, not only name validation.

The CHAT_MSG_MONEY anchor registers the **event name**, not removed payload-secrecy semantics. Generic frame methods are shared across widget types; a SetFont registration is not evidence that SimpleHTML tag/font conventions or font-asset validity are modeled. Full page audit must also account for lines 6–22 (blue-post notes), including debug secrecy, mouse taint, group formation, vehicle aura ownership and private-aura sound; the 131-row name register does not close those statements.

## 12.1.0/12.1 — 12 blockers plus three bounded residuals

The first 12 rows are all remaining **evidence-required** behavioral rows. Last three are **best-effort native-fidelity residuals**. The 432 FrameXML rows have no untriaged/evidence-required occurrences; 301 are stale-snapshot, 113 vendor-present, 12 compat, three reversed-snapshot, two LoD and one cross-flavor. Source-count resolution proof is not whole-page proof.

These `Patch12_1.*` names are synthetic audit labels, **not Lua APIs**: no generated declaration or simulator registration exists under that exact symbol. Related cached declaration locators expand to `~/.cache/wow-ui-sim/blizzard-ui/ptr/AddOns/Blizzard_APIDocumentationGenerated/` and identify actual APIs/types only. Native contract/probe work, not publication of synthetic globals, is the next action.

### 1. `changed:Patch12_1.UnitAura.AddonSecretError` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:81`: “Addon-tainted UnitAura and aura access raise the retail secret-value error shape.”
- Cached declaration: **none for synthetic label**; related `UnitAuraDocumentation.lua:189`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/globals/auras.rs:85`.
- Model suggestion: Capture the addon-tainted denial boundary, then enforce at authored-input/secret projection rather than replacing Blizzard consumers.

### 2. `changed:Patch12_1.UnitAura.BlizzardSecretAccess` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:87`: “Blizzard/internal callers receive the permitted secret-aura behavior distinct from addon-tainted callers.”
- Cached declaration: **none for synthetic label**; related `UnitAuraDocumentation.lua:189`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/globals/auras.rs:85`.
- Model suggestion: Use native clean-vs-tainted caller controls to define permitted secret projection; generic public AuraData is insufficient.

### 3. `changed:Patch12_1.UnitAura.SecretAuraData` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:93`: “Fully secret AuraData fields remain inaccessible to addons while preserving the retail object shape.”
- Cached declaration: **none for synthetic label**; related `UnitAuraDocumentation.lua:189`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/globals/auras.rs:85`.
- Model suggestion: Model exactly observed per-field secrecy on current aura records; distinguish object shape from access policy.

### 4. `changed:Patch12_1.UnitAura.SecretEventPayload` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:99`: “Secret UNIT_AURA payload values preserve retail secrecy and tuple shape.”
- Cached declaration: **none for synthetic label**; related `UnitAuraDocumentation.lua:621 (UNIT_AURA)`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/globals/auras.rs:85`.
- Model suggestion: Probe actual UNIT_AURA tuple/secret values and caller context; dispatch from aura-state mutation with rooted payloads.

### 5. `changed:Patch12_1.PrivateScriptObjects.ChildVisibility` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:117`: “Public traversal cannot expose forbidden or private children.”
- Cached declaration: **none for synthetic label**; related `SimpleFrameAPIDocumentation.lua:323`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/frame/methods/button_anchor_hierarchy/mod.rs:263`.
- Model suggestion: Probe public/private/forbidden traversal, then apply visibility at model child projection rather than vendor Lua.

### 6. `changed:Patch12_1.PrivateScriptObjects.ScriptStorage` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:129`: “Script handlers stored in private partitions are not publicly readable or writable.”
- Cached declaration: **none for synthetic label**; related `SimpleScriptRegionAPIDocumentation.lua:233,664`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/frame/methods/text_attribute_event/mod.rs:755–756`.
- Model suggestion: Probe partitioned GetScript/SetScript reads/writes; maintain storage partition and observable access rejection.

### 7. `changed:Patch12_1.PrivateScriptObjects.HookBoundary` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:123`: “Hooks cannot cross private or forbidden partitions except through permitted delegates.”
- Cached declaration: **none for synthetic label**; related `SimpleScriptRegionAPIDocumentation.lua:341`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/frame/methods/text_attribute_event/mod.rs:758`.
- Model suggestion: Probe hook receiver/taint boundaries and preserve permitted delegate routing; do not infer policy from HookScript presence.

### 8. `changed:Patch12_1.PrivateScriptObjects.SecureDelegateEnforcement` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:135`: “Public delegates invoke permitted private behavior without exposing private receiver state.”
- Cached declaration: **none for synthetic label**; related `SimpleScriptRegionAPIDocumentation.lua:664`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/frame/methods/text_attribute_event/mod.rs:755`.
- Model suggestion: Replay an actual vendor secure delegate to capture permitted receiver/taint behavior before strengthening enforcement.

### 9. `changed:Patch12_1.ForbiddenAspects.UntrustedScriptExecution` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:141`: “Operations requiring trusted script execution reject insecure callers.”
- Cached declaration: **none for synthetic label**; related `ForbiddenAspectConstantsDocumentation.lua:15`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/frame/methods/forbidden_aspects.rs:23`.
- Model suggestion: Capture trusted/insecure operations with aspect masks; reuse centralized pre-mutation aspect enforcement.

### 10. `changed:Patch12_1.ForbiddenAspects.UntrustedLayoutScriptExecution` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:147`: “Layout-script operations reject insecure callers lacking the required aspect.”
- Cached declaration: **none for synthetic label**; related `ForbiddenAspectConstantsDocumentation.lua:16`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/frame/methods/forbidden_aspects.rs:23`.
- Model suggestion: Capture layout-specific trusted boundary independently, preserving ordinary layout and engine input paths.

### 11. `changed:Patch12_1.AuraContainer.SecretVisibility` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:219`: “Secret aura values remain hidden while container and button structure stays usable.”
- Cached declaration: **none for synthetic label**; related `AuraContainerUtilDocumentation.lua`.
- Simulator registration: **not registered (synthetic label)**; related boundary `patch-tests/patch_12_1/aura_container.rs (related test, not a registration)`.
- Model suggestion: Drive an actual ManagedAuraContainer through secret/public partitions; preserve frames while hiding prohibited payload fields.

### 12. `changed:Patch12_1.Service.PrivateAura.Payloads` — evidence-required
- Source: `data/patch-api/sources/12.1-behaviors.json:375`: “Private-aura payloads preserve inaccessible and secret structural boundaries.”
- Cached declaration: **none for synthetic label**; related `UnitAuraDocumentation.lua:39`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/workarounds/temporary/private_aura_state.rs:133`.
- Model suggestion: Capture native private-aura anchor/update payloads before replacing deterministic placeholder state with modeled inaccessible payloads.

### 13. `changed:Patch12_1.PrivateScriptObjects.PrivateIdentity` — best-effort
- Source: `data/patch-api/sources/12.1-behaviors.json:105`: “Private or forbidden objects have identity distinct from their public frame view.”
- Cached declaration: **none for synthetic label**; related `SimpleFrameAPIDocumentation.lua:323`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/frame/methods/button_anchor_hierarchy/mod.rs:263`.
- Model suggestion: Re-audit native identity/interning across public/private projections; current simulator identity proof is bounded only.

### 14. `changed:Patch12_1.PrivateScriptObjects.InaccessiblePublicKeys` — best-effort
- Source: `data/patch-api/sources/12.1-behaviors.json:111`: “Private keys remain inaccessible through the public object.”
- Cached declaration: **none for synthetic label**; related `SimpleScriptRegionAPIDocumentation.lua:233`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/frame/methods/text_attribute_event/mod.rs:756`.
- Model suggestion: Capture the native key-access/export list; current partition isolation is not evidence for every native prohibited key.

### 15. `changed:Patch12_1.ForbiddenAspects.QueryFocus` — best-effort
- Source: `data/patch-api/sources/12.1-behaviors.json:171`: “Focus-query operations enforce the QueryFocus aspect restriction.”
- Cached declaration: **none for synthetic label**; related `SimpleEditBoxAPIDocumentation.lua:408 (HasFocus); SimpleScriptRegionAPIDocumentation.lua:469 (IsMouseMotionFocus)`.
- Simulator registration: **not registered (synthetic label)**; related boundary `src/lua_api/frame/methods/forbidden_aspects.rs:23`.
- Model suggestion: Re-audit exact native annotated queries with insecure callers; preserve unannotated global/physical queries.

## 12.1.5 — ten blockers plus five bounded residuals

First ten are the complete **evidence-required** set; last five are **best-effort** residual follow-ups. Current PTR cache prefix: `~/.cache/wow-ui-sim/blizzard-ui/ptr/AddOns/Blizzard_APIDocumentationGenerated/`. Ten blockers are documentation structures/fields/predicates, **not ten missing callable APIs**. Existing native capture is useful but has not upgraded their manifest dispositions.

`docs/baselines/ptr-12-1-5-remaining-structures.lua` and `docs/addons/Ptr125RemainingProbe/README.md` document PTR 12.1.5.69594 observations: positional region creation works; candidate options-table calls fail; both structure-named globals are nil; `GetNextSignal()` yields two numbers `(17,time)`. Secret/access experiments remain deferred. Treat these observations as contract evidence, not permission to manufacture new globals.

### 1. `added:CreateRegionParams.name` — evidence-required
- Source: `data/patch-api/sources/12.1.5-register.json:1448`: `{"before":null,"after":{"Name":"name","Nilable":true,"Type":"cstring"}}`
- Cached declaration: `SimpleFrameAPIDocumentation.lua:1625`.
- Simulator registration: not registered (documentation-only structure field).
- Model suggestion: Keep positional name behavior; native probe rejects options-table calls, so do not create a global/options API from this field.

### 2. `added:CreateRegionParams.drawLayer` — evidence-required
- Source: `data/patch-api/sources/12.1.5-register.json:1437`: `{"before":null,"after":{"Name":"drawLayer","Nilable":true,"Type":"DrawLayer"}}`
- Cached declaration: `SimpleFrameAPIDocumentation.lua:1626`.
- Simulator registration: not registered (documentation-only structure field).
- Model suggestion: Relate the field to actual positional layer inputs and native capture; reject invented public structure publication.

### 3. `added:CreateRegionParams.subLevel` — evidence-required
- Source: `data/patch-api/sources/12.1.5-register.json:1459`: `{"before":null,"after":{"Name":"subLevel","Nilable":true,"Type":"number"}}`
- Cached declaration: `SimpleFrameAPIDocumentation.lua:1628`.
- Simulator registration: not registered (documentation-only structure field).
- Model suggestion: Use captured positional sublevel round-trip to delimit behavior; options-table support is not evidenced.

### 4. `added:CreateRegionParams.templateName` — evidence-required
- Source: `data/patch-api/sources/12.1.5-register.json:1470`: `{"before":null,"after":{"Name":"templateName","Nilable":true,"Type":"cstring"}}`
- Cached declaration: `SimpleFrameAPIDocumentation.lua:1627`.
- Simulator registration: not registered (documentation-only structure field).
- Model suggestion: Use the captured Texture/FontString template controls; Line/Mask template and type/security edges remain unproven.

### 5. `added:CreateRegionParams` — evidence-required
- Source: `data/patch-api/sources/12.1.5-register.json:1426`: `{"before":null,"after":{"Fields":[{"Name":"name","Nilable":true,"Type":"cstring"},{"Name":"drawLayer","Nilable":true,"Type":"DrawLayer"},{"Name":"templateName","Nilable":true,"Type":"cstring"},{"Name":"subLevel","Nilable":true,"Type":"number"}],"Name":"CreateRegionParams","Type":"Structure"}}`
- Cached declaration: `SimpleFrameAPIDocumentation.lua:1621`.
- Simulator registration: not registered (documentation-only type; native global is nil).
- Model suggestion: Reclassify only on consumer/declaration-role proof; do not publish a global or options-table overload rejected by native capture.

### 6. `added:TimedSignalMapEntry.key` — evidence-required
- Source: `data/patch-api/sources/12.1.5-register.json:4200`: `{"before":null,"after":{"Name":"key","Nilable":false,"Type":"number"}}`
- Cached declaration: `TimedSignalMapSharedDocumentation.lua:11`.
- Simulator registration: not registered (documentation-only field).
- Model suggestion: Preserve captured GetNextSignal key as tuple result one; never invent an entry table solely from the type declaration.

### 7. `added:TimedSignalMapEntry.time` — evidence-required
- Source: `data/patch-api/sources/12.1.5-register.json:4211`: `{"before":null,"after":{"Name":"time","Nilable":false,"Type":"FrameTime"}}`
- Cached declaration: `TimedSignalMapSharedDocumentation.lua:12`.
- Simulator registration: not registered (documentation-only field).
- Model suggestion: Preserve captured absolute-time tuple result two; compare entry type role with actual consuming API.

### 8. `added:TimedSignalMapEntry` — evidence-required
- Source: `data/patch-api/sources/12.1.5-register.json:4189`: `{"before":null,"after":{"Fields":[{"Name":"key","Nilable":false,"Type":"number"},{"Name":"time","Nilable":false,"Type":"FrameTime"}],"Name":"TimedSignalMapEntry","Type":"Structure"}}`
- Cached declaration: `TimedSignalMapSharedDocumentation.lua:6`.
- Simulator registration: not registered (documentation-only type; native global is nil).
- Model suggestion: Native GetNextSignal returns (key,time), not a public entry table; locate real structured consumer before implementing anything.

### 9. `added:RequiresTimedSignalMapAccess` — evidence-required
- Source: `data/patch-api/sources/12.1.5-register.json:3838`: `{"before":null,"after":{"FailureMode":"Error","Name":"RequiresTimedSignalMapAccess","Type":"Precondition"}}`
- Cached declaration: `TimedSignalMapSharedDocumentation.lua:30`.
- Simulator registration: not registered (precondition metadata); related src/c_api/timed_signal_map.rs:82–87.
- Model suggestion: Probe tainted access to maps containing secret values; enforce at guarded methods, not as a callable global.

### 10. `added:SecretWhenLuaTableHasSecretKeys` — evidence-required
- Source: `data/patch-api/sources/12.1.5-register.json:3924`: `{"before":null,"after":{"Name":"SecretWhenLuaTableHasSecretKeys","Type":"Secret"}}`
- Cached declaration: `LuaValuePredicatesDocumentation.lua:10`.
- Simulator registration: not registered (secret predicate metadata).
- Model suggestion: Probe actual annotated consumers with secret table keys and rooted results; implement secrecy at those consumers only.

### 11. `added:C_AuraContainerUtil.CustomAuraButtonApplicationBarOptions.minApplications` — best-effort
- Source: `data/patch-api/sources/12.1.5-register.json:56`: `{"before":null,"after":{"Default":0,"Name":"minApplications","Nilable":false,"Type":"number"}}`
- Cached declaration: `AuraContainerUtilDocumentation.lua:246`.
- Simulator registration: not registered as a public structure field; related processing binding `src/c_api/c_aura_container_util.rs:227`.
- Model suggestion: Re-audit default zero/explicit numeric value and wrong-type/security controls without claiming application-bar rendering.

### 12. `added:C_AuraContainerUtil.CustomAuraButtonCasterNameOptions.showRealmName` — best-effort
- Source: `data/patch-api/sources/12.1.5-register.json:92`: `{"before":null,"after":{"Default":false,"Name":"showRealmName","Nilable":false,"Type":"bool"}}`
- Cached declaration: `AuraContainerUtilDocumentation.lua:263`.
- Simulator registration: not registered as a public structure field; related processing binding `src/c_api/c_aura_container_util.rs:230`.
- Model suggestion: Reuse normalized option state; probe display/realm semantics separately from default-false normalization.

### 13. `added:C_AuraContainerUtil.CustomAuraButtonCasterNameOptions.useClassColors` — best-effort
- Source: `data/patch-api/sources/12.1.5-register.json:104`: `{"before":null,"after":{"Default":false,"Name":"useClassColors","Nilable":false,"Type":"bool"}}`
- Cached declaration: `AuraContainerUtilDocumentation.lua:264`.
- Simulator registration: not registered as a public structure field; related processing binding `src/c_api/c_aura_container_util.rs:230`.
- Model suggestion: Reuse normalized flag storage; do not infer actual class-color rendering from processor round-trips.

### 14. `changed:table.freeze` — best-effort
- Source: `data/patch-api/sources/12.1.5-register.json:14955`: `{"before":{"Arguments":[{"Name":"table","Nilable":false,"Type":"LuaValueReference"}],"Name":"freeze","SecretArguments":"AllowedWhenUntainted","Type":"Function"},"after":{"Arguments":[{"Name":"table","Nilable":false,"Type":"table"}],"Name":"freeze","Returns":[{"Name":"frozen","Nilable":false,"Type":"table"}],"SecretArguments":"AllowedWhenUntainted","Type":"Function"}}`
- Cached declaration: `LuaTableExtensionsDocumentation.lua:79`.
- Simulator registration: src/lua_api/globals/real/table_freeze.rs:13.
- Model suggestion: Reuse root-only read-only state and lifecycle/GC regression; re-audit changed authenticated input policy separately.

### 15. `changed:table.isfrozen` — best-effort
- Source: `data/patch-api/sources/12.1.5-register.json:14998`: `{"before":{"Arguments":[{"Name":"table","Nilable":false,"Type":"LuaValueReference"}],"Name":"isfrozen","Returns":[{"Name":"frozen","Nilable":false,"Type":"bool"}],"SecretArguments":"AllowedWhenUntainted","Type":"Function"},"after":{"Arguments":[{"Name":"table","Nilable":false,"Type":"table"}],"Name":"isfrozen","Returns":[{"Name":"frozen","Nilable":false,"Type":"bool"}],"SecretArguments":"AllowedWhenUntainted","Type":"Function"}}`
- Cached declaration: `LuaTableExtensionsDocumentation.lua:163`.
- Simulator registration: src/lua_api/globals/real/table_freeze.rs:14.
- Model suggestion: Reuse root read-only query; re-audit AllowedWhenUntainted and secret-table keys without recursive freezing.

Related region methods are registered in `src/lua_api/frame/methods/button_anchor_hierarchy/mod.rs:282` (CreateTexture), `:293` (CreateMaskTexture), `:296` (CreateLine), `:300` (CreateFontString). Timed-signal-map methods are registered in `src/c_api/timed_signal_map.rs:82–87`. These are consumer boundaries, not registrations of documentation types.

## October 12.0.5 overlaps — reusable producer proof, not automatic same-delta credit

Compared exact occurrence `symbol` values against each current 12.0.5 capability’s `symbols`, then inspected direction/source delta and linked credited source_rows. Read-only symbol-match calculation:

```python
cap_symbols = {s for cap in coverage["capabilities"] for s in cap.get("symbols", [])}
matched = [r for r in manifest["rows"] if r["symbol"] in cap_symbols]
# A symbol match is only a candidate. Compare old before/after and literal 12.0.5
# source change; credit only the common proven delta/profile, never the whole row.
```

- `12.0.0`: **33** exact-symbol occurrence matches; **29** are still evidence-required.
- `12.0.7`: **3** exact-symbol occurrence matches; **0** are still evidence-required.
- `12.1-behaviors`: **0** exact-symbol occurrence matches; **0** are still evidence-required.
- `12.1-framexml`: **0** exact-symbol occurrence matches; **0** are still evidence-required.
- `12.1.5`: **0** exact-symbol occurrence matches; **0** are still evidence-required.

**Confirmed automatically transferable same-symbol + same-atomic-delta credits: none established.** This is not a claim that newer runtime work has no value. The earlier 12.0.0 source mostly says “API added”; October 12.0.5 rows prove narrower argument/return-security or identifier changes. A current producer and bounded modern tests cannot establish the full historical added contract/profile. Conversely, 12.0.7 removal of `C_Spell.GetMawPowerBorderAtlasBySpellID` is not satisfied by 12.0.5’s newly modeled query; it is the opposite direction. Ready-check additions in 12.0.7 are not the 12.0.5 chat-lockdown restriction delta. FontString 12.0.7 number→uiFontHeight/valid-font predicates are not 12.0.5 uiUnit→number/optional-flags credit. `table.freeze/isfrozen` 12.0.5 prose is still audit-pending; no approved October page credit can be imported from that row.

The following same-symbol **producer reuse candidates** explain the recommended 12.0.0 slice. Retain older row evidence-required until bounded historical publication/contract proof is reconciled; a matching 12.0.5 capability does not close older residuals.

| Older symbol | October 12.0.5 capability | Credited literal source IDs/deltas and proof status | Reuse boundary |
|---|---|---|---|
| `C_NamePlateManager.GetNamePlateHitTestInsets` | `nameplate-insets` | `prose-2026-03-25-081` Added an API to allow addons to override the offsets of nameplate hit-rects. (bounded-coverage) | Per-friendly/enemy type configuration and guarded input/callers; no fabricated plates, no actual world nameplate hit testing or bounds clamping. Zero defaults/caller policies inferred. |
| `C_NamePlateManager.SetNamePlateHitTestInsets` | `nameplate-insets` | `prose-2026-03-25-081` Added an API to allow addons to override the offsets of nameplate hit-rects. (bounded-coverage) | Per-friendly/enemy type configuration and guarded input/callers; no fabricated plates, no actual world nameplate hit testing or bounds clamping. Zero defaults/caller policies inferred. |
| `C_ActionBar.GetActionText` | `action-text` | `prose-2026-03-31-170` Added a new C_ActionBar.UsesActionText API and switched to using it inside ActionBarActionButtonMixin:Update. This should fix some Lua errors that were getting thrown for addons in that code due to secrets. (bounded-coverage) | Populated macro names from live action-slot state; macro move/rename/delete and spell/empty slots. Qualifying action kinds inferred; populated item/outfit action labels not proven. |
| `C_ActionBar.GetActionDisplayCount` | `action-count-outputs` | `global api-C_ActionBar-GetActionDisplayCount-237` SecretWhenActionCooldownRestricted -> SecretWhenCooldownsRestricted (bounded-coverage); `global api-C_ActionBar-GetActionUseCount-241` SecretWhenActionCooldownRestricted -> SecretWhenCooldownsRestricted (bounded-coverage) | Exact237/241 output predicates plus meaningful count providers: empty explicit slot-keyed use snapshot matched current spell binding; valid existing charge quantity supplies display only. Independent NUM use/STRING display with decimal/>maximum replacement/default9999/* formatting. Missing/zero/priority/domain/nil/width rules inferred, no fake acquisition/consumption/inventory. VM original args authentication before parse/model, public tainted allowed, secure actual secrets accepted; restricted real host NUM/STR including zero/empty, immediate root, host payload/Lua opacity/copy/GC/trust/live/isolation. No opaque nominal-type/native/allaction/UI/global-privacy parity. |
| `C_ActionBar.GetActionUseCount` | `action-count-outputs` | `global api-C_ActionBar-GetActionDisplayCount-237` SecretWhenActionCooldownRestricted -> SecretWhenCooldownsRestricted (bounded-coverage); `global api-C_ActionBar-GetActionUseCount-241` SecretWhenActionCooldownRestricted -> SecretWhenCooldownsRestricted (bounded-coverage) | Exact237/241 output predicates plus meaningful count providers: empty explicit slot-keyed use snapshot matched current spell binding; valid existing charge quantity supplies display only. Independent NUM use/STRING display with decimal/>maximum replacement/default9999/* formatting. Missing/zero/priority/domain/nil/width rules inferred, no fake acquisition/consumption/inventory. VM original args authentication before parse/model, public tainted allowed, secure actual secrets accepted; restricted real host NUM/STR including zero/empty, immediate root, host payload/Lua opacity/copy/GC/trust/live/isolation. No opaque nominal-type/native/allaction/UI/global-privacy parity. |
| `C_ActionBar.GetActionCharges` | `cooldown-charge-restriction` | `global api-C_ActionBar-GetActionCharges-231` SecretWhenActionCooldownRestricted -> SecretWhenCooldownsRestricted (partial-development-green); `global api-C_Spell-GetSpellCharges-303` # SecretWhenSpellCooldownRestricted -> SecretWhenCooldownsRestricted (partial-development-green); `global api-C_SpellBook-GetSpellBookItemCharges-320` # SecretWhenSpellCooldownRestricted -> SecretWhenCooldownsRestricted (partial-development-green) | Explicit false-default restriction predicate; three five-field charge tables with four restricted numeric fields and public max/table; guarded action/book selectors. Secret spell identifiers AllowedWhenTainted unresolved; no automatic hooks, per-spell exceptions, other cooldown output policies, older-profile execution or native parity. |
| `C_ActionBar.GetActionChargeDuration` | `zero-span-charge-durations` | `prose-2026-03-12-023` The following APIs will now return a zero-span duration object when the queried spell is at maximum charges: GetSpellBookItemChargeDuration, GetSpellChargeDuration, GetActionChargeDuration. (bounded-coverage); `prose-2026-03-12-026` Duration objects that measure a zero-span are now considered fully elapsed. (bounded-coverage) | B90 prose-2026-03-12-023/026: at maximum charges all three charge-duration queries return a nonnil zero-span duration object from the shared explicit charge-state producer; zero-span objects report expired, elapsed fraction1 and remaining fraction0 for default, future-start, reset and explicit zero intervals, with a below-maximum nonzero control. 'Fully elapsed' is concretized as HasExpired/fractions, HasStarted deliberately false. Configured charge state only; no automatic progression, native rate/security policy, secret-configured spans or other profiles. |
| `C_Spell.GetSpellDisplayCount` | `spell-count-outputs` | `global api-C_Spell-GetSpellCastCount-301` # SecretWhenSpellCooldownRestricted -> SecretWhenCooldownsRestricted (bounded-coverage); `global api-C_Spell-GetSpellDisplayCount-309` # SecretWhenSpellCooldownRestricted -> SecretWhenCooldownsRestricted (bounded-coverage) | Exact301/309 output predicates and meaningful empty-default spell-keyed quantities, distinct from action-slot counts; valid charges supply display only. Decimal strict >maximum replacement/default9999/* policies inferred. Display authenticates all original VM arguments before parsing/model; restricted real host NUM/STR preserve payload, opacity, copies, roots and GC. Cast AllowedWhenTainted secret-input parity remains UNMODELED: conservative rejection earns no input-policy or NeverSecret credit. No acquisition, consumption, nominal primitive type, native/global privacy or full-profile/UI parity. |
| `C_Spell.GetSpellMaxCumulativeAuraApplications` | `spell-max-cumulative-aura-applications` | `global api-C_Spell-GetSpellMaxCumulativeAuraApplications-315` # arg1.Type number -> SpellIdentifier (bounded-coverage); `global api-C_Spell-GetSpellMaxCumulativeAuraApplications-316` # SecretWhenSpellAuraRestricted -> SecretWhenUnitAuraRestricted (bounded-coverage) | B95 exact315/316 on a newly registered query: public spell identifier (number, name, alias-first) resolved through the shared reader to an explicit spell-keyed maximum, zero on a miss, independent of applied stacks; secret host number only under the explicit unit_auras_restricted flag. Secret identifiers rejected for every caller (declared AllowedWhenTainted unmodeled); nothing sets the flag automatically; no native maxima or C_Secrets predicate. |
| `C_UnitAuras.AuraIsBigDefensive` | `aura-spell-classification-identifiers` | `global api-C_UnitAuras-AuraIsBigDefensive-361` # arg1.Type number -> SpellIdentifier (bounded-coverage); `global api-C_UnitAuras-AuraIsPrivate-363` # arg1.Type number -> SpellIdentifier (bounded-coverage) | Exact361/363 inferred explicit empty host classification map, independent flags, strict public identifiers and aliases, one public boolean, immediate mutation/read-only/environment isolation, no generic aura/private-instance/cooldown inference, public caller-taint preservation and GC-rooted conservative actual-secret rejection/recovery. Native permissions, catalog/acquisition, identifier parity and result secrecy unknown. |
| `C_UnitAuras.DoesAuraHaveExpirationTime` | `aura-expiration-time-arguments` | `global api-C_UnitAuras-DoesAuraHaveExpirationTime-365` # SecretArguments AllowedWhenTainted -> AllowedWhenUntainted (bounded-coverage) | B86 exact365 AllowedWhenUntainted: unit and aura instance are both VM-authenticated before either is validated; untainted callers may pass authentic secret unit/ID/both, tainted callers are denied before validation or lookup including with a malformed public unit; caller taint and input secrecy retained, GC-rooted unit and numeric identity. One public boolean from stored nonzero expiration over player helpful/harmful and party records, live reads, inferred false for unknown/nil unit and strict representation errors. Blocked/target records, full-record immutability, extra arguments, RequiresUnitAuraAccess/RequiresValidUnitAuraInstance/SecretWhenUnitAuraRestricted output and profiles below retail-12-0-5 unasserted or unmodeled. |
| `C_UnitAuras.GetAuraApplicationDisplayCount` | `aura-application-display-count` | `global api-C_UnitAuras-GetAuraApplicationDisplayCount-367` + arg3 NeverSecret (bounded-coverage); `global api-C_UnitAuras-GetAuraApplicationDisplayCount-368` + arg4 NeverSecret (bounded-coverage); `global api-C_UnitAuras-GetAuraApplicationDisplayCount-369` # SecretArguments AllowedWhenTainted -> AllowedWhenUntainted (bounded-coverage) | Exact367/368 NeverSecret thresholds and369 AllowedWhenUntainted unit/ID. Authentic VM secrets, tainted denial/recovery and GC identity; unchanged blocked-inclusive typed count lookup. 14 display +80 controls =94 unique PASS; startup0 []. Strict representation/decimal/missing/min-first policies inferred. Native permissions/valid-instance/restricted-output secrecy excluded. |
| `C_UnitAuras.GetAuraBaseDuration` | `aura-refresh-duration` | `global api-C_UnitAuras-GetAuraBaseDuration-371` # arg3.Type number -> SpellIdentifier (bounded-coverage); `global api-C_UnitAuras-GetRefreshExtendedDuration-394` # arg3.Type number -> SpellIdentifier (bounded-coverage) | Parent accepts independent bounded optional identifier coverage, consumer GetAuraBaseDuration prerequisite and explicit unknown metadata: 18 duration + 73 controls = 91 distinct PASS, saved startup0 [], fresh fmt/check0. Formula/cap/permanence/nil/strict secret and seeded alias policies inferred, not native. Production metadata empty; queries return nil until explicit metadata. No Blizzard visual, native, whole-page or all-profile claim. Two nonblocking readability suggestions deferred; no behavior issue or adjacent restructure authorized. |
| `C_UnitAuras.GetAuraDuration` | `aura-duration-object` | `global api-C_UnitAuras-GetAuraDuration-380` # SecretArguments AllowedWhenTainted -> AllowedWhenUntainted (bounded-coverage) | B93 exact380 AllowedWhenUntainted on a newly registered query: one fresh duration object per call spanning stored expiration minus duration to expiration at rate 1 for player helpful/harmful and party records, zero span for permanent records, live reads with independent snapshots, errors (never nil) for unknown or cross-unit instances and malformed or missing arguments; both arguments VM-authenticated before validation, untainted secret acceptance, tainted denial incl. malformed public unit, taint and secrecy retained. Inferred invalid-instance and permanent policies; mixed zero fields not rejected; RequiresUnitAuraAccess, restricted output, blocked/target records, rate modifiers and native parity unmodeled or unasserted. |
| `C_UnitAuras.GetRefreshExtendedDuration` | `aura-refresh-duration` | `global api-C_UnitAuras-GetAuraBaseDuration-371` # arg3.Type number -> SpellIdentifier (bounded-coverage); `global api-C_UnitAuras-GetRefreshExtendedDuration-394` # arg3.Type number -> SpellIdentifier (bounded-coverage) | Parent accepts independent bounded optional identifier coverage, consumer GetAuraBaseDuration prerequisite and explicit unknown metadata: 18 duration + 73 controls = 91 distinct PASS, saved startup0 [], fresh fmt/check0. Formula/cap/permanence/nil/strict secret and seeded alias policies inferred, not native. Production metadata empty; queries return nil until explicit metadata. No Blizzard visual, native, whole-page or all-profile claim. Two nonblocking readability suggestions deferred; no behavior issue or adjacent restructure authorized. |

## Evidence/provenance limits

Inspected evidence directory inventory: 56 files, suffix counts `{'.lua': 2, '.txt': 25, '.md': 22, '.stdout': 7}`. Immediate subdirectories are only `12.0.5` and `12.0.5-session-2026-10-03`; no separately preserved other-patch evidence directory exists here. Other manifests cite repository tests/specs/source hashes and historical temporary proof ledgers. Those are recorded evidence, not fresh execution in this scout.

Current parsed 12.0.5 checkpoint: 110 capabilities, 362 rows, {'metadata-only': 33, 'bounded-coverage': 243, 'audit-pending': 71, 'partial-development-green': 15}. The page remains in-progress; do not equate its high bounded-coverage count with exhaustive native conformance.

Snapshot SHA-256 bindings (these are scout fingerprints, not validator/test proof):
- `data/patch-api/12.0.0.json`: `6cc66364c986fa0feed4a2a15ce5f050358ff27229c051790b658f550ed90e59`.
- `data/patch-api/12.0.7.json`: `320773e0ca7a77d432b8eb004b56b72463b4bd3d6743262af137d011bc4b562c`.
- `data/patch-api/12.1-behaviors.json`: `c79b27b197f2bcacc403df5e80756929f4fe650a76381852ddf4aa4596912d43`.
- `data/patch-api/12.1-framexml.json`: `c61d58945b32dcd94dba5845e519ad6ef2a7bb23f2543d6cae7c9f94d23809a5`.
- `data/patch-api/12.1.5.json`: `5ab145742938c355cca934a68afaa930de806cc577be1ef781cba3eb91769945`.
- `data/patch-api/sources/12.0.5-page-coverage.json`: `ecee38622055026312edc7b2d0ae98f713ec556095aae997948550c5f9193d2e`.

### Inspected saved evidence and epoch caveat

Read `data/patch-api/evidence/12.0.5-session-2026-10-03/b93-aura-duration-proof.md`: ACCEPT WITH QUALIFICATIONS, 33/33 saved independent results, newly registered producer `012cf889a`, and explicit access/restricted-output/mixed-zero/native/older-profile exclusions. Also read both B86 reports: `b86-expiration-independent-proof.md` rejects authentication ordering; the later `b86-expiration-reverify.md` accepts corrected `d1bbdc8e8` with 9 expiration + 18 duration controls and explicit native/older-profile limits. Do not cite the earlier rejection as final acceptance or erase it.

Crucially, `src/c_api/mod.rs:15–16` gates `aura_duration` on `retail-12-0-5`; `src/lua_api/globals/register.rs` gates its registration likewise. Several action-count producers are similarly gated. The suggested 12.0.0 reuse slice therefore begins with **historical epoch availability**, not blind crediting: current modern registration does not prove a build targeting 12.0.0 publishes it. If old declarations and allowed target semantics demand implementation, that is implementation work outside this read-only scout. No broadening of epochs, source/runtime changes or metadata credits was performed.

12.0.0's two exception-requested rows are `C_NamePlateManager.IsNamePlateUnitBehindCamera` and `Model.SetUseGBuffer`, both covered by the permanent no-3D scope rule. 12.0.7's sole exception is `ModelSceneActorBase.GetModelUnitGUID`. Keep all three outside the actionable producer queue.

Read `docs/addons/Ptr125RemainingProbe/README.md`: **secrets work is deferred**. Predicate suggestions above describe future contract work only when that deferral is explicitly reopened; do not run the probe's secret commands as a scouting continuation. Structure captures do not establish absence of every possible structured consumer.

## Scout completion check

All four patch sections contain exactly 15 concrete entries. All named next rows were found in their parsed machine/source registers; all recommended 12.0.0 rows are evidence-required. Report distinguishes best-effort follow-ups from open rows, synthetic audit labels from real API registrations, and documentation types/predicates from callable globals. Cached references and registration anchors are source locations, not runtime acceptance.

Only this scratch report was written/edited. Repository and cache files were read-only. No cargo, tests, builds, validators, source synchronization, Git mutation, agents, model CLIs or operational actions were invoked.

## Existing blocker snapshot (additional accounting artifact)

`docs/generated/patch-api-blockers.json` uses `patch-api-blockers/v1`, verified 2026-09-14. Its `rows` carry manifest/id/owner/missing_evidence/probe_protocol/scope/basis_references/probe_plan; `manifests` bind their current SHA-256 and row totals. Parsed total **1,083** includes four old 12.0.5-probe blockers outside this request; the four other patches contribute **1,079 evidence-required** rows. The three scope exceptions are deliberately excluded. All six stored manifest hashes still match current files, so the older verification stamp does not by itself invalidate this membership snapshot. Its gap descriptions must still be re-audited against newer code.

| Manifest | Blocker protocols (parsed Counter) |
|---|---|
| `12.0.0` | `{'event-producer': 88, 'api-state': 419, 'structure-producer': 317, 'publication-removal': 233}` |
| `12.0.7` | `{}` |
| `12.1-behaviors` | `{'deferred-security': 12}` |
| `12.1-framexml` | `{}` |
| `12.1.5` | `{'region-params': 5, 'access-predicate': 1, 'deferred-security': 1, 'timed-entry': 3}` |

The recorded `deferred-security` protocol says: “Security/secret enforcement is excluded from the active scope.” This is a historical blocker-plan boundary, not new authorization or a changed user scope. Report lists security gaps for visibility only; current task is read-only. No tracked blocker JSON generation command was found in tools/scripts/src/xtask.
