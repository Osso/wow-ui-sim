# Achievement eligibility (R88o.1)

## Native contract

**VERIFIED (Retail 12.1.5.70688, lt-97f5 #1109):** This is a legacy ordinary global. It also publishes in an existing shared secure table, not through a flag-bearing namespace entry. Whether the native secure table exists in a given session is unknown. The simulator registers before its secure snapshot, and updates an already-present shared table on re-registration.

- The first argument is a readable number or a readable, non-secret numeric string. Missing/nil, Boolean, table, function, nonnumeric text, hidden numeric value, and marked string fail with suffix `Usage: IsAchievementEligible(achievementID)`; extras are ignored. Truncate toward zero and convert to signed 32 bits. The masked-invalid conversion maps to `-2147483648`, which is looked up normally.
- Normal calls return exactly one Boolean. An absent Achievement record or absent runtime group passes. A present group with a null tree, or a visited node lacking a CriteriaTree row, fails.
- All-children nodes require every child to pass, including a vacuous true for the empty list. Default operation passes. A leaf fails only when its Criteria row has a nonzero selector ID and a present cached value differs from the configured value: missing Criteria, zero selector ID, and absent cache entry each pass. An absent entry is not a cached zero.
- Earned/completed status, visible criteria count, criteria-failed/timer flags, and faction/class restrictions are not arguments to this provider.

## Simulator data and writers

**INFERRED adapter:** reuses the existing achievement record map but supplies independent empty-by-default inputs for configured tree IDs, nullable runtime roots, row/operation kinds, Criteria rows and cached values. An unprovided tree ID or group is not evidence of a native fresh-session default. The field names `world_state_id` and `required_value` correspond to names inferred from the older 70391 Criteria export. The visible criteria model has no selection-tree mapping and is not used to infer eligibility.

**VERIFIED native writer boundary:** native single and batch updates, control-zone removal, and reset/clear update this cache; a value or auxiliary-flag change can conditionally notify a matching tracked tree. No matching cache writer or its input currently exists in the simulator; the outdoor PvP timer map is a separate host snapshot. This patch has host reads only, does not simulate cache-writer callbacks, and dispatches no synthetic events. `CRITERIA_UPDATE`, `ACHIEVEMENT_EARNED`, and `TRACKED_ACHIEVEMENT_UPDATE` are not substitutes. Live cache contents, auxiliary-flag publishing, and update ordering remain unknown.

## Original consumer and tests

The selected `Blizzard_ObjectiveTracker.toc` loads `Blizzard_AchievementObjectiveTracker.lua` and its XML. `LayoutContents` reads tracked IDs (:96–116); `AddAchievement` uses the per-criterion tuple for **any positive visible count** (:126–131), and calls the global for the zero-visible path before coloring the real line (:182–183). The catalog-backed #6 zero-visible tests each traverse both Normal and Failed with actual font-string style/RGB: missing versus null group, and matching versus mismatching supplied leaf. The #948 multi-criterion tracker proof remains unchanged; a single visible criterion is also on the positive-count bypass. See `tests/achievement_eligible_tracker.rs` and `tests/achievement_eligible_api.rs`.

Pinned owner runtime Lua is identical on Retail 12.1.5.70575, PTR 12.1.7.70493, and Forever 1.60.1.70707 for the tracker file. **INFERRED** PTR and Forever publication is based on those actual selected original calls (separate commit); neither result is a native contract for those profiles. Retail native evidence is separately build 70688. No Blizzard Lua changes.
