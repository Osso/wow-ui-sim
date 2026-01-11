use std::path::Path;
use wow_ui_sim::xml::{parse_xml_file, XmlElement};

const BLIZZARD_DIR: &str =
    "/home/osso/Projects/wow/reference-addons/wow-ui-source/Interface/AddOns/Blizzard_SharedXMLBase";

#[test]
fn test_parse_callback_registrant_xml() {
    let path = Path::new(BLIZZARD_DIR).join("CallbackRegistrant.xml");
    let ui = parse_xml_file(&path).expect("Failed to parse XML");

    // Should have Script and Frame elements
    assert!(!ui.elements.is_empty());

    let mut has_script = false;
    let mut has_frame = false;

    for element in &ui.elements {
        match element {
            XmlElement::Script(s) => {
                assert_eq!(s.file.as_deref(), Some("CallbackRegistrant.lua"));
                has_script = true;
            }
            XmlElement::Frame(f) => {
                assert_eq!(f.name.as_deref(), Some("CallbackRegistrantTemplate"));
                assert_eq!(f.mixin.as_deref(), Some("CallbackRegistrantMixin"));
                assert_eq!(f.is_virtual, Some(true));
                has_frame = true;
            }
            _ => {}
        }
    }

    assert!(has_script, "Expected Script element");
    assert!(has_frame, "Expected Frame element");
}

#[test]
fn test_parse_color_swatch_xml() {
    let path = Path::new(BLIZZARD_DIR).join("ColorSwatch.xml");
    let ui = parse_xml_file(&path).expect("Failed to parse XML");

    // Find the ColorSwatchTemplate frame
    for element in &ui.elements {
        if let XmlElement::Frame(f) = element {
            if f.name.as_deref() == Some("ColorSwatchTemplate") {
                // Check mixin
                assert_eq!(f.mixin.as_deref(), Some("ColorSwatchMixin"));

                // Check size
                let size = f.size().expect("Expected size");
                assert_eq!(size.x, Some(16.0));
                assert_eq!(size.y, Some(16.0));

                // Check layers exist
                let layers: Vec<_> = f.layers().collect();
                assert!(!layers.is_empty(), "Expected at least one layer");

                // Check for textures in layers
                let has_textures = layers.iter().any(|l| l.layers.iter().any(|layer| layer.textures().next().is_some()));
                assert!(has_textures, "Expected textures in layers");

                return;
            }
        }
    }
    panic!("ColorSwatchTemplate frame not found");
}

#[test]
fn test_parse_all_xml_in_shared_xml_base() {
    // Try to parse all XML files in the directory
    let dir = Path::new(BLIZZARD_DIR);
    let mut parsed = 0;
    let mut failed = Vec::new();

    for entry in std::fs::read_dir(dir).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension().map(|e| e == "xml").unwrap_or(false) {
            match parse_xml_file(&path) {
                Ok(_) => parsed += 1,
                Err(e) => failed.push((path.clone(), e)),
            }
        }
    }

    // Report results
    println!("Parsed {} XML files successfully", parsed);
    if !failed.is_empty() {
        println!("Failed to parse {} files:", failed.len());
        for (path, error) in &failed {
            println!("  {:?}: {}", path.file_name().unwrap(), error);
        }
    }

    // At least some files should parse
    assert!(parsed > 0, "Expected to parse at least some XML files");

    // Allow some failures for now (complex elements we haven't implemented)
    // but most should parse
    let total = parsed + failed.len();
    let success_rate = parsed as f64 / total as f64;
    assert!(
        success_rate >= 0.5,
        "Expected at least 50% success rate, got {:.0}%",
        success_rate * 100.0
    );
}

#[test]
fn test_xml_with_scripts() {
    // Test parsing XML with inline scripts
    let xml = r#"
        <Ui>
            <Frame name="TestFrame">
                <Scripts>
                    <OnLoad>
                        self:RegisterEvent("PLAYER_LOGIN")
                    </OnLoad>
                    <OnEvent method="OnEvent"/>
                    <OnShow inherit="append">
                        print("shown")
                    </OnShow>
                </Scripts>
            </Frame>
        </Ui>
    "#;

    let ui = wow_ui_sim::xml::parse_xml(xml).expect("Failed to parse XML");

    if let XmlElement::Frame(f) = &ui.elements[0] {
        let scripts = f.scripts().expect("Expected scripts");

        // Check OnLoad has inline code
        let on_load = scripts.on_load.as_ref().expect("Expected OnLoad");
        assert!(on_load.body.is_some());

        // Check OnEvent uses method reference
        let on_event = scripts.on_event.as_ref().expect("Expected OnEvent");
        assert_eq!(on_event.method.as_deref(), Some("OnEvent"));

        // Check OnShow has inherit attribute
        let on_show = scripts.on_show.as_ref().expect("Expected OnShow");
        assert_eq!(on_show.inherit.as_deref(), Some("append"));
    } else {
        panic!("Expected Frame element");
    }
}
