//! XML parsing functions.

use super::types::UiXml;

/// Parse a WoW UI XML file from a string.
pub fn parse_xml(xml: &str) -> Result<UiXml, quick_xml::DeError> {
    let fixed = preprocess_xml(xml)?;
    quick_xml::de::from_str(&fixed)
}

/// Parse a WoW UI XML file from disk.
///
/// Applies fixups for known Blizzard XML quirks before parsing.
pub fn parse_xml_file(path: &std::path::Path) -> Result<UiXml, XmlLoadError> {
    let contents = std::fs::read_to_string(path).map_err(|source| XmlLoadError::IoWithPath {
        path: path.to_path_buf(),
        source,
    })?;
    let mut ui = parse_xml(&contents)?;
    if ui
        .elements
        .iter()
        .any(|element| matches!(element, super::types::XmlElement::Binding(_)))
    {
        let locations = binding_source_lines(&contents)?;
        let mut locations = locations.into_iter();
        for element in &mut ui.elements {
            if let super::types::XmlElement::Binding(binding) = element {
                binding.source_path = Some(path.to_path_buf());
                binding.source_line = locations.next().ok_or_else(|| {
                    quick_xml::DeError::Custom("missing binding source location".to_string())
                })?;
            }
        }
    }
    Ok(ui)
}

fn binding_source_lines(contents: &str) -> Result<Vec<usize>, quick_xml::DeError> {
    use quick_xml::events::Event;

    let mut reader = quick_xml::Reader::from_str(contents);
    let mut lines = Vec::new();
    let mut offset = 0;
    let mut line = 1;
    let mut active = None;
    loop {
        let event = reader
            .read_event()
            .map_err(quick_xml::DeError::InvalidXml)?;
        let end = reader.buffer_position() as usize;
        let event_line = line;
        line += contents.as_bytes()[offset..end]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count();
        offset = end;
        if matches!(event, Event::Eof) {
            break;
        }
        record_binding_event(event, event_line, line, &mut active, &mut lines);
    }
    Ok(lines)
}

fn record_binding_event(
    event: quick_xml::events::Event<'_>,
    event_line: usize,
    line: usize,
    active: &mut Option<usize>,
    lines: &mut Vec<usize>,
) {
    use quick_xml::events::Event;
    match event {
        Event::Start(element) if element.name().as_ref() == b"Binding" => {
            *active = Some(lines.len());
            lines.push(line);
        }
        Event::Empty(element) if element.name().as_ref() == b"Binding" => lines.push(line),
        Event::Text(text) if active.is_some() => {
            let bytes = text.as_ref();
            let leading = bytes
                .iter()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count();
            if leading < bytes.len() {
                let leading_lines = bytes[..leading]
                    .iter()
                    .filter(|byte| **byte == b'\n')
                    .count();
                lines[active.take().expect("active binding")] = event_line + leading_lines;
            }
        }
        Event::End(element) if element.name().as_ref() == b"Binding" => *active = None,
        _ => {}
    }
}

fn preprocess_xml(xml: &str) -> Result<String, quick_xml::DeError> {
    let contents = strip_packager_xml_comments(xml);
    let contents = strip_regular_xml_comments(&contents);
    let fixed = strip_duplicate_self_closing(&contents, "Size")?;
    let fixed = strip_duplicate_self_closing(&fixed, "TexCoords")?;
    let fixed = strip_duplicate_script_handlers(&fixed);
    let fixed = normalize_attribute_equals_spacing(&fixed);
    let fixed = normalize_whitespace_padded_bools(&fixed);
    let fixed = normalize_whitespace_padded_numbers(&fixed);
    Ok(remove_empty_numeric_attrs(&fixed))
}

/// Normalize XML attributes whose equals sign has stray whitespace.
///
/// Retail accepts forms like `value ="0"`, but quick-xml's serde attribute lookup can miss the
/// field name in these addon/Blizzard XML files. Keep this as a text-level preparse fixup so the
/// downstream strongly typed XML model stays strict.
fn normalize_attribute_equals_spacing(xml: &str) -> String {
    use regex::Regex;
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(attribute_equals_spacing_regex);
    re.replace_all(xml, r#"$1=""#).into_owned()
}

fn attribute_equals_spacing_regex() -> regex::Regex {
    regex::Regex::new(r#"\b([A-Za-z_][A-Za-z0-9_:.-]*)\s+=""#)
        .expect("attribute-equals-spacing regex compiles")
}

/// Remove ordinary XML comments after source-packager comment wrappers have been handled.
///
/// quick-xml serde can still try to deserialize XML-looking text inside comments in some addon
/// files. Retail ignores normal comments, so strip them before typed deserialization.
fn strip_regular_xml_comments(xml: &str) -> String {
    use regex::Regex;
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(regular_xml_comment_regex);
    re.replace_all(xml, "").into_owned()
}

fn regular_xml_comment_regex() -> regex::Regex {
    regex::Regex::new(r#"(?s)<!--.*?-->"#).expect("regular-xml-comment regex compiles")
}

/// Strip leading/trailing whitespace inside boolean-valued XML attribute values.
///
/// Blizzard's source occasionally ships typos like `hidden=" true"` (note the leading space).
/// quick-xml's serde-bool deserializer rejects these because the literal does not match
/// `true`/`false`/`1`/`0` exactly. Examples in vendor XML:
///   - Blizzard_GarrisonTemplates/Blizzard_CovenantMissionTemplates.xml:598
///   - Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonLandingPage.xml:673
fn normalize_whitespace_padded_bools(xml: &str) -> String {
    let pairs = [
        (r#"=" true""#, r#"="true""#),
        (r#"="true ""#, r#"="true""#),
        (r#"=" false""#, r#"="false""#),
        (r#"="false ""#, r#"="false""#),
    ];
    let mut out = xml.to_string();
    for (from, to) in pairs {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }
    out
}

/// Strip leading/trailing whitespace inside numeric XML attribute values.
///
/// Blizzard's source occasionally ships typos like `y=" 39"` (note the leading space) — see
/// Blizzard_IslandsQueueUI.xml:170. serde's f32 deserializer rejects whitespace-padded numbers,
/// which silently aborts the entire XML deserialization for that file. Real WoW's XML loader
/// trims numeric attribute values, so we mirror that behavior here. The regex only matches
/// values that are pure numbers wrapped in whitespace, leaving real text attributes untouched.
fn normalize_whitespace_padded_numbers(xml: &str) -> String {
    use regex::Regex;
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(padded_number_attr_regex);
    re.replace_all(xml, trim_padded_number_attr).into_owned()
}

fn padded_number_attr_regex() -> regex::Regex {
    regex::Regex::new(r#"="(\s+-?\d[\d.]*\s*|-?\d[\d.]*\s+)""#)
        .expect("padded-number regex compiles")
}

fn trim_padded_number_attr(caps: &regex::Captures<'_>) -> String {
    format!(r#"="{}""#, caps[1].trim())
}

/// Remove blank values from XML attributes that the UI schema treats as numeric.
///
/// Some addon XML ships entries like `<AbsDimension x="-25" y="" />`. Retail's loader accepts
/// these as an omitted coordinate, while serde rejects the empty string before our optional fields
/// can default it later. Restrict this to known numeric attribute names so text attributes such as
/// `name=""` keep their original value.
fn remove_empty_numeric_attrs(xml: &str) -> String {
    use regex::Regex;
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(empty_numeric_attr_regex);
    re.replace_all(xml, "").into_owned()
}

fn empty_numeric_attr_regex() -> regex::Regex {
    regex::Regex::new(
        r#"\s(?:x|y|left|right|top|bottom|width|height|scale|alpha|value|inset|spacing|padding|degrees|duration|delay|order|level|bytes)="""#,
    )
    .expect("empty-numeric-attr regex compiles")
}

/// Strip CurseForge/BigWigs packager XML comment markers so source-form addons parse correctly.
///
/// Two forms are handled:
/// 1. Self-closing markers: `<!--@non-debug@-->` and `<!--@end-non-debug@-->` — stripped as
///    no-ops (content between them is already valid XML).
/// 2. Block wrappers: `<!--@non-debug@` ... `@end-non-debug@-->` — the comment markers are
///    removed so the wrapped content becomes active XML.
///    Same treatment for `<!--@debug@` / `@end-debug@-->` blocks (treat source as debug build).
///
/// `<!--@no-lib-strip@-->` / `<!--@end-no-lib-strip@-->` are similarly stripped.
fn strip_packager_xml_comments(xml: &str) -> String {
    // Self-closing markers: remove the entire comment line
    let xml = xml.replace("<!--@non-debug@-->", "");
    let xml = xml.replace("<!--@end-non-debug@-->", "");
    let xml = xml.replace("<!--@debug@-->", "");
    let xml = xml.replace("<!--@end-debug@-->", "");
    let xml = xml.replace("<!--@no-lib-strip@-->", "");
    let xml = xml.replace("<!--@end-no-lib-strip@-->", "");

    // Block-wrapper openers: `<!--@tag@` (no closing `-->` on same line)
    let xml = xml.replace(
        "<!--@non-debug@
",
        "",
    );
    let xml = xml.replace(
        "<!--@debug@
",
        "",
    );
    let xml = xml.replace(
        "<!--@no-lib-strip@
",
        "",
    );

    // Block-wrapper closers: `@end-tag@-->` (no opening `<!--` on same line)
    let xml = xml.replace("@end-non-debug@-->", "");
    let xml = xml.replace("@end-debug@-->", "");
    let xml = xml.replace("@end-no-lib-strip@-->", "");

    xml
}

/// Remove duplicate self-closing `<Tag .../>` elements within the same parent.
///
/// Blizzard's XML occasionally has duplicate elements in a single parent
/// (e.g. two `<Size>` in GuildRewards.xml, two `<TexCoords>` in Wowless
/// test.xml). quick-xml's serde can't handle duplicate fields. We keep only
/// the last occurrence (matching WoW's behavior where the last one wins).
fn strip_duplicate_self_closing(xml: &str, tag: &str) -> Result<String, quick_xml::DeError> {
    use quick_xml::events::Event;
    use std::collections::HashMap;

    let mut reader = quick_xml::Reader::from_str(xml);
    let mut parents = vec![0usize];
    let mut next_parent = 1usize;
    let mut seen = HashMap::new();
    let mut remove = Vec::new();
    loop {
        let start = reader.buffer_position() as usize;
        match reader.read_event()? {
            Event::Start(_) => {
                parents.push(next_parent);
                next_parent += 1;
            }
            Event::End(_) => {
                parents.pop();
            }
            Event::Empty(element) if element.name().as_ref() == tag.as_bytes() => {
                let end = reader.buffer_position() as usize;
                if let Some(previous) =
                    seen.insert(*parents.last().expect("XML parent"), (start, end))
                {
                    remove.push(previous);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    remove.sort_unstable();
    let mut result = String::with_capacity(xml.len());
    let mut previous = 0;
    for (start, end) in remove {
        result.push_str(&xml[previous..start]);
        previous = end;
    }
    result.push_str(&xml[previous..]);
    Ok(result)
}

/// Remove duplicate script handler elements within `<Scripts>` blocks.
///
/// Blizzard's XML occasionally has two handlers with the same tag name in one
/// `<Scripts>` block (e.g. two `<OnEnter>` in LFGList.xml). quick-xml's serde
/// can't collect non-contiguous duplicate elements into a Vec. We keep only
/// the last occurrence of each handler (matching WoW's behavior).
fn strip_duplicate_script_handlers(xml: &str) -> String {
    use std::collections::HashMap;

    let lines: Vec<&str> = xml.lines().collect();
    let mut remove = vec![false; lines.len()];
    let mut handlers: HashMap<String, (usize, usize)> = HashMap::new();

    let mut i = 0;
    while i < lines.len() {
        if !is_scripts_open(lines[i]) {
            i += 1;
            continue;
        }
        handlers.clear();
        i = dedup_scripts_block(&lines, i + 1, &mut handlers, &mut remove);
    }

    collect_kept_lines(&lines, &remove)
}

/// Scan one `<Scripts>` block starting after the opening tag at `start`.
/// Returns the line index after the closing `</Scripts>` tag.
fn dedup_scripts_block(
    lines: &[&str],
    start: usize,
    handlers: &mut std::collections::HashMap<String, (usize, usize)>,
    remove: &mut [bool],
) -> usize {
    let mut i = start;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed == "</Scripts>" {
            return i + 1;
        }

        let Some((tag_name, end)) = detect_handler_span(trimmed, lines, i) else {
            i += 1;
            continue;
        };

        if let Some((prev_start, prev_end)) = handlers.insert(tag_name, (i, end)) {
            remove[prev_start..=prev_end].fill(true);
        }
        i = end + 1;
    }
    i
}

/// Check if a line opens a `<Scripts>` block (not self-closing).
fn is_scripts_open(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("<Scripts")
        && trimmed.ends_with('>')
        && !trimmed.ends_with("/>")
        && !trimmed.contains("</Scripts>")
}

/// Detect a handler element starting at `start`, returning `(tag_name, end_line)`.
///
/// Returns `None` if the line isn't an opening tag. For self-closing tags (`/>`),
/// `end_line == start`. For multiline tags, scans forward for the closing tag.
fn detect_handler_span(trimmed: &str, lines: &[&str], start: usize) -> Option<(String, usize)> {
    if !trimmed.starts_with('<') || trimmed.starts_with("</") {
        return None;
    }

    let tag_name = extract_tag_name(trimmed);

    if trimmed.ends_with("/>") {
        return Some((tag_name.to_string(), start));
    }

    let closing_prefix = format!("</{tag_name}>");
    let end = (start + 1..lines.len())
        .find(|&j| lines[j].trim().starts_with(&closing_prefix))
        .unwrap_or(lines.len() - 1);
    Some((tag_name.to_string(), end))
}

/// Extract the tag name from an opening XML tag (e.g. `"OnEnter"` from `"<OnEnter function=...>"`).
fn extract_tag_name(trimmed: &str) -> &str {
    let after_lt = &trimmed[1..];
    let tag_end = after_lt.find([' ', '>', '/']).unwrap_or(after_lt.len());
    &after_lt[..tag_end]
}

/// Build output string from lines not marked for removal.
fn collect_kept_lines(lines: &[&str], remove: &[bool]) -> String {
    let mut out = String::with_capacity(lines.iter().map(|l| l.len() + 1).sum());
    for (idx, line) in lines.iter().enumerate() {
        if !remove[idx] {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(line);
        }
    }
    out
}

/// Error type for XML loading.
#[derive(Debug)]
pub enum XmlLoadError {
    /// IO error without a known path. Prefer `IoWithPath` when the path is
    /// available — it makes diagnosis dramatically easier.
    Io(std::io::Error),
    /// IO error with the failing path attached.
    IoWithPath {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    Parse(quick_xml::DeError),
}

impl From<std::io::Error> for XmlLoadError {
    fn from(e: std::io::Error) -> Self {
        XmlLoadError::Io(e)
    }
}

impl From<quick_xml::DeError> for XmlLoadError {
    fn from(e: quick_xml::DeError) -> Self {
        XmlLoadError::Parse(e)
    }
}

impl std::fmt::Display for XmlLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XmlLoadError::Io(e) => write!(f, "IO error: {}", e),
            XmlLoadError::IoWithPath { path, source } => {
                write!(f, "IO error loading {}: {}", path.display(), source)
            }
            XmlLoadError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for XmlLoadError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_whitespace_padded_numbers_trims_leading_space() {
        let xml = r#"<Size x="39" y=" 39"/>"#;
        let result = normalize_whitespace_padded_numbers(xml);
        assert_eq!(result, r#"<Size x="39" y="39"/>"#);
    }

    #[test]
    fn test_normalize_whitespace_padded_numbers_trims_trailing_space() {
        let xml = r#"<Size x="39 " y="39"/>"#;
        let result = normalize_whitespace_padded_numbers(xml);
        assert_eq!(result, r#"<Size x="39" y="39"/>"#);
    }

    #[test]
    fn test_normalize_whitespace_padded_numbers_preserves_text_attrs() {
        let xml = r#"<Anchor point="TOPLEFT" relativeTo=" $parent " name="Foo Bar"/>"#;
        let result = normalize_whitespace_padded_numbers(xml);
        assert_eq!(result, xml);
    }

    #[test]
    fn test_normalize_whitespace_padded_numbers_handles_negative_and_decimal() {
        let xml = r#"<Anchor x=" -3.5" y="2.0 "/>"#;
        let result = normalize_whitespace_padded_numbers(xml);
        assert_eq!(result, r#"<Anchor x="-3.5" y="2.0"/>"#);
    }

    #[test]
    fn test_strip_duplicate_size_keeps_last() {
        let xml = r#"<FontString parentKey="SumText">
    <Size x="0" y="28"/>
    <Anchors>
        <Anchor point="RIGHT"/>
    </Anchors>
    <Size x="0" y="0"/>
    <Color r="0" g="1" b="0"/>
</FontString>"#;
        let result = strip_duplicate_self_closing(xml, "Size").unwrap();
        assert!(!result.contains(r#"<Size x="0" y="28"/>"#));
        assert!(result.contains(r#"<Size x="0" y="0"/>"#));
    }

    #[test]
    fn test_strip_duplicate_size_no_change_single() {
        let xml = r#"<FontString>
    <Size x="10" y="20"/>
    <Color r="1" g="0" b="0"/>
</FontString>"#;
        let result = strip_duplicate_self_closing(xml, "Size").unwrap();
        assert!(result.contains(r#"<Size x="10" y="20"/>"#));
    }

    #[test]
    fn test_parse_xml_keeps_siblings_after_inline_scripts_block() {
        let xml = r#"<Ui>
    <Button name="BaseTemplate">
        <Frames><Frame>
            <Scripts><OnLoad method="OnLoad"/></Scripts>
        </Frame></Frames>
    </Button>
    <Button name="MiddleTemplate"/>
    <Button name="DerivedTemplate"/>
</Ui>"#;

        let ui = parse_xml(xml).unwrap();
        let names = ui
            .elements
            .into_iter()
            .filter_map(|element| match element {
                crate::XmlElement::Button(frame) => frame.name,
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec!["BaseTemplate", "MiddleTemplate", "DerivedTemplate"]
        );
    }

    #[test]
    fn test_strip_duplicate_script_handler_keeps_last() {
        let xml = r#"<Scripts>
    <OnEnter>
        old_handler();
    </OnEnter>
    <OnLeave function="GameTooltip_Hide"/>
    <OnLoad>
        self:RegisterForClicks("RightButtonUp");
    </OnLoad>
    <OnEnter function="NewHandler"/>
    <OnLeave function="NewLeaveHandler"/>
</Scripts>"#;
        let result = strip_duplicate_script_handlers(xml);
        // First OnEnter (multiline) should be removed
        assert!(!result.contains("old_handler()"));
        // Last OnEnter (self-closing) should remain
        assert!(result.contains(r#"<OnEnter function="NewHandler"/>"#));
        // First OnLeave removed, last kept
        assert!(!result.contains("GameTooltip_Hide"));
        assert!(result.contains("NewLeaveHandler"));
        // OnLoad untouched
        assert!(result.contains("RegisterForClicks"));
    }

    #[test]
    fn test_strip_duplicate_script_handler_no_change() {
        let xml = r#"<Scripts>
    <OnEnter function="Handler"/>
    <OnLeave function="Leave"/>
</Scripts>"#;
        let result = strip_duplicate_script_handlers(xml);
        assert!(result.contains(r#"<OnEnter function="Handler"/>"#));
        assert!(result.contains(r#"<OnLeave function="Leave"/>"#));
    }

    #[test]
    fn test_strip_duplicate_script_handler_separate_blocks() {
        // OnEnter in different Scripts blocks should NOT be considered duplicates
        let xml = r#"<Frame>
    <Scripts>
        <OnEnter function="Handler1"/>
    </Scripts>
</Frame>
<Frame>
    <Scripts>
        <OnEnter function="Handler2"/>
    </Scripts>
</Frame>"#;
        let result = strip_duplicate_script_handlers(xml);
        assert!(result.contains("Handler1"));
        assert!(result.contains("Handler2"));
    }

    #[test]
    fn test_strip_self_closing_triple_duplicate_keeps_last() {
        let xml = r#"<Frame>
    <Size x="1" y="1"/>
    <Size x="2" y="2"/>
    <Size x="3" y="3"/>
</Frame>"#;
        let result = strip_duplicate_self_closing(xml, "Size").unwrap();
        assert!(!result.contains(r#"<Size x="1" y="1"/>"#));
        assert!(!result.contains(r#"<Size x="2" y="2"/>"#));
        assert!(result.contains(r#"<Size x="3" y="3"/>"#));
    }

    #[test]
    fn test_strip_self_closing_with_trailing_comment() {
        let xml = r#"<Frame>
    <TexCoords left="0" right="1" top="0" bottom="0.5"/> <!-- old -->
    <TexCoords left="0" right="1" top="0" bottom="1"/>
</Frame>"#;
        let result = strip_duplicate_self_closing(xml, "TexCoords").unwrap();
        assert!(!result.contains("bottom=\"0.5\""));
        assert!(result.contains(r#"bottom="1""#));
    }

    #[test]
    fn test_strip_self_closing_empty_input() {
        let result = strip_duplicate_self_closing("", "Size").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_strip_self_closing_no_matching_tags() {
        let xml = r#"<Frame>
    <Color r="1" g="0" b="0"/>
</Frame>"#;
        let result = strip_duplicate_self_closing(xml, "Size").unwrap();
        assert!(result.contains(r#"<Color r="1" g="0" b="0"/>"#));
    }

    #[test]
    fn preserves_parent_and_child_sizes_with_equal_indentation() {
        let xml = r#"<Ui>
  <CheckButton name="SpellButtonTemplate" virtual="true">
    <Size x="37" y="37"/>
    <NormalTexture>
    <Size x="64" y="64"/>
    </NormalTexture>
  </CheckButton>
</Ui>"#;
        let ui = parse_xml(xml).unwrap();
        let crate::XmlElement::CheckButton(frame) = &ui.elements[0] else {
            panic!("expected check button");
        };
        let size = frame.size().unwrap();
        assert_eq!((size.x, size.y), (Some(37.0), Some(37.0)));
        let texture = frame.normal_texture().unwrap();
        let size = texture.size.as_ref().unwrap();
        assert_eq!((size.x, size.y), (Some(64.0), Some(64.0)));
    }

    #[test]
    fn test_strip_duplicate_size_different_depths() {
        let xml = r#"<Frame>
    <Size x="100" y="50"/>
    <Layers>
        <Layer>
            <Texture>
                <Size x="10" y="10"/>
            </Texture>
        </Layer>
    </Layers>
</Frame>"#;
        let result = strip_duplicate_self_closing(xml, "Size").unwrap();
        assert!(result.contains(r#"<Size x="100" y="50"/>"#));
        assert!(result.contains(r#"<Size x="10" y="10"/>"#));
    }

    #[test]
    fn test_strip_scripts_nested_scripts_blocks() {
        // A Scripts block nested inside another frame's Scripts block (shouldn't happen
        // in practice, but tests that the parser resets state at </Scripts>).
        let xml = r#"<Scripts>
    <OnLoad>first_load();</OnLoad>
</Scripts>
<Scripts>
    <OnLoad>second_load();</OnLoad>
</Scripts>"#;
        let result = strip_duplicate_script_handlers(xml);
        // Both OnLoad handlers should survive — they're in separate blocks
        assert!(result.contains("first_load()"));
        assert!(result.contains("second_load()"));
    }

    #[test]
    fn test_strip_scripts_empty_block() {
        let xml = r#"<Scripts>
</Scripts>"#;
        let result = strip_duplicate_script_handlers(xml);
        assert!(result.contains("<Scripts>"));
        assert!(result.contains("</Scripts>"));
    }

    #[test]
    fn test_strip_scripts_unclosed_handler() {
        // A multiline handler without a matching closing tag — the search runs to end of file.
        // The function should not panic and should still remove the first occurrence.
        let xml = r#"<Scripts>
    <OnEnter>
        first_enter();
    </OnEnter>
    <OnEnter>
        second_enter();
"#;
        let result = strip_duplicate_script_handlers(xml);
        // First OnEnter should be removed (duplicate), second kept
        assert!(!result.contains("first_enter()"));
        assert!(result.contains("second_enter()"));
    }

    #[test]
    fn test_strip_scripts_self_closing_then_multiline_same_name() {
        // Self-closing handler followed by multiline handler with same tag name.
        let xml = r#"<Scripts>
    <OnEnter function="OldHandler"/>
    <OnEnter>
        new_handler();
    </OnEnter>
</Scripts>"#;
        let result = strip_duplicate_script_handlers(xml);
        // Self-closing first occurrence removed, multiline last occurrence kept
        assert!(!result.contains("OldHandler"));
        assert!(result.contains("new_handler()"));
    }

    #[test]
    fn test_strip_scripts_multiline_then_self_closing_same_name() {
        // Multiline handler followed by self-closing handler with same tag name.
        let xml = r#"<Scripts>
    <OnEnter>
        old_handler();
    </OnEnter>
    <OnEnter function="NewHandler"/>
</Scripts>"#;
        let result = strip_duplicate_script_handlers(xml);
        // Multiline first occurrence removed, self-closing last occurrence kept
        assert!(!result.contains("old_handler()"));
        assert!(result.contains("NewHandler"));
    }

    #[test]
    fn test_strip_scripts_scripts_with_inherit_attribute() {
        // <Scripts inherit="prepend"> is recognized as a Scripts block.
        // Uses multiline handlers so dedup works (single-line <Tag>code</Tag> is not
        // recognized as self-closing by the current implementation).
        let xml = r#"<Scripts inherit="prepend">
    <OnLoad>
        first();
    </OnLoad>
    <OnLoad>
        second();
    </OnLoad>
</Scripts>"#;
        let result = strip_duplicate_script_handlers(xml);
        assert!(!result.contains("first()"));
        assert!(result.contains("second()"));
    }

    #[test]
    fn test_strip_scripts_single_line_handler_not_detected() {
        // Single-line <Tag>content</Tag> is NOT recognized as self-closing (only `/>` is).
        // The function treats it as an unclosed multiline handler and overshoots.
        // This documents a known limitation — all Blizzard handlers use either
        // `<Tag function="..."/>` or multiline `<Tag>\n...\n</Tag>`.
        let xml = r#"<Scripts>
    <OnLoad>first();</OnLoad>
    <OnLoad>second();</OnLoad>
</Scripts>"#;
        let result = strip_duplicate_script_handlers(xml);
        // Both survive because the parser doesn't detect the close on the same line
        assert!(result.contains("first()"));
        assert!(result.contains("second()"));
    }

    #[test]
    fn test_strip_packager_self_closing_non_debug() {
        // BlizzMove Libs.xml style: self-closing comment markers, content is already valid
        let xml = r#"<Ui>
    <!--@non-debug@-->
    <Script file="LibStub\LibStub.lua"/>
    <!--@end-non-debug@-->
</Ui>"#;
        let result = strip_packager_xml_comments(xml);
        assert!(!result.contains("<!--@non-debug@-->"));
        assert!(!result.contains("<!--@end-non-debug@-->"));
        assert!(result.contains(r#"<Script file="LibStub\LibStub.lua"/>"#));
    }

    #[test]
    fn test_strip_packager_block_non_debug() {
        // Block-comment form: content is inside comment wrapper
        let xml = "<!--@non-debug@
<Script file=\"LibStub.lua\"/>
@end-non-debug@-->";
        let result = strip_packager_xml_comments(xml);
        assert!(!result.contains("<!--@non-debug@"));
        assert!(!result.contains("@end-non-debug@-->"));
        assert!(result.contains(r#"<Script file="LibStub.lua"/>"#));
    }
}
