# Changed Rust readability audit

Scope: `src/c_api/c_garrison_trees.rs`, additive registration/SimState fields and both 7.2.5 test files at `43dd7ed8e`. Manual changed-line audit: no readability violations found. New production callbacks stay below 30 body lines; maximum nesting two; named query inputs, one state borrow scope and explicit detached array construction. No warning suppressions, alternate paths or shims. Deterministic catalog ordering is documented without native-order credit.

Tool behavior stays opt-in in `parse_top_level_api_bullets`, separate from all earlier parsers. Existing flags do not fit: legacy bullets require an API heading and nested New/Removals; BFA parsing ignores Changes summaries; prose-api-links handles wiki links rather than API templates. Fixture covers canonical display-label separation, multi-reference lines and rename direction.
