use std::path::Path;
use wow_ui_sim::xml::parse_xml_file;

fn main() {
    let path = Path::new("/home/osso/Projects/wow/reference-addons/wow-ui-source/Interface/AddOns/Blizzard_SharedXML/SharedBasicControls.xml");
    match parse_xml_file(path) {
        Ok(ui) => println!("Parsed {} elements", ui.elements.len()),
        Err(e) => println!("Error: {}", e),
    }
}
