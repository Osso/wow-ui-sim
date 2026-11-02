//! Supplied derived item-specialization records.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSpecRecord {
    /// Native permissive field, inferred as no traits and no specializations.
    pub no_traits_and_specializations: bool,
    /// Native permissive field, inferred as no stat-derived traits.
    pub no_stat_traits: bool,
    /// Native field selecting rejection of an unmatched specialization.
    pub reject_unmatched: bool,
    pub specialization_ids: Vec<u32>,
}
