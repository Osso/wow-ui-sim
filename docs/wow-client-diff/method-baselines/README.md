# Configuration-specific method baselines

Byte-for-byte copies of reviewed `complete-diff_methods_{extra,missing}.txt` files from [comparison evidence](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/method-profile-comparisons/), under the corresponding source `<profile>/` directory. Counts are diff rows relative to the frozen native inventory whose epoch, build, and product are unknown; these are not normative native oracles or historical Retail fixtures.

| Profile | Actual epoch | Forbidden | OnUpdate | Extra rows | Missing rows |
|---|---|---:|---:|---:|---:|
| retail | retail-12-1-5 | 1 | 1 | 12413 | 33 |
| ptr | retail-12-1-5 | 1 | 1 | 12412 | 33 |
| wrath | no-retail-epoch | 0 | 0 | 11633 | 38 |
| mists | no-retail-epoch | 0 | 0 | 11633 | 38 |
| era | no-retail-epoch | 0 | 0 | 11633 | 38 |
| anniversary | no-retail-epoch | 0 | 0 | 11633 | 38 |
| wowforever | no-retail-epoch | 1 | 1 | 11825 | 38 |

Layout: `<profile>/<actual-epoch>/forbidden-<0or1>-on-update-<0or1>/diff_methods_{extra,missing}.txt`.

PROVENANCE: Retail 12.1.5 baseline generated twice on 2026-11-03 from build `fef520c5b` with `integration method_diff_coverage::refresh_method_diff_files --exact --ignored --nocapture`. Both runs were byte-identical. Against the root Retail 12.1.0 baseline, the extra set adds 32 rounding rows and `MessageFrame:RemoveMessagesByAccessID`; the missing set is unchanged. Rounding is declared by pinned `Blizzard_APIDocumentationGenerated/SimpleScriptRegionAPIDocumentation.lua:202,653`. The message method remains on MessageFrame by the tested simulator contract; the frozen discovery is not a native oracle.

Configuration-specific selection follows the user-approved policy; these files retain reviewed simulator comparison results, not native catalogs. Selection is for the current explicit configuration, not automatic execution of the full matrix. The root [extra](../diff_methods_extra.txt) and [missing](../diff_methods_missing.txt) files retain the Retail 12.1.0 baseline. Frozen WowDiscovery remains unchanged.
