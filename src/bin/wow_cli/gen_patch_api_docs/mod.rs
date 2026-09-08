//! Semantic register generation from pinned Blizzard documentation snapshots.

mod evaluate;
mod records;
mod snapshot;

use clap::Args;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Args)]
pub struct Options {
    #[arg(long)]
    repository: PathBuf,
    #[arg(long)]
    base_commit: String,
    #[arg(long)]
    target_commit: String,
    #[arg(long)]
    patch: String,
    #[arg(long)]
    output: PathBuf,
}

pub fn run(options: Options) -> Result<()> {
    let bytes = generate(
        &options.repository,
        &options.base_commit,
        &options.target_commit,
        &options.patch,
    )?;
    std::fs::write(&options.output, bytes)?;
    Ok(())
}

fn generate(repository: &Path, base: &str, target: &str, patch: &str) -> Result<Vec<u8>> {
    let before = snapshot::read(repository, base)?;
    let after = snapshot::read(repository, target)?;
    let occurrences = diff(&before.records, &after.records);
    let register = json!({
        "schema": "patch-api-source-register/v1",
        "patch": patch,
        "source": {
            "base": before.metadata,
            "target": after.metadata,
            "method": "Semantic comparison of generated Blizzard API documentation evaluated in isolated rilua VMs",
            "hash_algorithm": "SHA-256 over source files sorted by UTF-8 path bytes; for each file append u64 big-endian path byte length, path bytes, u64 big-endian blob byte length, exact git blob bytes. No separators or newline conversion.",
            "limitations": [
                "Documentation snapshot delta, not proof of runtime behavior or first API introduction.",
                "Only immediate *Documentation.lua files in Blizzard_APIDocumentationGenerated; excludes FrameXML helpers, CVars and GlobalStrings not represented there.",
                "Documentation prose is ignored recursively; Enum/Constants references and their addition/subtraction remain symbolic strings; no numeric resolution is inferred.",
                "Two endpoints only; intermediate additions/removals are not observable."
            ]
        },
        "direction_counts": counts(&occurrences, "direction"),
        "category_counts": counts(&occurrences, "category"),
        "occurrences": occurrences,
    });
    let mut bytes = serde_json::to_vec_pretty(&register)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn counts(rows: &[Value], key: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for row in rows {
        *counts
            .entry(row[key].as_str().unwrap().to_owned())
            .or_default() += 1;
    }
    counts
}

fn diff(before: &records::Records, after: &records::Records) -> Vec<Value> {
    let symbols: BTreeSet<_> = before.keys().chain(after.keys()).collect();
    let mut rows = Vec::new();
    for direction in ["added", "changed", "removed"] {
        for symbol in &symbols {
            let old = before.get(*symbol);
            let new = after.get(*symbol);
            let change = match (old, new) {
                (None, Some(_)) => "added",
                (Some(_), None) => "removed",
                (Some(a), Some(b)) if a != b => "changed",
                _ => continue,
            };
            if change != direction {
                continue;
            }
            let record = new.or(old).unwrap();
            rows.push(json!({
                "direction": direction, "category": record.category, "symbol": symbol,
                "before": old.map(|v| &v.payload), "after": new.map(|v| &v.payload),
            }));
        }
    }
    rows
}

#[cfg(test)]
mod tests;
