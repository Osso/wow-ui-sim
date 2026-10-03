# Read-only verification — 92e4ea045

Scope: commit-pinned source inspected through `git show`; no repository edits, builds, tests, git mutations, agents or model CLIs. Main's reported 6/6 tooltip and 5/5 identity results are supplied evidence, not independently rerun proof.

## 1. Tooltip secrecy — ACCEPT WITH QUALIFICATIONS

Exact retained source verified: `data/patch-api/sources/12.0.5-api-changes.txt:75`: “Text lines in tooltips that contain secret text will no longer remain secret permanently.” Line 136: “Fixing a bug that would cause tooltips to get marked secret incorrectly when textures were added to them.” Distinct claims.

The five pre-existing passing fixtures are not constant-return/vacuous secrecy tests: authentic VM-wrapped strings, real per-frame `secret_text`, tainted caller denial and secure-readable content form positive controls. They establish bounded non-interference of texture insertion with existing SetText-origin restrictions, not new implementation in this commit, aggregate tooltip secrecy, AddLine secret ingestion or historical 12.0.5-only coverage. Wrapper decoding requires `forbidden-aspects` (`secret_origin.rs:17–23`); tests require that feature (`tests/tooltip_texture_secrecy.rs:3`).

Actual new repair: `line_data.rs:19–50` clears pooled child text and secret origin before OnTooltipCleared; pooled same-object reuse asserted at `tests/tooltip_texture_secrecy.rs:173–201`. General AddLine/DTO secrecy remains unmodeled.

### Tooltip earned coverage and regression risk

`docs/specs/tooltip-texture-secrecy.md:9–13,17–19` are supported for the bounded feature-enabled fixtures: tests at `tests/tooltip_texture_secrecy.rs:53–172` cover insertion, positive/negative secrecy controls, independent instances and replacement; `:173–201` covers actual same-FontString reuse after ClearLines. This grants bounded existing non-interference for row 136 and an actual cached-line lifetime fix for row 75, not whole-row historical/native acceptance. Line 47's source-observed defect is repaired. Line 46 execution is supported only by main's supplied 6/6 result; lines 48–49 remain gaps.

Clearing the text together with its origin is necessary: resetting only `secret_text` would expose old private bytes. Observable compatibility change: a retained cached FontString reference now returns nil after ClearLines instead of old content (or addon denial), before repopulation. `line_data.rs:35–38` performs cleanup before the existing callback. No fixture directly checks immediate post-clear cached GetText, right-side reuse, callback-visible contents or geometry; six tests do not prove those contracts. No concrete affected caller found. A bounded cached-retail scan found no ClearLines followed within 11 lines by GetText; this is not an exhaustive dataflow proof.

Cached retail handlers inspected: `Blizzard_UIWidgets/Blizzard_UIWidgetTemplateBase.lua:1573–1581` repopulates an embedded tooltip using SetOwner/SetItemByID, without reading old line text. `Blizzard_DebugTools/Blizzard_DebugTools.lua:271–272` forwards a callback event; no subscriber to that event was found elsewhere in the cached Lua scan. These sources show no old-content dependency, not loaded-Blizzard execution proof.

Geometry: cached text clearing marks children visually dirty (`src/widget/registry/mod.rs:212–215`), not a new sizing policy. Tooltip geometry remains derived from `TooltipData.lines` (`tooltip/sizing.rs:18–55`, `src/iced_app/tooltip.rs:68–135`), already emptied by ClearLines. The empty-content path retains existing dimensions and marks the tooltip rect dirty. Child text/measurement readouts can change as expected when content disappears; rendering/native pooling parity is untested.

The two supplied pre-existing failures have no demonstrated causal link:

- `tests/tooltip_item_context.rs:609–628` calls C_TooltipInfo.GetItemByID through secret-argument assertions, not GameTooltip/ClearLines. Its producer authenticates VM wrappers then validates/selects a DTO (`src/c_api/c_tooltip_info_item_context.rs:22–45`); this commit does not change that producer or VM authentication.
- `tests/tooltip_text_layout.rs:145–181` uses SetOwner/AddLine and viewport layout; it never calls ClearLines. SetOwner fires the existing callback but does not invoke the new cleanup (`tooltip/owner.rs:16–28`). No cached-line secrecy is involved; the measured text/anchor paths are unchanged.

Main's report that both fail with cleanup reverted is consistent with that source separation. Their underlying causes are not established by this review.

Documentation defect: `docs/specs/tooltip-texture-secrecy.md:3,46–47` still describes unexecuted/unapplied work after the producer landed. Update acceptance wording without closing historical or aggregate-secrecy gaps.

## 2. Reverse GUID identity lookup — ACCEPT WITH QUALIFICATIONS

`unit_misc.rs:203–217` suppresses existing active party mappings from host-owned `identity_secret_guids`; initializer is empty (`state.rs:216–220`), storage is environment-local (`sim_state.rs:234–239`). Five fixtures exercise two actual party mappings, host classification/removal, one public nil, secure/addon trust preservation, inactive/unknown controls and environment isolation. This is state-backed behavior, not unconditional nil.

The source names six categories at lines 48/84: arena, nameplate, boss, party, raid and target-of-target. Only party in UnitTokenFromGUID is modeled: one of six categories for one named API, with unspecified “various others” uncounted. PARTIAL is honest; no meaningful whole-sentence percentage can be assigned. Player/target/focus resolve before the party guard (`unit_misc.rs:220–233`); a classified party GUID also exposed as target/focus can still return that alias. This does not violate the bounded party-token policy, but does not conceal identity globally.

Earned behavioral checkboxes: `docs/specs/unit-token-identity-secrecy.md:7–10`; line 11 earns empty-default/environment-local/player controls only. Target/focus alias priority remains unchanged by inspection, but no fixture exercises their overlap with classified party identities. Lines 33–36 remain open. Line 32 combines implemented edits with build/RED evidence not independently available here. Stale documentation: inventory lines 21–24 still calls landed state/guard “proposed” and “not applied.”

## 3. Catalog getter ungating — ACCEPT WITH QUALIFICATIONS

Unconditional module `src/c_api/mod.rs:43`; unconditional registration `missing_surface.rs:247–249`, reached through `register.rs:224` and `missing_surface.rs:79–85,225–227`; unconditional state field `sim_state.rs:335` and initializer `state.rs:302`. The registrar publishes both getters (`c_catalog_shop_products.rs:128–137`). Thus source wiring includes profiles without retail-12-0-5.

All 480 module lines inspected: typed local DTOs, std HashMap, ungated shared helpers, Lua bridge and rilua; no references to a 12.0.5-only enum/type/module/helper. Getters clone the selected host record or return one nil (`:156–189`), with fresh nested serializers; no synthetic product fallback. Existing tests assert concrete stored DTOs and mutations, not wrapped constants.

No behavioral proof of the newly included profiles: `tests/catalog_shop_product_structures.rs:1` still gates every fixture behind retail-12-0-5. Accept structural ungating, not cross-profile build/runtime parity. Existing bounded requirements at `docs/specs/catalog-shop-product-structures.md:9–16,20–21,51` have substantive source and behavioral fixtures; they do not prove old-profile execution. Line 22 remains unchecked: malformed public selectors are tested (`tests/catalog_shop_product_structures.rs:525–538`), authentic secret selectors are not. Restrictions/purchases/panels remain outside credit. No concrete new dependency defect found.

## Artifact verification and merge risk

[EXIST] PASS — commit-pinned files retrieved successfully with git show: tooltip implementation 452 lines/test 201/spec 56; identity producer 449/test 135/spec 40; catalog DTO module 480/test 539/spec 57, plus registration/state dependencies.

[SUBSTANTIVE] PASS for bounded slices — authentic per-frame secret-origin restrictions and cleanup; mutable host identity classification; stored typed product/display maps and serializers. No whole-prose/native-parity completion granted.

[WIRED] PASS — tooltip method table `src/lua_api/frame/methods/widgets/tooltip.rs:21–28`; UnitTokenFromGUID registration `src/lua_api/globals/unit_misc.rs:432`; catalog registration chain cited above.

[ANTI-PATTERN] PASS for inspected new in-scope code — zero TODO/FIXME/HACK/XXX/except-pass markers; no new empty catch or commented-out implementation observed.

OVERALL: ACCEPT WITH QUALIFICATIONS. No confirmed new code defect in the three inspected slices. Merge risk: observable cached-line post-clear readout changes lack direct callback/right-line/native proof; newly enabled catalog profiles lack executed coverage. Identity must remain PARTIAL; tooltip rows must remain bounded to the tested secret-origin path. Spec prose is stale at the cited locations. No repository files modified and no tests/builds run.
