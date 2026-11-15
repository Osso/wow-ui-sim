//! Retail LFG reward writer from R88n.4/.4a/.4b (12.1.5.70688).
//! Host decoding is explicit: no public packet identity or wire decoder is implied.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use crate::lua_api::state::{LfgTooltipRewardGroup, SimState};

/// Enclosing decoded host message, not a JamLFGLockListSlot array element.
/// UNKNOWN transport identity, decoder and other clear routes stay unmodeled.
#[derive(Clone, Debug, Default)]
pub struct DecodedLfgRewardMessage {
    pub groups: Vec<LfgTooltipRewardGroup>,
}

/// Apply a decoded reward update, then notify installed Lua event listeners.
/// VERIFIED readiness/reset/rebuild/event order; Retail only.
pub fn apply_decoded_message(
    env: &WowLuaEnv,
    message: DecodedLfgRewardMessage,
) -> crate::Result<()> {
    if !retail_writer_enabled() {
        return Ok(());
    }
    {
        let mut sim = env.state().borrow_mut();
        rebuild_reward_groups(&mut sim, message);
    }
    env.fire_event("LFG_LOCK_INFO_RECEIVED")?;
    env.fire_event("LFG_UPDATE_RANDOM_INFO")
}

fn rebuild_reward_groups(sim: &mut SimState, message: DecodedLfgRewardMessage) {
    sim.lfg_tooltip_rewards_ready = false;
    sim.lfg_tooltip_reward_groups.clear();
    for incoming in message.groups {
        replace_reward_group(&mut sim.lfg_tooltip_reward_groups, incoming);
    }
    sim.lfg_tooltip_rewards_ready = true;
}

fn replace_reward_group(groups: &mut Vec<LfgTooltipRewardGroup>, incoming: LfgTooltipRewardGroup) {
    let index = groups.iter().position(|group| {
        group.dungeon_id == incoming.dungeon_id && group.dungeon_type_id == incoming.dungeon_type_id
    });
    let index = index.unwrap_or_else(|| {
        groups.push(LfgTooltipRewardGroup {
            dungeon_id: incoming.dungeon_id,
            dungeon_type_id: incoming.dungeon_type_id,
            ..LfgTooltipRewardGroup::default()
        });
        groups.len() - 1
    });
    let group = &mut groups[index];
    group.shortage_rewards = Default::default();
    group.shortage_rewards = incoming.shortage_rewards;
    // Vec length is the authoritative entry count; IDs/quantities/discriminators
    // remain exactly the decoded host entries, without catalog-derived invention.
    group.rewards = incoming.rewards;
}

fn retail_writer_enabled() -> bool {
    ACTIVE == ClientProfile::Retail
}
