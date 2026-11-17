# Addon metadata lookup performance (Questie)

`GetAddOnMetadata` scanned every TOC key with `eq_ignore_ascii_case` on each call. QuestieDB's Mists TOC has 408,495 keys and Questie reads rows one call at a time, so Mists startup with Questie 12.9.0 + QuestieDB 1.1.1 never settled within 90 s. A case-insensitive key index built while parsing each TOC fixed it (lt-45c1). A per-session TOC cache then removed the repeated parses of that 57 MB TOC (lt-498d).

## Symptoms

- VERIFIED (questie-perf handoff, `892fd3e83`): 109–3,307 µs per metadata read against 0.25–1.35 µs on the Era 1.15.9 client; `addon_metadata` 23.42% and `eq_ignore_ascii_case` 20.76% of flat `perf` samples; Questie timer callbacks up to 3.05 s; `lua-errors` with Questie timed out at 90 s.

## Root cause and fix

- `AddonInfo.metadata` was a `HashMap<String, String>`; `find_addon_metadata` iterated it for every lookup.
- `sim_toc::TocMetadata` keeps that exact-key map (exact readers deref to it) and adds an ASCII-lowercased key → last-line spelling index. The [metadata lookup contract](../../specs/addon-metadata-lookup.md) lists the preserved semantics.
- Case-variant duplicate keys (`X-Case`, `x-case`) previously followed random per-map `HashMap` order: 16 parses of one TOC returned all four combinations. They now resolve to the last TOC line.

## Measurements (Mists release, same-host A/B, 2026-11-17)

- VERIFIED: distinct-hit metadata read 2,408 → 1.52 µs; miss 892 → 0.47 µs; cold quest name 648 → 7.3 µs; cold quest objectives 4,080 → 11.3 µs. Non-metadata Lua probes unchanged.
- VERIFIED: startup with Questie completes in 18–26 s (exit 1 from the 4 pre-existing Mists Blizzard UI errors, identical to the no-Questie baseline); the old binary still times out at 90 s.
- VERIFIED: peak `VmHWM` 1,346,632–1,380,604 KiB against 1,144,792–1,154,188 KiB for the old binary (about +200 MiB). INFERRED: the index costs one lowercased key and one spelling copy per key for each live TOC metadata copy.

## Repeated TOC parsing (fixed, lt-498d)

- VERIFIED (flat `perf`, 17,820 samples, lt-45c1): after the index, top symbols were the Lua VM (`execute_inner` 12.9%) and TOC parsing work (SipHash, `HashMap<String, String>::insert`, `StrSearcher`, `malloc`/`free`, about 20% together).
- Cause: every reader parsed the TOC itself. Third-party startup parsed each TOC for screen eligibility, dependency sorting, enable state, list registration, startup eligibility, the Blizzard dependency check and the load; `LoadAddOn` parsed it again. Each parse built a new 408,495-key index, and the addon list kept a copy of the metadata map.
- Fix: the [TOC session cache](../../specs/toc-session-cache.md) parses each TOC path once per UI session and shares `Rc<TocFile>`; `TocMetadata` clones share one map. ReloadUI starts a new cache.
- VERIFIED (optional-tests regression, two 150,000-key TOCs, 10.06 MB): the unchanged loader read 66.3 MB at startup and 9.46 MB more in `LoadAddOn`; the cache reads 10.06 MB and 155 bytes.
- VERIFIED (Mists release, same-host A/B, 3 runs each, `998b136cd` against the fix): third-party pre-load phase 2.81–4.17 s → 0.33–0.58 s; startup wall time 12.6–19.2 s → 8.4–17.8 s (the 17.8 s run was the first after the build; host load made wall times noisy); peak `VmHWM` 1,381,212–1,386,340 KiB → 1,065,056–1,068,932 KiB (about −310 MiB, −23%). Both binaries print the same empty `lua-errors` JSON.

## Sources

- questie-perf handoff `REPORT.md` (untracked proof receipts) — original evidence.
- lt-45c1 metadata-index handoff `REPORT.md` (untracked proof receipts) — overlay proof, gates, A/B runs and profile.

## See Also

- [[addon-loading]] — TOC parsing and `AddonInfo` registration.
- [TOC session cache](../../specs/toc-session-cache.md) — one parse per TOC per UI session.
