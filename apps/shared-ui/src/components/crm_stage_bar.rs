use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CrmStatusOption {
    pub status_key: String,
    pub label: String,
    pub color: String,
    pub sort_order: i32,
    pub is_system: bool,
}

#[component]
pub fn CrmStageBar(
    stages: Vec<CrmStatusOption>,
    current_stage: Signal<String>,
    #[prop(into)] on_stage_change: Callback<String>,
) -> impl IntoView {
    // Helper to get Tailwind color classes based on the stage color string and active status.
    let get_stage_classes = move |color: &str, is_active: bool| -> String {
        let base = "flex-1 min-h-[44px] px-6 py-3 flex items-center justify-center text-xs jetbrains font-bold uppercase tracking-widest transition-all duration-300 relative border cursor-pointer select-none ";
        if is_active {
            match color {
                "blue" => format!("{} bg-blue-600 border-blue-600 text-white shadow-lg shadow-blue-500/20 scale-[1.02] z-10", base),
                "purple" => format!("{} bg-purple-600 border-purple-600 text-white shadow-lg shadow-purple-500/20 scale-[1.02] z-10", base),
                "indigo" => format!("{} bg-indigo-600 border-indigo-600 text-white shadow-lg shadow-indigo-500/20 scale-[1.02] z-10", base),
                "orange" => format!("{} bg-orange-500 border-orange-500 text-white shadow-lg shadow-orange-500/20 scale-[1.02] z-10", base),
                "emerald" => format!("{} bg-emerald-600 border-emerald-600 text-white shadow-lg shadow-emerald-500/20 scale-[1.02] z-10", base),
                "rose" => format!("{} bg-rose-600 border-rose-600 text-white shadow-lg shadow-rose-500/20 scale-[1.02] z-10", base),
                _ => format!("{} bg-slate-600 border-slate-600 text-white shadow-lg shadow-slate-500/20 scale-[1.02] z-10", base), // slate / fallback
            }
        } else {
            match color {
                "blue" => format!("{} bg-surface-container border-outline-variant/30 text-blue-500 hover:bg-blue-50/10 hover:border-blue-400/30", base),
                "purple" => format!("{} bg-surface-container border-outline-variant/30 text-purple-500 hover:bg-purple-50/10 hover:border-purple-400/30", base),
                "indigo" => format!("{} bg-surface-container border-outline-variant/30 text-indigo-500 hover:bg-indigo-50/10 hover:border-indigo-400/30", base),
                "orange" => format!("{} bg-surface-container border-outline-variant/30 text-orange-500 hover:bg-orange-50/10 hover:border-orange-400/30", base),
                "emerald" => format!("{} bg-surface-container border-outline-variant/30 text-emerald-500 hover:bg-emerald-50/10 hover:border-emerald-400/30", base),
                "rose" => format!("{} bg-surface-container border-outline-variant/30 text-rose-500 hover:bg-rose-50/10 hover:border-rose-400/30", base),
                _ => format!("{} bg-surface-container border-outline-variant/30 text-on-surface-variant hover:bg-slate-100/10 hover:border-slate-400/30", base), // slate / fallback
            }
        }
    };

    view! {
        <div class="w-full bg-surface-container-low border border-outline-variant/30 p-2 rounded-lg shadow-inner flex flex-col md:flex-row gap-1.5 overflow-hidden">
            <For
                each=move || stages.clone()
                key=|stage| stage.status_key.clone()
                children=move |stage| {
                    let key = stage.status_key.clone();
                    let color = stage.color.clone();
                    let label = stage.label.clone();
                    let click_key = stage.status_key.clone();

                    let key_for_active = key.clone();
                    let is_active = move || current_stage.get() == key_for_active;

                    let key_for_show = key.clone();
                    let is_active_show = move || current_stage.get() == key_for_show;

                    view! {
                        <div
                            on:click=move |_| {
                                on_stage_change.run(click_key.clone());
                            }
                            class=move || get_stage_classes(&color, is_active())
                            role="button"
                            tabindex="0"
                        >
                            // Active glowing pulse dot
                            <Show when=is_active_show>
                                <span class="absolute left-3 w-1.5 h-1.5 rounded-full bg-white animate-ping"></span>
                                <span class="absolute left-3 w-1.5 h-1.5 rounded-full bg-white"></span>
                            </Show>
                            
                            <span class="truncate">{label}</span>
                            
                            // Chevron/arrow effect on the right for non-last items
                            <span class="hidden md:block absolute right-0 top-0 bottom-0 w-3 pointer-events-none z-20">
                                // Handled via modular design structure cleanly
                            </span>
                        </div>
                    }
                }
            />
        </div>
    }
}
