//! InstanceProgramsPanel — per-app-instance G-36 program enablement checklist.

use leptos::prelude::*;
use uuid::Uuid;

use crate::api::programs::{
    Program, list_instance_enablements, list_programs, set_program_enabled_for_instance,
};

#[derive(Debug, Clone)]
struct InstanceProgramState {
    program: Program,
    enabled: bool,
    explicit: bool,
}

#[component]
pub fn InstanceProgramsPanel(app_instance_id: Uuid) -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let programs_res = LocalResource::new(move || async move {
        let programs = list_programs(true).await.unwrap_or_default();
        let mut rows = Vec::with_capacity(programs.len());
        for program in programs {
            let explicit = list_instance_enablements(program.id)
                .await
                .unwrap_or_default()
                .into_iter()
                .find(|row| row.app_instance_id == app_instance_id);
            rows.push(InstanceProgramState {
                program,
                enabled: explicit.as_ref().map(|row| row.is_enabled).unwrap_or(true),
                explicit: explicit.is_some(),
            });
        }
        rows
    });

    view! {
        <div class="section">
            <div class="section-hdr">
                <span class="section-title">"Growth programs"</span>
                <span class="text-[10px] text-on-surface-variant/60">
                    "Control which platform growth templates are available to this app instance."
                </span>
            </div>
            <Suspense fallback=move || view! {
                <div class="p-6 text-sm text-on-surface-variant animate-pulse">"Loading programs..."</div>
            }>
                {move || {
                    let rows = programs_res.get().unwrap_or_default();
                    if rows.is_empty() {
                        return view! {
                            <div class="p-6 text-sm text-on-surface-variant/70">
                                "No growth programs are configured yet."
                            </div>
                        }.into_any();
                    }

                    view! {
                        <div class="divide-y divide-outline-variant/10">
                            {rows.into_iter().map(|row| {
                                let program_id = row.program.id;
                                let enabled = row.enabled;
                                let toast = toast.clone();
                                view! {
                                    <div class="flex items-center justify-between gap-4 p-4">
                                        <div>
                                            <div class="flex items-center gap-2">
                                                <span class="text-sm font-semibold text-on-surface">{row.program.name.clone()}</span>
                                                <span class="pill">{row.program.program_kind.label()}</span>
                                                {(!row.program.is_active).then(|| view! {
                                                    <span class="text-[10px] text-on-surface-variant/50">"Inactive"</span>
                                                })}
                                            </div>
                                            <div class="text-xs text-on-surface-variant/70 mt-1">
                                                {row.program.description.clone().unwrap_or_else(|| "Growth program template".to_string())}
                                            </div>
                                            <div class="text-[10px] font-mono text-on-surface-variant/50 mt-1">
                                                {row.program.slug.clone()}
                                                {if row.explicit { " · explicit override" } else { " · default enabled" }}
                                            </div>
                                        </div>
                                        <button
                                            class=if enabled { "btn btn-primary btn-sm" } else { "btn btn-ghost btn-sm" }
                                            on:click=move |_| {
                                                let next = !enabled;
                                                let toast = toast.clone();
                                                leptos::task::spawn_local(async move {
                                                    match set_program_enabled_for_instance(program_id, app_instance_id, next).await {
                                                        Ok(_) => {
                                                            toast.show_toast("Program updated", "Growth program enablement saved.", "success");
                                                            programs_res.refetch();
                                                        }
                                                        Err(e) => toast.show_toast("Error", &e, "error"),
                                                    }
                                                });
                                            }
                                        >
                                            {if enabled { "Enabled" } else { "Disabled" }}
                                        </button>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </Suspense>
        </div>
    }
}
