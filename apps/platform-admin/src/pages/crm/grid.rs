use leptos::prelude::*;
use leptos_router::hooks::{use_query_map, use_navigate};
use crate::pages::crm::tabs::{
    leads_tab::LeadsTab,
    accounts_tab::AccountsTab,
    contacts_tab::ContactsTab,
    opportunities_tab::OpportunitiesTab,
};

/// CRM grid — thin shell that renders the tab bar and lazy-mounts the
/// active tab component. Each tab component owns its own LocalResource,
/// so only the selected tab fires an API call.
#[component]
pub fn CrmGrid() -> impl IntoView {
    let query    = use_query_map();
    let navigate = use_navigate();

    let active_tab = move || {
        query.with(|q| q.get("tab").map(|s| s.to_string()).unwrap_or_else(|| "leads".to_string()))
    };

    let page_title = move || match active_tab().as_str() {
        "leads"         => "Leads",
        "accounts"      => "Accounts",
        "contacts"      => "Contacts",
        "opportunities" => "Opportunities",
        _               => "CRM",
    };

    view! {
        <div class="main-area">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">{page_title}</h1>
                    <p class="page-subtitle">"G-31 Canonical Record Store · Platform-wide"</p>
                </div>
                <div style="display:flex;gap:8px;">
                    <button class="btn btn-ghost btn-sm">"Export CSV"</button>
                    <button class="btn btn-primary btn-sm">
                        <svg viewBox="0 0 14 14" width="12" height="12" fill="currentColor" style="margin-right:4px;">
                            <path d="M7 2a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H8v3a1 1 0 1 1-2 0V8H3a1 1 0 1 1 0-2h3V3a1 1 0 0 1 1-1z"/>
                        </svg>
                        {move || format!("New {}", match active_tab().as_str() {
                            "leads"         => "Lead",
                            "accounts"      => "Account",
                            "contacts"      => "Contact",
                            "opportunities" => "Opportunity",
                            _               => "Record",
                        })}
                    </button>
                </div>
            </div>

            // ── Tab Bar ──
            <div class="tab-bar">
                {[
                    ("leads",         "Leads"),
                    ("accounts",      "Accounts"),
                    ("contacts",      "Contacts"),
                    ("opportunities", "Opportunities"),
                ].into_iter().map(|(tab, label)| {
                    let navigate   = navigate.clone();
                    let tab_str    = tab.to_string();
                    let tab_active = tab_str.clone();
                    let tab_nav    = tab_str.clone();
                    view! {
                        <button
                            class=move || format!("tab {}", if active_tab() == tab_active { "active" } else { "" })
                            on:click=move |_| navigate(&format!("/crm?tab={}", tab_nav), Default::default())
                        >
                            {label}
                        </button>
                    }
                }).collect_view()}
            </div>

            // ── Active Tab Body ──
            // Using Show so only the active branch is mounted and its
            // LocalResource only fires when that tab is visible.
            <Show when=move || active_tab() == "leads">
                <LeadsTab />
            </Show>
            <Show when=move || active_tab() == "accounts">
                <AccountsTab />
            </Show>
            <Show when=move || active_tab() == "contacts">
                <ContactsTab />
            </Show>
            <Show when=move || active_tab() == "opportunities">
                <OpportunitiesTab />
            </Show>
        </div>
    }
}
