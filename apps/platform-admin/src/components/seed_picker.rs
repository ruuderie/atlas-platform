use leptos::prelude::*;
use crate::api::seeds::{SeedPackInfo, get_seed_packs, apply_seed_pack};

// ──────────────────────────────────────────────────────────────────────────────
// SEED PACK CARD
// ──────────────────────────────────────────────────────────────────────────────

#[component]
fn SeedPackCard(
    pack: SeedPackInfo,
    app_instance_id: String,
    /// Called after a successful apply so the parent can refetch the list.
    on_applied: Callback<()>,
) -> impl IntoView {
    let applying = RwSignal::new(false);
    let result_msg = RwSignal::new(Option::<(bool, String)>::None); // (success, message)

    let pack_id = pack.id.clone();
    let pack_title = pack.title.clone();
    let ai = app_instance_id.clone();

    let apply = move |_| {
        let pid = pack_id.clone();
        let ai = ai.clone();
        let on_applied = on_applied.clone();
        applying.set(true);
        result_msg.set(None);
        leptos::task::spawn_local(async move {
            match apply_seed_pack(&ai, &pid).await {
                Ok(resp) if resp.success => {
                    applying.set(false);
                    result_msg.set(Some((true, resp.message)));
                    on_applied.run(());
                }
                Ok(resp) => {
                    applying.set(false);
                    result_msg.set(Some((false, resp.message)));
                }
                Err(e) => {
                    applying.set(false);
                    result_msg.set(Some((false, format!("Request failed: {e}"))));
                }
            }
        });
    };

    // Format the last_applied_at timestamp for display
    let applied_label = pack.last_applied_at.as_deref().map(|ts| {
        // Show just the date portion: "2026-04-30" from the ISO string
        ts.get(..10).unwrap_or(ts).to_string()
    });

    view! {
        <div class="bg-card border border-border rounded-xl p-5 flex flex-col gap-4 hover:border-primary/40 transition-colors">
            // ── Card header ──────────────────────────────────────────────────
            <div class="flex items-start justify-between gap-3">
                <div class="flex-1 min-w-0">
                    <h4 class="font-semibold text-foreground text-sm leading-tight">{pack.title.clone()}</h4>
                    <p class="text-muted-foreground text-xs mt-1 leading-relaxed">{pack.description.clone()}</p>
                </div>
                // Applied badge
                {applied_label.clone().map(|date| view! {
                    <span class="shrink-0 inline-flex items-center gap-1 text-xs font-medium text-emerald-700 bg-emerald-50 border border-emerald-200 rounded-full px-2.5 py-0.5">
                        "✓ " {date}
                    </span>
                })}
            </div>

            // ── Content summary pill ──────────────────────────────────────────
            <div class="flex items-center gap-2">
                <span class="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded-full font-mono">
                    {pack.content_summary.clone()}
                </span>
            </div>

            // ── Result feedback ───────────────────────────────────────────────
            {move || result_msg.get().map(|(ok, msg)| {
                let cls = if ok {
                    "text-xs text-emerald-700 bg-emerald-50 border border-emerald-200 rounded-lg px-3 py-2"
                } else {
                    "text-xs text-red-700 bg-red-50 border border-red-200 rounded-lg px-3 py-2"
                };
                view! { <p class=cls>{msg}</p> }
            })}

            // ── Action button ─────────────────────────────────────────────────
            <button
                id=format!("seed-apply-{}", pack.id)
                class="mt-auto w-full py-2 px-4 rounded-lg text-sm font-semibold transition-all \
                       bg-primary text-primary-foreground hover:bg-primary/90 \
                       disabled:opacity-50 disabled:cursor-not-allowed"
                disabled=move || applying.get()
                on:click=apply
            >
                {move || {
                    if applying.get() {
                        "Applying…".to_string()
                    } else if applied_label.is_some() {
                        "Re-apply Seed".to_string()
                    } else {
                        "Apply Seed".to_string()
                    }
                }}
            </button>
        </div>
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// SEED PICKER (main exported component)
// ──────────────────────────────────────────────────────────────────────────────

#[component]
pub fn SeedPicker(app_instance_id: String) -> impl IntoView {
    let ai = app_instance_id.clone();

    let packs: LocalResource<Result<Vec<SeedPackInfo>, String>> =
        LocalResource::new(move || {
            let ai = ai.clone();
            async move { get_seed_packs(&ai).await }
        });

    let ai_id = app_instance_id.clone();

    view! {
        <div class="space-y-4">
            <div>
                <h3 class="text-base font-semibold text-foreground">"Demo & Test Seed Data"</h3>
                <p class="text-sm text-muted-foreground mt-0.5">
                    "Populate this app instance with realistic demo data for testing or presentations. \
                     Seeds are idempotent — re-applying a pack is safe."
                </p>
            </div>

            <Suspense fallback=move || view! {
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {(0..3usize).map(|_| view! {
                        <div class="bg-muted/50 border border-border rounded-xl h-48 animate-pulse" />
                    }).collect_view()}
                </div>
            }>
                {move || {
                    let packs_result = packs.get();
                    let ai_inner = ai_id.clone();

                    match packs_result {
                        None => view! { <div /> }.into_any(),
                        Some(Err(e)) => view! {
                            <div class="bg-red-50 border border-red-200 rounded-xl p-4 text-sm text-red-700">
                                "Failed to load seed packs: " {e}
                            </div>
                        }.into_any(),
                        Some(Ok(list)) if list.is_empty() => view! {
                            <div class="bg-muted/30 border border-dashed border-border rounded-xl p-8 text-center">
                                <p class="text-muted-foreground text-sm">
                                    "No seed packs are available for this app type."
                                </p>
                            </div>
                        }.into_any(),
                        Some(Ok(list)) => {
                            view! {
                                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                                    {list.into_iter().map(|pack| {
                                        let ai = ai_inner.clone();
                                        let refetch_packs = packs.clone();
                                        let on_applied = Callback::new(move |_: ()| {
                                            refetch_packs.refetch();
                                        });
                                        view! {
                                            <SeedPackCard
                                                pack=pack
                                                app_instance_id=ai
                                                on_applied=on_applied
                                            />
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </Suspense>
        </div>
    }
}
