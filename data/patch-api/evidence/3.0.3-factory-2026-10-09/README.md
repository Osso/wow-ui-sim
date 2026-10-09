# Separate supported retail 3.0.3 bare-factory measurement

Base `1ba5b66735ab09e6fcd7208f8e720662278eb3d6`, isolated `p303-factory` worktree. Historical evidence is unchanged; this directory is not a replacement for the original source ledger.

| Exact source name | Published | Value | Default |
|---|---|---|---|
| syncronizeConfig | No | nil | nil |
| synchronizeBindings | Yes | "1" | "1" |
| synchronizeMacros | No | nil | nil |

Discovery at `2f8f3596d`: empty gaps, expected exit 101, one passing unknown control and one failing sweep with exactly two gaps. Reviewed regression at `7ac3902cf`: exit 0, 3/3; all three sweep results byte-identical to discovery. `P303_FABRICATED_UNKNOWN_CVAR` is absent with nil value/default. No CVar assignments performed.

## Receipts

- `development-proof-ledger.json`: exact revisions, cwd, argv, relevant inherited environment and explicit overrides, full-stream hashes, GREEN input hashes and formatting receipt.
- `discovery-command.json` / `green-command.json`: individual execution metadata; corresponding `.stdout.log` / `.stderr.log` retain full streams including warnings.
- `inventory.json`, `known-gaps.json`, `reviewed-values.json`: exact three own source occurrences, reviewed two-gap set and concrete regression outputs.
- `measurement-ledger.json`: three fresh measurement IDs referencing historical inventory IDs; source policy/default boundary and semantic limits.
- `preserved-inputs.json` / `final-preservation.json`: direct equality of original 16 seals, manifest and eight source ledger IDs/bytes; `artifact-hashes.json` hashes fresh receipts without recursive self-sealing.

## Limits

`WowLuaEnv::new` only, shared factory classifier without cache/deprecation publishers. Supported current retail, not native historical 3.0.3 or loaded Blizzard UI. Source 0/1 synchronization policy specifies no default. Local arbitrary CVar storage does not prove synchronization; all UI-settings/bindings/macros synchronization semantics and native/server/persistence effects remain UNPROVEN. No aliases, corrected spelling, guessed defaults, runtime/model/shim/vendor/cache edits or final gates. Main owns full-UI/native integration. Existing library/binary/vendor and unused cached-helper warnings remain unsuppressed in retained streams.
