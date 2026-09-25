# Local CASC Index Generation Selection

A local CASC reader defect, not absent game data, hid two known 69977 records when it let a later directory entry replace a newer index generation. Published dependency revisions now select only the highest numeric generation. Four Forever texture records remain genuinely unindexed; the runtime cache is still incomplete after prior sync failures.

## Evidence

The local index directory order is `a8`, `ac`, `ab`, `a9`, `aa`. The old `IndexManager::load_all` overwrote a bucket as it enumerated that directory, leaving later `aa` selected over newer `ac`. It returned `None` for both known entries even though the sorted section in `04000007ac.idx` contains both keys:

| FDID | Encoding key | Correct local location |
|---|---|---|
| `7216281` | `5f957e00e5b07e2ee0c17b9bcbf5b506` | `data.089`, offset `415085389`, stored `258` bytes |
| `4637050` | `160aea39820ed10cedbb32e71bf790d4` | `data.089`, offset `717108714`, stored `352` bytes |

Selecting explicit newest bucket `04` in `ac` returns archive `89` and those offsets. Direct local reads decode BLTE and match the expected MD5 content keys, proving the entries are present in installed 69977 content. This supersedes the earlier diagnosis that a local `None` from the old aggregate loader established content absence.

The correct rule is strict: select the highest numeric local index generation only. Do not merge older generations or rescue absent entries from them; older records may be stale.

## Published dependency chain

`9ac8d0554` pins the simulator to the published reader chain:

- `cascette-rs` `d8ec31f4dabaaff80af64b2510c54104a1871fec`
- `asset-resolver` `b2bd6c88303b4db9cd620f66a1a8fc28a4737651`
- `casc-extract` `b25e415a3aab00d0758e3d615ba5a9107a6465df`

The cascette fix is the published `fix-local-index-generations` work: highest-generation selection without an older-generation merge or fallback. Focused RED evidence exists at `/tmp/wow-character-bug/savedindex-generation-red.log`; independent GREEN is pending. The simulator dependency pin is committed locally but not pushed.

## Forever source-sync boundary

The selected Forever target is installed build `1.60.1.69977`. Gethe revision `c6e899…` supplies manifest paths only; it is not a runtime source-byte fallback. Gethe's newer `70009` branch is not selected.

The four remaining texture blockers are still genuinely absent from the local index: Stats `8197104`, `8175455`, `8245174`, and `8254784`. They remain separate from the corrected reader defect. Prior CDN session reuse, bounded retry, index-memory release, and Zenity plain-text error work are committed in `27aed3051`, `2bbff1cb1`, and `7639c234d`, but require a new operational build/run. The cached Blizzard UI source tree is incomplete from earlier sync failures; this record does not claim it is restored.

## Sources

- `/tmp/wow-character-bug/index-probe.stdout` — old aggregate-loader `None` results and explicit newest-generation locations.
- `/tmp/wow-character-bug/local-content-verified.json` — local `data.089` BLTE decode and expected-MD5 matches.
- `/tmp/wow-character-bug/savedindex-generation-red.log` — focused pre-fix RED evidence.
- `9ac8d0554` — simulator dependency pins.
- `27aed3051`, `2bbff1cb1`, `7639c234d` — pending operational CDN-sync changes.

## See Also

- [[casc-asset-cache]] — cache and source-sync behavior.
- [[forever-character-panel]] — four remaining real content blockers.
- [[casc-root-v2-parsing-missing-textures]] — earlier independent root-parser failure mode.
