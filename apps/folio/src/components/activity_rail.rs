//! Property Activity right rail — property-scoped feed (not portfolio `/l/maintenance`).

use crate::components::status_pill::{StatusPill, StatusPillTone};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityRailItem {
    pub id: String,
    pub kind_label: String,
    pub title: String,
    pub meta: String,
    pub href: String,
    pub tone: StatusPillTone,
    /// Layout slot for WO/receipt photo; true shows a 40px thumb placeholder until attachment URLs exist.
    pub show_photo_slot: bool,
}

#[component]
pub fn ActivityRail(
    #[prop(into)] items: Signal<Vec<ActivityRailItem>>,
    #[prop(into)] see_all_href: String,
    #[prop(optional, into)] subtitle: Option<&'static str>,
    #[prop(optional, into)] new_wo_href: Option<String>,
    #[prop(optional, into)] schedule_href: Option<String>,
) -> impl IntoView {
    let hint = subtitle.unwrap_or("This property");
    let new_wo = new_wo_href;
    let schedule = schedule_href;

    view! {
        <aside class="hub-activity-rail" aria-label="Property activity">
            <div class="hub-activity-rail__head">
                <div>
                    <h3 class="hub-activity-rail__title">"Activity"</h3>
                    <p class="hub-activity-rail__hint">{hint}</p>
                </div>
                <a class="hub-activity-rail__all" href=see_all_href.clone()>"Full feed"</a>
            </div>
            <div class="hub-activity-rail__list">
                <Show
                    when=move || !items.get().is_empty()
                    fallback=|| view! {
                        <div class="folio-empty folio-empty--compact">
                            <p>"No open items on this property."</p>
                        </div>
                    }
                >
                    <For
                        each=move || items.get()
                        key=|i| i.id.clone()
                        children=move |i| {
                            let show_photo = i.show_photo_slot;
                            view! {
                                <a class="hub-activity-rail__row press" href=i.href.clone()>
                                    {show_photo.then(|| view! {
                                        <div class="hub-activity-rail__thumb" aria-hidden="true">
                                            <span class="material-symbols-outlined">"photo"</span>
                                        </div>
                                    })}
                                    <StatusPill label=i.kind_label.clone() tone=i.tone/>
                                    <div class="hub-activity-rail__body">
                                        <p class="hub-activity-rail__row-title">{i.title.clone()}</p>
                                        <p class="hub-activity-rail__row-meta">{i.meta.clone()}</p>
                                    </div>
                                </a>
                            }
                        }
                    />
                </Show>
            </div>
            {(new_wo.is_some() || schedule.is_some()).then(|| {
                let wo = new_wo.clone();
                let sch = schedule.clone();
                view! {
                    <div class="hub-activity-rail__footer">
                        {wo.map(|href| view! {
                            <a class="folio-btn folio-btn--primary folio-btn--sm press" href=href>"New WO"</a>
                        })}
                        {sch.map(|href| view! {
                            <a class="folio-btn folio-btn--ghost folio-btn--sm press" href=href>"Schedule"</a>
                        })}
                    </div>
                }
            })}
        </aside>
    }
}
