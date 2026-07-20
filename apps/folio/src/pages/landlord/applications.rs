//! Applications inbox — `/l/applications`
//! Thin list of rental applications with Offer lease after approve.

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::pages::landlord::tenant_profile::{
    list_applications, ApplicationRecord, ApplicationStatus,
};
use leptos::prelude::*;

#[component]
pub fn ApplicationsInbox() -> impl IntoView {
    let apps = Resource::new(|| (), |_| async move { list_applications().await });

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Applications".to_string())
                subtitle=Signal::derive(|| {
                    "Rental applications across your portfolio.".to_string()
                })
            >
                <a
                    class="folio-btn folio-btn--ghost press"
                    href=FolioRoute::LandlordLeaseCreate.path()
                >
                    "New lease"
                </a>
            </PageHeader>

            <Suspense fallback=|| view! { <div class="folio-empty">"Loading applications…"</div> }>
                {move || apps.get().map(|res| match res {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <p class="folio-empty__heading">"Could not load applications"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(list) if list.is_empty() => view! {
                        <div class="folio-empty">
                            <p class="folio-empty__heading">"No applications yet"</p>
                            <p class="folio-empty__sub">
                                "When renters apply, they show up here for review."
                            </p>
                        </div>
                    }.into_any(),
                    Ok(list) => view! {
                        <div class="leases-table-wrap">
                            <table class="leases-table">
                                <thead>
                                    <tr>
                                        <th class="leases-th">"Status"</th>
                                        <th class="leases-th">"Income"</th>
                                        <th class="leases-th">"Submitted"</th>
                                        <th class="leases-th"></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {list.into_iter().map(|app: ApplicationRecord| {
                                        let status = ApplicationStatus::from_str(&app.status);
                                        let income = app.monthly_income_cents
                                            .map(|c| format!("${:.0}/mo", c as f64 / 100.0))
                                            .unwrap_or_else(|| "\u{2014}".into());
                                        let submitted = app.submitted_at
                                            .map(|d| d.format("%Y-%m-%d").to_string())
                                            .unwrap_or_else(|| "\u{2014}".into());
                                        let profile_href = FolioRoute::LandlordTenantProfile
                                            .path()
                                            .replace(":id", &app.applicant_user_id.to_string());
                                        let offer_href = match app.target_asset_id {
                                            Some(aid) => format!(
                                                "{}?asset_id={}&user_id={}",
                                                FolioRoute::LandlordLeaseCreate.path(),
                                                aid,
                                                app.applicant_user_id
                                            ),
                                            None => format!(
                                                "{}?user_id={}",
                                                FolioRoute::LandlordLeaseCreate.path(),
                                                app.applicant_user_id
                                            ),
                                        };
                                        let approved = status == ApplicationStatus::Approved;
                                        view! {
                                            <tr class="leases-row">
                                                <td class="leases-td">
                                                    <span class={format!("lease-status-badge {}", status.pill_class())}>
                                                        {status.as_str()}
                                                    </span>
                                                </td>
                                                <td class="leases-td">{income}</td>
                                                <td class="leases-td">{submitted}</td>
                                                <td class="leases-td">
                                                    <div class="unit-actions" style="justify-content:flex-end;">
                                                        <a class="folio-btn folio-btn--ghost folio-btn--sm press" href=profile_href>
                                                            "Profile"
                                                        </a>
                                                        {if approved {
                                                            view! {
                                                                <a class="folio-btn folio-btn--primary folio-btn--sm press" href=offer_href>
                                                                    "Offer lease"
                                                                </a>
                                                            }.into_any()
                                                        } else {
                                                            ().into_any()
                                                        }}
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
