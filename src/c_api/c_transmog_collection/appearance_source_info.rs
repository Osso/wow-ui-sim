//! Explicit source-ID keyed inputs; no fabricated appearance or query producer.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppearanceSourceInfo {
    pub category: i32,
    pub item_appearance_id: i64,
    pub can_have_illusion: bool,
    pub icon: i64,
    pub is_collected: bool,
    pub item_link: String,
    pub transmoglink: String,
    pub source_type: Option<i32>,
    pub item_subclass: i32,
    pub ignore_model_attachment_checks_for_illusion: bool,
}
