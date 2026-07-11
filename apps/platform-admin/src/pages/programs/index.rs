//! G-36 Programs catalog.

use leptos::prelude::*;

use crate::api::programs::{
    Program, ProgramAnalytics, ProgramUpdatePatch, get_program_analytics, list_programs,
    update_program,
};
use crate::components::gtm_process_strip::{GtmProcessStrip, GtmStage};

#[component]
pub fn ProgramsPage() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let include_inactive = RwSignal::new(true);

    let programs_res = LocalResource::new(move || async move {
        list_programs(include_inactive.get())
            .await
            .unwrap_or_default()
    });

    let analytics_res = LocalResource::new(move || async move {
        let programs = list_programs(true).await.unwrap_or_default();
        let mut total_actions = 0_i64;
        let mut total_grants = 0_i64;
        for program in programs {
            if let Ok(analytics) = get_program_analytics(program.id).await {
                total_actions += analytics.total_actions;
                total_grants += analytics.total_grants;
            }
        }
        ProgramAnalytics {
            total_actions,
            total_grants,
            actions_by_status: vec![],
            outcomes_by_status: vec![],
            grants_by_status: vec![],
        }
    });

    let toggle_active = {
        let toast = toast.clone();
        move |program: Program| {
            let toast = toast.clone();
            leptos::task::spawn_local(async move {
                let next = !program.is_active;
                match update_program(
                    program.id,
                    ProgramUpdatePatch {
                        is_active: Some(next),
                        ..Default::default()
                    },
                )
                .await
                {
                    Ok(_) => {
                        toast.show_toast(
                            "Program updated",
                            if next {
                                "Program activated."
                            } else {
                                "Program deactivated."
                            },
                            "success",
                        );
                        programs_res.refetch();
                    }
                    Err(e) => toast.show_toast("Error", &e, "error"),
                }
            });
        }
    };

    view! {
        <div class="main-canvas">
            <GtmProcessStrip
                active=GtmStage::Programs
                subtitle="Package repeatable acquisition, referral, invite, and reward loops."
            />

            <div class="page-header">
                <div>
                    <div class="page-title">"Programs"</div>
                    <div class="page-subtitle">
                        "Growth templates for productized referrals, network invites, lead capture, and rewards."
                    </div>
                </div>
            </div>

            <Suspense fallback=move || view! {
                <div class="kpi-row">
                    <KpiCard label="Programs" value="...".to_string() hint="loading" />
                    <KpiCard label="Active" value="...".to_string() hint="loading" />
                    <KpiCard label="Actions" value="...".to_string() hint="loading" />
                    <KpiCard label="Grants" value="...".to_string() hint="loading" />
                </div>
            }>
                {move || {
                    let programs = programs_res.get().unwrap_or_default();
                    let analytics = analytics_res.get();
                    let total = programs.len();
                    let active = programs.iter().filter(|p| p.is_active).count();
                    let actions = analytics.as_ref().map(|a| a.total_actions).unwrap_or(total as i64);
                    let grants = analytics.as_ref().map(|a| a.total_grants).unwrap_or(active as i64);

                    view! {
                        <div class="kpi-row">
                            <KpiCard label="Programs" value=total.to_string() hint="catalog" />
                            <KpiCard label="Active" value=active.to_string() hint="available" />
                            <KpiCard label="Actions" value=actions.to_string() hint="tracked" />
                            <KpiCard label="Grants" value=grants.to_string() hint="rewarded" />
                        </div>
                    }
                }}
            </Suspense>

            <div class="section">
                <div class="section-hdr">
                    <span class="section-title">"Program Catalog"</span>
                    <label class="flex items-center gap-2 text-[11px] text-on-surface-variant/70">
                        <input
                            type="checkbox"
                            prop:checked=move || include_inactive.get()
                            on:change=move |_| {
                                include_inactive.update(|v| *v = !*v);
                                programs_res.refetch();
                            }
                        />
                        "Show inactive"
                    </label>
                </div>

                <table class="w-full text-left">
                    <thead>
                        <tr class="bg-surface-container-high/20 border-b border-outline-variant/10 text-[10px] uppercase tracking-wider text-on-surface-variant/70">
                            <th class="px-5 py-3 font-semibold">"Program"</th>
                            <th class="px-5 py-3 font-semibold">"Kind"</th>
                            <th class="px-5 py-3 font-semibold">"Active"</th>
                            <th class="px-5 py-3 font-semibold">"Actor Roles"</th>
                            <th class="px-5 py-3 font-semibold">"Target Roles"</th>
                            <th class="px-5 py-3 font-semibold text-right">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-outline-variant/10">
                        <Suspense fallback=move || view! {
                            <tr><td colspan="6" class="px-5 py-8 text-center text-sm text-on-surface-variant">"Loading programs..."</td></tr>
                        }>
                            {move || {
                                let programs = programs_res.get().unwrap_or_default();
                                if programs.is_empty() {
                                    return view! {
                                        <tr><td colspan="6" class="px-5 py-8 text-center text-sm text-on-surface-variant/70">
                                            "No programs found."
                                        </td></tr>
                                    }.into_any();
                                }

                                programs.into_iter().map(|program| {
                                    let id = program.id;
                                    let program_for_toggle = program.clone();
                                    let status_text = if program.is_active { "Active" } else { "Inactive" };
                                    view! {
                                        <tr class="hover:bg-surface-bright/5 transition-colors">
                                            <td class="px-5 py-3">
                                                <a href=format!("/programs/{id}") class="font-semibold text-on-surface hover:text-primary">
                                                    {program.name.clone()}
                                                </a>
                                                <div class="font-mono text-[10px] text-on-surface-variant/50">
                                                    {program.slug.clone()}
                                                </div>
                                                {program.description.clone().map(|d| view! {
                                                    <div class="text-xs text-on-surface-variant/70 mt-1 max-w-xl">{d}</div>
                                                })}
                                            </td>
                                            <td class="px-5 py-3">
                                                <span class="pill">{program.program_kind.label()}</span>
                                            </td>
                                            <td class="px-5 py-3">
                                                <span class=if program.is_active { "text-emerald-400 text-xs font-bold" } else { "text-on-surface-variant/50 text-xs font-bold" }>
                                                    {status_text}
                                                </span>
                                            </td>
                                            <td class="px-5 py-3 text-xs text-on-surface-variant">
                                                {empty_dash(program.actor_roles_display())}
                                            </td>
                                            <td class="px-5 py-3 text-xs text-on-surface-variant">
                                                {empty_dash(program.target_roles_display())}
                                            </td>
                                            <td class="px-5 py-3 text-right">
                                                <button
                                                    class="btn btn-ghost btn-sm"
                                                    on:click=move |_| toggle_active(program_for_toggle.clone())
                                                >
                                                    {if program.is_active { "Deactivate" } else { "Activate" }}
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any()
                            }}
                        </Suspense>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn KpiCard(label: &'static str, value: String, hint: &'static str) -> impl IntoView {
    view! {
        <div class="kpi-card">
            <div class="kpi-label">{label}</div>
            <div class="kpi-value">{value}</div>
            <div class="kpi-delta positive">{hint}</div>
        </div>
    }
}

fn empty_dash(value: String) -> String {
    if value.trim().is_empty() {
        "-".to_string()
    } else {
        value
    }
}
