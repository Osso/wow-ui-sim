# PTR eager load-order fixture

`ptr-12-1-5-eager-load-order.txt` pins all 211 names returned by `discover_blizzard_addons` for PTR **12.1.5.69594**, product `wowxptr`, Gethe `ptr2` revision `49b69918fcdc77e109813281e4f537d45ec7dcbf`. Authoritative bytes: `data/blizzard-ui-builds/ptr.json`, build key `4a9973f37906f8cfb344f8a9fe6777e0`; paths: `data/blizzard-ui-files/ptr.txt`.

Captured after `9d96dbba0` fixed metadata game-type filtering. This is the eager discovery contract, not the newer startup stream containing bootstrap-only nodes. Retail's inline snapshot remains unchanged.

Source-backed differences from retail's 219-name snapshot:

- Ten removed `Blizzard_Deprecated*` addons are absent from both the PTR manifest and content index.
- `Blizzard_TutorialTemplates` is a required dependency of `Blizzard_BoostTutorial`.
- `Blizzard_PTRFeedback` is present with `OnlyBetaAndPTR: 1`.
- FrameXML's `Blizzard_UnitPopup` and `Blizzard_MirrorTimer` dependencies are Classic-only. Filtering them moves those addons and the UnitPopup dependency chain to their remaining dependency/alphabetical positions.

The corrected dump is `/tmp/pi-ptr125-corrected-order.txt`; `/tmp/pi-ptr125-corrected-order-dump.*` records generation. Its dump helper intentionally exits 101 and is not passing-test evidence. All 211 names were checked against the pinned content index. The exact snapshot assertion must pass separately.
