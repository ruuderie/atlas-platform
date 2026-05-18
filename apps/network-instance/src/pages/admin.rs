//! Network-instance Admin Dashboard
//!
//! Operator-facing admin page for network-instance tenants.
//! Provides dynamic module-driven navigation using `AdminModuleSidebar`
//! with `SidebarTheme::Network` (light, sans-serif aesthetic).
//!
//! ## Auth gate
//! Uses `check_admin_access()`, a server function that reads the session cookie
//! and verifies the caller holds `Owner | Admin | PlatformSuperAdmin` role via
//! the backend `GET /api/admin/modules` auth path. This prevents reliance on the
//! stale `user.is_admin` boolean that was removed in migration `m20260504_000002`.
//!
//! ## Module Loading
//! Calls `GET /api/admin/modules` (forwarding the session cookie) to load the
//! module set for this tenant. Falls back to an empty list on error.
//!
//! ## Adding new content panels
//! Add a new `AdminModuleType` arm to the `match active_tab.get()` block below.
//! No changes to the sidebar, theming, or backend are required.

use leptos::prelude::*;
use shared_ui::components::admin_module_sidebar::{
    AdminModuleConfig, AdminModuleType, SidebarTheme,
};
use crate::auth::api_base_url;

// ─────────────────────────────────────────────────────────────────────────────
// Server Functions
// ─────────────────────────────────────────────────────────────────────────────

/// Verifies that the caller's session belongs to a user with an admin-level
/// role (`Owner | Admin | PlatformSuperAdmin`) on this tenant.
///
/// Returns `Ok(true)` if access is granted, `Ok(false)` if session is missing
/// or user has insufficient role. Uses `GET /api/admin/modules` as the
/// authoritative check — if the backend accepts the request, the user is admin.
///
/// This is the correct pattern (matching anchor's `check_session()`) — it avoids
/// relying on `UserProfile.is_admin` which was removed in migration m20260504_000002.
#[server(CheckNetworkAdminAccess, "/api")]
pub async fn check_admin_access() -> Result<bool, ServerFnError> {
    use axum::http::request::Parts;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Ok(false);
    };

    // Probe the admin modules endpoint — it enforces Owner/Admin/PlatformSuperAdmin
    // and returns 200 if the role gate passes, 403 otherwise.
    let url = format!("{}/api/admin/modules", api_base_url());
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(res.status().is_success())
}

/// Fetches the enabled admin module set for the authenticated network-instance tenant.
/// Forwards the session cookie so the backend can resolve the caller's tenant context.
///
/// Returns `Vec<AdminModuleConfig>` sorted by `sort_order` ascending.
/// Returns an empty vec on any error — safe fallback.
#[server(GetNetworkAdminModules, "/api")]
pub async fn get_network_admin_modules() -> Result<Vec<AdminModuleConfig>, ServerFnError> {
    use axum::http::request::Parts;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Ok(vec![]);
    };

    let url = format!("{}/api/admin/modules", api_base_url());
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        res.json::<Vec<AdminModuleConfig>>()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    } else {
        // Non-200 (e.g. 403 after session expiry) — return empty list gracefully
        Ok(vec![])
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Admin Page Component
// ─────────────────────────────────────────────────────────────────────────────

/// Network-instance admin dashboard page.
/// Registered at `/admin` in `app.rs`.
///
/// Auth gate uses `check_admin_access()` — a backend-verified server fn —
/// rather than the frontend `AuthContext.user.is_admin` field (stale/removed flag).
#[component]
pub fn NetworkAdmin() -> impl IntoView {
    // SSR-blocking resource: resolves before the page HTML is emitted.
    // This prevents the page from briefly rendering to non-admins during hydration.
    let auth_resource = Resource::new_blocking(
        || (),
        |_| async move { check_admin_access().await.unwrap_or(false) },
    );

    view! {
        <Suspense fallback=move || view! {
            <div class="min-h-screen flex items-center justify-center bg-background">
                <div class="w-10 h-10 border-4 border-primary border-t-transparent rounded-full animate-spin" />
            </div>
        }>
            {move || {
                match auth_resource.get() {
                    None => view! {
                        // Still loading — spinner already rendered by Suspense fallback
                        <div />
                    }.into_any(),
                    Some(false) => view! {
                        // Not authenticated or insufficient role
                        <div class="min-h-screen flex items-center justify-center">
                            <div class="text-center space-y-4 max-w-md mx-auto p-8">
                                <span class="material-symbols-outlined text-5xl text-muted-foreground block">
                                    "lock"
                                </span>
                                <h1 class="text-2xl font-bold text-foreground">"Access Restricted"</h1>
                                <p class="text-muted-foreground">
                                    "Sign in with an operator account to access the admin dashboard."
                                </p>
                                <a href="/auth/login"
                                   class="inline-block bg-primary text-primary-foreground px-6 py-2 rounded-lg text-sm font-medium">
                                    "Sign In"
                                </a>
                            </div>
                        </div>
                    }.into_any(),
                    Some(true) => view! { <AdminDashboard /> }.into_any(),
                }
            }}
        </Suspense>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Authenticated admin shell
// ─────────────────────────────────────────────────────────────────────────────

/// The inner admin shell, only rendered when the user is authenticated + admin.
/// Owns the module resource and the active_tab signal.
#[component]
fn AdminDashboard() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(AdminModuleType::Dashboard);

    // Load module list — blocking resource for SSR hydration consistency.
    let modules_resource = Resource::new_blocking(
        || (),
        |_| async move { get_network_admin_modules().await.unwrap_or_default() },
    );

    let on_logout = Callback::new(move |_: ()| {
        leptos::task::spawn_local(async move {
            // Best-effort revoke; redirect regardless of result.
            let _ = reqwest::Client::new()
                .post(format!("{}/api/auth/session/revoke", api_base_url()))
                .send()
                .await;
            if let Some(w) = web_sys::window() {
                let _ = w.location().replace("/");
            }
        });
    });

    view! {
        <div class="min-h-screen flex bg-background">
            // ── Sidebar ───────────────────────────────────────────────────
            <Suspense fallback=move || view! {
                <div class="w-56 h-screen border-r border-border bg-background animate-pulse" />
            }>
                {move || {
                    let modules = modules_resource.get().unwrap_or_default();
                    view! {
                        <shared_ui::components::admin_module_sidebar::AdminModuleSidebar
                            modules=modules
                            active_tab=active_tab
                            set_active_tab=set_active_tab
                            on_logout=on_logout
                            theme=SidebarTheme::Network
                            brand_label="NETWORK ADMIN".to_string()
                        />
                    }
                }}
            </Suspense>

            // ── Main content ─────────────────────────────────────────────
            <main class="flex-1 overflow-auto p-8">
                // Header
                <div class="mb-8 pb-6 border-b border-border">
                    <p class="text-xs text-muted-foreground uppercase tracking-widest font-medium mb-1">
                        "Network Administration"
                    </p>
                    <h1 class="text-3xl font-bold text-foreground">
                        {move || format!("{}", active_tab.get()
                            .to_display_name_dyn()
                            .unwrap_or("Dashboard")
                        )}
                    </h1>
                </div>

                // Content dispatch
                <div class="space-y-6">
                    {move || match active_tab.get() {
                        AdminModuleType::Dashboard  => view! { <AdminOverviewPanel /> }.into_any(),
                        AdminModuleType::Listings   => view! { <ListingsPanel /> }.into_any(),
                        AdminModuleType::Leads      => view! { <LeadsPanel /> }.into_any(),
                        AdminModuleType::Contacts   => view! { <ContactsPanel /> }.into_any(),
                        AdminModuleType::Settings   => view! { <SettingsPanel /> }.into_any(),
                        AdminModuleType::Security   => view! { <SecurityPanel /> }.into_any(),
                        AdminModuleType::Navigation => view! { <NavigationPanel /> }.into_any(),
                        _ => view! {
                            <div class="flex flex-col items-center justify-center h-64 border-2 border-dashed border-border rounded-xl text-muted-foreground space-y-3">
                                <span class="material-symbols-outlined text-4xl">"construction"</span>
                                <p class="text-sm font-medium">"This module is coming soon."</p>
                            </div>
                        }.into_any(),
                    }}
                </div>
            </main>
        </div>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Content Panels
// Each panel maps to one AdminModuleType arm in AdminDashboard.
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn AdminOverviewPanel() -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-6">
            <StatCard label="Active Listings" value="—" icon="home" />
            <StatCard label="Pending Leads"   value="—" icon="person_add" />
            <StatCard label="Members"         value="—" icon="group" />
        </div>
        <div class="mt-8 p-6 rounded-xl border border-border bg-card">
            <h2 class="text-sm font-semibold text-foreground mb-2">"Network Health"</h2>
            <p class="text-sm text-muted-foreground">
                "Real-time metrics will appear here once telemetry is connected."
            </p>
        </div>
    }
}

#[component]
fn ListingsPanel() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-border bg-card p-8 text-center space-y-3">
            <span class="material-symbols-outlined text-4xl text-primary block">"home"</span>
            <h2 class="text-lg font-semibold text-foreground">"Listings"</h2>
            <p class="text-sm text-muted-foreground max-w-md mx-auto">
                "Review, approve, and manage all listings submitted to this network."
            </p>
        </div>
    }
}

#[component]
fn LeadsPanel() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-border bg-card p-8 text-center space-y-3">
            <span class="material-symbols-outlined text-4xl text-primary block">"person_add"</span>
            <h2 class="text-lg font-semibold text-foreground">"Leads"</h2>
            <p class="text-sm text-muted-foreground max-w-md mx-auto">
                "Inbound inquiries and unvetted contacts awaiting qualification. "
                "Leads are distinct from Contacts — they have not yet opted in to communications."
            </p>
        </div>
    }
}

#[component]
fn ContactsPanel() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-border bg-card p-8 text-center space-y-3">
            <span class="material-symbols-outlined text-4xl text-primary block">"contacts"</span>
            <h2 class="text-lg font-semibold text-foreground">"Contacts"</h2>
            <p class="text-sm text-muted-foreground max-w-md mx-auto">
                "Vetted, opted-in members eligible to receive network communications."
            </p>
        </div>
    }
}

#[component]
fn SettingsPanel() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-border bg-card p-8 text-center space-y-3">
            <span class="material-symbols-outlined text-4xl text-muted-foreground block">"settings"</span>
            <h2 class="text-lg font-semibold text-foreground">"Network Settings"</h2>
            <p class="text-sm text-muted-foreground">"Configure your network appearance, domain, and membership settings."</p>
        </div>
    }
}

#[component]
fn SecurityPanel() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-border bg-card p-8 text-center space-y-3">
            <span class="material-symbols-outlined text-4xl text-muted-foreground block">"security"</span>
            <h2 class="text-lg font-semibold text-foreground">"Security"</h2>
            <p class="text-sm text-muted-foreground">"Manage passkeys, active sessions, and access control."</p>
        </div>
    }
}

#[component]
fn NavigationPanel() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-border bg-card p-8 text-center space-y-3">
            <span class="material-symbols-outlined text-4xl text-muted-foreground block">"menu"</span>
            <h2 class="text-lg font-semibold text-foreground">"Navigation"</h2>
            <p class="text-sm text-muted-foreground">"Configure the network's public-facing navigation menu."</p>
        </div>
    }
}

/// Reusable stat card for the overview panel.
#[component]
fn StatCard(label: &'static str, value: &'static str, icon: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center gap-4 p-5 rounded-xl border border-border bg-card">
            <div class="w-10 h-10 rounded-lg bg-primary/10 text-primary flex items-center justify-center shrink-0">
                <span class="material-symbols-outlined text-[20px]">{icon}</span>
            </div>
            <div>
                <p class="text-2xl font-bold text-foreground">{value}</p>
                <p class="text-xs text-muted-foreground">{label}</p>
            </div>
        </div>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Display name helper (local — avoids cross-crate dep on backend enum)
// ─────────────────────────────────────────────────────────────────────────────

trait DisplayName {
    fn to_display_name_dyn(&self) -> Option<&'static str>;
}

impl DisplayName for AdminModuleType {
    fn to_display_name_dyn(&self) -> Option<&'static str> {
        Some(match self {
            Self::Dashboard      => "Dashboard",
            Self::Settings       => "Settings",
            Self::Security       => "Security",
            Self::Blog           => "Blog",
            Self::ResumeProfiles => "Resume Profiles",
            Self::ResumeEntries  => "Resume Entries",
            Self::LandingPages   => "Landing Pages",
            Self::Webforms       => "Webforms",
            Self::Navigation     => "Navigation",
            Self::Footer         => "Footer",
            Self::PageHeaders    => "Page Headers",
            Self::Leads          => "Leads",
            Self::Contacts       => "Contacts",
            Self::LeadOptions    => "Lead Options",
            Self::Services       => "Services",
            Self::CaseStudies    => "Case Studies",
            Self::Highlights     => "Highlights",
            Self::Properties     => "Properties",
            Self::Listings       => "Listings",
            Self::Custom         => "Custom",
        })
    }
}
