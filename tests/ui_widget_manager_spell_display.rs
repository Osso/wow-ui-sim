//! Supplied SpellDisplay getter/feed/consumer tests; execution proof lives in the spec.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[path = "widget_spell_display/consumer.rs"]
mod consumer;
#[path = "widget_discrete/fixtures.rs"]
mod discrete_fixtures;
#[path = "widget_spell_display/feed.rs"]
mod feed;
#[path = "widget_spell_display/fixtures.rs"]
mod fixtures;
#[path = "widget_spell_display/queries.rs"]
mod queries;
#[path = "widget_spell_display/security.rs"]
mod security;
#[path = "widget_spell_display/unit_apis.rs"]
mod unit_apis;
