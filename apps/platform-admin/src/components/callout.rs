//! Callout — a semantic alert/notice block distinct from data cards.
//!
//! Use this for static system messages, contextual warnings, and informational
//! banners. Never use a `<Card>` for this purpose — the visual distinction
//! between an alert surface and a data surface is load-bearing for operators.
//!
//! ## Variants
//! - `"info"`    — primary-tinted, for neutral guidance
//! - `"warning"` — amber-tinted, for important notices that require attention
//! - `"success"` — green-tinted, for positive confirmations
//! - `"error"`   — red-tinted, for critical failures or destructive-action warnings

use leptos::prelude::*;

// ── Palette mapping ────────────────────────────────────────────────────────────

fn outer_classes(variant: &str) -> &'static str {
    match variant {
        "warning" => "w-full px-4 py-3.5 rounded-xl border bg-amber-500/10 border-amber-500/20 text-xs text-on-surface-variant leading-relaxed",
        "success" => "w-full px-4 py-3.5 rounded-xl border bg-emerald-500/10 border-emerald-500/20 text-xs text-on-surface-variant leading-relaxed",
        "error"   => "w-full px-4 py-3.5 rounded-xl border bg-error/10 border-error/20 text-xs text-on-surface-variant leading-relaxed",
        _         => "w-full px-4 py-3.5 rounded-xl border bg-primary/10 border-primary/20 text-xs text-on-surface-variant leading-relaxed",
    }
}

fn title_classes(variant: &str) -> &'static str {
    match variant {
        "warning" => "text-amber-400 font-bold",
        "success" => "text-emerald-400 font-bold",
        "error"   => "text-error font-bold",
        _         => "text-primary font-bold",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn Callout(
    /// Visual variant: "info" | "warning" | "success" | "error".
    /// Defaults to "info".
    #[prop(default = "info")]
    variant: &'static str,
    /// Optional bold title shown before the body text.
    #[prop(optional)]
    title: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=outer_classes(variant)>
            {title.map(|t| view! {
                <span class=title_classes(variant)>{t}{" — "}</span>
            })}
            {children()}
        </div>
    }
}
