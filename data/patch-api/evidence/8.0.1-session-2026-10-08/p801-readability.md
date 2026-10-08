# Rust readability audit

Manually read changed Rust at 69b4f0912. Projection/selection/argument parsing/vector construction are separate named functions; no vendor behavior override, speculative default geometry, lint suppression, or nested control flow beyond two levels. Affine extents and continent/map identity are explicit state, not string/API special cases. Tests assert coordinates, state changes, missing inputs, overlap and isolation, not implementation shape. No blocking readability finding. Native overlap selection and geometry remain explicit missing data/behavior, not fallbacks.
