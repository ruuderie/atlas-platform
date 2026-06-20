use leptos::prelude::*;

/// Shared CRM pagination bar.
/// `page`: current 1-based page signal.
/// `per_page`: records per page (used to detect last page).
/// `count`: reactive count of records in the current page result.
#[component]
pub fn Pagination(
    page: RwSignal<u64>,
    per_page: u64,
    count: Signal<usize>,
) -> impl IntoView {
    let on_prev = move |_| { if page.get() > 1 { page.update(|p| *p -= 1); } };
    let on_next = move |_| { if count.get() as u64 >= per_page { page.update(|p| *p += 1); } };

    view! {
        <div class="crm-pagination">
            <span>
                "Page " {move || page.get().to_string()}
                " · " {move || count.get().to_string()}
                " records"
            </span>
            <div class="crm-pagination-btns">
                <button
                    class=move || {
                        if page.get() <= 1 { "btn btn-ghost btn-sm" } else { "btn btn-ghost btn-sm" }
                    }
                    disabled=move || page.get() <= 1
                    on:click=on_prev
                >
                    "← Prev"
                </button>
                <button
                    class="btn btn-ghost btn-sm"
                    disabled=move || (count.get() as u64) < per_page
                    on:click=on_next
                >
                    "Next →"
                </button>
            </div>
        </div>
    }
}
