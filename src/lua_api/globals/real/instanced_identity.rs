//! Host-declared instance identity context; no world or control acquisition.

use std::collections::HashSet;

use crate::lua_api::state::{SEEDED_LOCAL_CHARACTER_GUID, SimState};

#[derive(Default)]
pub struct InstanceIdentityContext {
    /// INFERRED: false default; host sets the current map context explicitly.
    pub on_instanced_map: bool,
    /// Host-declared player-owned pet/vehicle identities exposed by known GUID aliases.
    pub player_owned_guids: HashSet<String>,
    /// Host-recorded control context. Deliberately not a secrecy input: control
    /// alone neither removes an ally exemption nor grants one to a visitor.
    pub mind_controlled_guids: HashSet<String>,
}

pub(crate) fn identity_is_secret(sim: &SimState, unit: &str) -> bool {
    let Some(guid) = super::super::unit_misc::existing_guid_for_unit(sim, unit) else {
        return false;
    };
    // INFERRED: sibling explicit classification overrides map exemptions.
    if sim.identity_secret_guids.contains(&guid) {
        return true;
    }
    if guid == SEEDED_LOCAL_CHARACTER_GUID {
        return false;
    }
    let exempt =
        is_group_member(sim, &guid) || sim.instance_identity.player_owned_guids.contains(&guid);
    sim.instance_identity.on_instanced_map && !exempt
}

fn is_group_member(sim: &SimState, guid: &str) -> bool {
    sim.party_group_active
        && sim
            .party_members
            .iter()
            .enumerate()
            .any(|(index, _)| guid == super::super::unit_misc::party_guid_for_index(index))
}
