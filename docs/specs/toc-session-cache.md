# TOC session cache

The simulator reads and parses each addon TOC file once per UI session and shares the parsed TOC between every reader. The QuestieDB Mists TOC is 57 MB with 408,495 metadata keys; before this cache, Mists startup parsed it several times (lt-498d).

## What it must do

- [x] One UI session reads each TOC path at most once. A session starts at launch or at ReloadUI and ends at the next ReloadUI, which discards the addon runtime and its cache, so a reload reads every TOC again as before.
- [x] Every session reader uses the cache: third-party discovery, dependency sorting, enable-state dependency propagation, addon-list registration, startup eligibility, the Blizzard dependency closure for third-party addons, startup loading and dependency screen checks; Blizzard discovery, `LoadWith` registration and startup loading; `C_AddOns` runtime TOC lookup, the wrong-interface check and `LoadAddOn`; `WowLuaEnv::scan_and_register_addons`.
- [x] A read error is not stored: each reader still receives the current error, as when every reader read the file itself.
- [x] Results do not change: addon lists, metadata, enable state, dependency order and load order are the same as with one parse per reader. The addon list and the parsed TOC share one metadata map; a write to a shared map copies it first.
- [x] Discovery functions without a session (`discover_blizzard_addons_for_screen`, `discover_all_blizzard_addons` and the closure helpers) parse each TOC once per call. The `*_with_tocs` variants take the session cache.

Within one session a TOC edited on disk after its first read is not read again. Native client TOC re-read timing is UNKNOWN; this matches the simulator's ReloadUI boundary, not a native claim.

## How it works

- `src/toc/cache.rs` — `TocCache` maps a TOC path to `Rc<TocFile>`.
- `src/lua_api/state/ui_reload.rs` — `discard_addon_runtime` replaces `SimState::toc_cache`.
- `crates/sim-toc/src/metadata.rs` — `TocMetadata` keeps its maps behind one `Arc`.

## Tests asserting this spec

- `src/toc/cache.rs` unit tests — a session keeps its first parse and a new session reads the edited file; a read error is reported again and not stored.
- `crates/sim-toc/src/metadata.rs` — writing a clone leaves the original unchanged.
- `src/bin/wow_sim/addon_loading/toc_parse_once_tests.rs` (`optional-tests`, Linux) — two 150,000-key TOCs: third-party startup reads each once, and `C_AddOns.LoadAddOn` of the load-on-demand addon reads it no more, measured with the thread's `rchar` counter.
- Existing addon loading, metadata, load-order and ReloadUI tests cover unchanged results.
