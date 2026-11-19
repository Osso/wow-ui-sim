#![cfg(all(feature = "gui", feature = "client-retail"))]

use std::path::{Path, PathBuf};
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn write_frame_addon(
    root: &Path,
    name: &str,
    secure: bool,
    width: u32,
    height: u32,
    target: &str,
) -> PathBuf {
    let addon = root.join(name);
    std::fs::create_dir(&addon).unwrap();
    let secure_flag = if secure {
        "## UseSecureEnvironment: 1\n"
    } else {
        ""
    };
    let toc = format!("## Interface: 120105\n## Title: {name}\n{secure_flag}Main.xml\n");
    std::fs::write(addon.join(format!("{name}.toc")), toc).unwrap();
    let xml = format!(
        "<Ui><Frame name=\"XmlDuplicateIdentityFrame\" parent=\"UIParent\"><Size x=\"{width}\" y=\"{height}\"/><Scripts><OnLoad>{target} = self</OnLoad></Scripts></Frame></Ui>"
    );
    std::fs::write(addon.join("Main.xml"), xml).unwrap();
    addon.join(format!("{name}.toc"))
}

#[test]
fn duplicate_named_secure_xml_frame_uses_its_createframe_identity() {
    let root = tempfile::tempdir().unwrap();
    let ordinary_toc = write_frame_addon(
        root.path(),
        "Blizzard_XmlIdentityOrdinary",
        false,
        31,
        41,
        "FirstXmlOnLoadSelf",
    );
    let secure_toc = write_frame_addon(
        root.path(),
        "Blizzard_XmlIdentitySecure",
        true,
        73,
        79,
        "SecondXmlOnLoadSelf",
    );
    let env = WowLuaEnv::new().expect("create XML identity fixture");
    env.state().borrow_mut().addon_base_paths = vec![root.path().to_path_buf()];

    let ordinary_result =
        load_addon(&env.loader_env(), &ordinary_toc).expect("load ordinary XML frame");
    assert!(
        ordinary_result.warnings.is_empty(),
        "{:?}",
        ordinary_result.warnings
    );
    let secure_result =
        load_addon(&env.loader_env(), &secure_toc).expect("load secure duplicate XML frame");
    assert!(
        secure_result.warnings.is_empty(),
        "{:?}",
        secure_result.warnings
    );

    let (distinct, first_width, first_height, second_width, second_height): (bool, f64, f64, f64, f64) = env
        .eval(
            "local first = FirstXmlOnLoadSelf; local second = rawget(__secureenv, 'SecondXmlOnLoadSelf'); \
             return not rawequal(first, second), first:GetWidth(), first:GetHeight(), second:GetWidth(), second:GetHeight()",
        )
        .expect("read both XML OnLoad self values");
    assert!(
        distinct,
        "each XML OnLoad must receive its own CreateFrame result"
    );
    assert_eq!((first_width, first_height), (31.0, 41.0));
    assert_eq!((second_width, second_height), (73.0, 79.0));
}
