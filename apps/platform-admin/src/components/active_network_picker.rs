//! Compact Active network picker for Admin ops pages (Integrations, Developer, Settings API keys).
//! Binds to the global `active_network` / `set_active_network` context from `app.rs`.

use crate::api::models::PlatformAppModel;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn ActiveNetworkPicker(
    /// Optional short hint under the control.
    #[prop(optional)]
    hint: Option<&'static str>,
) -> impl IntoView {
    let dirs_res = use_context::<LocalResource<Vec<PlatformAppModel>>>().expect("dirs context");
    let active_network =
        use_context::<ReadSignal<Option<Uuid>>>().expect("active network context");
    let set_active_network =
        use_context::<WriteSignal<Option<Uuid>>>().expect("set active network context");

    let hint_text = hint.unwrap_or(
        "API credentials and webhooks are scoped to a tenant. Select a network to load live data.",
    );

    view! {
        <div
            class="tenant-bar"
            style="display:flex;align-items:center;gap:10px;flex-wrap:wrap;padding:10px 12px;background:var(--bg-surface);border:1px solid var(--border-default);border-radius:6px;"
        >
            <span
                style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);"
            >
                "Active network"
            </span>
            <select
                class="bg-[#1C2236] border border-outline-variant/30 text-on-surface text-xs rounded px-2.5 py-1.5 focus:ring-1 focus:ring-primary focus:border-primary max-w-[260px]"
                prop:value=move || {
                    active_network
                        .get()
                        .map(|id| id.to_string())
                        .unwrap_or_default()
                }
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    if val.is_empty() {
                        set_active_network.set(None);
                    } else if let Ok(parsed) = Uuid::parse_str(&val) {
                        set_active_network.set(Some(parsed));
                    }
                }
            >
                <option value="">"None selected"</option>
                <Suspense fallback=move || view! { <option>"Loading…"</option> }>
                    {move || {
                        dirs_res.get().map(|networks| {
                            view! {
                                <For
                                    each=move || networks.clone()
                                    key=|dir| dir.tenant_id.clone()
                                    children=move |dir| {
                                        let tid = dir.tenant_id.clone();
                                        let label = format!("{} · {}", dir.name, dir.app_type);
                                        view! {
                                            <option value=tid>{label}</option>
                                        }
                                    }
                                />
                            }
                        })
                    }}
                </Suspense>
            </select>
            <Show when=move || active_network.get().is_none()>
                <span style="font-size:11px;color:var(--amber);">{hint_text}</span>
            </Show>
            <Show when=move || active_network.get().is_some()>
                <a
                    href="/internal-instances"
                    class="btn btn-ghost btn-sm"
                    style="margin-left:auto;font-size:11px;"
                >
                    "Manage instances →"
                </a>
            </Show>
        </div>
    }
}
