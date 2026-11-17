# Addon metadata lookup performance (Questie)

`GetAddOnMetadata` scanned every TOC key with `eq_ignore_ascii_case` on each call. QuestieDB's Mists TOC has 408,495 keys and Questie reads rows one call at a time, so Mists startup with Questie 12.9.0 + QuestieDB 1.1.1 never settled within 90 s. A case-insensitive key index built while parsing each TOC fixed it (lt-45c1).

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

## Next hotspot (not fixed)

- VERIFIED (flat `perf`, 17,820 samples): the metadata scan is gone from the profile. Top symbols are the Lua VM (`execute_inner` 12.9%) and TOC parsing work (SipHash, `HashMap<String, String>::insert`, `StrSearcher`, `malloc`/`free`, about 20% together).
- INFERRED: third-party startup parses the 57 MB QuestieDB TOC several times (eligibility, metadata registration, enable state, load). The pre-load phase took 3.7–6.9 s with the index against 2.3–3.7 s without it; host load made these runs noisy.

## Sources

- `/home/osso/.local/state/wow-ui-sim/handoff/questie-perf/REPORT.md` — original evidence.
- `/home/osso/.local/state/wow-ui-sim/handoff/metadata-index/REPORT.md` — overlay proof, gates, A/B runs and profile.

## See Also

- [[addon-loading]] — TOC parsing and `AddonInfo` registration.
