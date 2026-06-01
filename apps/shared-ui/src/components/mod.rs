pub mod badge;
pub mod card;
pub mod data_table;
pub mod modal;
pub mod tabs;
pub mod file_attachments;
pub mod hooks;
pub mod ui;
pub mod attribute_icon;
pub mod auth;
pub mod properties_editor;
pub mod icon;
pub mod theme_provider;
// Platform admin module registry sidebar
pub mod admin_module_sidebar;
pub mod crm_stage_bar;
pub mod crm_timeline;
pub mod crm_timeline_generic;
pub mod email_composer;
/// G-27 Scorecard Template Configurator — create/edit templates, dimensions, and Combinator config.
pub mod configurator;

/// G-27 Scorecard — component suite (widget, nudge prompt, display rules builder).
/// Use this for new features; configurator.rs remains for backward compat.
pub mod scorecard;
