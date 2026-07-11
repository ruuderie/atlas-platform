//! InstanceFeaturesPanel — per-app-instance feature flag grant/deny/inherit.

use leptos::prelude::*;
use uuid::Uuid;

use crate::api::admin::{
    FlagEffect, InstanceFlagRow, InstanceFlagUpdateItem, get_instance_feature_flags,
    update_instance_feature_flags,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectChoice {
    Inherit,
    Grant,
    Deny,
}

impl EffectChoice {
    fn from_row(row: &InstanceFlagRow) -> Self {
        match row.effect {
            None => Self::Inherit,
            Some(FlagEffect::Grant) => Self::Grant,
            Some(FlagEffect::Deny) => Self::Deny,
        }
    }

    fn to_api(self) -> Option<FlagEffect> {
        match self {
            Self::Inherit => None,
            Self::Grant => Some(FlagEffect::Grant),
            Self::Deny => Some(FlagEffect::Deny),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Inherit => "Inherit",
            Self::Grant => "Grant",
            Self::Deny => "Deny",
        }
    }
}

#[derive(Debug, Clone)]
struct FlagRowState {
    row: InstanceFlagRow,
    choice: EffectChoice,
    dirty: bool,
}

#[component]
pub fn InstanceFeaturesPanel(app_instance_id: Uuid) -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let rows = RwSignal::new(Vec::<FlagRowState>::new());
    let saving = RwSignal::new(false);
    let load_err = RwSignal::new(Option::<String>::None);

    let flags_res = LocalResource::new(move || {
        let id = app_instance_id.to_string();
        async move { get_instance_feature_flags(&id).await }
    });

    Effect::new(move |_| {
        if let Some(result) = flags_res.get() {
            match result {
                Ok(resp) => {
                    load_err.set(None);
                    rows.set(
                        resp.flags
                            .into_iter()
                            .map(|row| {
                                let choice = EffectChoice::from_row(&row);
                                FlagRowState {
                                    row,
                                    choice,
                                    dirty: false,
                                }
                            })
                            .collect(),
                    );
                }
                Err(e) => load_err.set(Some(e)),
            }
        }
    });

    let dirty_count = move || rows.get().iter().filter(|r| r.dirty).count();

    view! {
        <div class="section">
            <div class="section-hdr" style="display:flex;align-items:center;justify-content:space-between;gap:12px;">
                <div>
                    <span class="section-title">"Feature flags"</span>
                    <span class="text-[10px] text-on-surface-variant/60" style="display:block;margin-top:2px;">
                        "Grant or deny catalog flags for this instance. Inherit uses tenant override → global rollout."
                    </span>
                </div>
                <button
                    class="btn btn-primary btn-sm"
                    disabled=move || saving.get() || dirty_count() == 0
                    on:click=move |_| {
                        let toast = toast.clone();
                        let id = app_instance_id.to_string();
                        let updates: Vec<InstanceFlagUpdateItem> = rows
                            .get_untracked()
                            .into_iter()
                            .filter(|r| r.dirty)
                            .map(|r| InstanceFlagUpdateItem {
                                flag_key: r.row.flag_key.clone(),
                                effect: r.choice.to_api(),
                                rollout_pct: if r.choice == EffectChoice::Grant {
                                    Some(100)
                                } else {
                                    None
                                },
                            })
                            .collect();
                        if updates.is_empty() {
                            return;
                        }
                        saving.set(true);
                        leptos::task::spawn_local(async move {
                            match update_instance_feature_flags(&id, updates).await {
                                Ok(resp) => {
                                    rows.set(
                                        resp.flags
                                            .into_iter()
                                            .map(|row| {
                                                let choice = EffectChoice::from_row(&row);
                                                FlagRowState {
                                                    row,
                                                    choice,
                                                    dirty: false,
                                                }
                                            })
                                            .collect(),
                                    );
                                    toast.show_toast(
                                        "Features updated",
                                        "Instance feature flag enablements saved.",
                                        "success",
                                    );
                                }
                                Err(e) => toast.show_toast("Error", &e, "error"),
                            }
                            saving.set(false);
                        });
                    }
                >
                    {move || {
                        if saving.get() {
                            "Saving…".to_string()
                        } else {
                            let n = dirty_count();
                            if n == 0 {
                                "Save".to_string()
                            } else {
                                format!("Save ({n})")
                            }
                        }
                    }}
                </button>
            </div>

            <Suspense fallback=move || view! {
                <div class="p-6 text-sm text-on-surface-variant animate-pulse">"Loading feature flags…"</div>
            }>
                {move || {
                    let _ = flags_res.get();
                    if let Some(err) = load_err.get() {
                        return view! {
                            <div class="p-6 text-sm text-error">{err}</div>
                        }.into_any();
                    }
                    let current = rows.get();
                    if current.is_empty() {
                        return view! {
                            <div class="p-6 text-sm text-on-surface-variant/70">
                                "No feature flags in the catalog yet. Create them on the Flags page."
                            </div>
                        }.into_any();
                    }

                    view! {
                        <div class="divide-y divide-outline-variant/10">
                            {current.into_iter().map(|state| {
                                let key = state.row.flag_key.clone();
                                let desc = state.row.description.clone();
                                let catalog_on = state.row.catalog_enabled;
                                let global_pct = state.row.global_rollout_pct;
                                let choice = state.choice;
                                let dirty = state.dirty;
                                view! {
                                    <div class="flex items-center justify-between gap-4 p-4">
                                        <div style="min-width:0;flex:1;">
                                            <div class="flex items-center gap-2 flex-wrap">
                                                <span class="text-sm font-semibold text-on-surface font-mono">{key.clone()}</span>
                                                {if dirty {
                                                    view! { <span class="text-[10px] text-amber-400">"unsaved"</span> }.into_any()
                                                } else {
                                                    view! { <></> }.into_any()
                                                }}
                                                {if !catalog_on {
                                                    view! {
                                                        <span class="text-[10px] text-error/80">"catalog off"</span>
                                                    }.into_any()
                                                } else {
                                                    view! { <></> }.into_any()
                                                }}
                                            </div>
                                            <div class="text-xs text-on-surface-variant/70 mt-1">
                                                {if desc.is_empty() {
                                                    "No description".to_string()
                                                } else {
                                                    desc
                                                }}
                                            </div>
                                            <div class="text-[10px] font-mono text-on-surface-variant/50 mt-1">
                                                {format!("global {global_pct}%")}
                                            </div>
                                        </div>
                                        <div class="flex items-center gap-1">
                                            {[EffectChoice::Inherit, EffectChoice::Grant, EffectChoice::Deny]
                                                .into_iter()
                                                .map(|opt| {
                                                    let key = key.clone();
                                                    let active = choice == opt;
                                                    view! {
                                                        <button
                                                            class=if active {
                                                                "btn btn-primary btn-sm"
                                                            } else {
                                                                "btn btn-ghost btn-sm"
                                                            }
                                                            on:click=move |_| {
                                                                rows.update(|list| {
                                                                    if let Some(row) = list
                                                                        .iter_mut()
                                                                        .find(|r| r.row.flag_key == key)
                                                                    {
                                                                        let original =
                                                                            EffectChoice::from_row(&row.row);
                                                                        row.choice = opt;
                                                                        row.dirty = row.choice != original;
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            {opt.label()}
                                                        </button>
                                                    }
                                                })
                                                .collect_view()}
                                        </div>
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
