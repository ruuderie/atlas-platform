use leptos::prelude::*;
use uuid::Uuid;
use crate::api::admin::{get_all_passkeys, revoke_passkey_admin, PasskeyAdminModel};
use crate::app::GlobalToast;

#[component]
pub fn SecurityPasskeys() -> impl IntoView {
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");
    let refetch = RwSignal::new(0u32);
    let search = RwSignal::new(String::new());

    let passkeys_res = LocalResource::new(move || {
        let _ = refetch.get();
        async move { get_all_passkeys(None).await.unwrap_or_default() }
    });

    let revoke_action = Action::new_local(move |id: &Uuid| {
        let t = toast.clone();
        let pk_id = *id;
        async move {
            match revoke_passkey_admin(pk_id).await {
                Ok(()) => {
                    t.show_toast("Revoked", "Passkey has been revoked.", "success");
                    refetch.update(|v| *v += 1);
                }
                Err(e) => t.show_toast("Error", &e, "error"),
            }
        }
    });

    let filtered = Signal::derive(move || {
        let pks = passkeys_res.get().unwrap_or_default();
        let q = search.get().to_lowercase();
        if q.is_empty() {
            pks
        } else {
            pks.into_iter()
                .filter(|pk| {
                    pk.user_email.to_lowercase().contains(&q)
                        || pk.name.to_lowercase().contains(&q)
                })
                .collect()
        }
    });

    view! {
        <div class="space-y-8 animate-in slide-in-from-bottom-4 duration-500 ease-out fade-in">
            // Header
            <header class="flex justify-between items-center bg-surface-container border border-outline-variant/10 p-6 rounded-2xl shadow-sm">
                <div>
                    <h1 class="text-3xl font-light tracking-tight text-on-surface mb-2 font-['Inter']">
                        "Passkey Registry"
                    </h1>
                    <p class="text-on-surface-variant text-sm tracking-wide">
                        "All WebAuthn passkeys registered across the platform. Revoke any credential instantly."
                    </p>
                </div>
                <button
                    on:click=move |_| refetch.update(|v| *v += 1)
                    class="flex items-center gap-2 px-4 py-2 rounded-lg border border-outline/20 text-sm font-medium hover:bg-surface-bright/10 transition-all"
                >
                    <span class="material-symbols-outlined text-sm">"refresh"</span>
                    "Refresh"
                </button>
            </header>

            // KPI strip
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <div class="p-5 rounded-2xl bg-surface-container border border-outline-variant/10 shadow-sm flex flex-col gap-1">
                    <span class="text-3xl font-bold text-primary">
                        {move || passkeys_res.get().map(|p| p.len()).unwrap_or(0).to_string()}
                    </span>
                    <span class="text-sm text-on-surface-variant">"Total Passkeys"</span>
                </div>
                <div class="p-5 rounded-2xl bg-surface-container border border-outline-variant/10 shadow-sm flex flex-col gap-1">
                    <span class="text-3xl font-bold text-success">
                        {move || {
                            passkeys_res.get().map(|pks| {
                                let mut users = std::collections::HashSet::new();
                                for pk in &pks { users.insert(pk.user_id); }
                                users.len()
                            }).unwrap_or(0).to_string()
                        }}
                    </span>
                    <span class="text-sm text-on-surface-variant">"Users with Passkeys"</span>
                </div>
                <div class="p-5 rounded-2xl bg-surface-container border border-outline-variant/10 shadow-sm flex flex-col gap-1">
                    <span class="text-3xl font-bold text-on-surface">
                        {move || {
                            passkeys_res.get().map(|pks| {
                                pks.iter().map(|pk| pk.sign_count).sum::<i32>()
                            }).unwrap_or(0).to_string()
                        }}
                    </span>
                    <span class="text-sm text-on-surface-variant">"Total Auth Events"</span>
                </div>
            </div>

            // Search + table
            <section class="p-6 rounded-2xl bg-surface-container border border-outline-variant/10 shadow-sm">
                <div class="flex items-center gap-4 mb-6">
                    <div class="relative flex-1 max-w-sm">
                        <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-on-surface-variant text-base">"search"</span>
                        <input
                            type="text"
                            placeholder="Search by email or device name…"
                            class="w-full pl-9 pr-4 py-2.5 bg-surface-container-highest border border-outline/20 rounded-lg text-sm text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/30"
                            prop:value=move || search.get()
                            on:input=move |ev| search.set(event_target_value(&ev))
                        />
                    </div>
                </div>

                <Suspense fallback=move || view! {
                    <div class="flex items-center justify-center p-12 text-on-surface-variant text-sm gap-3">
                        <span class="material-symbols-outlined animate-spin text-primary">"progress_activity"</span>
                        "Loading passkeys…"
                    </div>
                }>
                    {move || {
                        let pks = filtered.get();
                        if pks.is_empty() {
                            view! {
                                <div class="text-center py-12 text-sm text-on-surface-variant border border-dashed border-outline-variant/30 rounded-xl">
                                    "No passkeys found."
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto rounded-xl border border-outline-variant/10">
                                    <table class="w-full text-sm text-left">
                                        <thead class="bg-surface-container-highest text-on-surface-variant text-xs uppercase tracking-wider">
                                            <tr>
                                                <th class="px-4 py-3 font-medium">"User"</th>
                                                <th class="px-4 py-3 font-medium">"Device Name"</th>
                                                <th class="px-4 py-3 font-medium text-center">"Sign Count"</th>
                                                <th class="px-4 py-3 font-medium">"Last Used"</th>
                                                <th class="px-4 py-3 font-medium">"Registered"</th>
                                                <th class="px-4 py-3 font-medium text-right">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-outline-variant/10">
                                            <For
                                                each=move || pks.clone()
                                                key=|pk| pk.id
                                                children=move |pk: PasskeyAdminModel| {
                                                    let pk_id = pk.id;
                                                    let last_used = pk.last_used_at.as_deref().unwrap_or("—").to_string();
                                                    let created = &pk.created_at[..10];
                                                    let created = created.to_string();
                                                    view! {
                                                        <tr class="hover:bg-surface-bright/5 transition-colors">
                                                            <td class="px-4 py-3.5 font-medium text-on-surface">
                                                                <div class="flex flex-col">
                                                                    <span>{pk.user_email.clone()}</span>
                                                                    <span class="font-mono text-[10px] text-on-surface-variant">{pk.user_id.to_string().chars().take(8).collect::<String>()}"…"</span>
                                                                </div>
                                                            </td>
                                                            <td class="px-4 py-3.5 text-on-surface-variant">{pk.name}</td>
                                                            <td class="px-4 py-3.5 text-center">
                                                                <span class="px-2.5 py-0.5 rounded-full bg-primary/10 text-primary text-xs font-bold">
                                                                    {pk.sign_count.to_string()}
                                                                </span>
                                                            </td>
                                                            <td class="px-4 py-3.5 text-on-surface-variant text-xs">{last_used}</td>
                                                            <td class="px-4 py-3.5 text-on-surface-variant text-xs">{created}</td>
                                                            <td class="px-4 py-3.5 text-right">
                                                                <button
                                                                    on:click=move |_| { revoke_action.dispatch(pk_id); }
                                                                    class="px-3 py-1.5 text-xs font-semibold text-error border border-error/30 rounded-lg hover:bg-error/10 transition-all"
                                                                >
                                                                    "Revoke"
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }
                                                }
                                            />
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any()
                        }
                    }}
                </Suspense>
            </section>

            // Info callout
            <div class="p-4 rounded-xl bg-primary-container/20 border border-primary/20 text-sm text-on-surface flex items-start gap-3">
                <span class="material-symbols-outlined text-primary mt-0.5 shrink-0">"info"</span>
                <div>
                    <span class="font-semibold">"Security note: "</span>
                    "Revoking a passkey is immediate and irreversible. The user will need to register a new passkey to use WebAuthn login. Email / password login is unaffected."
                </div>
            </div>
        </div>
    }
}
