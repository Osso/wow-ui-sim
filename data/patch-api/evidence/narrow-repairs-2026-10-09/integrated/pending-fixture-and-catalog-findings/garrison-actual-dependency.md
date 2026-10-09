# Garrison damage-class catalog: dependency source inspection

Scope: bounded, read-only continuation. Corrects the prior `/tmp/garrison-catalog-local-data.md` dependency claim: the absence of `~/Repos/asset-resolver` did not establish absent dependency source. Inspected the actual Cargo git checkout and its docs/source, then checked only relevant local data and cached Garrison callers. No network, build, test, CASC extraction, write to repository/cache, or operation. No enum-to-row mapping inferred.

## Corrected dependency availability

- The pinned dependency checkout is present at `/home/osso/.cargo/git/checkouts/asset-resolver-b5f2cd5846cc4706/462097b`; pinned revision is `462097bba2aa72d1afed8e0ca5b7c7315096427d`. Manifest: `/home/osso/.cargo/git/checkouts/asset-resolver-b5f2cd5846cc4706/462097b/Cargo.toml`, SHA-256 `dfae1fb6151b29fec20ad54160a7ca2c6a59cd04b0a1503d02ddf7351de61a5f`. Thus dependency source is available in Cargo's checkout although `~/Repos/asset-resolver` is absent.
- Its docs are `docs/specs/listfile-resolution.md` (SHA-256 `cc8ed10be58f4bc37a9569e13825d7d62967f5f697ec912ff0acb9974c53aede`) and `docs/specs/installed-build-selection.md` (SHA-256 `eb109588f47174cbd41ad4da6b30891c6c84e3180cacd0c4a48fca43fb5fa158`). The listfile source is `src/listfile_cache.rs`, SHA-256 `8edf6e69a53d6ded02c350b141a8933aea353fb544439d4d6bcc0feabaec5a95`; it imports `fdid;path` listfile rows into SQLite `listfile_entries(fdid,path,lower_path)`, not DB2 records. `src/casc_cache.rs` reads CASC `root.bin`/`encoding.bin` and builds a fileDataID-to-content/encoding-key resolution cache, not a localized game-data catalog. The bounded `docs/` and `src/` inventory contains only listfile and CASC asset-resolution documentation/code; no DB2 schema/table reader or native catalog rows.
- The dependency does not define the runtime extraction route for this requested catalog. Its listfile gives asset paths/FDIDs; CASC resolution yields bytes for assets and is not evidence that the source data is a DB2 export or has the needed localized rows.

## Exact local evidence

- The cached vendor consumer is `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_GarrisonUI/Mainline/Blizzard_AdventuresCombatLog.lua`, SHA-256 `4ba843a93ee6cf4bc4febc410a67e781c9e39356904ad4dd660db7fce826e700`. Lines 88–91 call `C_Garrison.GetAutoCombatDamageClassValues()` and index `damageClassValue` and `locString`. This establishes required consumer shape, not values.
- `/home/osso/Projects/wow/wow-ui-sim/docs/wow-client-diff/WowDiscovery.lua`, SHA-256 `4cd7ba572f01c0abf7308fe218230fe004983b0b49d4a752cdfedcf672e4a5e6`, lists the method at line 7466 and `Enum.Damageclass` at lines 24087 onward. These are API/enum discovery metadata; no catalog result rows or `locString` entries appear there.
- The only local CSV data under `data/db2/` with relevant localization is `/home/osso/Projects/wow/wow-ui-sim/data/db2/wowforever-1.60.1.69913/GlobalStrings.csv`, SHA-256 `784c705a19dba90216d670fead458c40297fb3c33062aa95f6928ba935778e7d`. It is a WoWForever 1.60.1.69913 export (different client/build), contains general spell-school/global strings, and does not contain a damage-class-specific catalog key/schema or the requested API rows. Other shipped CSV exports are map/atlas data; no Garrison/auto-combat/damage-class DB2 CSV was present. No relevant raw DB2/ADB catalog file appeared in the bounded repo data or simulator cache paths.
- The examined cached retail Garrison UI has 33 Lua/XML/TOC files. Only the consumer above mentions the target method/fields; those files do not provide the backing table.

## Row result and next grounded path

Known missing-row proof: **zero verified `(damageClassValue, locString)` rows** from all inspected sources. Enum members cannot fill this gap because no method-to-row mapping or locale strings are established. No exact native DB2 table name/schema was found; do not guess one.

The dependency checkout offers no further source-only extraction path. Next read-only probe, if continuing, must use a source that actually exposes the target native result: call `C_Garrison.GetAutoCombatDamageClassValues()` in a matching real WoW client/native probe and record returned rows and locale/client build, or inspect an exact-build DB2 export only after its producer/schema identifies a candidate table and localized-string join. The current local evidence does not ground a particular table name, so do not query one by guess. If a native probe is unavailable, the local-data question remains unresolved; do not synthesize rows from `Enum.Damageclass`.

No target-row file/hash exists because no target rows were found.
