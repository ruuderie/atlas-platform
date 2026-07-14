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
}

#[component]
pub fn ActivityRail(
    #[prop(into)] items: Signal<Vec<ActivityRailItem>>,
    #[prop(into)] see_all_href: String,
) -> impl IntoView {
    view! {
        <aside class="hub-activity-rail" aria-label="Property activity">
            <div class="hub-activity-rail__head">
                <h3 class="hub-activity-rail__title">"Activity"</h3>
                <a class="hub-activity-rail__all" href=see_all_href>"See all"</a>
            </div>
            <p class="hub-activity-rail__hint">"This property only — portfolio feed is Activity"</p>
            <div class="hub-activity-rail__list">
                <Show
                    when=move || !items.get().is_empty()
                    fallback=|| view! {
                        <div class="folio-empty folio-empty--compact">
                            <p>"No recent activity on this property."</p>
                        </div>
                    }
                >
                    <For
                        each=move || items.get()
                        key=|i| i.id.clone()
                        children=move |i| {
                            view! {
                                <a class="hub-activity-rail__row press" href=i.href.clone()>
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
        </aside>
    }
}
