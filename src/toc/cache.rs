//! Parsed TOC files of one UI session.

use super::TocFile;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Each TOC path is read and parsed once; callers share the result.
///
/// A session is one UI environment: `SimState` replaces its cache when
/// ReloadUI discards the addon runtime, so a reload reads every TOC again.
/// Read errors are not stored, so each caller still sees the current error.
#[derive(Default)]
pub struct TocCache {
    tocs: RefCell<HashMap<PathBuf, Rc<TocFile>>>,
}

impl TocCache {
    /// The parsed TOC at `toc_path`, read from disk on first use.
    pub fn get(&self, toc_path: &Path) -> std::io::Result<Rc<TocFile>> {
        if let Some(toc) = self.tocs.borrow().get(toc_path) {
            return Ok(Rc::clone(toc));
        }
        let toc = Rc::new(TocFile::from_file(toc_path)?);
        self.tocs
            .borrow_mut()
            .insert(toc_path.to_path_buf(), Rc::clone(&toc));
        Ok(toc)
    }
}

#[cfg(test)]
mod tests {
    use super::TocCache;

    #[test]
    fn a_session_keeps_the_first_parse_and_a_new_session_reads_again() {
        let dir = tempfile::tempdir().expect("tempdir");
        let toc_path = dir.path().join("Probe.toc");
        std::fs::write(&toc_path, "## Title: First\nCore.lua\n").expect("write toc");
        let session = TocCache::default();
        let first = session.get(&toc_path).expect("parse");
        std::fs::write(&toc_path, "## Title: Second\nCore.lua\n").expect("rewrite toc");

        let same_session = session.get(&toc_path).expect("cached");
        let next_session = TocCache::default().get(&toc_path).expect("reparse");

        assert_eq!(
            first.metadata.get("Title").map(String::as_str),
            Some("First")
        );
        assert_eq!(
            same_session.metadata.get("Title").map(String::as_str),
            Some("First")
        );
        assert_eq!(
            next_session.metadata.get("Title").map(String::as_str),
            Some("Second")
        );
    }

    #[test]
    fn a_read_error_is_reported_again_and_not_stored() {
        let dir = tempfile::tempdir().expect("tempdir");
        let toc_path = dir.path().join("Missing.toc");
        let session = TocCache::default();
        assert!(session.get(&toc_path).is_err());
        std::fs::write(&toc_path, "## Title: Late\n").expect("write toc");
        let toc = session.get(&toc_path).expect("read after the file appears");
        assert_eq!(toc.metadata.get("Title").map(String::as_str), Some("Late"));
    }
}
