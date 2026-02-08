//! XML parsing functions.

use super::types::UiXml;

/// Parse a WoW UI XML file from a string.
pub fn parse_xml(xml: &str) -> Result<UiXml, quick_xml::DeError> {
    quick_xml::de::from_str(xml)
}

/// Parse a WoW UI XML file from disk.
///
/// Applies fixups for known Blizzard XML quirks before parsing.
pub fn parse_xml_file(path: &std::path::Path) -> Result<UiXml, XmlLoadError> {
    let contents = std::fs::read_to_string(path)?;
    let fixed = strip_duplicate_size_elements(&contents);
    Ok(parse_xml(&fixed)?)
}

/// Remove duplicate `<Size .../>` elements within the same parent element.
///
/// Blizzard's XML occasionally has two `<Size>` elements in a single
/// FontString/Texture (e.g. GuildRewards.xml). quick-xml's serde can't
/// handle non-contiguous duplicate elements. We keep only the last
/// occurrence (matching WoW's behavior where the last Size wins).
fn strip_duplicate_size_elements(xml: &str) -> String {
    use std::collections::HashMap;

    let lines: Vec<&str> = xml.lines().collect();
    let mut result: Vec<Option<usize>> = Vec::with_capacity(lines.len());

    // Track Size element line indices per indentation depth.
    // When we see a second Size at the same depth, remove the first.
    let mut size_at_depth: HashMap<usize, usize> = HashMap::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let depth = line.len() - line.trim_start().len();

        // Closing tags end a child scope - clear Sizes tracked inside it
        if trimmed.starts_with("</") {
            size_at_depth.retain(|&d, _| d <= depth);
        }

        if trimmed.starts_with("<Size ") && trimmed.ends_with("/>") {
            if let Some(prev_idx) = size_at_depth.insert(depth, i) {
                // Mark previous Size line for removal
                result[prev_idx] = None;
            }
            result.push(Some(i));
        } else {
            result.push(Some(i));
        }
    }

    let mut out = String::with_capacity(xml.len());
    for entry in &result {
        if let Some(idx) = entry {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(lines[*idx]);
        }
    }
    out
}

/// Error type for XML loading.
#[derive(Debug)]
pub enum XmlLoadError {
    Io(std::io::Error),
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
            XmlLoadError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for XmlLoadError {}

#[cfg(test)]
mod tests {
    use super::*;

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
        let result = strip_duplicate_size_elements(xml);
        assert!(!result.contains(r#"<Size x="0" y="28"/>"#));
        assert!(result.contains(r#"<Size x="0" y="0"/>"#));
    }

    #[test]
    fn test_strip_duplicate_size_no_change_single() {
        let xml = r#"<FontString>
    <Size x="10" y="20"/>
    <Color r="1" g="0" b="0"/>
</FontString>"#;
        let result = strip_duplicate_size_elements(xml);
        assert!(result.contains(r#"<Size x="10" y="20"/>"#));
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
        let result = strip_duplicate_size_elements(xml);
        assert!(result.contains(r#"<Size x="100" y="50"/>"#));
        assert!(result.contains(r#"<Size x="10" y="10"/>"#));
    }
}
