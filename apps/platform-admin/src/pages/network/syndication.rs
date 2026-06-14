use leptos::prelude::*;

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
        toast.show_toast("Success", &format!("Syndication access granted to {}", instance), "success");
    };

    let handle_revoke = move |_| {
        show_revoke_modal.set(false);
        toast.show_toast("Revoked", &format!("Syndication access revoked for {}", revoke_target.get()), "info");
    };

    view! {
        <div class="space-y-6">
            // ── Page Header ──
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                <div>
                    <h1 class="text-2xl font-extrabold tracking-tight text-on-surface">"Syndication Manager"</h1>
                    <p class="text-xs text-on-surface-variant mt-1">"Control which network instances a tenant's listings are dynamically syndicated to"</p>
                </div>
                <div class="flex items-center gap-3">
                    <button 
                        class="btn-ghost px-4 py-2 rounded-lg text-sm font-semibold border border-outline-variant/30 hover:bg-surface-bright/20 hover:text-on-surface transition-all active:scale-95"
                        on:click=move |_| {
                            let window = web_sys::window().unwrap();
                            let _ = window.history().unwrap().back();
                        }
                    >
                        "← Back to Tenant"
                    </button>
                    <button 
                        class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-semibold text-on-primary-container shadow-md shadow-primary/10 hover:opacity-90 active:scale-95 transition-all"
                        on:click=move |_| show_grant_modal.set(true)
                    >
                        "+ Grant Access"
                    </button>
                </div>
            </div>

            // ── Context Banner ──
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 bg-surface-container-low border border-outline-variant/20 p-5 rounded-xl shadow-sm">
                <div class="space-y-1">
                    <span class="text-[9px] font-bold text-on-surface-variant/50 uppercase tracking-wider">"Managing syndication for"</span>
                    <p class="text-sm font-bold text-on-surface">"Oakwood Property Management"</p>
                </div>
                <div class="space-y-1">
                    <span class="text-[9px] font-bold text-on-surface-variant/50 uppercase tracking-wider">"Tenant ID"</span>
                    <p class="text-xs font-mono text-on-surface-variant/80">"ten_7f2a1b9c-4e3d"</p>
                </div>
                <div class="space-y-1">
                    <span class="text-[9px] font-bold text-on-surface-variant/50 uppercase tracking-wider">"Active Plan"</span>
                    <p class="text-sm font-semibold text-on-surface">"Professional"</p>
                </div>
                <div class="space-y-1">
                    <span class="text-[9px] font-bold text-on-surface-variant/50 uppercase tracking-wider">"Active Listings"</span>
                    <p class="text-sm font-bold text-primary">"34"</p>
                </div>
            </div>

            // ── Active Grants Section ──
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm space-y-4 p-6">
                <div class="flex items-center gap-2 border-b border-outline-variant/10 pb-4 mb-4">
                    <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Active Syndication Grants (2)"</h3>
                </div>

                // Grant 1
                <div class="bg-surface-container p-5 rounded-xl border border-outline-variant/20 space-y-4">
                    <div class="flex justify-between items-start gap-4">
                        <div>
                            <div class="flex items-center gap-2">
                                <h4 class="text-sm font-bold text-on-surface">"leira Rentals"</h4>
                                <span class="px-2 py-0.5 rounded text-[8px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider">"Active"</span>
                            </div>
                            <p class="text-[10px] text-on-surface-variant/70 mt-1">"leira-rentals.app · LTR Directory · Capability: ltr_listings"</p>
                            <p class="text-[9px] text-on-surface-variant/50 mt-1">"Granted Jun 3, 2026 by admin@atlas.com"</p>
                        </div>
                        <div class="text-right">
                            <span class="text-sm font-bold text-on-surface font-mono">"28"</span>
                            <span class="text-xs text-on-surface-variant">" / 34 listings"</span>
                        </div>
                    </div>
                    <div class="space-y-1">
                        <div class="flex justify-between text-[10px] text-on-surface-variant">
                            <span>"Syncing 28 of 34 listings"</span>
                            <span class="font-bold">"82%"</span>
                        </div>
                        <div class="w-full h-1 bg-surface-container-low rounded-full overflow-hidden">
                            <div class="h-full bg-primary" style="width: 82%"></div>
                        </div>
                    </div>
                    <div class="flex gap-2 pt-2">
                        <button class="btn-ghost px-3 py-1.5 rounded-lg border border-outline-variant/30 text-[10px] font-bold uppercase tracking-wider">"View Synced Listings"</button>
                        <button 
                            class="bg-error-container/20 border border-error/30 text-error hover:bg-error-container/30 px-3 py-1.5 rounded-lg text-[10px] font-bold uppercase tracking-wider active:scale-95 transition-all"
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
                <div class="bg-surface-container p-5 rounded-xl border border-outline-variant/20 space-y-4">
                    <div class="flex justify-between items-start gap-4">
                        <div>
                            <div class="flex items-center gap-2">
                                <h4 class="text-sm font-bold text-on-surface">"leira Stays"</h4>
                                <span class="px-2 py-0.5 rounded text-[8px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider">"Active"</span>
                            </div>
                            <p class="text-[10px] text-on-surface-variant/70 mt-1">"leira-stays.app · STR Directory · Capability: str_listings"</p>
                            <p class="text-[9px] text-on-surface-variant/50 mt-1">"Granted Jun 3, 2026 by admin@atlas.com"</p>
                        </div>
                        <div class="text-right">
                            <span class="text-sm font-bold text-on-surface font-mono">"6"</span>
                            <span class="text-xs text-on-surface-variant">" / 34 listings"</span>
                        </div>
                    </div>
                    <div class="space-y-1">
                        <div class="flex justify-between text-[10px] text-on-surface-variant">
                            <span>"Syncing 6 of 34 listings"</span>
                            <span class="font-bold">"18%"</span>
                        </div>
                        <div class="w-full h-1 bg-surface-container-low rounded-full overflow-hidden">
                            <div class="h-full bg-emerald-400" style="width: 18%"></div>
                        </div>
                    </div>
                    <div class="flex gap-2 pt-2">
                        <button class="btn-ghost px-3 py-1.5 rounded-lg border border-outline-variant/30 text-[10px] font-bold uppercase tracking-wider">"View Synced Listings"</button>
                        <button 
                            class="bg-error-container/20 border border-error/30 text-error hover:bg-error-container/30 px-3 py-1.5 rounded-lg text-[10px] font-bold uppercase tracking-wider active:scale-95 transition-all"
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
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm p-6 space-y-4">
                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/10 pb-4">"Grant History"</h3>
                <div class="divide-y divide-outline-variant/10">
                    <div class="py-3 flex items-center justify-between text-xs">
                        <div class="flex items-center gap-3">
                            <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
                            <span class="text-on-surface font-medium">"Granted access to leira Rentals"</span>
                        </div>
                        <span class="text-on-surface-variant/60 font-mono">"Jun 3, 2026 · by admin@atlas.com"</span>
                    </div>
                    <div class="py-3 flex items-center justify-between text-xs">
                        <div class="flex items-center gap-3">
                            <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
                            <span class="text-on-surface font-medium">"Granted access to leira Stays"</span>
                        </div>
                        <span class="text-on-surface-variant/60 font-mono">"Jun 3, 2026 · by admin@atlas.com"</span>
                    </div>
                </div>
            </div>

            // ── Modal: Grant Syndication ──
            <Show when=move || show_grant_modal.get()>
                <div class="fixed inset-0 z-[1000] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
                    <div class="bg-surface-container border border-outline-variant/20 rounded-2xl p-6 max-w-md w-full shadow-2xl space-y-4">
                        <div>
                            <h3 class="text-lg font-bold text-on-surface">"Grant Syndication Access"</h3>
                            <p class="text-xs text-on-surface-variant">"Select a network instance to syndicate Oakwood PM's listings to."</p>
                        </div>
                        
                        <div class="space-y-4">
                            <div class="space-y-1">
                                <label class="text-xs font-semibold text-on-surface-variant">"Network Instance"</label>
                                <select 
                                    class="w-full bg-surface-container-high border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 cursor-pointer outline-none"
                                    on:change=move |ev| selected_grant_instance.set(event_target_value(&ev))
                                >
                                    <option value="">"— choose —"</option>
                                    <option value="leira Pros">"leira Pros (leira-pros.app)"</option>
                                    <option value="leira Miami">"leira Miami (leira-miami.app)"</option>
                                </select>
                            </div>
                        </div>

                        <div class="flex justify-end gap-3 pt-4 border-t border-outline-variant/10">
                            <button 
                                class="btn-ghost px-4 py-2 rounded-lg text-sm font-semibold border border-outline-variant/30 hover:bg-surface-bright/20 transition-all"
                                on:click=move |_| show_grant_modal.set(false)
                            >
                                "Cancel"
                            </button>
                            <button 
                                class="btn-primary px-4 py-2 rounded-lg text-sm font-semibold transition-all"
                                on:click=handle_grant
                            >
                                "Grant Access"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Modal: Revoke Syndication ──
            <Show when=move || show_revoke_modal.get()>
                <div class="fixed inset-0 z-[1000] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
                    <div class="bg-surface-container border border-outline-variant/20 rounded-2xl p-6 max-w-md w-full shadow-2xl space-y-4">
                        <div>
                            <h3 class="text-lg font-bold text-error">"Revoke Syndication Access?"</h3>
                            <p class="text-xs text-on-surface-variant mt-2 leading-relaxed">
                                "Revoking access will immediately remove Oakwood PM's listings from " <strong class="text-on-surface">{move || revoke_target.get()}</strong> ". This cannot be undone automatically."
                            </p>
                        </div>

                        <div class="flex justify-end gap-3 pt-4 border-t border-outline-variant/10">
                            <button 
                                class="btn-ghost px-4 py-2 rounded-lg text-sm font-semibold border border-outline-variant/30 hover:bg-surface-bright/20 transition-all"
                                on:click=move |_| show_revoke_modal.set(false)
                            >
                                "Cancel"
                            </button>
                            <button 
                                class="bg-error-container/20 border border-error/30 text-error hover:bg-error-container/30 px-4 py-2 rounded-lg text-sm font-semibold transition-all active:scale-95"
                                on:click=handle_revoke
                            >
                                "Revoke Access"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
