# Method registration gates — independent source/artifact verification

Verified: 2026-10-09. **Reports require corrections; source wiring verified, no runtime/native acceptance.** Followed `/home/osso/AgentConfig/skills/verify/SKILL.md` as the independent verifier. No delegation, backend, build, test, snapshot refresh, source edit, service/process operation, or deployment. Only this requested report is written.

## Exact evidence epochs

- Initial checkout HEAD: `9cf08121af8f0167d5ea4646ba3b4e21e6cabdcf`; status showed only pre-existing untracked `.code-index.db`. Later observed HEAD: `903711da8b9d6472f718e407af350c576fe4a7e0` (commit timestamp `2026-10-09T22:06:55-05:00`). Read-only Git diff between those revisions returned exit 0, empty diff for Cargo, client-profile guards, frame initialization, method attachment/filter, all frame-method modules, and method-diff test. Source findings are scoped to those inspected revisions, not a floating future HEAD.
- Historical full-suite artifact: `96e88494a6844833b79a574d37a2115dc03b44eb`, commit timestamp `2026-10-09T15:56:12-05:00`; saved metadata starts `2026-10-09T16:02:03-0500`, finishes `2026-10-09T16:54:27-0500`. Evidence: `/home/osso/.local/state/wow-ui-sim/verification/fullsuite-96-comparison/{data.json,failure-diagnostics.json,selected-evidence.txt}`. Saved log SHA-256 metadata: `96a9874d9a712839f7d631b666905b552d074c5c8232f22dad026b794a3a5e20`. This audit parsed retained diagnostic text, not a new execution or independently rehashed original full log.
- Same-epoch source check: Git object reads at revision 96 and empty `git diff 96… HEAD -- <file>` independently establish unchanged Cargo feature definitions, `client_profile.rs`, frame initialization/attachment/filter, all registrar files used below, and `tests/method_diff_coverage.rs`. This is source equality, not proof the historical executable was rebuilt from those bytes.
- Handoff reports contain no pinned source revision. Their present bytes, not an inferred authoring epoch, are audited. SHA-256:
  - `extra-text-gates-current.md`: `01d33e61ddf1362a35480d9933e785a32731fded8f8f4e7c5ed23f7813976844`
  - corrected `extra-cooldown-gates-current.md`: `e1a8913f4771b28b5f5c8c1a2d979d28333f2d88451164d88d16618c8abfa5ff`
  - `extra-model-gates-current.md`: `25d61f980e4b6cae1d3ca1c82d9a26c60b6e97b22d2749093140a11524790a30`
  - `metatable-filter-root-current.md`: `f70981bdcbcf5b8eae2ec9e16016684d01d8cd7d1074659ad295fa829b8cea2a`

## Accepted/rejected handoff claims

| Report | Accepted | Rejected / correction |
|---|---|---|
| extra-text | Literal Retail patch cfgs; parent roleset gate plus method-level Retail gate; cited wikitext publication rows. | **Forever does enable `forbidden-aspects`**, through `client-wowforever → forbidden-animation-aspects → forbidden-aspects`. Thus its three access-restriction registrations are present in the standard Forever bundle. Reject the summary that Forever exposes none of these twelve. All text registrations use the common table; FontString/Texture labels are logical families or source-document receiver labels, not type-specific metatable publication filters. |
| extra-cooldown, after Main correction | All ten source profile entries. Formatter gate is specifically `retail-12-0-5`; three threshold entries are unconditional; radial four and resize require `retail-12-1-0`. Shared publication, not Cooldown-only/Animation-only. | No remaining receiver-scope correction needed in the audited corrected bytes. Family names do not prove accepted invocation receiver or native support. |
| extra-model | `AddSecretAspect` uses shared forbidden capability; Forever transitive feature chain; six model Retail gates; unconditional movie publication and explicit MovieFrame receiver check. Permanent 3D skip/false bodies are compatibility entries, not native 3D implementation. | **`CanBeAccessedInContext` is not Forever-only overall.** Report inspected only `misc/secret.rs:34–40`, missing the second registrar `text_attribute_event/access_restrictions.rs:33–38` under `forbidden-aspects`. Retail/PTR and Forever publish it. On Forever, later text registration overwrites the earlier misc registration. |
| metatable-filter-root | Shared base table, cached per-type shallow clones, explicit exclusions rather than positive allowlist, no `is_method_allowed` occurrence in current `src`; diff test compares keys, not invocations; capture absence alone does not establish native absence. | **11,985 versus 491 is not a discrepancy:** baseline size versus historical additions delta. The retained diagnostics independently establish the exact 491 distribution. **“Introspection-only, not runtime-dispatch gate” is unsupported and contradicted by attachment source:** the filtered clone is the actual frame table metatable, and ordinary method access uses its `__index`. Registration remains shared, but hiding a key affects ordinary lookup on that attached table. No runtime test was run; no assertion about every possible alternate dispatch path. |

## Enclosing register and filter chains

All paths below are relative to `/home/osso/Projects/wow/wow-ui-sim`.

1. `src/lua_api/frame/mod.rs:3` and `frame/methods/mod.rs:1–13` expose the method modules without a profile gate around these registrars. `env_init/frames.rs:46–61` creates one base frame metatable and registers core, misc, text, button/anchor/hierarchy and widgets on the same `frame_mt_ref`. In particular, **misc runs before text**. `:72–92` clones the resulting index and installs it on the base table.
2. Misc chain: `frame/methods/misc/mod.rs:30–45 → alpha_gradient::register / secret::register`. Text chain: `text_attribute_event/mod.rs:32–47 → register_text_methods` (`:189–194 → register_text_layout / register_styled_text`, `:232–236 → measurement/scaling`, `:313–317 → colors`), rolesets (`:36–40`), forbidden methods (`:44–45 → :57–59 → access_restrictions::register`). The access module declaration itself is `#[cfg(feature = "forbidden-aspects")]` at `:6–7`.
3. Widget chain: `widgets/mod.rs:13–26` declares cooldown/model/movie/texture modules; `:85–102 → texture::register_texture / cooldown::register_cooldown / model::register_model / movie::register_movie`. Their arrays/entries populate the same argument table. Radial chain: `button_anchor_hierarchy/mod.rs:721–728 → register_animations`, `:336 → register_animation_config`, `:581–604 → RADIAL_PROGRESS_METHODS`. Resize chain: `core_state/mod.rs:36–37 → register_size`, `:59–60`.
4. Attachment/filter: `src/lua_api/methods.rs:296–317` obtains WidgetType, gets the type metatable and attaches it to the frame table; comments explicitly state methods are accessed through `__index`. `methods/frame_metatable.rs:10–40` caches/clones (WorldFrame uses base); `:51–88` names the ScrollFrame/MessageFrame/StatusBar exclusions; `:91–128` installs a filtered index clone. **None of the 31 audited names is excluded.** Family registration does not imply family-restricted publication.

## Current-source profile matrix — all 31 named rows

Columns mean standard Cargo bundles only: R = default `client-retail` (Retail API 12.1.0); P = `client-ptr` (12.1.5); C = each of Wrath/Mists/Era/Anniversary; F = `client-wowforever`. Y/N are source registration results, not measured runtime or native acceptance. Historical count is additions to the extra snapshot in the retained revision-96 diagnostic, **not current exposure count**.

Paths abbreviated relative to `src/lua_api/frame/methods/`: T=`text_attribute_event`, M=`misc`, W=`widgets`, B=`button_anchor_hierarchy`, C0=`core_state`.

| Name | R | P | C | F | Effective enclosing gate / registrar | Historical extra additions |
|---|---|---|---|---|---|---:|
| AddAccessRestrictions | Y | Y | N | Y | forbidden-aspects; T/mod.rs:6,44,57–59 → T/access_restrictions.rs:18 | 16 |
| GetAccessRestrictions | Y | Y | N | Y | same chain → T/access_restrictions.rs:24 | 16 |
| HasAnyAccessRestrictions | Y | Y | N | Y | same chain → T/access_restrictions.rs:30 | 16 |
| IsRolesetFiltered | Y | Y | N | N | parent cfg!(Retail12.1.0 or Forever), entry Retail12.1.0; T/mod.rs:36–40 → T/rolesets.rs:20–21 | 16 |
| GetAlphaGradient | Y | Y | N | N | Retail12.0.0; M/alpha_gradient.rs:14–15 | 15 |
| GetScaleAnimationMode | Y | Y | N | N | Retail12.0.0; T/mod.rs:286–300 | 15 |
| SetScaleAnimationMode | Y | Y | N | N | Retail12.0.0; T/mod.rs:286–300 | 15 |
| GetSmoothScaling | Y | Y | N | N | Retail12.0.5; T/mod.rs:301–305 | 16 |
| SetSmoothScaling | Y | Y | N | N | Retail12.0.5; T/mod.rs:301–305 | 16 |
| GetUnboundedStringWidthForText | Y | Y | N | N | Retail12.0.5; T/mod.rs:256–263 | 16 |
| SetDesaturateEmbeddedTextures | Y | Y | N | N | Retail12.1.0; T/mod.rs:324–331 | 16 |
| SetVertexColorFromBoolean | Y | Y | N | N | Retail12.0.0; W/texture/mod.rs:47–51 → :171–174 | 14 |
| GetCountdownFormatter | Y | Y | N | N | Retail12.0.5; W/cooldown.rs:761–762 → :807–811 | 16 |
| SetCountdownFormatter | Y | Y | N | N | Retail12.0.5; W/cooldown.rs:759–760 → :807–811 | 16 |
| GetCountdownAbbrevThreshold | Y | Y | Y | Y | unconditional entry; W/cooldown.rs:776–779 → :807–811 | 16 |
| GetCountdownMillisecondsThreshold | Y | Y | Y | Y | unconditional entry; W/cooldown.rs:784–787 → :807–811 | 16 |
| SetCountdownMillisecondsThreshold | Y | Y | Y | Y | unconditional entry; W/cooldown.rs:780–783 → :807–811 | 16 |
| GetFromPercent | Y | Y | N | N | Retail12.1.0 call and array; B/mod.rs:581–604 | 16 |
| GetToPercent | Y | Y | N | N | same; B/mod.rs:581–604 | 16 |
| SetFromPercent | Y | Y | N | N | same; B/mod.rs:581–604 | 16 |
| SetToPercent | Y | Y | N | N | same; B/mod.rs:581–604 | 16 |
| ResizeToBoundsRect | Y | Y | N | N | Retail12.1.0; C0/mod.rs:36–37,54–60 | 16 |
| AddSecretAspect | Y | Y | N | Y | forbidden-aspects; M/secret.rs:20–21 | 16 |
| CanBeAccessedInContext | Y | Y | N | Y | forbidden-aspects via T/access_restrictions.rs:33–38; also Forever entry M/secret.rs:34–40, earlier/overwritten | 16 |
| EnableSubtitles | Y | Y | Y | Y | unconditional; W/movie.rs:33–35; receiver check :21–28 is separate | 16 |
| GetModelUnitGUID | Y | Y | N | N | Retail12.0.7 module and array; W/model.rs:10–11,653–654 → :808–811 | 16 |
| IsPreferringModelCollisionBounds | Y | Y | N | N | Retail12.0.0 block; W/model.rs:812–827 | 16 |
| SetPreferModelCollisionBounds | Y | Y | N | N | same; W/model.rs:812–827 | 16 |
| SetGradientMaskWithDyes | Y | Y | N | N | Retail12.0.5 array entry; W/model.rs:711–712 → :808–811 | 16 |
| SetSheathedCategory | Y | Y | N | N | Retail12.0.5 array entry; W/model.rs:713–714 → :808–811 | 16 |
| UseUnitSheatheCategory | Y | Y | N | N | Retail12.0.5 array entry; W/model.rs:715–716 → :808–811 | 16 |

Matrix totals: R31, P31, each C4, F9 named registrations. Explicitly enabled optional shared capabilities can change a nonRetail bundle's rows; these totals do not describe arbitrary custom feature combinations.

### Feature-chain falsification and historical apparent conflict

`Cargo.toml:118–122` makes Retail epochs cumulative. `:140–141` defines `forbidden-animation-aspects = ["forbidden-aspects"]`; `:155` enables forbidden-animation-aspects on Forever. `:149–154` assigns standard Retail/PTR/classic bundles. Thus the extra-text report's claim that Forever lacks forbidden-aspects is false.

At revision 96, `src/client_profile.rs:247–261` rejects Forever combined with another profile marker; `:279–289` rejects a Retail epoch unless profile-retail or client-ptr is enabled. Hence a **genuinely Forever-only registration** cannot coexist with Retail-only registrations in one valid ordinary build. But the alleged conflict for `CanBeAccessedInContext` is resolved by the **same-revision second registrar**: `git show 96…:src/lua_api/frame/methods/text_attribute_event/access_restrictions.rs` contains that name at :36, reachable under shared forbidden-aspects. These files and Cargo/profile guards are unchanged at the inspected current epochs.

Consequently the revision-96 observed 16 rows for CanBeAccessedInContext can coexist with Retail-only method observations without violating mutually exclusive profile guards. It is an attribution error in the handoff, not evidence that the run combined Forever and Retail. No build provenance or profile is inferred merely from these rows. Saved selected-evidence.txt:1 records integration command `cargo nextest run --test integration --no-fail-fast --test-threads 16 --offline --locked`; no explicit feature override is shown there. That alone does not prove the historical executable's feature set.

## Historical delta versus committed baseline

Parsed the entire two method-failure strings from retained failure-diagnostics.json. Extra failure spans original-log lines **6973–7565**:

- **96 stale removals**: six minimap setters, each across 16 types (`SetBlipTexture`, `SetCorpsePOIArrowTexture`, `SetIconTexture`, `SetPOIArrowTexture`, `SetPlayerTexture`, `SetStaticPOIArrowTexture`).
- **491 additions / 31 distinct names**: 27×16 + 3×15 + 1×14 = 491. The three 15-row names are GetAlphaGradient/GetScaleAnimationMode/SetScaleAnimationMode; the 14-row name is SetVertexColorFromBoolean. Remaining 27 names have 16 each, as tabulated above.
- **11,985 nonblank baseline rows across 16 types**: independently counted current `docs/wow-client-diff/diff_methods_extra.txt`; Git-object read at revision 96 has identical bytes and row count.
- Delta-applied historical extra set size is **12,380** (=11,985−96+491); this is set reconstruction from retained diagnostics, not a new runtime snapshot. The 587 diagnostic entries are removals plus additions, not the total extra surface.

Missing failure spans lines **7594–7620**: 5 stale removals and 16 additions (SetPreventSecretValues on 16 types). This is a separate missing-snapshot delta. In same-epoch `misc/secret.rs:41–47`, SetPreventSecretValues is gated by **not Retail12.0.5**; its absence in the 12.1.0 standard bundle is consistent with missing rows, not an extra registration.

`tests/method_diff_coverage.rs:7–48` enumerates metatable index keys; `:185–220` computes actual-minus-discovered extra and discovered-minus-actual missing; `:242–264` computes diagnostic additions/removals against committed snapshots. The 491 counts are **new extra-diff entries**, not all present keys, not unique APIs, not callable-method tests, and not current-source exposure counts. Discovery can already include a name on some types, explaining 15/14 rather than 16 extra additions without requiring a current per-type filter.

## Source-publication evidence

Verified exact `entries[].symbol`, `direction = added`, and `wikitext_line` in checked JSON source registers (wikitext line metadata, not JSON physical lines):

| Register | Verified names/metadata |
|---|---|
| sources/11.2.0-wikitext-register.json | FontString:GetAlphaGradient, line161 |
| sources/12.0.0-wikitext-register.json | Region:SetVertexColorFromBoolean694; FontString:GetScaleAnimationMode695, SetScaleAnimationMode696 |
| sources/12.0.5-wikitext-register.json | FontString:GetSmoothScaling735, GetUnboundedStringWidthForText736, SetSmoothScaling737; Cooldown:GetCountdownAbbrevThreshold741, GetCountdownFormatter742, GetCountdownMillisecondsThreshold743, SetCountdownFormatter744, SetCountdownMillisecondsThreshold745 |
| sources/12.1.0-wikitext-register.json | FrameScriptObject:AddAccessRestrictions1117, GetAccessRestrictions1121, HasAnyAccessRestrictions1126; FontString:SetDesaturateEmbeddedTextures1128; RadialProgress:GetFromPercent1142, GetToPercent1143, SetFromPercent1144, SetToPercent1145; Frame:IsRolesetFiltered1150, ResizeToBoundsRect1152 |

A whole-name scan of top-level `data/patch-api/*.json` matched GetModelUnitGUID in `12.0.7.json` and none of the other eight model-report names. These checked publication artifacts do not prove a native API contract, client presence, accepted receiver or behavior. This audit does not assign the checked Discovery capture a verified build/profile: README states collection on **2026-02-23**, but that is documentary provenance, not independently validated native capture provenance. The README's old aggregate method counts are not this audit's baseline count authority.

## Verification checklist and limits

- **EXIST — PASS:** all four handoff reports and cited enclosing source files exist. One report mis-cites `frame/methods/frame_metatable.rs`; actual file is `src/lua_api/methods/frame_metatable.rs`.
- **SUBSTANTIVE — MIXED:** reports contain concrete registrations/source references, but the three corrections above prevent retaining their broad conclusions unchanged. Permanent model skip/false compatibility handlers were inspected and are not counted as implemented 3D behavior.
- **WIRED — PASS, source only:** traced every one of 31 names through enclosing gates/common registration and final index filtering; checked both registrars for CanBeAccessedInContext. All 31 survive the named filter when their feature gate registers them.
- **ANTI-PATTERN — NOT APPLICABLE to new code:** no implementation changes were requested or made. Existing permanent 3D handlers are explicit unsupported-domain entries, not new placeholder findings.

**No native or current runtime acceptance.** Historical observations remain attached to revision 96; current matrix is source evidence. No snapshot correction, filter redesign, compatibility fix, or parent-goal closure is authorized by this report.
