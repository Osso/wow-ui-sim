# Commentator camera speeds

`C_Commentator.SetFollowCameraSpeeds` requires two numeric arguments and returns no values. It does not move a camera or store speed state: 3D camera behavior is intentionally unsupported. The permanent workaround is published for Retail, PTR, WoW Forever, Mists, Era and Anniversary.

## Best-guess basis (best-guess, native UNKNOWN)

The pinned Retail, PTR and WoW Forever `Blizzard_APIDocumentationGenerated/CommentatorFrameDocumentation.lua:1372–1382`, and Mists, Era and Anniversary versions at lines 1212–1222, declare the two numeric arguments. Retail `Blizzard_Commentator/Mainline/Blizzard_Commentator.lua:23–31` calls the function for its camera presets. The no-effect behavior is a simulator limitation, not a claim about native camera behavior.

## Known gaps

`RequiresActiveCommentator` is not modeled. Native camera behavior and native validation details are unknown.
