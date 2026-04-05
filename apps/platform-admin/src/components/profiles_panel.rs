use leptos::prelude::*;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::table::{Table as DataTable, TableHeader as DataTableHeader, TableRow as DataTableRow, TableHead as DataTableHead, TableBody as DataTableBody, TableCell as DataTableCell};
use leptos_router::hooks::use_params_map;
use shared_ui::components::ui::label::Label;

#[component]
pub fn ProfilesPanel() -> impl IntoView {
    let params = use_params_map();
    let site_id = move || params.with(|p| p.get("id").unwrap_or_default());
    let site_id_str = site_id().to_string();

    let (show_invite, set_show_invite) = signal(false);
    let (managing_user_name, set_managing_user_name) = signal(None::<String>);
    let invite_email = RwSignal::new("".to_string());

    let _profiles_res = LocalResource::new({
        let sid = site_id_str.clone();
        move || {
            let sid = sid.clone();
            async move { crate::api::admin::get_users(uuid::Uuid::parse_str(&sid).ok()).await.unwrap_or_default() }
        }
    });

    let mock_profiles = vec![
        ("usr_8821", "Alice Admin", "alice@example.com", "Site Admin"),
        ("usr_3194", "Bob Driver", "bob@example.com", "Contributor"),
        ("usr_5561", "Charlie Dispatch", "charlie@example.com", "Editor"),
    ];

    let handle_invite = move |_| {
        let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
        // Simulated Magic Link Request
        toast.message.set(Some(format!("Magic link dispatched to: {}", invite_email.get())));
        invite_email.set("".to_string());
        set_show_invite.set(false);
    };

    view! {
        <div class="w-full animation-fade-in relative">
            <div class="flex justify-between items-center mb-6">
                <div>
                    <h3 class="text-xl font-semibold dark:text-white">"Identity & Access Management"</h3>
                    <p class="text-slate-500 text-sm">"Manage access controls, roles, and connected profiles."</p>
                </div>
                <Button variant=ButtonVariant::Default on:click=move |_| set_show_invite.set(true)>
                    "Invite Team Member"
                </Button>
            </div>

            <div class="w-full rounded-md border border-border">
                <DataTable class="w-full">
                    <DataTableHeader>
                        <DataTableRow class="hover:bg-transparent">
                            <DataTableHead>"User"</DataTableHead>
                            <DataTableHead>"Email"</DataTableHead>
                            <DataTableHead>"Role"</DataTableHead>
                            <DataTableHead class="text-right">"Actions"</DataTableHead>
                        </DataTableRow>
                    </DataTableHeader>
                    <DataTableBody>
                        {mock_profiles.into_iter().map(|(id, name, email, role)| {
                            let n = name.to_string();
                            let n2 = name.to_string();
                            view! {
                                <DataTableRow>
                                    <DataTableCell class="font-medium">
                                        <div class="flex items-center gap-2">
                                            <div class="w-8 h-8 rounded-full bg-indigo-100 dark:bg-indigo-900 flex items-center justify-center text-indigo-700 dark:text-indigo-300 font-bold text-xs">
                                                {n.chars().next().unwrap_or('A')}
                                            </div>
                                            <span>{n}</span>
                                            <span class="text-xs text-muted-foreground ml-2 font-mono">"#" {id}</span>
                                        </div>
                                    </DataTableCell>
                                    <DataTableCell>{email}</DataTableCell>
                                    <DataTableCell>
                                        <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-slate-100 dark:bg-slate-800 text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-slate-700">
                                            {role}
                                        </span>
                                    </DataTableCell>
                                    <DataTableCell class="text-right">
                                        <Button variant=ButtonVariant::Ghost class="h-8 px-2" on:click=move |_| set_managing_user_name.set(Some(n2.clone()))>
                                            "Manage Access"
                                        </Button>
                                    </DataTableCell>
                                </DataTableRow>
                            }
                        }).collect_view()}
                    </DataTableBody>
                </DataTable>
            </div>

            <Show when=move || show_invite.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_invite.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Invite Team Member"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Send a Magic Link invitation to grant access."</p>
                        <div class="space-y-4 mb-6">
                            <div class="grid gap-2 text-left">
                                <Label>"Email Address"</Label>
                                <Input r#type=InputType::Email placeholder="user@example.com".to_string() bind_value=invite_email />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_invite.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=handle_invite>"Send Invite"</Button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || managing_user_name.get().is_some()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_managing_user_name.set(None)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">{move || format!("Manage {}", managing_user_name.get().unwrap_or_default())}</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Configure robust access and permissions."</p>
                        <div class="flex justify-end gap-3 mt-8">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_managing_user_name.set(None)>"Close"</Button>
                            <Button variant=ButtonVariant::Destructive on:click=move |_| {
                                let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
                                toast.message.set(Some("User access rescinded.".to_string()));
                                set_managing_user_name.set(None);
                            }>"Revoke Access"</Button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
