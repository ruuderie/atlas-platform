//! Property documents & expenses — `/l/assets/:id/documents`

use crate::atlas_client::{authenticated_get, session_token_from_request};
use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::property_tab_bar::{PropertyTab, PropertyTabBar};
use crate::components::status_pill::{StatusPill, StatusPillTone};
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyDocumentKind {
    Vault,
    Expense,
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

#[server(GetPropertyDocuments, "/api")]
pub async fn get_property_documents(
    asset_id: Uuid,
    project_id: Option<Uuid>,
) -> Result<Vec<PropertyDocumentRow>, ServerFnError> {
    let token = session_token_from_request().await.map_err(ServerFnError::new)?;
    let mut path = format!("/api/folio/assets/{asset_id}/documents");
    if let Some(pid) = project_id {
        path.push_str(&format!("?project_id={pid}"));
    }
    authenticated_get(&path, &token, None)
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

    let docs = Resource::new(
        move || (asset_id.get(), project_id.get()),
        |(aid, pid)| async move {
            if aid.is_nil() {
                return Ok(Vec::new());
            }
            get_property_documents(aid, pid).await
        },
    );

    view! {
        <div class="landlord-list-page docs-page">
            <PageHeader
                title=Signal::derive(|| "Documents & expenses".to_string())
                subtitle=Signal::derive(|| "Vault files + paid work-order costs".to_string())
            >
                <a class="folio-btn folio-btn--primary" href=FolioRoute::LandlordVault.path()>"Open vault"</a>
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
            {move || project_id.get().map(|p| view! {
                <p class="proj-section__hint" style="margin-bottom:1rem;">
                    {format!("Filtered by project {p}")}
                    " · "
                    <a
                        class="hub-activity-rail__all"
                        href=FolioRoute::LandlordAssetDocuments.path().replace(":id", &asset_id.get().to_string())
                    >"Clear filter"</a>
                </p>
            })}
            <Suspense fallback=move || view! { <div class="folio-empty">"Loading documents…"</div> }>
                {move || match docs.get() {
                    Some(Ok(list)) if list.is_empty() => view! {
                        <div class="folio-empty">
                            <p>"No documents or expenses for this property yet."</p>
                            <p class="proj-section__hint">"Upload to the vault or complete paid work orders."</p>
                        </div>
                    }.into_any(),
                    Some(Ok(list)) => view! {
                        <section class="proj-section">
                            <For
                                each=move || list.clone()
                                key=|r| r.id
                                children=move |r| {
                                    let kind_label = match r.kind {
                                        PropertyDocumentKind::Vault => "Vault",
                                        PropertyDocumentKind::Expense => "Expense",
                                    };
                                    let amount = r.amount_cents
                                        .map(|c| format!("${:.0}", c as f64 / 100.0))
                                        .unwrap_or_else(|| "—".into());
                                    view! {
                                        <div class="hub-activity-rail__row">
                                            <StatusPill label=kind_label.to_string() tone=StatusPillTone::Neutral/>
                                            <div class="hub-activity-rail__body">
                                                <p class="hub-activity-rail__row-title">{r.title.clone()}</p>
                                                <p class="hub-activity-rail__row-meta">{r.category.clone()}</p>
                                            </div>
                                            <strong>{amount}</strong>
                                        </div>
                                    }
                                }
                            />
                        </section>
                    }.into_any(),
                    Some(Err(e)) => view! {
                        <div class="folio-empty">
                            <p>{format!("Could not load documents: {e}")}</p>
                        </div>
                    }.into_any(),
                    None => view! { <div class="folio-empty">"Loading…"</div> }.into_any(),
                }}
            </Suspense>
        </div>
    }
}
