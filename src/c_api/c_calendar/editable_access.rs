//! Local authority for edits to the separately supplied Calendar event.
use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::state::SimState;

pub(super) fn can_edit(host: &SimState) -> bool {
    let policy = &host.calendar_context_policy;
    policy.editable.as_ref().is_some_and(|event| {
        if event.category_flags & 0x40 != 0 {
            policy.announcement_edit_allowed
        } else {
            has_local_authority(host)
        }
    })
}

pub(super) fn has_local_authority(host: &SimState) -> bool {
    host.calendar_context_policy
        .editable
        .as_ref()
        .is_some_and(|event| {
            event.local_edit_credential.is_some_and(|id| id != 0)
                || existing_guid_for_unit(host, "player")
                    .as_deref()
                    .is_some_and(|id| !id.is_empty() && id == event.creator_identity)
        })
}
