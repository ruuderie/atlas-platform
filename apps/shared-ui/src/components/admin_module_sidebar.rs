//! AdminModuleSidebar — Dynamic admin navigation component.
//!
//! A platform-level component that renders the admin sidebar navigation from
//! a dynamically loaded module list. Consumed by all Atlas app admin pages
//! (`anchor/admin.rs`, `network-instance/admin.rs`, future apps).
//!
//! ## Theming
//!
//! Each app provides a `SidebarTheme` variant. New apps add a new variant or
//! use `SidebarTheme::Custom(tokens)` — the component internals never change.
//!
//! ## Usage
//!
//! ```rust,ignore
//! // In anchor/src/pages/admin.rs
//! <AdminModuleSidebar
//!     modules=modules_vec
//!     active_tab=active_tab
//!     set_active_tab=set_active_tab
//!     on_logout=Callback::new(|_| { /* logout logic */ })
//!     theme=SidebarTheme::Anchor
//! />
//! ```

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// WIRE TYPES
// These mirror `backend/src/models/admin_module.rs` and must stay in sync
// with the JSON shape returned by `GET /api/admin/modules`.
// ─────────────────────────────────────────────────────────────────────────────

/// Logical grouping for admin module types (mirrors backend `ModuleCategory`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModuleCategory {
    Platform,
    Content,
    Appearance,
    CrmAndComms,
    B2B,
    Advanced,
}

/// The canonical set of admin module types (mirrors backend `AdminModuleType`).
/// Used as the tab identity key throughout the admin UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdminModuleType {
    Dashboard,
    Settings,
    Security,
    Blog,
    ResumeProfiles,
    ResumeEntries,
    LandingPages,
    Webforms,
    Navigation,
    Footer,
    PageHeaders,
    Leads,
    Contacts,
    LeadOptions,
    Services,
    CaseStudies,
    Highlights,
    Properties,
    Listings,
    Custom,
}

impl AdminModuleType {
    /// Returns the default material symbol icon name for this module type.
    pub fn default_icon(self) -> &'static str {
        match self {
            Self::Dashboard      => "dashboard",
            Self::Settings       => "settings",
            Self::Security       => "security",
            Self::Blog           => "article",
            Self::ResumeProfiles => "person",
            Self::ResumeEntries  => "work",
            Self::LandingPages   => "web",
            Self::Webforms       => "dynamic_form",
            Self::Navigation     => "menu",
            Self::Footer         => "vertical_align_bottom",
            Self::PageHeaders    => "title",
            Self::Leads          => "person_add",
            Self::Contacts       => "contacts",
            Self::LeadOptions    => "tune",
            Self::Services       => "build",
            Self::CaseStudies    => "cases",
            Self::Highlights     => "star",
            Self::Properties     => "home",
            Self::Listings       => "list",
            Self::Custom         => "category",
        }
    }
}

/// The serialized wire type for a single module — matches backend `AdminModuleConfig`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminModuleConfig {
    pub module_type: AdminModuleType,
    pub display_name: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_fixed: bool,
    pub category: ModuleCategory,
}

// ─────────────────────────────────────────────────────────────────────────────
// THEMING
// ─────────────────────────────────────────────────────────────────────────────

/// Token set consumed by `AdminModuleSidebar` for layout and color rendering.
/// Each field is a Tailwind class string (or bare CSS value for non-Tailwind contexts).
#[derive(Clone, PartialEq)]
pub struct SidebarThemeTokens {
    /// Container background class (e.g. "bg-[#0a0a0a]" or "bg-white")
    pub bg: &'static str,
    /// Logo / brand area text class
    pub brand_text: &'static str,
    /// Inactive nav item text class
    pub text_inactive: &'static str,
    /// Active nav item text class
    pub text_active: &'static str,
    /// Active nav item background class
    pub bg_active: &'static str,
    /// Font family class (e.g. "font-mono" or "font-sans")
    pub font_class: &'static str,
    /// Border / separator class used between sections
    pub border: &'static str,
    /// Logout button text class
    pub logout_text: &'static str,
}

/// Declarative theme selector for `AdminModuleSidebar`.
///
/// New Atlas apps add a new variant or use `Custom(tokens)`.
/// The component internals never change when a new theme is needed.
#[derive(Clone, PartialEq)]
pub enum SidebarTheme {
    /// Dark monospace aesthetic — anchor-app default.
    Anchor,
    /// Light sans-serif aesthetic — network-instance.
    Network,
    /// Fully custom token set — for any future app.
    Custom(SidebarThemeTokens),
}

impl SidebarTheme {
    pub fn tokens(&self) -> SidebarThemeTokens {
        match self {
            Self::Anchor => SidebarThemeTokens {
                bg:            "bg-[#0a0a0a]",
                brand_text:    "text-white font-mono text-sm tracking-widest",
                text_inactive: "text-slate-400 hover:text-white hover:bg-white/5",
                text_active:   "text-white bg-white/10 border-l-2 border-white",
                bg_active:     "bg-white/10",
                font_class:    "font-mono",
                border:        "border-white/10",
                logout_text:   "text-slate-500 hover:text-red-400",
            },
            Self::Network => SidebarThemeTokens {
                bg:            "bg-white",
                brand_text:    "text-slate-900 font-semibold text-sm",
                text_inactive: "text-slate-500 hover:text-slate-900 hover:bg-slate-50",
                text_active:   "text-blue-600 bg-blue-50 border-l-2 border-blue-600",
                bg_active:     "bg-blue-50",
                font_class:    "font-sans",
                border:        "border-slate-200",
                logout_text:   "text-slate-400 hover:text-red-500",
            },
            Self::Custom(t) => t.clone(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// COMPONENT
// ─────────────────────────────────────────────────────────────────────────────

/// Platform-level dynamic admin sidebar component.
///
/// Renders the full left-side navigation column for any Atlas app admin page.
/// Tab order and visibility are driven by the `modules` prop — no hardcoding.
///
/// # Props
/// - `modules` — ordered list of enabled modules from `GET /api/admin/modules`
/// - `active_tab` — reactive signal of the currently selected module type
/// - `set_active_tab` — write signal to change the active tab
/// - `on_logout` — callback invoked when the user clicks Sign Out
/// - `theme` — visual theme variant (defaults to `SidebarTheme::Anchor`)
/// - `brand_label` — optional override for the top brand label text
#[component]
pub fn AdminModuleSidebar(
    modules: Vec<AdminModuleConfig>,
    active_tab: ReadSignal<AdminModuleType>,
    set_active_tab: WriteSignal<AdminModuleType>,
    on_logout: Callback<()>,
    #[prop(default = SidebarTheme::Anchor)]
    theme: SidebarTheme,
    #[prop(into, optional)]
    brand_label: Option<String>,
) -> impl IntoView {
    let tokens = theme.tokens();
    let bg = tokens.bg;
    let border = tokens.border;
    let brand_text_class = tokens.brand_text;
    let font_class = tokens.font_class;
    let text_inactive = tokens.text_inactive;
    let text_active = tokens.text_active;
    let logout_text = tokens.logout_text;

    let label = brand_label.unwrap_or_else(|| "ATLAS".to_string());

    view! {
        <nav class=format!(
            "flex flex-col h-full w-56 shrink-0 border-r {border} {bg} {font_class}"
        )>
            // ── Brand header ────────────────────────────────────────────────
            <div class=format!("px-5 py-5 border-b {border}")>
                <span class=format!("uppercase tracking-widest {brand_text_class}")>
                    {label}
                </span>
            </div>

            // ── Module navigation ────────────────────────────────────────────
            <div class="flex-1 overflow-y-auto py-3 px-2 space-y-0.5">
                {modules.into_iter().map(|m| {
                    let module_type = m.module_type;
                    let display_name = m.display_name.clone();
                    let icon_name = m.icon
                        .clone()
                        .unwrap_or_else(|| module_type.default_icon().to_string());
                    let text_inactive = text_inactive.to_string();
                    let text_active   = text_active.to_string();

                    view! {
                        <SidebarNavItem
                            module_type=module_type
                            display_name=display_name
                            icon_name=icon_name
                            active_tab=active_tab
                            set_active_tab=set_active_tab
                            text_inactive=text_inactive
                            text_active=text_active
                        />
                    }
                }).collect_view()}
            </div>

            // ── Logout ───────────────────────────────────────────────────────
            <div class=format!("px-2 py-3 border-t {border}")>
                <button
                    class=format!(
                        "w-full flex items-center gap-2.5 px-3 py-2 rounded-md \
                         text-xs transition-colors {logout_text}"
                    )
                    on:click=move |_| on_logout.run(())
                >
                    <span class="material-symbols-outlined text-[16px]">
                        "logout"
                    </span>
                    "Sign Out"
                </button>
            </div>
        </nav>
    }
}

// Internal nav item sub-component — not exported.
#[component]
fn SidebarNavItem(
    module_type: AdminModuleType,
    display_name: String,
    icon_name: String,
    active_tab: ReadSignal<AdminModuleType>,
    set_active_tab: WriteSignal<AdminModuleType>,
    text_inactive: String,
    text_active: String,
) -> impl IntoView {
    let text_inactive_clone = text_inactive.clone();
    let text_active_clone   = text_active.clone();

    let item_class = move || {
        if active_tab.get() == module_type {
            format!(
                "w-full flex items-center gap-2.5 px-3 py-2 rounded-md \
                 text-xs transition-colors cursor-pointer {text_active_clone}"
            )
        } else {
            format!(
                "w-full flex items-center gap-2.5 px-3 py-2 rounded-md \
                 text-xs transition-colors cursor-pointer {text_inactive_clone}"
            )
        }
    };

    view! {
        <button
            class=item_class
            on:click=move |_| set_active_tab.set(module_type)
        >
            <span class="material-symbols-outlined text-[16px] shrink-0">
                {icon_name.clone()}
            </span>
            <span class="truncate uppercase tracking-wider text-[10px]">
                {display_name.clone()}
            </span>
        </button>
    }
}
