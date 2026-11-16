# Retail C_ColorOverrides state

`C_ColorOverrides` provides Retail item-quality colors and active override state. The ordinary Retail namespace uses the modeled six-method API; explicit save/load seams use `playerColorOverrides`, but no automatic lifecycle trigger is connected.

## Content

### API and state

**VERIFIED (Retail 12.1.5.70688, R88v.v1–v3):** the namespace publishes `ClearColorOverrides`, `GetColorForQuality`, `GetColorOverrideInfo`, `GetDefaultColorForQuality`, `RemoveColorOverride`, and `SetColorOverride`. Override types are 0–7. `SetColorOverride` converts RGBA table components to single precision, clamps finite values to `[0, 1]`, rounds them to byte channels, and replaces a matching type or adds a new record. `RemoveColorOverride` removes a matching record; `ClearColorOverrides` empties the active records.

**VERIFIED (simulator source and test):** the model lives in `src/c_api/c_color_overrides.rs`, stores active records in a private Lua-registry table, and registers only when `client_profile::ACTIVE` is Retail. `color_defaults.rs` retains `GetColorOverrideInfo` and `GetColorForQuality` before secure-environment creation. The secure snapshot's member set matches the two-member `origin/master` baseline at `ffc9675274b9cc21922daea9fa901c3efd1723ee`; Retail then replaces only the ordinary namespace with the six modeled methods. The test checks member names, not secure method behavior or flag-3 publication.

**VERIFIED (Retail 12.1.5.70688, R88v.v2):** `GetColorOverrideInfo` returns one nil when no record matches. A match returns an `overrideType`, an RGB `ColorMixin` table without an `a` field, and an `overrideColorString`. `GetColorForQuality` returns a `ColorMixin` RGBA table, selects a matching override when one exists, and otherwise selects that quality's default color.

**INFERRED (R88v.v1 and the local `ColorOverridesDocumentation.lua` cache, whose exact build is unverified):** the native update callback is published as synchronous `COLOR_OVERRIDE_UPDATED(overrideType)`, and the reset callback is published as synchronous `COLOR_OVERRIDES_RESET()`. The simulator dispatches these event names. The tests do not assert this mapping.

### GlobalColor defaults

**VERIFIED (Retail 12.1.5.70688, R88v.v3):** quality values 0–8 select the `GlobalColor` names `ITEM_POOR_COLOR`, `ITEM_STANDARD_COLOR`, `ITEM_GOOD_COLOR`, `ITEM_SUPERIOR_COLOR`, `ITEM_EPIC_COLOR`, `ITEM_LEGENDARY_COLOR`, `ITEM_ARTIFACT_COLOR`, `ITEM_SCALING_STAT_COLOR`, and `ITEM_WOW_TOKEN_COLOR`. A missing lookup returns packed magenta `0xFFFF00FF`.

**INFERRED (Retail 12.1.5.70688 semantics applied to the simulator's generated Retail 12.1.5.70783 rows):** the current model obtains these names from `RETAIL_GLOBAL_COLORS`. The native 70688 evidence does not establish the values of its nine rows, and the 70783 snapshot is not proof of the live 70688 catalog.

**VERIFIED (Retail integration tests at revision-1 source `779aacb1161979d930100e89a4daf8feb9b85e96`):** the selected original `ColorManager` consumer returns the tested Poor and Epic defaults and uses two supplied overrides. The original bag-quality consumer uses its default without an override. Exact selectors pass; receipts are in the revision-1 handoff report. The lookup uses shared packed-color channel conversion in [`c_ui_color.rs`](../../src/c_api/c_ui_color.rs). See the [GlobalColor catalog](global-color-catalog.md) for row provenance and version limits.

### Persistence boundary

**VERIFIED (Retail 12.1.5.70688, R88v.v3):** native active records are in-memory state. A separate save/load path uses the account setting `playerColorOverrides` and saves override type and RGBA components there.

**VERIFIED (simulator source and round-trip test):** crate-level `save_color_overrides` and `load_color_overrides` seams read and write `playerColorOverrides`. Loading clears current records before it restores the saved records. The test saves two distinct types, starts a new Lua environment on the same temporary session path, adds a stale type, loads, and checks the restored RGBA values and stale-type removal.

**INFERRED:** the simulator serializes rows as JSON byte arrays `[overrideType, red, green, blue, alpha]`; the test asserts round-trip effects, not the stored encoding. Automatic save/load triggers remain unconnected.

**UNKNOWN (Retail 12.1.5.70688):** the complete ReloadUI/logout chain, final storage step, and failed-save behavior are not established. Do not treat the model as persisted across a new Lua environment unless a caller invokes the explicit load seam.

### Consumer selection and RED correction

**VERIFIED (Retail cache and simulator runtime):** `Blizzard_Colors.toc` permits both screens and selects `[Family]\\ColorManager.lua`; Retail selects the `mainline` game type. The Retail manifest includes the TOC and `Mainline/ColorManager.lua`. The full-environment regression test confirms that `ColorManager` is loaded.

**CORRECTION (VERIFIED):** the initial RED at test commit `589d1278958b6d14867abba6d6dfe6736cb91378` called `GetDefaultColorForQuality` directly before the original consumer. It did not prove a ColorManager failure. The corrected RED at test-only commit `9f94060da3c33f8dc32b8474cc51da5301bb5600` removes that ordinary method through a test adapter before Blizzard_Colors loads; the original `ColorManager.lua:66` call then fails on the missing method. The test source blob is unchanged at GREEN (`c973568cd3c22fbecf56bafc2fa26b30e47add34`).

**VERIFIED (secure snapshot RED/GREEN):** the unchanged test body at test commit `1f0c2605b407b9024e758bcecd812c98f3b45858` failed with an empty secure member set against the expected master members `GetColorForQuality` and `GetColorOverrideInfo`. After the pre-snapshot defaults were restored, the same test passed at the revision-1 source. Its test-body SHA-256 is `a971afd72d030ce1d519cfc5e0e8f5dc000fcd0462ed46aa7b3b682e7e56fa21`.

## Sources

- [Retail C_ColorOverrides contract](../../specs/color-overrides.md) — tested requirements and current gaps.
- [GlobalColor catalog](global-color-catalog.md) — source table and version boundary.
- [Consumer selection and RED proof](../investigations/consumer-selection-and-red-proof.md#retail-colormanager-and-c-coloroverrides) — TOC, profile selection, and original consumer proof.
- Simulator source: [C_ColorOverrides model](../../../src/c_api/c_color_overrides.rs) and [pre-secure color defaults](../../../src/lua_api/workarounds/temporary/color_defaults.rs).
- Retail API documentation cache: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ColorOverridesDocumentation.lua`.
- Local native evidence: `pr46-49-decomp-answers.md`, section `R88v`, Retail 12.1.5.70688.

## See Also

- [[global-color-catalog]] — generated GlobalColor source and native export behavior.
- [[lua-api]] — C API registration boundary.
- [[client-profiles]] — profile-specific runtime selection.
