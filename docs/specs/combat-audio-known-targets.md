# Combat audio known targeting list

Retail and PTR `C_CombatAudioAlert.AddToKnownTargetingList(unit)` and `RemoveFromKnownTargetingList(unit)` return one boolean on normal paths. Retail `CombatAudioAlertDocumentation.lua:11-23,121-133` declares the APIs. WM49-D verifies token storage and full resolved-GUID comparisons, and emulates normal boolean publication. WM52 verifies source refusal before argument parsing; refused calls return zero values via the shared reason-0 reporter.

The simulator retains unit tokens, not GUID snapshots, and resolves them again on each scan. A new resolved identity adds successfully; a duplicate, unresolvable unit or absent removal returns false. Removal matches the first current resolved identity. Wrong types and missing arguments raise an error. The model supports the simulator's player, target, focus and party unit identities; other unit tokens are unresolvable until those identities are modeled. No list-change event is emitted.
