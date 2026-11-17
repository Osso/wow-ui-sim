## [2026-11-17] investigation | Addon metadata lookup performance

Added the [investigation](investigations/addon-metadata-lookup-performance.md) and [contract](../specs/addon-metadata-lookup.md); updated [[addon-loading]]. **VERIFIED:** `GetAddOnMetadata` reads drop from 892–2,408 µs to 0.47–1.52 µs on the QuestieDB Mists TOC, and Mists startup with Questie completes instead of timing out at 90 s. Case-variant duplicate keys now resolve to the last TOC line instead of random hash order.

