# Local CASC Index Generation Selection

A local reader defect hid installed 69977 records by allowing directory enumeration to replace the latest numeric base generation with an older one. Published dependencies now select the newest evidenced generation; isolated 69977 CASC sync completed all 4,398 Forever source files and the four former panel-asset misses are MD5-verified in the normal cache.

## Root cause and rule

For the affected 69977 local index buckets, the directory order was `a8`, `ac`, `ab`, `a9`, `aa`. The old aggregate loader let the later `aa` directory entry replace `ac`, returning `None` for records stored in `data.089`. The evidence establishes `ac` as the highest numeric base generation for those buckets; it does **not** license a global assumption that every bucket's newest entry is named `ac`.

Correct behavior selects the highest numeric generation per bucket. It must not use enumeration order, merge older generations, rescue missing entries from older data, or fabricate a resolver fallback.

| FDID | Encoding key | Verified local location |
|---|---|---|
| 7216281 | `5f957e00e5b07e2ee0c17b9bcbf5b506` | `data.089`, offset `415085389`, 258 bytes |
| 4637050 | `160aea39820ed10cedbb32e71bf790d4` | `data.089`, offset `717108714`, 352 bytes |

Direct BLTE decoding matched the expected MD5 content keys. This proves local 69977 content presence for those records.

## Published dependency and source proof

`9ac8d0554` pins the published reader chain:

- `cascette-rs` `d8ec31f4dabaaff80af64b2510c54104a1871fec`
- `asset-resolver` `b2bd6c88303b4db9cd620f66a1a8fc28a4737651`
- `casc-extract` `b25e415a3aab00d0758e3d615ba5a9107a6465df`

Final scoped verification at `72d6d8b45` passed all three CDN/status cases, as part of 12/12 focused checks; `cargo fmt --check` and `cargo check` also passed. This does not erase the unchanged two Python casing failures, five dependency warnings, or eight older lib-test warnings.

The isolated fixture `/tmp/wow-character-bug/frozen-69977-install` synced all 4,398 wowforever manifest files: 4,068 local reads, 330 CDN downloads, one reused session, 91.279 seconds. `/tmp/wow-character-bug/verified-69977/cache-promotion.json` records byte-equal promotion into the default wowforever cache and explicitly excludes desktop install registration. Gethe `c6e899…` supplied manifest paths only; 4,397 comparable files match after CRLF normalization, while `LoadLocale` remains `ptPT` rather than `enUS`.

## Former asset misses

The four former character-panel misses are now exact 69977 CASC files in the standard byte cache, with content MD5 records in `/tmp/wow-character-bug/assets-staged/promoted.json`: `8175455`, `8245174`, `8254784`, and `8197104`. This is acquisition evidence only. It adds no source-file fallback, no substituted texture, no claim of native English fidelity, and no registered-host startup pass.

## Limits

The desktop `.build.info` still lacks `wow_classic_beta`; the observed Syncthing debug copy was modified on September 24, 2026 at 19:39 CDT, and this work did not edit real installation metadata. Thus the actual registered install cannot currently supply host-startup proof. The release alias runs with Forever features, but final release visual acceptance remains pending. The unrelated minimap mask miss remains open.

## Sources

- `/tmp/wow-character-bug/index-probe.stdout` and `local-content-verified.json` — generation and BLTE/MD5 evidence.
- `/tmp/wow-character-bug/verified-69977/cache-promotion.json` — 4,398-file byte-equal promotion.
- `/tmp/wow-character-bug/assets-staged/promoted.json` — verified formerly missing assets.
- `9ac8d0554`, `72d6d8b45` — dependency pins and final semantic verification revision.

## See Also

- [[casc-asset-cache]] — cache layers and runtime source sync.
- [[forever-character-panel]] — bounded consumer evidence and limits.
- [[backpack-background-texture]] — independent exact slot-texture acquisition.
