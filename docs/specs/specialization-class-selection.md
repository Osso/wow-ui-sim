# Specialization class selection

`C_SpecializationInfo.GetSpecializationInfo` accepts the optional seventh `classID` argument added by the pinned [12.0.0 source contract](../../data/patch-api/sources/12.0.0-register.json). `src/c_api/c_spec.rs` resolves explicit class selections using existing specialization records, without changing player state.

## What it must do

- [x] On retail 12.0.0 and later, a known explicit class ID selects the requested one-based specialization within that class, returning the existing ten outputs.
- [x] Missing or nil class ID preserves the current-player path, including existing active-specialization fallback for an invalid requested index.
- [x] Explicit invalid class or specialization index returns `(0, nil, nil, nil, nil, nil, 0, nil, 0, true)` rather than selecting the player's class. Positive integral class/index validation and using the pinned default-shaped tuple for invalid selections are bounded simulator policy, not native error claims.
- [x] Queries leave player class and active specialization unchanged.
- [x] The unmodified cached `CooldownViewerUtil.GetClassAndSpecTagText` produces `Mage - Arcane` for tag `81` and `Paladin - Holy` for tag `21` when the player is a Paladin.
- [ ] Profiles without the retail 12.0.0 epoch retain their existing player-class selection behavior.

## How it works

- [12.0.0 audit](../wiki/investigations/patch-12-0-0-api-audit.md)

## Implementation inventory

- `src/c_api/c_spec.rs`: explicit class/index resolution and invalid-selection defaults.
- `data/specializations.rs`: existing class/spec identities and metadata.

## Tests asserting this spec

- `tests/talent_spec_probes.rs`: `specialization_class_id_` tests cover explicit class selection, exact output shape, player-state preservation, invalid-selection policy, unmodified CooldownViewer consumer, and non-retail control.

## Known gaps (current cycle)

- [ ] Independent verification remains pending.
- [ ] Non-retail control execution remains pending.

## Out of scope

Inspect, pet, gender, and group behavior are unchanged and not newly validated. Secret/security enforcement, native invalid-input errors/coercion, localization fidelity, and gameplay unlock rules are not claimed. Current cached consumer proof does not establish historical vendor-source equivalence.
