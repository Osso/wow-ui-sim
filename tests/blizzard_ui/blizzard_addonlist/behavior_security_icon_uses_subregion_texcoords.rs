//! AddonList security icon texture-coordinate behavior for `Blizzard_AddOnList`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AddOnList";

// Wrath has no AddonList vendor widget/handler input; mixed and modeled probes remain enabled.
timerunning_util_panel_fixture! {
post_wrath_ui_fixture! {
#[test]
fn security_icon_uses_indexed_sixteen_pixel_subregions() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let actual_texcoords: String = env
            .eval(
                r#"
                local texture = UIParent:CreateTexture(nil, "ARTWORK")
                local coords = {}

                for index = 1, 4 do
                    AddonList_SetSecurityIcon(texture, index)
                    local tlx, tly, blx, bly, trx, try, brx, bry = texture:GetTexCoord()
                    table.insert(coords, string.format(
                        "%.2f,%.2f,%.2f,%.2f,%.2f,%.2f,%.2f,%.2f",
                        tlx, tly, blx, bly, trx, try, brx, bry
                    ))
                end

                return table.concat(coords, "|")
                "#,
            )
            .expect("AddonList security icon texcoord probe must run cleanly");

        assert_security_icon_texcoords(&actual_texcoords);
    });
}
}
}

fn assert_security_icon_texcoords(actual_texcoords: &str) {
    let expected_texcoords = "0.00,0.00,0.00,1.00,0.25,0.00,0.25,1.00|\
        0.25,0.00,0.25,1.00,0.50,0.00,0.50,1.00|\
        0.50,0.00,0.50,1.00,0.75,0.00,0.75,1.00|\
        0.75,0.00,0.75,1.00,1.00,0.00,1.00,1.00";
    assert_eq!(
        actual_texcoords, expected_texcoords,
        "`AddonList_SetSecurityIcon` must map indexes 1-4 to consecutive 16/64 subregions"
    );
}
