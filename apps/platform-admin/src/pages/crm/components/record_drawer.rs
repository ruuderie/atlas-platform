use leptos::prelude::*;

/// Generic slide-in detail drawer used by all CRM tabs.
/// `open`: signal that controls open state.
/// `title`, `subtitle`: strings shown in the panel header.
/// `detail_href`: URL for "View Full Record" button.
/// `children`: the panel body content.
/// `extra_actions`: optional extra action buttons rendered next to "View Full Record".
#[component]
pub fn RecordDrawer(
    open: RwSignal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] subtitle: Signal<String>,
    #[prop(into)] detail_href: Signal<String>,
    #[prop(default = None)] extra_actions: Option<AnyView>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=move || format!("panel-backdrop {}", if open.get() { "open" } else { "" })
            on:click=move |_| open.set(false)
        ></div>
        <div class=move || format!("detail-panel {}", if open.get() { "open" } else { "" })>
            <div class="panel-header">
                <div class="panel-header-top">
                    <div class="panel-identity">
                        <div class="panel-title-text">{move || title.get()}</div>
                        <div class="panel-subtitle-text">{move || subtitle.get()}</div>
                    </div>
                    <button class="panel-close" on:click=move |_| open.set(false)>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" style="width:14px;height:14px;">
                            <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
                        </svg>
                    </button>
                </div>
                <div class="panel-actions">
                    <a href=move || detail_href.get() class="btn btn-ghost btn-sm" style="text-decoration:none">
                        "View Full Record"
                    </a>
                    {extra_actions}
                </div>
                <div class="panel-tabs">
                    <button class="panel-tab active">"Overview"</button>
                </div>
            </div>
            <div class="panel-content" style="padding: 16px 20px;">
                {children()}
            </div>
        </div>
    }
}
