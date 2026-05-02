use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use uuid::Uuid;

use crate::api::menus::{
    list_menus, create_menu, update_menu, delete_menu,
    AppMenu, CreateMenuPayload, UpdateMenuPayload,
};

/// Parses the tenant_id from the query string (?tenant_id=...).
fn get_tenant_id() -> Uuid {
    let query = use_query_map();
    query
        .get_untracked()
        .get("tenant_id")
        .and_then(|s| s.parse::<Uuid>().ok())
        .unwrap_or(Uuid::nil())
}

#[component]
pub fn MenuEditor() -> impl IntoView {
    let tenant_id = get_tenant_id();

    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // ── Data fetch ────────────────────────────────────────────────────────────
    let (trigger_fetch, set_trigger_fetch) = signal(0u32);
    let menus_res = LocalResource::new(move || {
        trigger_fetch.get();
        async move { list_menus(tenant_id).await.unwrap_or_default() }
    });

    // ── Form signals (new menu item) ──────────────────────────────────────────
    let new_label       = RwSignal::new("".to_string());
    let new_href        = RwSignal::new("/".to_string());
    let new_menu_type   = RwSignal::new("header".to_string());
    let new_order       = RwSignal::new(0i32);
    let new_visible     = RwSignal::new(true);

    // ── Edit state ────────────────────────────────────────────────────────────
    let editing_id: RwSignal<Option<Uuid>>   = RwSignal::new(None);
    let edit_label      = RwSignal::new("".to_string());
    let edit_href       = RwSignal::new("".to_string());
    let edit_order      = RwSignal::new(0i32);
    let edit_visible    = RwSignal::new(true);

    // ── Handlers ──────────────────────────────────────────────────────────────
    let handle_create = move |_| {
        if new_label.get().trim().is_empty() {
            toast.message.set(Some("Label is required.".to_string()));
            return;
        }
        leptos::task::spawn_local(async move {
            let payload = CreateMenuPayload {
                menu_type:     new_menu_type.get(),
                label:         new_label.get(),
                href:          Some(new_href.get()),
                parent_id:     None,
                display_order: Some(new_order.get()),
                is_visible:    Some(new_visible.get()),
            };
            match create_menu(tenant_id, payload).await {
                Ok(_) => {
                    set_trigger_fetch.update(|v| *v += 1);
                    new_label.set("".to_string());
                    new_href.set("/".to_string());
                    new_order.set(0);
                    new_visible.set(true);
                    toast.message.set(Some("Menu item created.".to_string()));
                }
                Err(e) => toast.message.set(Some(e)),
            }
        });
    };

    let begin_edit = move |menu: AppMenu| {
        editing_id.set(Some(menu.id));
        edit_label.set(menu.label.clone());
        edit_href.set(menu.href.unwrap_or_else(|| "/".to_string()));
        edit_order.set(menu.display_order);
        edit_visible.set(menu.is_visible);
    };

    let handle_save_edit = move |_| {
        let Some(id) = editing_id.get() else { return };
        leptos::task::spawn_local(async move {
            let payload = UpdateMenuPayload {
                label:         Some(edit_label.get()),
                href:          Some(edit_href.get()),
                parent_id:     None,
                display_order: Some(edit_order.get()),
                is_visible:    Some(edit_visible.get()),
            };
            match update_menu(tenant_id, id, payload).await {
                Ok(_) => {
                    editing_id.set(None);
                    set_trigger_fetch.update(|v| *v += 1);
                    toast.message.set(Some("Menu item updated.".to_string()));
                }
                Err(e) => toast.message.set(Some(e)),
            }
        });
    };

    let handle_delete = move |id: Uuid| {
        leptos::task::spawn_local(async move {
            match delete_menu(tenant_id, id).await {
                Ok(_) => {
                    set_trigger_fetch.update(|v| *v += 1);
                    toast.message.set(Some("Menu item deleted.".to_string()));
                }
                Err(e) => toast.message.set(Some(e)),
            }
        });
    };

    view! {
        <div class="space-y-8">
            // ── Header ──
            <div class="flex flex-col md:flex-row md:items-end justify-between gap-4">
                <div>
                    <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                        <span>"Content"</span>
                        <span class="material-symbols-outlined text-xs">"chevron_right"</span>
                        <span class="text-primary/70">"Navigation Menus"</span>
                    </nav>
                    <h1 class="text-4xl font-extrabold tracking-tight text-on-surface mb-2">"Navigation Menus"</h1>
                    <p class="text-on-surface-variant text-sm max-w-2xl">
                        "Manage the navigation menus provisioned for this tenant. Items created here are "
                        "immediately available to the live site via "
                        <code class="text-primary/80 text-xs bg-surface-container-high px-1 rounded">
                            "GET /api/public/menus/{tenant_id}"
                        </code>
                        "."
                    </p>
                </div>
                // Tenant warning
                {if tenant_id == Uuid::nil() {
                    view! {
                        <div class="flex items-center gap-2 px-4 py-2 bg-error-container text-error rounded-lg text-xs shrink-0">
                            <span class="material-symbols-outlined text-sm">"warning"</span>
                            "No tenant — add ?tenant_id=<uuid>"
                        </div>
                    }.into_any()
                } else {
                    view! { <span /> }.into_any()
                }}
            </div>

            <div class="grid grid-cols-1 xl:grid-cols-3 gap-8">
                // ── LEFT: Menu Table ──────────────────────────────────────────
                <div class="xl:col-span-2 space-y-6">
                    <Suspense fallback=move || view! {
                        <div class="flex items-center gap-2 text-on-surface-variant text-sm">
                            <span class="material-symbols-outlined animate-spin text-primary">"progress_activity"</span>
                            "Loading menus..."
                        </div>
                    }>
                        {move || menus_res.get().map(|menus: Vec<AppMenu>| {
                            // Group by menu_type
                            let mut groups: std::collections::BTreeMap<String, Vec<AppMenu>> = std::collections::BTreeMap::new();
                            for m in menus {
                                groups.entry(m.menu_type.clone()).or_default().push(m);
                            }
                            let groups_vec: Vec<(String, Vec<AppMenu>)> = groups.into_iter().collect();

                            if groups_vec.is_empty() {
                                view! {
                                    <div class="flex flex-col items-center justify-center py-20 bg-surface-container rounded-2xl border border-dashed border-outline-variant/30">
                                        <span class="material-symbols-outlined text-on-surface-variant text-5xl mb-4">"menu_open"</span>
                                        <p class="text-on-surface font-bold text-lg mb-1">"No menu items yet"</p>
                                        <p class="text-on-surface-variant text-sm text-center max-w-xs">
                                            "A default Header menu was seeded during provisioning. "
                                            "If none appear, run the provisioning step for this tenant."
                                        </p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="space-y-6">
                                        <For
                                            each=move || groups_vec.clone()
                                            key=|(t, _)| t.clone()
                                            children=move |(menu_type, items)| {
                                                let icon = match menu_type.as_str() {
                                                    "header" => "menu",
                                                    "footer" => "bottom_panel_open",
                                                    "sidebar" => "view_sidebar",
                                                    _ => "more_horiz",
                                                };
                                                view! {
                                                    <div class="bg-surface-container rounded-2xl border border-outline-variant/20 overflow-hidden">
                                                        // Group header
                                                        <div class="flex items-center gap-3 px-6 py-4 border-b border-outline-variant/10 bg-surface-container-low">
                                                            <span class="material-symbols-outlined text-primary">{icon}</span>
                                                            <h2 class="font-bold text-on-surface uppercase tracking-wider text-sm">
                                                                {menu_type.clone()}" Navigation"
                                                            </h2>
                                                            <span class="ml-auto text-[10px] text-on-surface-variant bg-surface-container-high px-2 py-0.5 rounded-full">
                                                                {items.len()}" items"
                                                            </span>
                                                        </div>
                                                        // Items table
                                                        <table class="w-full text-left">
                                                            <thead>
                                                                <tr class="text-[10px] uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/5 bg-surface-container-low/50">
                                                                    <th class="px-6 py-3 font-bold">"Order"</th>
                                                                    <th class="px-4 py-3 font-bold">"Label"</th>
                                                                    <th class="px-4 py-3 font-bold">"URL"</th>
                                                                    <th class="px-4 py-3 font-bold">"Visible"</th>
                                                                    <th class="px-4 py-3 font-bold text-right">"Actions"</th>
                                                                </tr>
                                                            </thead>
                                                            <tbody>
                                                                <For
                                                                    each=move || items.clone()
                                                                    key=|m| m.id
                                                                    children=move |menu| {
                                                                        // Extract all non-Copy fields before any closures so we
                                                                        // don't accidentally make this FnOnce by moving menu.
                                                                        let menu_id        = menu.id;
                                                                        let menu_order     = menu.display_order;
                                                                        let menu_label     = menu.label.clone();
                                                                        let menu_href      = menu.href.clone().unwrap_or_else(|| "—".to_string());
                                                                        let menu_visible   = menu.is_visible;
                                                                        // Snapshot for begin_edit — cloned once, not reactive
                                                                        let menu_snap      = menu.clone();

                                                                        let is_editing = Signal::derive(move || editing_id.get() == Some(menu_id));

                                                                        view! {
                                                                            <tr class="border-b border-outline-variant/5 hover:bg-surface-container-high/50 transition-colors group">
                                                                                {move || if is_editing.get() {
                                                                                    view! {
                                                                                        <td class="px-6 py-3">
                                                                                            <input
                                                                                                type="number"
                                                                                                class="w-16 bg-surface-container-high border border-primary/30 rounded px-2 py-1 text-sm text-on-surface"
                                                                                                prop:value=move || edit_order.get()
                                                                                                on:input=move |ev| { if let Ok(n) = event_target_value(&ev).parse::<i32>() { edit_order.set(n); } }
                                                                                            />
                                                                                        </td>
                                                                                        <td class="px-4 py-3">
                                                                                            <input
                                                                                                type="text"
                                                                                                class="w-full bg-surface-container-high border border-primary/30 rounded px-2 py-1 text-sm text-on-surface"
                                                                                                prop:value=move || edit_label.get()
                                                                                                on:input=move |ev| edit_label.set(event_target_value(&ev))
                                                                                            />
                                                                                        </td>
                                                                                        <td class="px-4 py-3">
                                                                                            <input
                                                                                                type="text"
                                                                                                class="w-full bg-surface-container-high border border-primary/30 rounded px-2 py-1 text-sm font-mono text-on-surface"
                                                                                                prop:value=move || edit_href.get()
                                                                                                on:input=move |ev| edit_href.set(event_target_value(&ev))
                                                                                            />
                                                                                        </td>
                                                                                        <td class="px-4 py-3">
                                                                                            <input
                                                                                                type="checkbox"
                                                                                                class="rounded border-outline-variant text-primary focus:ring-0"
                                                                                                prop:checked=move || edit_visible.get()
                                                                                                on:change=move |ev| edit_visible.set(event_target_checked(&ev))
                                                                                            />
                                                                                        </td>
                                                                                        <td class="px-4 py-3 text-right">
                                                                                            <div class="flex items-center justify-end gap-2">
                                                                                                <button
                                                                                                    class="px-3 py-1 text-[10px] font-bold uppercase tracking-wider bg-primary text-on-primary rounded-md hover:opacity-90 transition-all"
                                                                                                    on:click=handle_save_edit
                                                                                                >
                                                                                                    "Save"
                                                                                                </button>
                                                                                                <button
                                                                                                    class="px-3 py-1 text-[10px] font-bold uppercase tracking-wider bg-surface-container-high text-on-surface rounded-md hover:opacity-90 transition-all"
                                                                                                    on:click=move |_| editing_id.set(None)
                                                                                                >
                                                                                                    "Cancel"
                                                                                                </button>
                                                                                            </div>
                                                                                        </td>
                                                                                    }.into_any()
                                                                                } else {
                                                                                    // Clones for display — these are captured by value not by ref
                                                                                    let label_d  = menu_label.clone();
                                                                                    let href_d   = menu_href.clone();
                                                                                    let snap     = menu_snap.clone();
                                                                                    view! {
                                                                                        <td class="px-6 py-3">
                                                                                            <span class="inline-flex w-7 h-7 items-center justify-center rounded-full bg-surface-container-high text-xs font-bold text-on-surface-variant">
                                                                                                {menu_order}
                                                                                            </span>
                                                                                        </td>
                                                                                        <td class="px-4 py-3 font-bold text-sm text-on-surface">
                                                                                            {label_d}
                                                                                        </td>
                                                                                        <td class="px-4 py-3">
                                                                                            <span class="text-xs text-on-surface-variant font-mono">{href_d}</span>
                                                                                        </td>
                                                                                        <td class="px-4 py-3">
                                                                                            {if menu_visible {
                                                                                                view! { <span class="px-2 py-0.5 text-[10px] font-bold bg-tertiary/10 text-tertiary rounded-full uppercase">"Visible"</span> }.into_any()
                                                                                            } else {
                                                                                                view! { <span class="px-2 py-0.5 text-[10px] font-bold bg-surface-container-high text-on-surface-variant rounded-full uppercase">"Hidden"</span> }.into_any()
                                                                                            }}
                                                                                        </td>
                                                                                        <td class="px-4 py-3 text-right opacity-0 group-hover:opacity-100 transition-opacity">
                                                                                            <div class="flex items-center justify-end gap-2">
                                                                                                <button
                                                                                                    class="p-1.5 rounded-md bg-surface-container hover:bg-surface-bright/50 text-on-surface-variant hover:text-primary transition-all"
                                                                                                    on:click=move |_| begin_edit(snap.clone())
                                                                                                >
                                                                                                    <span class="material-symbols-outlined text-[16px]">"edit"</span>
                                                                                                </button>
                                                                                                <button
                                                                                                    class="p-1.5 rounded-md bg-surface-container hover:bg-error-container/20 text-on-surface-variant hover:text-error transition-all"
                                                                                                    on:click=move |_| handle_delete(menu_id)
                                                                                                >
                                                                                                    <span class="material-symbols-outlined text-[16px]">"delete"</span>
                                                                                                </button>
                                                                                            </div>
                                                                                        </td>
                                                                                    }.into_any()
                                                                                }}
                                                                            </tr>
                                                                        }
                                                                    }
                                                                />
                                                            </tbody>
                                                        </table>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </div>

                // ── RIGHT: Add Item Form ──────────────────────────────────────
                <div class="space-y-4">
                    <div class="bg-surface-container rounded-2xl border border-outline-variant/20 p-6 sticky top-4">
                        <div class="flex items-center gap-2 mb-6">
                            <span class="material-symbols-outlined text-primary">"add_circle"</span>
                            <h3 class="font-bold text-on-surface text-lg">"Add Menu Item"</h3>
                        </div>
                        <div class="space-y-4">
                            // Menu Type
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Menu Type"</label>
                                <select
                                    class="w-full bg-surface-container-high border-none rounded-lg p-3 text-sm text-on-surface focus:ring-1 focus:ring-primary appearance-none"
                                    on:change=move |ev| new_menu_type.set(event_target_value(&ev))
                                    prop:value=move || new_menu_type.get()
                                >
                                    <option value="header">"Header"</option>
                                    <option value="footer">"Footer"</option>
                                    <option value="sidebar">"Sidebar"</option>
                                </select>
                            </div>
                            // Label
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Label"</label>
                                <input
                                    type="text"
                                    placeholder="Home"
                                    class="w-full bg-surface-container-high border-none rounded-lg p-3 text-sm text-on-surface focus:ring-1 focus:ring-primary transition-all"
                                    prop:value=move || new_label.get()
                                    on:input=move |ev| new_label.set(event_target_value(&ev))
                                />
                            </div>
                            // URL
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"URL / Path"</label>
                                <input
                                    type="text"
                                    placeholder="/"
                                    class="w-full bg-surface-container-high border-none rounded-lg p-3 text-sm font-mono text-on-surface focus:ring-1 focus:ring-primary transition-all"
                                    prop:value=move || new_href.get()
                                    on:input=move |ev| new_href.set(event_target_value(&ev))
                                />
                            </div>
                            // Display Order
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Display Order"</label>
                                <input
                                    type="number"
                                    class="w-full bg-surface-container-high border-none rounded-lg p-3 text-sm text-on-surface focus:ring-1 focus:ring-primary transition-all"
                                    prop:value=move || new_order.get()
                                    on:input=move |ev| {
                                        if let Ok(n) = event_target_value(&ev).parse::<i32>() {
                                            new_order.set(n);
                                        }
                                    }
                                />
                            </div>
                            // Visible toggle
                            <div class="flex items-center gap-3 py-2">
                                <input
                                    type="checkbox"
                                    id="new_visible"
                                    class="rounded border-outline-variant text-primary focus:ring-0"
                                    prop:checked=move || new_visible.get()
                                    on:change=move |ev| new_visible.set(event_target_checked(&ev))
                                />
                                <label for="new_visible" class="text-sm text-on-surface cursor-pointer">"Visible to site visitors"</label>
                            </div>
                            // Submit
                            <button
                                class="w-full btn-primary-gradient text-on-primary py-3 rounded-lg text-sm font-bold uppercase tracking-wider shadow-lg shadow-primary/10 hover:opacity-90 active:scale-[0.98] transition-all mt-2"
                                on:click=handle_create
                            >
                                "Add Menu Item"
                            </button>
                        </div>
                    </div>

                    // Quick tips card
                    <div class="bg-surface-container rounded-2xl border border-outline-variant/20 p-5 space-y-3">
                        <div class="flex items-center gap-2">
                            <span class="material-symbols-outlined text-secondary text-sm">"info"</span>
                            <span class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Quick Reference"</span>
                        </div>
                        <ul class="space-y-2 text-xs text-on-surface-variant">
                            <li class="flex items-start gap-2">
                                <span class="material-symbols-outlined text-[14px] mt-0.5 text-primary/60">"radio_button_unchecked"</span>
                                "Items with lower Display Order appear first."
                            </li>
                            <li class="flex items-start gap-2">
                                <span class="material-symbols-outlined text-[14px] mt-0.5 text-primary/60">"radio_button_unchecked"</span>
                                "Hidden items are excluded from the public API response."
                            </li>
                            <li class="flex items-start gap-2">
                                <span class="material-symbols-outlined text-[14px] mt-0.5 text-primary/60">"radio_button_unchecked"</span>
                                <span>
                                    "Public endpoint: "
                                    <code class="bg-surface-container-high px-1 rounded text-primary/80">"GET /api/public/menus/{tenant_id}"</code>
                                </span>
                            </li>
                            <li class="flex items-start gap-2">
                                <span class="material-symbols-outlined text-[14px] mt-0.5 text-primary/60">"radio_button_unchecked"</span>
                                <span>
                                    "Filter by type: "
                                    <code class="bg-surface-container-high px-1 rounded text-primary/80">"/tree/header"</code>
                                </span>
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    }
}
