//! TOC `## Key: Value` pairs with a case-insensitive key index.

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

/// Metadata of one parsed TOC.
///
/// Dereferences to the exact-key map, so exact readers such as
/// `get("AllowLoad")` stay case-sensitive. `get_ignore_ascii_case` serves
/// `GetAddOnMetadata` without scanning every key: the index maps each
/// ASCII-lowercased key to the spelling of the last TOC line with that key.
///
/// Clones share one map: a 400k-key TOC stored in the addon list and in the
/// parsed TOC is held once. A write to a shared clone copies it first.
#[derive(Clone, Default)]
pub struct TocMetadata(Arc<MetadataIndex>);

#[derive(Clone, Default)]
struct MetadataIndex {
    values: HashMap<String, String>,
    spellings: HashMap<String, String>,
}

impl TocMetadata {
    /// Store `value` under `key`, replacing an earlier line with the same key.
    pub fn insert(&mut self, key: &str, value: String) {
        let index = Arc::make_mut(&mut self.0);
        index.values.insert(key.to_string(), value);
        index.record_spelling(key);
    }

    /// Append `value` to the comma-separated list stored under `key`.
    pub(crate) fn append_list_value(&mut self, key: &str, value: String) {
        let index = Arc::make_mut(&mut self.0);
        index
            .values
            .entry(key.to_string())
            .and_modify(|existing| {
                existing.push(',');
                existing.push_str(&value);
            })
            .or_insert(value);
        index.record_spelling(key);
    }

    /// Value of the last TOC line whose key equals `key` ignoring ASCII case.
    pub fn get_ignore_ascii_case(&self, key: &str) -> Option<&String> {
        let spelling = self.0.spellings.get(&key.to_ascii_lowercase())?;
        self.0.values.get(spelling)
    }
}

impl MetadataIndex {
    fn record_spelling(&mut self, key: &str) {
        self.spellings
            .insert(key.to_ascii_lowercase(), key.to_string());
    }
}

impl Deref for TocMetadata {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0.values
    }
}

impl std::fmt::Debug for TocMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.values.fmt(f)
    }
}

/// Entries are applied in order, as TOC lines are.
impl<K: AsRef<str>, V: Into<String>> FromIterator<(K, V)> for TocMetadata {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(entries: I) -> Self {
        let mut metadata = Self::default();
        for (key, value) in entries {
            metadata.insert(key.as_ref(), value.into());
        }
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::TocMetadata;

    #[test]
    fn exact_reads_stay_case_sensitive() {
        let metadata = TocMetadata::from_iter([("AllowLoad", "Glue")]);
        assert_eq!(metadata.get("AllowLoad").map(String::as_str), Some("Glue"));
        assert_eq!(metadata.get("allowload"), None);
        assert_eq!(
            metadata
                .get_ignore_ascii_case("ALLOWLOAD")
                .map(String::as_str),
            Some("Glue")
        );
    }

    #[test]
    fn case_variant_lines_resolve_to_the_last_line() {
        let metadata = TocMetadata::from_iter([
            ("X-Case", "upper"),
            ("x-case", "lower"),
            ("X-Case", "again"),
        ]);
        assert_eq!(metadata.len(), 2);
        assert_eq!(metadata.get("x-case").map(String::as_str), Some("lower"));
        assert_eq!(
            metadata.get_ignore_ascii_case("x-CASE").map(String::as_str),
            Some("again")
        );
        assert_eq!(metadata.get_ignore_ascii_case("X-Missing"), None);
    }

    #[test]
    fn writing_a_clone_leaves_the_original_unchanged() {
        let original = TocMetadata::from_iter([("Title", "Original")]);
        let mut copy = original.clone();
        copy.insert("title", "Copy".into());
        assert_eq!(original.len(), 1);
        assert_eq!(
            original.get_ignore_ascii_case("TITLE").map(String::as_str),
            Some("Original")
        );
        assert_eq!(
            copy.get_ignore_ascii_case("TITLE").map(String::as_str),
            Some("Copy")
        );
    }

    #[test]
    fn appended_list_values_stay_visible_through_the_index() {
        let mut metadata = TocMetadata::default();
        metadata.append_list_value("OptionalDeps", "A".into());
        metadata.append_list_value("OptionalDeps", "B".into());
        assert_eq!(
            metadata
                .get_ignore_ascii_case("optionaldeps")
                .map(String::as_str),
            Some("A,B")
        );
    }
}
