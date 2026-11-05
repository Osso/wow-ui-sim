//! WM94 §8: the original `SetArtifactItem` accessor selects the Artifact UI's
//! current item identity and composes the ordinary item tooltip from it.

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::state::ArtifactInfo;

tooltip_selection_getters_fixture! {
    fn viewed_artifact(item_id: i32, name: &str) -> ArtifactInfo {
        ArtifactInfo {
            item_id,
            alt_item_id: 0,
            name: name.to_string(),
            icon: String::new(),
            total_xp: 0,
            points_spent: 0,
            quality: 6,
            artifact_appearance_id: 0,
            appearance_mod_id: 0,
            item_appearance_id: 0,
            alt_item_appearance_id: 0,
            alt_on_top: false,
            tier: 1,
            maxed: false,
            disabled: false,
            category: 0,
        }
    }

    #[test]
    fn tooltip_artifact_item_original_accessor_reads_viewed_artifact() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        // The artifact need not be equipped or in a bag: the panel view is the source.
        env.exec("A_Admin.EquipItem(16, 231268)").unwrap();
        for (item_id, name) in [(128872, "The Dreadblades"), (128934, "Shield of the Sea Queen")] {
            env.state().borrow_mut().viewed_artifact.info =
                Some(viewed_artifact(item_id, name));
            env.exec(&format!(
                r#"
                GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                GameTooltip:SetArtifactItem()
                local info = GameTooltip:GetPrimaryTooltipInfo()
                local data = GameTooltip:GetPrimaryTooltipData()
                assert(info and info.getterName == 'GetArtifactItem', 'artifact accessor missing')
                assert(data and data.type == Enum.TooltipDataType.Item and data.id == {item_id})
                assert(data.lines[1].leftText == {name:?})
                assert(GameTooltip:GetLeftLine(1):GetText() == {name:?})
                "#,
            ))
            .unwrap();
        }
    }

    #[test]
    fn tooltip_artifact_item_without_viewed_artifact_is_a_miss() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            A_Admin.EquipItem(16, 231268)
            assert(select('#', C_TooltipInfo.GetArtifactItem()) == 0,
                'no viewed artifact must take the missing route')
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetArtifactItem()
            assert(GameTooltip:GetPrimaryTooltipData() == nil,
                'no equipped-artifact fallback is allowed')
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_artifact_item_without_catalog_item_is_a_miss() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.state().borrow_mut().viewed_artifact.info = Some(viewed_artifact(999_999, "Missing"));
        env.exec(
            r#"
            assert(select('#', C_TooltipInfo.GetArtifactItem()) == 0,
                'unresolved artifact identity must take the missing route')
            "#,
        )
        .unwrap();
    }
}
