//! G-37 Ambassadors — mint codes, dual QR card packs, fulfillment stubs.
//!
//! Route: /ambassadors

use leptos::prelude::*;

use crate::api::admin::{
    AmbassadorModel, CreateAmbassadorInput, CreateFulfillmentInput, create_ambassador,
    create_ambassador_fulfillment, download_ambassador_qr, list_ambassadors,
};

#[component]
pub fn AmbassadorsPage() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let show_create = RwSignal::new(false);
    let code = RwSignal::new(String::new());
    let display_name = RwSignal::new(String::new());
    let partner_type = RwSignal::new("referral".to_string());
    let creating = RwSignal::new(false);
    let selected_id = RwSignal::<Option<uuid::Uuid>>::new(None);
    let refresh_tick = RwSignal::new(0u32);

    let list_res = LocalResource::new(move || {
        let _ = refresh_tick.get();
        async move { list_ambassadors().await }
    });

    view! {
        <div class="main-canvas space-y-6">
            <style>{r#"
                .ambassador-press-btn {
                    transition: transform 100ms ease, opacity 120ms ease;
                }
                .ambassador-press-btn:active {
                    transform: scale(0.97);
                }
                @media (prefers-reduced-motion: reduce) {
                    .ambassador-press-btn,
                    .ambassador-press-btn:active {
                        transition: none;
                        transform: none;
                    }
                }
            "#}</style>

            <div class="flex items-start justify-between flex-wrap gap-4">
                <div>
                    <h1 class="page-title">"Ambassadors"</h1>
                    <p class="text-xs text-on-surface-variant mt-1 max-w-xl">
                        "Mint partner codes for Friends & Family card packs. Each ambassador gets landlord and vendor QR links."
                    </p>
                </div>
                <button class="btn btn-primary" on:click=move |_| show_create.set(true)>
                    "+ New ambassador"
                </button>
            </div>

            <Show when=move || show_create.get()>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 space-y-4 max-w-lg">
                    <h2 class="text-sm font-bold text-on-surface">"Create ambassador"</h2>
                    <div>
                        <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Code"</label>
                        <input
                            type="text"
                            placeholder="alice"
                            class="w-full bg-surface-container border border-outline-variant/30 rounded-lg px-3 py-2 text-xs font-mono"
                            prop:value=move || code.get()
                            on:input=move |ev| code.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Display name"</label>
                        <input
                            type="text"
                            placeholder="Alice Chen"
                            class="w-full bg-surface-container border border-outline-variant/30 rounded-lg px-3 py-2 text-xs"
                            prop:value=move || display_name.get()
                            on:input=move |ev| display_name.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Partner type"</label>
                        <select
                            class="w-full bg-surface-container border border-outline-variant/30 rounded-lg px-3 py-2 text-xs"
                            on:change=move |ev| partner_type.set(event_target_value(&ev))
                        >
                            <option value="referral">"Referral"</option>
                            <option value="influencer">"Influencer"</option>
                            <option value="affiliate">"Affiliate"</option>
                            <option value="recruiter">"Recruiter"</option>
                        </select>
                    </div>
                    <div class="flex gap-2">
                        <button
                            class="btn btn-primary"
                            disabled=move || creating.get()
                            on:click={
                                let toast = toast.clone();
                                move |_| {
                                    let toast = toast.clone();
                                    let c = code.get().trim().to_string();
                                    let n = display_name.get().trim().to_string();
                                    if c.is_empty() || n.is_empty() {
                                        toast.show_toast("Missing fields", "Code and display name are required.", "error");
                                        return;
                                    }
                                    creating.set(true);
                                    let input = CreateAmbassadorInput {
                                        code: c,
                                        display_name: n,
                                        partner_type: partner_type.get(),
                                        notes: None,
                                        campaign_ids: vec![],
                                    };
                                    leptos::task::spawn_local(async move {
                                        match create_ambassador(input).await {
                                            Ok(m) => {
                                                toast.show_toast(
                                                    "Ambassador created",
                                                    &format!("{} attached to F&F campaigns", m.code),
                                                    "success",
                                                );
                                                show_create.set(false);
                                                code.set(String::new());
                                                display_name.set(String::new());
                                                selected_id.set(Some(m.id));
                                                refresh_tick.update(|t| *t += 1);
                                            }
                                            Err(e) => toast.show_toast("Create failed", &e, "error"),
                                        }
                                        creating.set(false);
                                    });
                                }
                            }
                        >
                            {move || if creating.get() { "Creating…" } else { "Create" }}
                        </button>
                        <button class="btn btn-ghost" on:click=move |_| show_create.set(false)>"Cancel"</button>
                    </div>
                </div>
            </Show>

            <div class="grid grid-cols-1 xl:grid-cols-5 gap-6">
                <div class="xl:col-span-2 space-y-2">
                    <Suspense fallback=|| view! { <p class="text-xs text-on-surface-variant/60 animate-pulse">"Loading…"</p> }>
                        {move || list_res.get().map(|res| match res {
                            Err(e) => view! {
                                <p class="text-xs text-error">{e}</p>
                            }.into_any(),
                            Ok(list) if list.is_empty() => view! {
                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-8 text-center">
                                    <p class="text-sm font-semibold text-on-surface-variant">"No ambassadors yet"</p>
                                    <p class="text-xs text-on-surface-variant/60 mt-1">"Create one to mint dual QR card packs."</p>
                                </div>
                            }.into_any(),
                            Ok(list) => view! {
                                    <div class="space-y-2">
                                        {list.into_iter().map(|a| {
                                            let id = a.id;
                                            view! {
                                                <button
                                                    class="w-full text-left bg-surface-container-low border border-outline-variant/20 rounded-xl px-4 py-3 hover:border-primary/40 transition-colors ambassador-press-btn"
                                                    on:click=move |_| selected_id.set(Some(id))
                                                >
                                                    <div class="flex items-center justify-between gap-2">
                                                        <div>
                                                            <div class="text-sm font-semibold text-on-surface">{a.display_name.clone()}</div>
                                                            <div class="text-[11px] font-mono text-primary/80 mt-0.5">{a.code.clone()}</div>
                                                        </div>
                                                        <span class="text-[9px] uppercase tracking-wider text-on-surface-variant border border-outline-variant/30 rounded px-1.5 py-0.5">
                                                            {a.partner_type.clone()}
                                                        </span>
                                                    </div>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any(),
                        })}
                    </Suspense>
                </div>

                <div class="xl:col-span-3">
                    <Suspense fallback=|| view! { <p class="text-xs text-on-surface-variant/60">""</p> }>
                        {move || list_res.get().map(|res| {
                            let Ok(list) = res else {
                                return view! { <div></div> }.into_any();
                            };
                            match selected_id.get().and_then(|sid| list.into_iter().find(|a| a.id == sid)) {
                                None => view! {
                                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-10 text-center text-xs text-on-surface-variant/60">
                                        "Select an ambassador to download QRs or request card packs."
                                    </div>
                                }.into_any(),
                                Some(a) => view! {
                                    <AmbassadorDetail ambassador=a on_changed=Callback::new(move |_| {
                                        refresh_tick.update(|t| *t += 1);
                                    }) />
                                }.into_any(),
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

#[component]
fn AmbassadorDetail(
    ambassador: AmbassadorModel,
    on_changed: Callback<()>,
) -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let a = RwSignal::new(ambassador);
    let requesting = RwSignal::new(false);
    let id = a.get_untracked().id;

    view! {
        <div class="space-y-4">
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5">
                <h2 class="text-lg font-extrabold text-on-surface tracking-tight">{move || a.get().display_name}</h2>
                <p class="text-xs font-mono text-primary/80 mt-1">{move || a.get().code}</p>
                <p class="text-[10px] uppercase tracking-wider text-on-surface-variant mt-2">
                    {move || format!("{} · {}", a.get().partner_type, a.get().status)}
                </p>
            </div>

            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 space-y-3">
                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Share URLs"</h3>
                <div class="space-y-2">
                    <div class="flex items-center gap-2 flex-wrap">
                        <code class="text-[11px] text-emerald-400/90 break-all flex-1">{move || a.get().landlord_url}</code>
                        <button
                            class="btn btn-ghost btn-sm ambassador-press-btn"
                            on:click={
                                let toast = toast.clone();
                                move |_| {
                                    let url = a.get().landlord_url;
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        if let Some(clipboard) = web_sys::window().map(|w| w.navigator().clipboard()) {
                                            let _ = clipboard.write_text(&url);
                                            toast.show_toast("Copied", "Landlord URL copied", "success");
                                        }
                                    }
                                    #[cfg(not(target_arch = "wasm32"))]
                                    { let _ = (&toast, url); }
                                }
                            }
                        >"Copy"</button>
                    </div>
                    <div class="flex items-center gap-2 flex-wrap">
                        <code class="text-[11px] text-emerald-400/90 break-all flex-1">{move || a.get().vendor_url}</code>
                        <button
                            class="btn btn-ghost btn-sm ambassador-press-btn"
                            on:click={
                                let toast = toast.clone();
                                move |_| {
                                    let url = a.get().vendor_url;
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        if let Some(clipboard) = web_sys::window().map(|w| w.navigator().clipboard()) {
                                            let _ = clipboard.write_text(&url);
                                            toast.show_toast("Copied", "Vendor URL copied", "success");
                                        }
                                    }
                                    #[cfg(not(target_arch = "wasm32"))]
                                    { let _ = (&toast, url); }
                                }
                            }
                        >"Copy"</button>
                    </div>
                </div>
            </div>

            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 space-y-3">
                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Card pack QRs"</h3>
                <p class="text-xs text-on-surface-variant/70">"Download landlord and vendor PNGs for print (25+25 packs)."</p>
                <div class="flex flex-wrap gap-2">
                    <button
                        class="btn btn-primary btn-sm ambassador-press-btn"
                        on:click={
                            let toast = toast.clone();
                            move |_| {
                                let toast = toast.clone();
                                leptos::task::spawn_local(async move {
                                    match download_ambassador_qr(id, "landlord").await {
                                        Ok(()) => toast.show_toast("QR downloaded", "Landlord cards PNG ready", "success"),
                                        Err(e) => toast.show_toast("QR failed", &e, "error"),
                                    }
                                });
                            }
                        }
                    >"Download landlord QR"</button>
                    <button
                        class="btn btn-primary btn-sm ambassador-press-btn"
                        on:click={
                            let toast = toast.clone();
                            move |_| {
                                let toast = toast.clone();
                                leptos::task::spawn_local(async move {
                                    match download_ambassador_qr(id, "vendor").await {
                                        Ok(()) => toast.show_toast("QR downloaded", "Vendor cards PNG ready", "success"),
                                        Err(e) => toast.show_toast("QR failed", &e, "error"),
                                    }
                                });
                            }
                        }
                    >"Download vendor QR"</button>
                    <button
                        class="btn btn-ghost btn-sm ambassador-press-btn"
                        disabled=move || requesting.get()
                        on:click={
                            let toast = toast.clone();
                            move |_| {
                                let toast = toast.clone();
                                requesting.set(true);
                                leptos::task::spawn_local(async move {
                                    match create_ambassador_fulfillment(
                                        id,
                                        CreateFulfillmentInput {
                                            kind: "business_cards".into(),
                                            landlord_qty: 25,
                                            vendor_qty: 25,
                                        },
                                    )
                                    .await
                                    {
                                        Ok(updated) => {
                                            a.set(updated);
                                            toast.show_toast(
                                                "Cards requested",
                                                "25 landlord + 25 vendor (stub)",
                                                "success",
                                            );
                                            on_changed.run(());
                                        }
                                        Err(e) => toast.show_toast("Request failed", &e, "error"),
                                    }
                                    requesting.set(false);
                                });
                            }
                        }
                    >
                        {move || if requesting.get() { "Requesting…" } else { "Request cards (25+25)" }}
                    </button>
                </div>
            </div>

            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 space-y-3">
                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Fulfillment stubs"</h3>
                {move || {
                    let reqs = a.get().fulfillment_requests;
                    let arr = reqs.as_array().cloned().unwrap_or_default();
                    if arr.is_empty() {
                        view! {
                            <p class="text-xs text-on-surface-variant/60">"No fulfillment requests yet."</p>
                        }.into_any()
                    } else {
                        view! {
                            <ul class="space-y-2">
                                {arr.into_iter().map(|r| {
                                    let kind = r.get("kind").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                    let status = r.get("status").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                    let ll = r.get("landlord_qty").and_then(|v| v.as_i64()).unwrap_or(0);
                                    let vv = r.get("vendor_qty").and_then(|v| v.as_i64()).unwrap_or(0);
                                    let at = r.get("requested_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    view! {
                                        <li class="text-xs border border-outline-variant/15 rounded-lg px-3 py-2 flex items-center justify-between gap-2 flex-wrap">
                                            <span>
                                                <span class="font-semibold text-on-surface">{kind}</span>
                                                <span class="text-on-surface-variant">
                                                    {format!(" · {ll} landlord / {vv} vendor · {status}")}
                                                </span>
                                            </span>
                                            <span class="font-mono text-[10px] text-on-surface-variant/60">{at}</span>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
