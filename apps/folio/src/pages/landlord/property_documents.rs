//! Property documents & expenses — `/l/assets/:id/documents`

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::property_tab_bar::{PropertyTab, PropertyTabBar};
use crate::components::status_pill::{StatusPill, StatusPillTone};
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyDocumentKind {
    Vault,
    Expense,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DocsGroupBy {
    Year,
    Unit,
    Asset,
    Kind,
}

impl DocsGroupBy {
    fn label(self) -> &'static str {
        match self {
            Self::Year => "Year",
            Self::Unit => "Unit",
            Self::Asset => "Asset",
            Self::Kind => "Kind",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyDocumentRow {
    pub id: Uuid,
    pub kind: PropertyDocumentKind,
    pub title: String,
    pub category: String,
    pub amount_cents: Option<i64>,
    pub asset_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub project_id: Option<Uuid>,
}

fn category_label(category: &str) -> String {
    if category.is_empty() {
        return "Uncategorized".into();
    }
    if category == "work_order_cost" {
        return "Work order cost".into();
    }
    category
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn group_label(row: &PropertyDocumentRow, by: DocsGroupBy, property_id: Uuid) -> String {
    match by {
        DocsGroupBy::Year => row.created_at.format("%Y").to_string(),
        DocsGroupBy::Unit => match row.asset_id {
            None => "Building".into(),
            Some(id) if id == property_id => "Building".into(),
            Some(_) => "Unit".into(),
        },
        DocsGroupBy::Asset => category_label(&row.category),
        DocsGroupBy::Kind => match row.kind {
            PropertyDocumentKind::Vault => "Vault".into(),
            PropertyDocumentKind::Expense => "Expense".into(),
        },
    }
}

fn group_documents(
    list: &[PropertyDocumentRow],
    by: DocsGroupBy,
    property_id: Uuid,
) -> Vec<(String, Vec<PropertyDocumentRow>)> {
    let mut map: BTreeMap<String, Vec<PropertyDocumentRow>> = BTreeMap::new();
    for row in list {
        let key = group_label(row, by, property_id);
        map.entry(key).or_default().push(row.clone());
    }
    let mut groups: Vec<_> = map.into_iter().collect();
    if by == DocsGroupBy::Year {
        groups.sort_by(|a, b| b.0.cmp(&a.0));
    }
    groups
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(GetPropertyDocuments, "/api")]
pub async fn get_property_documents(
    asset_id: Uuid,
    project_id: Option<Uuid>,
) -> Result<Vec<PropertyDocumentRow>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    let mut path = format!("/api/folio/assets/{asset_id}/documents");
    if let Some(pid) = project_id {
        path.push_str(&format!("?project_id={pid}"));
    }
    crate::atlas_client::authenticated_get(&path, &token, None)
        .await
        .map_err(ServerFnError::new)
}

#[component]
pub fn PropertyDocuments() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let asset_id = Memo::new(move |_| {
        params
            .get()
            .get("id")
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or(Uuid::nil())
    });
    let project_id = Memo::new(move |_| {
        query
            .get()
            .get("project")
            .and_then(|s| Uuid::parse_str(&s).ok())
    });
    let kind_filter = Memo::new(move |_| {
        query
            .get()
            .get("kind")
            .map(|s| s.to_ascii_lowercase())
            .filter(|s| s == "photo")
    });

    let group_by = RwSignal::new(DocsGroupBy::Year);
    let show_export = RwSignal::new(false);

    let docs = Resource::new(
        move || (asset_id.get(), project_id.get()),
        |(aid, pid)| async move {
            if aid.is_nil() {
                return Ok(Vec::new());
            }
            get_property_documents(aid, pid).await
        },
    );

    let upload_href = move || {
        format!(
            "{}?entity_type=atlas_assets&entity_id={}",
            FolioRoute::LandlordVault.path(),
            asset_id.get()
        )
    };
    let log_paid_href = format!(
        "{}?mode=paid",
        FolioRoute::LandlordMaintenanceNew.path()
    );
    let clear_filter_href = move || {
        FolioRoute::LandlordAssetDocuments
            .path()
            .replace(":id", &asset_id.get().to_string())
    };

    view! {
        <div class="landlord-list-page docs-page">
            <PageHeader
                title=Signal::derive(|| "Documents & expenses".to_string())
                subtitle=Signal::derive(|| "Vault files and paid work-order costs.".to_string())
            >
                <button
                    type="button"
                    class="folio-btn folio-btn--ghost"
                    on:click=move |_| show_export.set(true)
                >
                    "Export"
                </button>
                <a class="folio-btn folio-btn--ghost" href=log_paid_href.clone()>
                    "Log paid WO"
                </a>
                <a class="folio-btn folio-btn--primary" href=upload_href>
                    "Upload"
                </a>
            </PageHeader>
            {move || {
                let id = asset_id.get();
                view! {
                    <PropertyTabBar
                        asset_id=id
                        active=Signal::derive(|| PropertyTab::Documents)
                    />
                }
            }}
            {move || project_id.get().map(|_| view! {
                <p class="proj-section__hint docs-page__filter-hint">
                    "Filtered by renovation project"
                    " · "
                    <a class="hub-activity-rail__all" href=clear_filter_href>
                        "Clear filter"
                    </a>
                </p>
            })}
            {move || kind_filter.get().map(|_| view! {
                <p class="proj-section__hint docs-page__filter-hint">
                    "Photos — drop onto the Photos card on the property hub or unit, or upload here via vault."
                    " · "
                    <a class="hub-activity-rail__all" href=clear_filter_href>
                        "Show all documents"
                    </a>
                </p>
            })}
            <div class="folio-field docs-page__group-by">
                <span class="folio-field__label">"Group by"</span>
                <div class="folio-segment-bar" role="tablist" aria-label="Group documents by">
                    {[
                        DocsGroupBy::Year,
                        DocsGroupBy::Unit,
                        DocsGroupBy::Asset,
                        DocsGroupBy::Kind,
                    ]
                        .into_iter()
                        .map(|opt| {
                            view! {
                                <button
                                    type="button"
                                    role="tab"
                                    class=move || {
                                        if group_by.get() == opt {
                                            "folio-segment folio-segment--active"
                                        } else {
                                            "folio-segment"
                                        }
                                    }
                                    aria-selected=move || (group_by.get() == opt).to_string()
                                    on:click=move |_| group_by.set(opt)
                                >
                                    {opt.label()}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
            <Suspense fallback=move || view! { <div class="folio-empty">"Loading documents…"</div> }>
                {move || match docs.get() {
                    Some(Ok(list)) if list.is_empty() => view! {
                        <div class="folio-empty">
                            <p>"No documents or expenses for this property yet."</p>
                            <p class="proj-section__hint">
                                "Upload a file for this property, or log paid work orders to show expenses here."
                            </p>
                        </div>
                    }.into_any(),
                    Some(Ok(list)) => {
                        let property_id = asset_id.get();
                        let by = group_by.get();
                        let list = if kind_filter.get().is_some() {
                            list.into_iter()
                                .filter(|r| {
                                    r.kind == PropertyDocumentKind::Vault
                                        && {
                                            let c = r.category.to_lowercase();
                                            c.contains("photo")
                                                || c.contains("image")
                                                || c.contains("picture")
                                                || c.contains("gallery")
                                                || c.contains("cover")
                                        }
                                })
                                .collect::<Vec<_>>()
                        } else {
                            list
                        };
                        if list.is_empty() && kind_filter.get().is_some() {
                            return view! {
                                <div class="folio-empty">
                                    <p>"No photo-tagged vault files yet."</p>
                                    <p class="proj-section__hint">
                                        "Upload images in the vault for this property — photo gallery is not a separate surface yet."
                                    </p>
                                    <a
                                        class="folio-btn folio-btn--primary press"
                                        style="margin-top:0.75rem;"
                                        href=upload_href
                                    >
                                        "Upload to vault"
                                    </a>
                                </div>
                            }.into_any();
                        }
                        let groups = group_documents(&list, by, property_id);
                        view! {
                            <section class="proj-section docs-page__list">
                                <For
                                    each=move || groups.clone()
                                    key=|g| g.0.clone()
                                    children=move |(label, rows)| {
                                        let count = rows.len();
                                        view! {
                                            <div class="docs-group">
                                                <div class="docs-group__head">
                                                    <p class="docs-group__title">{label.clone()}</p>
                                                    <p class="docs-group__meta">
                                                        {format!(
                                                            "{} {}",
                                                            count,
                                                            if count == 1 { "item" } else { "items" }
                                                        )}
                                                    </p>
                                                </div>
                                                <For
                                                    each=move || rows.clone()
                                                    key=|r| r.id
                                                    children=move |r| {
                                                        let kind_label = match r.kind {
                                                            PropertyDocumentKind::Vault => "Vault",
                                                            PropertyDocumentKind::Expense => "Expense",
                                                        };
                                                        let amount = r.amount_cents
                                                            .map(|c| format!("${:.0}", c as f64 / 100.0))
                                                            .unwrap_or_else(|| "—".into());
                                                        let meta = category_label(&r.category);
                                                        view! {
                                                            <div class="hub-activity-rail__row">
                                                                <StatusPill
                                                                    label=kind_label.to_string()
                                                                    tone=StatusPillTone::Neutral
                                                                />
                                                                <div class="hub-activity-rail__body">
                                                                    <p class="hub-activity-rail__row-title">
                                                                        {r.title.clone()}
                                                                    </p>
                                                                    <p class="hub-activity-rail__row-meta">{meta}</p>
                                                                </div>
                                                                <strong>{amount}</strong>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            </div>
                                        }
                                    }
                                />
                            </section>
                        }.into_any()
                    }
                    Some(Err(e)) => view! {
                        <div class="folio-empty">
                            <p>{format!("Could not load documents: {e}")}</p>
                        </div>
                    }.into_any(),
                    None => view! { <div class="folio-empty">"Loading…"</div> }.into_any(),
                }}
            </Suspense>

            <Show when=move || show_export.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:24rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Export for taxes"</h3>
                            <button
                                type="button"
                                class="modal-close"
                                on:click=move |_| show_export.set(false)
                            >
                                "✕"
                            </button>
                        </div>
                        <div class="modal-body">
                            <p class="proj-section__hint" style="margin:0;font-size:0.875rem;line-height:1.45;">
                                "Tax ZIP and CSV export is "
                                <strong>"Not available"</strong>
                                " yet. Group and review expenses here; download packs will land in a later release."
                            </p>
                        </div>
                        <div class="modal-footer">
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary"
                                on:click=move |_| show_export.set(false)
                            >
                                "Close"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
