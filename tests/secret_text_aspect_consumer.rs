//! WM61 S6 Text aspect through the selected CompactUnitFrame consumer.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

#[test]
fn compact_unit_name_marks_text_aspect_for_secret_identity() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    // Blizzard_QuickJoin.toc:3-6 enables the FriendsFrame dependency by default;
    // QuickJoinToast.lua:160 supplies UpdateDisplayedFriendCount.
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_CompactRaidFrames", "Blizzard_QuickJoin"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_UnitFrame"));
    env.exec("A_Admin.SetPartySize(1); A_Admin.SetPartyMember(1, 'Aspect Name', 1, 80); CompactUnitFrame_SetUnit(CompactArenaFrame.memberUnitFrames[1], 'party1')")
        .unwrap();
    let guid: String = env.eval("return UnitGUID('party1')").unwrap();
    env.state().borrow_mut().identity_secret_guids.insert(guid);
    env.exec(
        r#"
        local consumer = CompactUnitFrame_UpdateName
        local source = debug.getinfo(consumer, 'S')
        assert(source.what == 'Lua' and source.source:find('CompactUnitFrame.lua', 1, true))
        local frame = CompactArenaFrame.memberUnitFrames[1]
        assert(frame and frame.name)
        assert(not frame.name:HasSecretAspect(8), 'fixture starts without Text aspect')
        local name = UnitName('party1')
        assert(issecretvalue(name) and secretunwrap(name) == 'Aspect Name', 'fixture has a secret name')
        consumer(frame)
        assert(frame.name:HasSecretAspect(8), 'secret text must mark Text aspect 8')
        assert(frame.name:HasAnySecretAspect())
        assert(not frame.name:IsPreventingSecretValues(), 'Text aspect is not prevention')
        assert(not frame.name:IsAnchoringSecret(), 'Text aspect is not anchoring secrecy')
        "#,
    )
    .unwrap();
}
