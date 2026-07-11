use crate::api::models::TemplateModel;
use crate::api::templates::get_templates;
use leptos::prelude::*;

#[component]
pub fn Templates() -> impl IntoView {
    let (templates, set_templates) = signal(Vec::<TemplateModel>::new());

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(data) = get_templates().await {
                set_templates.set(data);
            }
        });
    });

    view! {
        <div class="main-canvas">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Data Templates"</h1>
                    <p class="page-subtitle">"Manage dynamic data schemas for listings based on categories."</p>
                </div>
                <div class="page-actions">
                    <a href="/network/templates/new">
                        <button class="btn btn-primary">"+ Create Template"</button>
                    </a>
                </div>
            </div>

            // ── Table ──
            <div class="section">
                <div class="section-header">
                    <div class="section-title">"Templates"<span class="section-count">{move || templates.get().len().to_string()}</span></div>
                </div>
                <div style="overflow-x:auto;">
                    <table>
                        <thead>
                            <tr>
                                <th>"Name"</th>
                                <th>"Type"</th>
                                <th>"Status"</th>
                                <th>"Description"</th>
                                <th style="text-align:right">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || templates.get().into_iter().map(|item| {
                                let id = item.id.clone();
                                let detail_url = format!("/network/templates/{}", id);
                                let (status_color, status_bg, status_text) = if item.is_active {
                                    ("var(--green)", "var(--green-dim)", "Active")
                                } else {
                                    ("var(--text-muted)", "var(--bg-surface)", "Inactive")
                                };
                                view! {
                                    <tr>
                                        <td><div style="font-weight:600">{item.name}</div></td>
                                        <td>
                                            <span class="plan-badge" style="color:var(--cobalt);border-color:var(--cobalt);background:var(--cobalt-dim)">{item.template_type}</span>
                                        </td>
                                        <td>
                                            <span class="plan-badge" style=format!("color:{};border-color:{};background:{}", status_color, status_color, status_bg)>{status_text}</span>
                                        </td>
                                        <td style="max-width:250px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--text-muted)">{item.description}</td>
                                        <td style="text-align:right">
                                            <a href=detail_url><button class="btn btn-ghost btn-sm">"Manage"</button></a>
                                        </td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
