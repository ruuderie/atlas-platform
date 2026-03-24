use leptos::prelude::*;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};
use shared_ui::components::badge::{Badge, BadgeIntent};
use crate::api::admin::{get_users, toggle_admin, UserModel};

#[component]
pub fn PlatformAdmins() -> impl IntoView {
    let (trigger_fetch, set_trigger_fetch) = signal(0);
    let (selected_directory, set_selected_directory) = signal(None::<uuid::Uuid>);
    
    let users_res = LocalResource::new(
        move || { 
            trigger_fetch.get();
            let dir_id = selected_directory.get();
            async move { get_users(dir_id).await.unwrap_or_default() }
        }
    );

    let dirs = use_context::<LocalResource<Vec<crate::api::models::DirectoryModel>>>().expect("dirs context");

    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let handle_toggle_admin = move |id: uuid::Uuid| {
        leptos::task::spawn_local(async move {
            match toggle_admin(id).await {
                Ok(updated) => {
                    toast.message.set(Some(format!("Updated admin status for {}", updated.email)));
                    set_trigger_fetch.update(|v| *v += 1);
                }
                Err(e) => {
                    toast.message.set(Some(format!("Failed: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="space-y-8 max-w-[1200px]">
            <div class="flex flex-col md:flex-row md:items-end justify-between gap-4">
                <div class="flex-1">
                    <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                        <span>"Platform Hub"</span>
                        <span class="material-symbols-outlined text-xs">"chevron_right"</span>
                        <span class="text-primary/70">"Users"</span>
                    </nav>
                    <h1 class="text-4xl font-extrabold tracking-tight text-on-surface mb-2">"Platform Users"</h1>
                    <p class="text-on-surface-variant text-sm max-w-2xl">"Manage global accounts and filter visibility by directory instances."</p>
                </div>
                <div class="flex items-center gap-3">
                    <select
                        class="bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg focus:ring-primary focus:border-primary block p-2.5 min-w-[200px]"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            if val.is_empty() {
                                set_selected_directory.set(None);
                            } else if let Ok(parsed) = uuid::Uuid::parse_str(&val) {
                                set_selected_directory.set(Some(parsed));
                            }
                        }
                    >
                        <option value="">"All Directories"</option>
                        <Suspense fallback=move || view! { <option>"Loading..."</option> }>
                            {move || dirs.get().map(|directories| view! {
                                <For
                                    each=move || directories.clone()
                                    key=|dir| dir.id.clone()
                                    children=move |dir| {
                                        view! {
                                            <option value=dir.id.to_string()>{dir.name.clone()}</option>
                                        }
                                    }
                                />
                            })}
                        </Suspense>
                    </select>
                </div>
            </div>

            <Suspense fallback=move || view! { <div class="text-on-surface-variant">"Loading users..."</div> }>
                <div class="bg-surface-container-high rounded-xl border border-outline-variant/30 overflow-hidden">
                    <DataTable class="w-full text-sm">
                        <DataTableHeader class="bg-surface-container-highest border-b border-outline-variant/30">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-on-surface-variant">"Email"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-on-surface-variant">"Status"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-on-surface-variant">"Role"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-on-surface-variant">"Actions"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            {move || users_res.get().map(|users| view! {
                                <For
                                    each=move || users.clone()
                                    key=|u: &UserModel| u.id.clone()
                                    children=move |u| {
                                        let uid = u.id.clone();
                                        let is_admin = u.is_admin;
                                        view! {
                                            <DataTableRow class="transition-colors hover:bg-surface-container-highest data-[state=selected]:bg-muted">
                                                <DataTableCell class="p-4 align-middle font-medium text-on-surface">{u.email.clone()}</DataTableCell>
                                                <DataTableCell class="p-4 align-middle">
                                                    {if u.is_active {
                                                        view! { <Badge intent=BadgeIntent::Success>"Active"</Badge> }.into_any()
                                                    } else {
                                                        view! { <Badge intent=BadgeIntent::Default>"Inactive"</Badge> }.into_any()
                                                    }}
                                                </DataTableCell>
                                                <DataTableCell class="p-4 align-middle">
                                                    {if u.is_admin {
                                                        view! { <Badge intent=BadgeIntent::Primary>"Platform Admin"</Badge> }.into_any()
                                                    } else {
                                                        view! { <Badge intent=BadgeIntent::Default>"User"</Badge> }.into_any()
                                                    }}
                                                </DataTableCell>
                                                <DataTableCell class="p-4 align-middle text-right space-x-2">
                                                    <a href=format!("/users/{}", uid)>
                                                        <Button variant=ButtonVariant::Outline class="h-8 px-2 text-xs".to_string()>"View"</Button>
                                                    </a>
                                                    <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary text-xs".to_string() on:click=move |_| handle_toggle_admin(uid.clone())>
                                                        {if is_admin { "Revoke Admin" } else { "Make Admin" }}
                                                    </Button>
                                                </DataTableCell>
                                            </DataTableRow>
                                        }
                                    }
                                />
                            })}
                        </DataTableBody>
                    </DataTable>
                </div>
            </Suspense>
        </div>
    }
}
