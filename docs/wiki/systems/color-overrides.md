# Retail C_ColorOverrides state

`C_ColorOverrides` provides Retail item-quality colors and color override state. The simulator registers the API in the ordinary Lua environment and stores active overrides per Lua environment. The exact persistence lifecycle remains a neutral seam.

## Content

### API and state

**VERIFIED (Retail 12.1.5.70688, R88v.v1–v3):** the namespace publishes `ClearColorOverrides`, `GetColorForQuality`, `GetColorOverrideInfo`, `GetDefaultColorForQuality`, `RemoveColorOverride`, and `SetColorOverride`. Override types are 0–7. `SetColorOverride` converts RGBA table components to single precision, clamps finite values to `[0, 1]`, rounds them to byte channels, and replaces a matching type or adds a new record. `RemoveColorOverride` removes a matching record; `ClearColorOverrides` empties the active records.

**VERIFIED (simulator source):** the model lives in `src/c_api/c_color_overrides.rs`, uses a private Lua-registry table for the active records, and registers only when `client_profile::ACTIVE` is Retail. The registration does not publish these methods into the secure environment. The two temporary defaults formerly installed by `color_defaults.rs` are removed.

**VERIFIED (Retail 12.1.5.70688, R88v.v2):** `GetColorOverrideInfo` returns one nil when no record matches. A match returns an `overrideType`, an RGB `ColorMixin` table without an `a` field, and an `overrideColorString`. `GetColorForQuality` returns a `ColorMixin` RGBA table, selects a matching override when one exists, and otherwise selects that quality's default color.

**INFERRED (R88v.v1 and the local `ColorOverridesDocumentation.lua` cache, whose exact build is unverified):** the native update callback is published as synchronous `COLOR_OVERRIDE_UPDATED(overrideType)`, and the reset callback is published as synchronous `COLOR_OVERRIDES_RESET()`. The simulator dispatches these event names. The consumer test does not assert this mapping.

### GlobalColor defaults

**VERIFIED (Retail 12.1.5.70688, R88v.v3):** quality values 0–8 select the `GlobalColor` names `ITEM_POOR_COLOR`, `ITEM_STANDARD_COLOR`, `ITEM_GOOD_COLOR`, `ITEM_SUPERIOR_COLOR`, `ITEM_EPIC_COLOR`, `ITEM_LEGENDARY_COLOR`, `ITEM_ARTIFACT_COLOR`, `ITEM_SCALING_STAT_COLOR`, and `ITEM_WOW_TOKEN_COLOR`. A missing lookup returns packed magenta `0xFFFF00FF`.

**INFERRED (Retail 12.1.5.70688 semantics applied to the simulator's generated Retail 12.1.5.70783 rows):** the current model obtains these names from `RETAIL_GLOBAL_COLORS`. The native 70688 evidence does not establish the values of its nine rows, and the 70783 snapshot is not proof of the live 70688 catalog.

**VERIFIED (Retail integration tests at implementation commit `2d1a14c8049ae890fdb28fdfc1752de88d3409b3`):** the selected original `ColorManager` consumer returns the tested Poor and Epic defaults and uses two supplied overrides; the target tests pass. The lookup uses the shared packed-color channel conversion in [`c_ui_color.rs`](../../src/c_api/c_ui_color.rs). See the [GlobalColor catalog](global-color-catalog.md) for row provenance and version limits.

### Persistence boundary

**VERIFIED (Retail 12.1.5.70688, R88v.v3):** native active records are in-memory state. A separate save/load path uses the account setting `playerColorOverrides` and serializes override type and RGBA components there.

**UNKNOWN (Retail 12.1.5.70688):** the complete ReloadUI/logout chain, final storage step, and failed-save behavior are not established. The simulator does not connect the active registry to a guessed serializer or lifecycle trigger. Do not treat the model as persisted across a new Lua environment.

### Consumer selection

**VERIFIED (Retail cache and simulator runtime):** `Blizzard_Colors.toc` permits both screens and selects `[Family]\\ColorManager.lua`; Retail selects the `mainline` game type. The Retail manifest includes the TOC and `Mainline/ColorManager.lua`. The selected original `ColorManager` calls `GetColorForQuality` at line 23 and `GetDefaultColorForQuality` at line 66. The full-environment regression test confirms that `ColorManager` is loaded and exercises those consumers.

## Sources

- [Retail C_ColorOverrides contract](../../specs/color-overrides.md) — tested requirements and current gaps.
- [GlobalColor catalog](global-color-catalog.md) — source table and version boundary.
- [Consumer selection and RED proof](../investigations/consumer-selection-and-red-proof.md#retail-colormanager-and-c-coloroverrides) — TOC, profile selection, and original consumer proof.
- Local native evidence: `pr46-49-decomp-answers.md`, section `R88v`, Retail 12.1.5.70688.

## See Also

- [[global-color-catalog]] — generated GlobalColor source and native export behavior.
- [[lua-api]] — C API registration boundary.
- [[client-profiles]] — profile-specific runtime selection.
