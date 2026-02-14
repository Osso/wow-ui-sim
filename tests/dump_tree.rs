use wow_ui_sim::dump::{build_tree, build_warning_dump, strip_wow_escapes};
use wow_ui_sim::widget::{Anchor, AnchorPoint, Frame, WidgetRegistry, WidgetType};

fn make_frame(id: u64, parent: Option<u64>, w: f32, h: f32) -> Frame {
    let mut f = Frame::default();
    f.id = id;
    f.parent_id = parent;
    f.width = w;
    f.height = h;
    f
}

fn anchor(point: AnchorPoint, rel_id: Option<usize>, rel_point: AnchorPoint) -> Anchor {
    Anchor { point, relative_to_id: rel_id, relative_to: None, relative_point: rel_point, x_offset: 0.0, y_offset: 0.0 }
}

fn build_basic_registry() -> WidgetRegistry {
    let mut reg = WidgetRegistry::new();
    let mut uip = make_frame(1, None, 1024.0, 768.0);
    uip.name = Some("UIParent".to_string());
    uip.children = vec![10, 11];
    reg.register(uip);

    let mut btn = make_frame(10, Some(1), 200.0, 36.0);
    btn.name = Some("MyButton".to_string());
    btn.visible = true;
    btn.anchors = vec![anchor(AnchorPoint::Center, None, AnchorPoint::Center)];
    btn.children = vec![20];
    btn.children_keys.insert("Icon".to_string(), 20);
    reg.register(btn);

    let mut tex = make_frame(20, Some(10), 32.0, 32.0);
    tex.widget_type = WidgetType::Texture;
    tex.name = Some("__tex_123".to_string());
    tex.visible = true;
    tex.texture = Some("Interface/Icons/foo".to_string());
    tex.anchors = vec![anchor(AnchorPoint::Center, None, AnchorPoint::Center)];
    reg.register(tex);

    let mut hidden = make_frame(11, Some(1), 100.0, 50.0);
    hidden.name = Some("HiddenFrame".to_string());
    hidden.visible = false;
    reg.register(hidden);

    reg
}

// ── strip_wow_escapes ───────────────────────────────────────

#[test]
fn test_strip_plain_text() {
    assert_eq!(strip_wow_escapes("Hello World"), "Hello World");
}

#[test]
fn test_strip_color_codes() {
    assert_eq!(strip_wow_escapes("|cff00ff00Green|r Text"), "Green Text");
}

#[test]
fn test_strip_texture_escape() {
    assert_eq!(strip_wow_escapes("Before |TInterface/Icons/foo:16|t After"), "Before  After");
}

#[test]
fn test_strip_hyperlink() {
    assert_eq!(strip_wow_escapes("|Hitem:12345|h[Sword]|h"), "[Sword]");
}

#[test]
fn test_strip_nested_escapes() {
    assert_eq!(
        strip_wow_escapes("|cffff0000|Hspell:1234|hFireball|h|r"),
        "Fireball"
    );
}

// ── build_tree integration ──────────────────────────────────

#[test]
fn test_build_tree_includes_children() {
    let reg = build_basic_registry();
    let lines = build_tree(&reg, None, None, false, 1024.0, 768.0);
    let has_button = lines.iter().any(|l| l.contains("MyButton"));
    let has_icon = lines.iter().any(|l| l.contains(".Icon"));
    assert!(has_button, "Should contain MyButton");
    assert!(has_icon, "Should contain .Icon (parentKey)");
}

#[test]
fn test_build_tree_filter() {
    let reg = build_basic_registry();
    let lines = build_tree(&reg, Some("MyButton"), None, false, 1024.0, 768.0);
    assert!(lines.iter().any(|l| l.contains("MyButton")));
    assert!(!lines.iter().any(|l| l.contains("HiddenFrame")));
}

#[test]
fn test_build_tree_visible_only() {
    let reg = build_basic_registry();
    let lines = build_tree(&reg, None, None, true, 1024.0, 768.0);
    assert!(!lines.iter().any(|l| l.contains("HiddenFrame")));
}

#[test]
fn test_build_tree_shows_texture_path() {
    let reg = build_basic_registry();
    let lines = build_tree(&reg, None, None, false, 1024.0, 768.0);
    assert!(lines.iter().any(|l| l.contains("[texture] Interface/Icons/foo")));
}

#[test]
fn test_build_tree_shows_anchor_lines() {
    let reg = build_basic_registry();
    let lines = build_tree(&reg, None, None, false, 1024.0, 768.0);
    assert!(lines.iter().any(|l| l.contains("[anchor]")));
}

#[test]
fn test_build_warning_dump_includes_header() {
    let reg = build_basic_registry();
    let lines = build_warning_dump(&reg, 1024.0, 768.0);
    assert!(lines[0].contains("Frame Dump"));
    assert!(lines[1].contains("1024x768"));
}
