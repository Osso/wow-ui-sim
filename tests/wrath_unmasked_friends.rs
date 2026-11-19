//! Production friend producers with the unchanged FriendsMicroButton subtree.
#![cfg(feature = "client-wrath")]
#[path = "wrath_unmasked_support/mod.rs"]
mod support;
use support::{env, load, original, run};
use wow_ui_sim::lua_api::state::SocialFriend;

#[test]
fn wrath_unmasked_original_friends_count_tracks_supplied_online_rows() {
    for (online, expected) in [
        (vec![true, false, true], 2),
        (vec![false, true, false], 1),
        (vec![], 0),
    ] {
        let env = env();
        env.state().borrow_mut().social_friends = online
            .iter()
            .enumerate()
            .map(|(i, &is_online)| SocialFriend {
                name: format!("Friend{i}"),
                level: 80,
                area: "Dalaran".into(),
                class_name: "Paladin".into(),
                note: String::new(),
                is_online,
                guid: format!("Player-1-{i:08}"),
            })
            .collect();
        load(&env, &["Fonts.xml", "FontStyles.xml"]);
        run(
            &env,
            r#"
            DEFAULT_CHAT_FRAME = CreateFrame('Frame', 'UnmaskedChatAnchor', UIParent)
            DEFAULT_CHAT_FRAME.buttonFrame = CreateFrame('Frame', nil, DEFAULT_CHAT_FRAME)
        "#,
        );
        // FrameXML/FloatingChatFrame.xml:936,944-950. Parse the COMPLETE
        // document (the old parser failure boundary), then create the unedited
        // original button subtree. Neither count function is replaced.
        let (_, source) = original("FloatingChatFrame.xml");
        let ui = wow_ui_sim::xml::parse_xml(&source).unwrap();
        let frame = ui
            .elements
            .iter()
            .find_map(|element| match element {
                wow_ui_sim::xml::XmlElement::Button(frame)
                    if frame.name.as_deref() == Some("FriendsMicroButton") =>
                {
                    Some(frame)
                }
                _ => None,
            })
            .expect("original FriendsMicroButton");
        let result = wow_ui_sim::loader::create_frame_from_xml(
            &env.loader_env(),
            frame,
            "Button",
            None,
            None,
            None,
            &mut wow_ui_sim::loader::LoadTiming::default(),
        );
        run(&env, "assert(true)");
        assert_eq!(result.unwrap().as_deref(), Some("FriendsMicroButton"));
        run(
            &env,
            &format!(
                r#"
            local total, online = GetNumFriends()
            assert(select('#', GetNumFriends()) == 2)
            assert(total == {} and online == {expected})
            assert(total == C_FriendList.GetNumFriends())
            assert(online == C_FriendList.GetNumOnlineFriends())
            local _, bnOnline = BNGetNumFriends()
            assert(bnOnline == 0) -- Existing temporary BNet source, not a model.
            assert(FriendsMicroButtonCount:GetObjectType() == 'FontString')
            assert(FriendsMicroButtonCount:GetParent() == FriendsMicroButton)
            assert(FriendsMicroButtonCount:GetText() == '{expected}')
        "#,
                online.len()
            ),
        );
    }
}
