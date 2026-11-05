# Artifact-item tooltip

## Requirements

- `C_TooltipInfo.GetArtifactItem()` uses the Artifact UI's current item identity,
  the same value as `C_ArtifactUI.GetArtifactItemID`, and composes the shared
  catalog item lines for it.
- The getter takes no arguments and performs no argument secret check
  (`native-evidence-70688/WM94-tooltipinfo-getters.md` §8).
- The item need not be equipped or in a bag; there is no equipped-artifact
  fallback.
- An absent viewed artifact, or an identity without generated catalog metadata,
  uses the shared `unknown_getter_failure` seam. Zero Lua return values are
  **EMULATED** simulator policy.

## Model limits

- WM94 establishes that this request carries an artifact-specific flag with its
  own instance-descriptor preparation and suppresses some ordinary item
  additions. That branch's exact line set is UNKNOWN, so the producer publishes
  the shared item lines and adds no artifact-only content.

## Profile boundary

Retail, PTR and Forever pinned `TooltipInfoDocumentation.lua:43` declare the
getter; it is registered only on those profiles.

## Evidence and proof

`Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:542` installs the original
`SetArtifactItem` accessor. `tests/tooltip_artifact_item.rs` sets two distinct
viewed artifacts and drives that accessor, then pins the no-view and
missing-catalog cases. No other pinned Blizzard addon calls the accessor
directly; `C_ArtifactUI.GetArtifactItemID` is the shared identity source.
