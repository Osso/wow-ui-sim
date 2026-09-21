//! Read the resolved compiler revision from Cargo-generated lock records.

pub fn parse_locked_rilua_revision(lock: &str) -> Result<&str, &'static str> {
    let lines: Vec<_> = lock.lines().collect();
    let mut packages = lines
        .split(|line| line.trim() == "[[package]]")
        .filter(|record| quoted_field(record, "name") == Some("rilua"));
    let package = packages.next().ok_or("missing rilua package")?;
    if packages.next().is_some() {
        return Err("multiple rilua packages; compiler identity is ambiguous");
    }
    let source = quoted_field(package, "source").ok_or("rilua is not Git-sourced")?;
    let (repository, revision) = source
        .rsplit_once('#')
        .ok_or("rilua Git source has no resolved revision")?;
    let has_git_repository = repository
        .strip_prefix("git+")
        .is_some_and(|repository| !repository.is_empty());
    let has_full_revision =
        revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit());
    if !has_git_repository || !has_full_revision {
        return Err("rilua source must identify a Git repository and full 40-digit revision");
    }
    Ok(revision)
}

fn quoted_field<'a>(record: &[&'a str], field: &str) -> Option<&'a str> {
    let (_, value) = record
        .iter()
        .filter_map(|line| line.split_once('='))
        .find(|(key, _)| key.trim() == field)?;
    value.trim().strip_prefix('"')?.strip_suffix('"')
}
