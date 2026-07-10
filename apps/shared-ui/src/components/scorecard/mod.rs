//! G-27 Scorecard — UI component suite.
//!
//! # Module layout
//!
//! ```text
//! scorecard/
//!   models.rs          — Shared data models (TemplateForm, DimensionForm,
//!                         DisplayRuleForm, SessionDimension, OptionForm)
//!   sections/
//!     display_rules.rs — Display Rules Builder (Configurator section 4)
//!   widgets/
//!     scorecard_widget.rs — Full session form (entity detail pages)
//!     nudge_prompt.rs     — Compact post/pre-activity floating prompt
//! ```
//!
//! The original monolithic `configurator.rs` remains in place for
//! backward compatibility. Apps can migrate to the new split modules
//! incrementally:
//!
//! 1. Adopt `scorecard::models` types in their own state management.
//! 2. Replace the inline `DimensionsSection` with the split version that
//!    includes `is_inverted`.
//! 3. Add the `DisplayRulesSection` as a new tab in the Configurator.
//! 4. Replace the custom rating form with `ScorecardWidget`.
//! 5. Wire `NudgePrompt` to `atlas_activity` create events.

pub mod models;
pub mod sections;
pub mod widgets;

// ── Re-exports (public API) ───────────────────────────────────────────────────

// Models — used by all consumers
pub use models::{
    TemplateForm, DimensionForm, OptionForm, DisplayRuleForm, SessionDimension,
    DisplayConfigForm, ConfiguratorMode, TemplateSavePayload,
};

// Configurator sections
pub use sections::display_rules::DisplayRulesSection;

// Widgets
pub use widgets::scorecard_widget::{ScorecardWidget, ScoreSubmission};
pub use widgets::nudge_prompt::NudgePrompt;
