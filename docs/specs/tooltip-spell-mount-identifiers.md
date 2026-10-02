# Tooltip spell and mount identifiers

Retail 12.0.5 `C_TooltipInfo.GetMountBySpellID` and `GetSpellByID` must accept the chosen shared public spell-identifier model and enforce their documented NeverSecret optional arguments. Bounded next56 implementation is present; parent GREEN and acceptance remain pending. No native-verified capability is claimed. See [Lua API architecture](../lua-api.md) and [frame data flow](../frame-data-flow.md).

## What it must do

### Identifiers and meaningful payloads

- [ ] Use the existing strict public `c_spell::read_public_spell_identifier_at` contract: finite integral u32 NUMBER or UTF-8 STRING, with seeded lowercase aliases preceding numeric identity. This representation/validation policy is **inferred**, not native argument-type evidence.
- [ ] Numeric 19750 retains the real generated Flash of Light title, modeled cast/description lines, spell type, ID and width. Numeric mount 23338 retains the first actual default `world.mounts` match, Swift Palomino, its title and existing `Summons this mount.` description.
- [ ] DTO equivalence compares Color values through exactly four public numeric RGBA components (`GetRGBA()` or public numeric `r/g/b/a`), not per-instance method/object identity. All non-color fields retain strict recursive value/type equality with diagnostic paths.
- [ ] Explicit numeric-string, case-normalized name and full colored-link aliases produce DTO-equivalent results to their resolved numeric IDs for both queries. A seeded numeric alias wins over numeric identity; a full-link alias wins over its embedded number. Output ID is the resolved ID.
- [ ] Read alias changes/removal live. Queries do not mutate aliases or declared mounts; independent environments remain isolated. Mutating a returned DTO cannot change later results or another returned DTO.
- [ ] Preserve current unknown numeric policy: exactly one identified, line-empty, Spell-typed tooltip, including valid u32 endpoints.
- [ ] **INFERRED new unresolved-public-STRING miss policy:** exactly one fresh, unidentified, empty Spell-typed TooltipData with a fresh lines table. Never invent an ID. Unseeded numeric strings, catalog names, short links and full colored links remain misses; no general parsing/catalog-name lookup/acquisition/override graph.
- [ ] Results and nested DTO fields remain public under the chosen simulator contract. Native output secrecy remains unknown.

### Secret boundary and caller context

- [ ] Reject authentic VM-secret optional arguments in all documented positions, whether caller is secure or ordinarily tainted, before alias/catalog/payload acquisition. Mount arg2 `checkIndoors`; spell args2–6 `isPet`, `showSubtext`, `dontOverride`, `difficultyID`, `isLink`.
- [ ] **INFERRED conservative arg1 policy:** reject secret identifiers through the same public identifier helper without unwrapping or inspecting payloads. Native `AllowedWhenTainted` permissions are unknown; rejection does not establish them.
- [ ] Reject actual host-secret BOOL true and false, NUMBER, STRING, wrapped real Frame table and ordinary table across every optional position and both arg1 boundaries. Known numeric/name inputs and numeric/string misses cannot bypass NeverSecret checks.
- [ ] Failed queries preserve authentic wrapper secrecy, global/list/stack roots, live wrapper allocation identity, ordinary caller properties and owned state maps; valid public recovery still works after forced GC and failures.
- [ ] Preserve stack taint for rejection and ordinary public calls. Restore secure context after returning from an explicitly tainted closure.
- [ ] **INFERRED error policy:** provide nonempty public API context, without private string payload disclosure. Exact native errors and precedence are unknown; tests do not assert source text or a native error message.

### Public optional compatibility and frame route

- [ ] Preserve **current ignored-provider behavior**, not new flag semantics: documented nil/BOOL combinations for mount arg2 and spell args2/3/4/6, plus nil or public number17 at spell arg5, leave actual payload unchanged. No new public optional type/domain/finite validation is required for unused flags.
- [ ] Actual `GameTooltip:SetSpellByID` and `SetMountBySpellID` pass the original identifier to the same real namespace query. Numeric/name/full-link aliases preserve every direct-query TooltipData field/value, including public RGBA components, and equivalent actual rendered lines. Processing may add per-line `lineIndex` only: when present it must be public numeric and equal the actual line position. Other extra fields or lines fail equivalence. No query replacement, method-call spy, VM-shape assertion or duplicate builder counts as frame proof.
- [ ] Secret frame identifiers and mount arg2 fail without replacing prior exposed TooltipData or rendered lines; valid public frame recovery remains meaningful.

## How it works

- [Lua API architecture](../lua-api.md)
- [Frame data flow](../frame-data-flow.md)
- [Widget system](../widget-system.md)

## Implementation inventory

### Exact declarations and four uncredited source IDs

`data/patch-api/sources/12.0.5-api-changes.txt`, register `12.0.5-register.json`, and coverage `12.0.5-page-coverage.json` retain these exact IDs:

| Source ID | Literal delta | Current gap |
| --- | --- | --- |
| `global api-C_TooltipInfo-GetMountBySpellID-333` | arg1.Type number → SpellIdentifier | Shared strict public alias-first resolver implemented; GREEN pending. |
| `global api-C_TooltipInfo-GetMountBySpellID-334` | arg2 NeverSecret | Actual VM-secret rejection before lookup implemented; GREEN pending. |
| `global api-C_TooltipInfo-GetSpellByID-336` | arg1.Type number → SpellIdentifier | Shared strict public alias-first resolver implemented; GREEN pending. |
| `global api-C_TooltipInfo-GetSpellByID-337` | arg2–6 NeverSecret | Actual VM-secret rejection before lookup implemented; GREEN pending. |

All four currently have `audit-pending` status and empty capabilities. This implementation changes no source/register/coverage/accounting. Parent's 214 pending / 134 bounded / 14 partial checkpoint receives no credit from these fixtures.

Actual profile cache inspected October 2, 2026:
`~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua`, complete function blocks at lines629–644 and1045–1063:

| Query | Required arg1 | Nilable NeverSecret optional arguments |
| --- | --- | --- |
| `GetMountBySpellID` | `spellID: SpellIdentifier` | arg2 `checkIndoors: bool` |
| `GetSpellByID` | `spellID: SpellIdentifier` | arg2 `isPet: bool`; arg3 `showSubtext: bool`; arg4 `dontOverride: bool`; arg5 `difficultyID: number`; arg6 `isLink: bool` |

Both declare `SecretArguments = "AllowedWhenTainted"`, `MayReturnNothing = true`, and a `data: TooltipData` return with `Nilable = false`. These are declarations, not runtime/error/permission/miss evidence. The selected unresolved-string policy does **not** claim native MayReturnNothing semantics.

### Existing meaningful producers and state

- `src/c_api/c_tooltip_info_spell_mount.rs`: owns both handlers and their sole `retail-12-0-5` publication. Rejects actual VM-secret arg1 and all declared optional positions before shared identifier resolution; public errors include method, argument index and policy without payload inspection or taint changes. Ordinary public optionals remain ignored.
- `src/c_api/mod.rs`: feature-gates the focused module under `retail-12-0-5`.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs`: invokes the C API registrar after the namespace exists in globals. A single crate-visible `tooltip_for_spell_identifier` bridge dispatches resolved numeric IDs to unchanged shared producers, or unresolved strings to the existing fresh empty Spell DTO builder. Only the tooltip module visibility is widened; child producers/builders remain private.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs`: both old numeric handlers and registrations compile only without `retail-12-0-5`; earlier profile behavior remains unchanged.
- `src/lua_api/globals/missing_surface/tooltip_info/spell.rs`: shared finite spell builder and mount builder, spell catalog first then `world.mounts`; retains numeric unknown identity. Mount description is currently at sparse lines index3.
- `src/lua_api/globals/missing_surface/tooltip_info/builders.rs`: existing fresh empty/identified TooltipData builders and line/color construction; reused later, never copied.
- `data/spells.rs`: real generated 19750 entry at lines52582–52588, name `Flash of Light`; no fake production catalog entry.
- `src/spell_lookup.rs`: existing spell catalog facade; 23338 has no spell row in inspected generated catalog and reaches host mount state.
- `src/lua_api/state_defaults.rs`: actual default mount18 Swift Palomino, spell23338, icon132261, collected/usable true, mount_type230. A later default mount69 shares23338; current first match wins. Tests assert the existing fixture precondition rather than fabricate catalog data.
- `src/lua_api/state_types/collections.rs`: declared `MountData` fields; no new state/producer fixtures or modified model.
- `src/c_api/c_spell.rs`: already crate-visible strict public identifier helper and shared alias-first resolver over existing `spell_id_aliases`; unchanged.
- `src/lua_api/frame/methods/widgets/tooltip/content.rs`: actual setters forward original argument values to the real namespace query, store exposed `processingInfo.tooltipData`, and apply rendered lines. Mount forwards arg2; spell setter currently forwards only arg1.
- `src/lua_api/frame/methods/widgets/tooltip/line_data.rs`: **frame precondition limit**: `GetSpell()` uses `TooltipData.spell_id`, which setters populate only from original numeric inputs. String aliases currently have no GetSpell name/ID; numeric mount23338 names fall back to `Spell 23338`, not the mount title. Thus these fixtures compare actual rendered lines and exposed query DTO, not an invented GetSpell equivalence. No frame patch is authorized here.
- `src/lua_api/methods.rs`: original frames are backed `Val::Table` objects, not presumed Userdata. Tests require actual `GetObjectType()` and existing table backing metadata before wrapping the real frame.

Namespace registration uses the constructor's `GcRef<Table>`, already globally rooted before callback installation. Shared payload constructors return `Val`; handlers push that value immediately before any further allocation. Existing producer allocation/rooting and frame processing behavior are unchanged. No whole-tooltip refactor, copied builders, catalog/state additions, color/frame payload patch or vendor behavior patch.

## Tests asserting this spec

`tests/tooltip_spell_mount_identifiers.rs`: **22 focused tests**, `#[cfg(feature = "retail-12-0-5")]`. Existing `build.rs` discovers top-level test modules into the grouped `integration` target (`tests/integration.rs`); no new Cargo target or harness edit.

| Capability | Fixture cases | Proof level |
| --- | --- | --- |
| Real spell/mount DTO and aliases | Numeric producer guards; both alias DTO families; numeric precedence; link precedence; live changes | Historical numeric controls PASS; initial alias failures observed. Corrected compiled RED pending. |
| Miss/read-only/isolation/strictness | Numeric endpoints; unidentified public string misses; result mutation/freshness; two environments; invalid identifiers | Corrected parent RED: controls PASS; alias/miss/read-only/isolation/strictness failures genuine. GREEN pending. Chosen strictness/string miss inferred. |
| Public flags and taint | Documented ignored optional combinations; secure/ordinary-tainted public DTO equivalence | Corrected public RGBA baseline passes before alias boundary failure. GREEN pending. Ignored flags receive no semantic credit. |
| Authentic secret boundary | All six VM secret kinds for arg1/mount arg2; three paired kind matrices for each spell position; forced GC | Corrected parent compiled RED observed genuine failures; GREEN pending. Optional ordering requires parent source audit, not claimed read-observation instrumentation. |
| Actual frame consumer | Original identifier DTO/rendered-line equivalence; rejected secret inputs retain prior payload and recover | Corrected query-preservation/lineIndex oracle reaches genuine alias boundary failure; GREEN pending. Secret retention test unchanged. GetSpell alias identity and full frame optional semantics excluded. |

GC identity snapshots compare actual host `Val`, userdata GcRef and live allocation sequence before/after returning to the secure host boundary, then compare globally rooted list/stack-export entries. Metadata snapshots do not create VM roots or inspect private payloads. Lua verifies secrecy and original caller/frame/table properties. Pinned rilua6044544 denies tainted secret-BOOL `rawequal`; these fixtures never require it or relax queries. Secret publication roots wrappers during insertion and roots original real tables while wrapping them.

Existing meaningful controls remain in `tests/tooltip_mount.rs`, `tests/tooltip_item_sources.rs` and `tests/tooltip_item_spell.rs`: numeric mount title/ID and frame lines, generated spell identity, and actual tooltip content. They do not establish the new alias/NeverSecret contract. Batch53 `tests/break_up_large_numbers.rs` supplies the authentic host-wrapper/root identity pattern, not a replacement tooltip query.

### Input-stage proof ledger

- Base inspected: `b1d8486563b85e02ec4e2d9787833020e4a95d47`; unrelated dirty source preserved without body access.
- Owned formatter only: `rustfmt --edition 2024 --config skip_children=true tests/tooltip_spell_mount_identifiers.rs`, explicit repository cwd. Result recorded in handoff; no compiler invoked.
- Historical input revision `7ea54c824e3dfd4e1ccaebc0fad6399baa5de45c`: parent `cargo test --test integration --no-run --message-format=json` exited0 in104.26011972106062s; selected22 run exited101 with3PASS/19FAIL in3.852031323942356s. Artifacts: `/tmp/patch-12.0.5-batch56-red-build-result.json`, `/tmp/patch-12.0.5-batch56-red-run.json` and saved full outputs. This is mixed fixture/producer evidence, not genuine corrected RED: public-nil/repeat numeric DTO and numeric frame comparisons failed the old scalar oracle; remaining17 failures include alias input/security/strictness boundaries, not17 established root causes.
- Read-only diagnosis `/tmp/patch-12.0.5-tooltip-dto-equivalence-diagnosis.md` inferred fresh-color identity as a likely fixture cause, not confirmed channel behavior. `builders::color_table` invokes `CreateColor`; naked-environment `color_defaults.rs` creates per-instance methods, while cached `Blizzard_SharedXMLBase/Color.lua` uses `ColorMixin`. Both `GetRGBA()` contracts return `r/g/b/a`; comparator now extracts only these four public numeric components without comparing method identity. No executed corrected fixture proof yet.
- Parent's separate full-UI diagnostic `/tmp/patch-12.0.5-tooltip-dto-diagnostic.lua` and `.stdout/.stderr` reportedly exited0 in5.204861s. Saved output has no repeated numeric spell/mount differences and reports spell-frame additions `lines.1..4.lineIndex`; this does not prove naked-environment color equivalence. Frame oracle now preserves original query fields and validates only correctly positioned lineIndex enrichment; existing rendered-line and secret failure-retention assertions remain.
- Correction `215f0bb84cc86b611dcc2f4288f87754436967b6` changes only semantic public RGBA comparison and equivalent frame fields/lineIndex. Parent corrected compilation exited0 in58.19581049506087s; selected22 exited101 with3PASS/19 genuineFAIL in3.5979648019419983s. Numeric19750, default mount23338 and unknown-numeric controls PASS; both former DTO scalar baseline failures now pass their numeric baseline before actual alias number/string boundary failure. Artifacts: `/tmp/patch-12.0.5-batch56-red-fixed-build-result.json`, `-build.jsonl`, `-build.stderr`, `-run.json`, `-run.stdout`, `-run.stderr`. Executable SHA256 `74043f3c4ada1833904a24f00c16d89250eeeca1f4362f1a3b62a2e616e2cbc1`. Proof includes preserved unowned dirty source hash `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`, not clean-revision proof. Historical uncorrected run is not the producer RED gate.
- Implementation uses that corrected compiled RED; no new tests/build/check/readability/coverage/gates/operations/delegation/model CLI/push were run by the implementer. Owned Rust formatting only; parent must compile the changed producer and observe GREEN/source audit/acceptance. No requirement boxes or four source IDs are promoted.

## Known gaps (current cycle)

Corrected parent compiled RED is recorded above; the two focused C API boundaries are implemented. Fixture preconditions remain explicit and unchanged; no monkey-patching or weakened query assertions.
- [ ] Parent GREEN/source-audit/acceptance; four source rows remain uncredited until accepted proof.
- [ ] Ordinary optional flags are currently ignored: indoor eligibility, pet/subtext/override selection, difficulty and link meanings remain **gaps**, not semantic support.
- [ ] Native alias vocabulary, arg types, permissions for secret arg1, error precedence, miss behavior and result secrecy remain unknown. Native probes are unavailable and not a completion gate for this chosen simulator policy.

## Out of scope

- Item330/331 `GetItemByID` contexts remain separate and pending; no item context model.
- Production state changes and producer work before parent compiled RED.
- Generic string parsing, catalog acquisition, fake catalog entries, mount acquisition, override graph and whole-tooltip refactor.
- New semantics or unrequested public optional type/domain/finite validation; old-profile contract changes.
- GetSpell alias-ID bookkeeping changes, missing frame optional forwarding, sparse mount-line layout redesign and output/native parity claims.
- Vendor patches, accounting promotion, broad gates, builds, test execution, push and native probes in this slice.
