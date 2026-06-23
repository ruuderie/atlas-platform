use leptos::prelude::*;
use crate::api::networks::grant_syndication;
use crate::api::networks::revoke_syndication;

#[component]
pub fn SyndicationManager() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // Modal states
    let show_grant_modal = RwSignal::new(false);
    let show_revoke_modal = RwSignal::new(false);
    
    // Selected target to revoke
    let revoke_target = RwSignal::new("leira Rentals".to_string());
    
    // Select dropdown in grant modal
    let selected_grant_instance = RwSignal::new("".to_string());

    let handle_grant = move |_| {
        let instance = selected_grant_instance.get();
        if instance.is_empty() { return; }
        show_grant_modal.set(false);
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match grant_syndication(&instance).await {
                Ok(_) => t_toast.show_toast("Syndication Granted", &format!("Syndication access granted to {}.", instance), "success"),
                Err(e) => t_toast.show_toast("Error", &format!("Failed to grant syndication: {}", e), "error"),
            }
        });
    };

    let handle_revoke = move |_| {
        let target = revoke_target.get();
        show_revoke_modal.set(false);
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match revoke_syndication(&target).await {
                Ok(_) => t_toast.show_toast("Syndication Revoked", &format!("Access for {} has been revoked.", target), "info"),
                Err(e) => t_toast.show_toast("Error", &format!("Failed to revoke syndication: {}", e), "error"),
            }
        });
    };

    view! {
        <div class="main-canvas">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Syndication Manager"</h1>
                    <p class="page-subtitle">"Control which network instances a tenant's listings appear on"</p>
                </div>
                <div class="page-header-actions">
                    <button 
                        class="btn btn-ghost btn-sm"
                        on:click=move |_| {
                            let window = web_sys::window().unwrap();
                            let _ = window.history().unwrap().back();
                        }
                    >
                        "← Back to tenant"
                    </button>
                    <button 
                        class="btn btn-primary"
                        on:click=move |_| show_grant_modal.set(true)
                    >
                        "+ Grant access"
                    </button>
                </div>
            </div>

            // ── Context Banner ──
            <div class="context-banner">
                <div>
                    <p class="context-label">"Managing syndication for"</p>
                    <p class="context-value">"Oakwood Property Management"</p>
                </div>
                <div class="context-sep"></div>
                <div>
                    <p class="context-label">"Tenant ID"</p>
                    <p style="font-size:11px;color:var(--text-muted);font-family:'SFMono-Regular',monospace;">"ten_7f2a1b9c-4e3d"</p>
                </div>
                <div class="context-sep"></div>
                <div>
                    <p class="context-label">"Plan"</p>
                    <p class="context-value">"Professional"</p>
                </div>
                <div class="context-sep"></div>
                <div>
                    <p class="context-label">"Active listings"</p>
                    <p class="context-value">"34"</p>
                </div>
            </div>

            // ── Active Grants Section ──
            <div class="section">
                <div class="section-header">
                    <span class="section-title">
                        <span class="status-dot" style="background:var(--green);"></span>
                        "Active syndication grants"
                        <span class="section-count">"2 active"</span>
                    </span>
                </div>

                // Grant 1
                <div class="grant-card">
                    <div style="display:flex;align-items:flex-start;justify-content:space-between;gap:12px;">
                        <div style="flex:1;">
                            <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;">
                                <h4 class="grant-title">"leira Rentals"</h4>
                                <span class="tag tag-pm">"Active"</span>
                            </div>
                            <p class="grant-meta">"leira-rentals.app · LTR Directory · Capability: ltr_listings"</p>
                            <p class="grant-meta" style="color:var(--text-muted);margin-top:2px;">"Granted Jun 3, 2026 by admin@atlas.com"</p>
                        </div>
                        <div style="text-align:right;">
                            <span style="font-size:14px;font-weight:700;color:var(--text-primary);font-family:monospace;">"28"</span>
                            <span style="font-size:11px;color:var(--text-muted);">" / 34 listings"</span>
                        </div>
                    </div>
                    <div style="margin-top:10px;">
                        <div style="display:flex;justify-content:space-between;font-size:10px;color:var(--text-secondary);margin-bottom:2px;">
                            <span>"Syncing 28 of 34 listings"</span>
                            <span style="font-weight:700;">"82%"</span>
                        </div>
                        <div class="sync-bar">
                            <div class="sync-bar-fill" style="width: 82%;"></div>
                        </div>
                    </div>
                    <div class="grant-actions">
                        <button class="btn btn-ghost btn-sm" on:click=move |_| { let _ = web_sys::window().unwrap().location().set_href("/apps"); }>"View Synced Listings"</button>
                        <button 
                            class="btn btn-sm btn-reject"
                            style="border-color:var(--red);color:var(--red);"
                            on:click=move |_| {
                                revoke_target.set("leira Rentals".to_string());
                                show_revoke_modal.set(true);
                            }
                        >
                            "Revoke Access"
                        </button>
                    </div>
                </div>

                // Grant 2
                <div class="grant-card">
                    <div style="display:flex;align-items:flex-start;justify-content:space-between;gap:12px;">
                        <div style="flex:1;">
                            <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;">
                                <h4 class="grant-title">"leira Stays"</h4>
                                <span class="tag tag-pm">"Active"</span>
                            </div>
                            <p class="grant-meta">"leira-stays.app · STR Directory · Capability: str_listings"</p>
                            <p class="grant-meta" style="color:var(--text-muted);margin-top:2px;">"Granted Jun 3, 2026 by admin@atlas.com"</p>
                        </div>
                        <div style="text-align:right;">
                            <span style="font-size:14px;font-weight:700;color:var(--text-primary);font-family:monospace;">"6"</span>
                            <span style="font-size:11px;color:var(--text-muted);">" / 34 listings"</span>
                        </div>
                    </div>
                    <div style="margin-top:10px;">
                        <div style="display:flex;justify-content:space-between;font-size:10px;color:var(--text-secondary);margin-bottom:2px;">
                            <span>"Syncing 6 of 34 listings"</span>
                            <span style="font-weight:700;">"18%"</span>
                        </div>
                        <div class="sync-bar">
                            <div class="sync-bar-fill" style="width: 18%; background:var(--green);"></div>
                        </div>
                    </div>
                    <div class="grant-actions">
                        <button class="btn btn-ghost btn-sm" on:click=move |_| { let _ = web_sys::window().unwrap().location().set_href("/apps"); }>"View Synced Listings"</button>
                        <button 
                            class="btn btn-sm btn-reject"
                            style="border-color:var(--red);color:var(--red);"
                            on:click=move |_| {
                                revoke_target.set("leira Stays".to_string());
                                show_revoke_modal.set(true);
                            }
                        >
                            "Revoke Access"
                        </button>
                    </div>
                </div>
            </div>

            // ── Grant History Section ──
            <div class="section">
                <div class="section-header">
                    <span class="section-title">"Grant History"</span>
                </div>
                <div class="audit-row">
                    <span class="audit-dot" style="background:var(--green);"></span>
                    <span style="flex:1;color:var(--text-primary);">"Granted access to leira Rentals"</span>
                    <span style="color:var(--text-muted);font-family:monospace;">"Jun 3, 2026 · by admin@atlas.com"</span>
                </div>
                <div class="audit-row">
                    <span class="audit-dot" style="background:var(--green);"></span>
                    <span style="flex:1;color:var(--text-primary);">"Granted access to leira Stays"</span>
                    <span style="color:var(--text-muted);font-family:monospace;">"Jun 3, 2026 · by admin@atlas.com"</span>
                </div>
            </div>
        </div>

        // ── Modal: Grant Syndication ──
        <Show when=move || show_grant_modal.get()>
            <div class="modal-overlay open">
                <div class="modal">
                    <h3 style="font-size:15px;font-weight:700;margin-bottom:8px;color:var(--text-primary);">"Grant Syndication Access"</h3>
                    <p style="font-size:12px;color:var(--text-secondary);margin-bottom:16px;">"Select a network instance to syndicate Oakwood PM's listings to."</p>
                    
                    <div style="margin-bottom:20px;">
                        <label class="form-label">"Network Instance"</label>
                        <select 
                            class="form-select"
                            on:change=move |ev| selected_grant_instance.set(event_target_value(&ev))
                        >
                            <option value="">"— choose —"</option>
                            <option value="leira Pros">"leira Pros (leira-pros.app)"</option>
                            <option value="leira Miami">"leira Miami (leira-miami.app)"</option>
                        </select>
                    </div>

                    <div style="display:flex;justify-content:flex-end;gap:8px;">
                        <button class="btn btn-ghost" on:click=move |_| show_grant_modal.set(false)>"Cancel"</button>
                        <button class="btn btn-primary" on:click=handle_grant>"Grant Access"</button>
                    </div>
                </div>
            </div>
        </Show>

        // ── Modal: Revoke Syndication ──
        <Show when=move || show_revoke_modal.get()>
            <div class="modal-overlay open">
                <div class="modal">
                    <h3 style="font-size:15px;font-weight:700;margin-bottom:8px;color:var(--red);">"Revoke Syndication Access?"</h3>
                    <p style="font-size:12px;color:var(--text-secondary);margin-bottom:16px;line-height:1.6;">
                        "Revoking access will immediately remove Oakwood PM's listings from " <strong style="color:var(--text-primary);">{move || revoke_target.get()}</strong> ". This cannot be undone automatically."
                    </p>

                    <div style="display:flex;justify-content:flex-end;gap:8px;">
                        <button class="btn btn-ghost" on:click=move |_| show_revoke_modal.set(false)>"Cancel"</button>
                        <button 
                            class="btn"
                            style="background:transparent;border-color:var(--red);color:var(--red);"
                            on:click=handle_revoke
                        >
                            "Revoke Access"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
