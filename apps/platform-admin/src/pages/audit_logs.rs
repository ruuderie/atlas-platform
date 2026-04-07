use leptos::prelude::*;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::api::audit_logs::{get_audit_logs, AuditLogModel};
use crate::api::models::UserInfo;
use crate::app::GlobalToast;

pub fn format_datetime_diff(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_json_diff(val: &Option<Value>) -> String {
    match val {
        Some(v) => serde_json::to_string_pretty(v).unwrap_or_else(|_| "{}".to_string()),
        None => "None".to_string(),
    }
}

#[component]
pub fn AuditLogs() -> impl IntoView {
    let (logs, set_logs) = signal(Vec::<AuditLogModel>::new());
    let (loading, set_loading) = signal(true);
    
    // Diff Modal
    let (selected_log, set_selected_log) = signal(None::<AuditLogModel>);
    let (show_modal, set_show_modal) = signal(false);

    let active_network = use_context::<ReadSignal<Option<uuid::Uuid>>>().expect("active network missing");
    let user = use_context::<ReadSignal<Option<UserInfo>>>().expect("user context missing");
    let toast = use_context::<GlobalToast>().expect("toast context missing");

    let fetch_logs = move || {
        set_loading.set(true);
        leptos::task::spawn_local(async move {
            let tenant_id = active_network.get();
            // In a real app we might pass other filters based on state
            match get_audit_logs(tenant_id, None, None).await {
                Ok(data) => {
                    set_logs.set(data);
                }
                Err(e) => {
                    toast.show_toast("Error", &e, "error");
                    leptos::logging::error!("Failed to fetch logs: {}", e);
                }
            }
            set_loading.set(false);
        });
    };

    // Refetch on mount or network change
    Effect::new(move |_| {
        fetch_logs();
    });


    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight text-on-surface">"Security Audit Ledger"</h1>
                    <p class="text-on-surface-variant text-sm mt-1">"Immutable log of critical system operations and state changes."</p>
                </div>
                <button 
                    class="btn-secondary px-4 py-2 rounded-md shadow flex items-center gap-2"
                    on:click=move |_| fetch_logs()
                >
                    <span class="material-symbols-outlined text-sm">"refresh"</span>
                    "Refresh Logs"
                </button>
            </div>

            <div class="glass-panel rounded-xl border border-outline-variant/30 overflow-hidden shadow-sm">
                <div class="overflow-x-auto">
                    <table class="w-full text-left text-sm whitespace-nowrap">
                        <thead class="bg-surface-container-high border-b border-outline-variant/20 uppercase text-xs font-semibold text-on-surface-variant tracking-wider">
                            <tr>
                                <th class="px-6 py-4">"Timestamp"</th>
                                <th class="px-6 py-4">"Action Type"</th>
                                <th class="px-6 py-4">"Entity Type"</th>
                                <th class="px-6 py-4">"Actor ID"</th>
                                <th class="px-6 py-4">"Entity ID"</th>
                                <th class="px-6 py-4 text-center">"Diff"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-outline-variant/10">
                            <Show when=move || !loading.get() fallback=move || view! {
                                <tr>
                                    <td colspan="6" class="px-6 py-12 text-center text-on-surface-variant">
                                        <div class="animate-pulse flex flex-col items-center">
                                            <span class="material-symbols-outlined text-4xl mb-2 opacity-50">"sync"</span>
                                            <p>"Loading ledger entries..."</p>
                                        </div>
                                    </td>
                                </tr>
                            }>
                                <Show when=move || !logs.get().is_empty() fallback=move || view! {
                                    <tr>
                                        <td colspan="6" class="px-6 py-12 text-center text-on-surface-variant">
                                            <div class="flex flex-col items-center">
                                                <span class="material-symbols-outlined text-4xl mb-2 opacity-50">"history_toggle_off"</span>
                                                <p>"No audit logs found for this configuration."</p>
                                            </div>
                                        </td>
                                    </tr>
                                }>
                                    <For
                                        each=move || logs.get()
                                        key=|log| log.id
                                        children=move |log| {
                                            view! {
                                                <tr class="hover:bg-surface-bright/10 transition-colors">
                                                    <td class="px-6 py-4 text-on-surface-variant font-mono text-xs">{format_datetime_diff(log.created_at)}</td>
                                                    <td class="px-6 py-4 text-primary font-medium tracking-wide">{log.action_type.clone()}</td>
                                                    <td class="px-6 py-4 text-on-surface font-semibold">{log.entity_type.clone()}</td>
                                                    <td class="px-6 py-4 text-on-surface-variant font-mono text-xs">{log.actor_id.map(|id| id.to_string()).unwrap_or_else(|| "System".to_string())}</td>
                                                    <td class="px-6 py-4 text-on-surface-variant font-mono text-xs">{log.entity_id.to_string()}</td>
                                                    <td class="px-6 py-4 text-center">
                                                        <button 
                                                            class="text-xs font-semibold px-3 py-1 bg-primary/10 text-primary border border-primary/20 rounded-md hover:bg-primary/20 transition-all active:scale-95"
                                                            on:click={
                                                                let log_clone = log.clone();
                                                                move |_| {
                                                                    set_selected_log.set(Some(log_clone.clone()));
                                                                    set_show_modal.set(true);
                                                                }
                                                            }
                                                        >
                                                            "View Delta"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    />
                                </Show>
                            </Show>
                        </tbody>
                    </table>
                </div>
            </div>

            // Diff Modal
            <Show when=move || show_modal.get()>
                <div class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-background/80 backdrop-blur-sm">
                    <div class="glass-panel border border-outline-variant/40 rounded-2xl shadow-2xl w-full max-w-4xl max-h-[90vh] flex flex-col animate-in fade-in zoom-in duration-200">
                        <div class="p-6 border-b border-outline-variant/20 flex justify-between items-center bg-surface-container-high/50 rounded-t-2xl">
                            <div>
                                <h3 class="text-xl font-bold text-on-surface flex items-center gap-2">
                                    <span class="material-symbols-outlined text-primary">"difference"</span>
                                    "State Differential"
                                </h3>
                                <p class="text-xs text-on-surface-variant mt-1 font-mono">
                                    {move || selected_log.get().map(|log| format!("Audit ID: {}", log.id)).unwrap_or_default()}
                                </p>
                            </div>
                            <button 
                                class="p-2 bg-surface-container hover:bg-error-container text-on-surface hover:text-on-error-container rounded-full transition-all duration-200 active:scale-95"
                                on:click=move |_| set_show_modal.set(false)
                            >
                                <span class="material-symbols-outlined">"close"</span>
                            </button>
                        </div>
                        
                        <div class="p-6 overflow-y-auto flex-1 font-mono text-sm grid grid-cols-2 gap-6 bg-[#060e20]">
                            // Old State
                            <div class="space-y-3">
                                <div class="px-3 py-1 border border-error/30 bg-error/10 text-error inline-block rounded-md text-xs font-bold w-full backdrop-blur-sm shadow-sm">
                                    "BEFORE (old_state)"
                                </div>
                                <pre class="bg-surface-container-highest p-4 rounded-xl border border-outline-variant/20 overflow-x-auto text-[#e06c75] min-h-[200px] shadow-inner text-xs">
                                    {move || format_json_diff(&selected_log.get().map(|l| l.old_state).unwrap_or_default())}
                                </pre>
                            </div>
                            
                            // New State
                            <div class="space-y-3">
                                <div class="px-3 py-1 border border-success/30 bg-success/10 text-success inline-block rounded-md text-xs font-bold w-full backdrop-blur-sm shadow-sm">
                                    "AFTER (new_state)"
                                </div>
                                <pre class="bg-surface-container-highest p-4 rounded-xl border border-outline-variant/20 overflow-x-auto text-[#98c379] min-h-[200px] shadow-inner text-xs">
                                    {move || format_json_diff(&selected_log.get().map(|l| l.new_state).unwrap_or_default())}
                                </pre>
                            </div>
                        </div>
                        
                        <div class="p-4 border-t border-outline-variant/20 bg-surface-container-low rounded-b-2xl flex justify-between items-center text-xs text-on-surface-variant">
                            <span>"Note: This ledger entry is immutable and cannot be deleted or modified."</span>
                            <span class="flex items-center gap-1 font-mono">
                                <span class="material-symbols-outlined text-sm">"lock"</span>
                                "Cryptographic Sealing Enabled"
                            </span>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
