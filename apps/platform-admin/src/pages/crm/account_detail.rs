/// Account Detail Page — full stitch-aligned implementation for atlas_accounts
use leptos::prelude::*;
use crate::api::crm::get_contacts;
use crate::api::models::{AccountModel, ContactModel};

fn fmt_opt(v: &Option<String>) -> String {
    v.as_deref().unwrap_or("—").to_string()
}

fn fmt_i32(v: &Option<i32>) -> String {
    v.map(|n| n.to_string()).unwrap_or_else(|| "—".into())
}

fn fmt_f64(v: &Option<f64>) -> String {
    v.map(|n| format!("${:.0}", n)).unwrap_or_else(|| "—".into())
}

fn fmt_i16(v: &Option<i16>) -> String {
    v.map(|n| n.to_string()).unwrap_or_else(|| "—".into())
}

fn org_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .map(|c| c.to_uppercase().to_string())
        .collect()
}

#[component]
pub fn AccountDetail(account: AccountModel) -> impl IntoView {
    let a = account.clone();
    let id = a.id.clone();

    let active_tab = RwSignal::new("overview");

    // ── Contacts for this account (fetched once, filtered client-side) ────────
    let account_id_for_filter = StoredValue::new(id.clone());
    let contacts_res: LocalResource<Vec<ContactModel>> = {
        LocalResource::new(move || {
            let fid = account_id_for_filter.get_value();
            async move {
                get_contacts(None, 1, 200, None)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|c| c.account_id == fid)
                    .collect()
            }
        })
    };

    // ── Pre-extracted display values ──────────────────────────────────────────
    let name          = a.name.clone();
    let initials      = org_initials(&name);
    let type_label    = a.account_type.label();
    let type_class    = a.account_type.badge_class();
    let status_label  = a.status.label();
    let status_color  = a.status.color();
    let account_id    = StoredValue::new(a.id.clone());
    let domain_val    = fmt_opt(&a.domain);
    let website_val   = fmt_opt(&a.website);
    let phone_val     = fmt_opt(&a.company_phone);
    let email_val     = fmt_opt(&a.company_email);
    let industry_val  = fmt_opt(&a.industry);
    let employees_val = fmt_i32(&a.num_employees);
    let revenue_val   = fmt_f64(&a.annual_revenue);
    let city_val      = fmt_opt(&a.city);
    let country_val   = fmt_opt(&a.country);

    let location_str  = {
        let mut parts = Vec::new();
        if city_val != "—"    { parts.push(city_val.clone()); }
        if country_val != "—" { parts.push(country_val.clone()); }
        if parts.is_empty() { "—".into() } else { parts.join(", ") }
    };

    // ── Detail rows (stored to avoid FnOnce) ─────────────────────────────────
    let detail_rows = StoredValue::new(vec![
        ("Account ID",      a.id.clone(),                     true),
        ("Name",            a.name.clone(),                   false),
        ("DBA Name",        fmt_opt(&a.dba_name),             false),
        ("Type",            type_label.to_string(),           false),
        ("Status",          status_label.to_string(),         false),
        ("Domain",          fmt_opt(&a.domain),               false),
        ("Website",         fmt_opt(&a.website),              false),
        ("Company Phone",   fmt_opt(&a.company_phone),        false),
        ("Company Email",   fmt_opt(&a.company_email),        false),
        ("Industry",        fmt_opt(&a.industry),             false),
        ("Company Type",    fmt_opt(&a.company_type),         false),
        ("Employees",       fmt_i32(&a.num_employees),        false),
        ("Annual Revenue",  fmt_f64(&a.annual_revenue),       false),
        ("Year Est.",       fmt_i16(&a.year_established),     false),
        ("Street",          fmt_opt(&a.street_address),       false),
        ("City",            fmt_opt(&a.city),                 false),
        ("State",           fmt_opt(&a.state),                false),
        ("Postal Code",     fmt_opt(&a.postal_code),          false),
        ("Country",         fmt_opt(&a.country),              false),
        ("Data Source",     fmt_opt(&a.data_source),          false),
        ("Created At",      fmt_opt(&a.created_at),           true),
        ("Updated At",      fmt_opt(&a.updated_at),           true),
    ]);

    view! {
        <div style="display:flex;flex-direction:column;height:100%;overflow:hidden;">

            // ── Hero ─────────────────────────────────────────────────────────
            <div style="padding:14px 24px;border-bottom:1px solid var(--border-default);flex-shrink:0;">
                <div style="font-size:11px;color:var(--text-muted);margin-bottom:8px;display:flex;align-items:center;gap:5px;">
                    <a href="/accounts" style="color:var(--text-link);text-decoration:none;">"Accounts"</a>
                    " › "
                    {name.clone()}
                </div>
                <div style="display:flex;align-items:flex-start;justify-content:space-between;">
                    <div style="display:flex;gap:14px;align-items:flex-start;">
                        // Org avatar
                        <div style="width:52px;height:52px;border-radius:10px;background:var(--cobalt-dim);border:1px solid var(--cobalt);display:flex;align-items:center;justify-content:center;font-size:18px;font-weight:700;color:var(--cobalt);flex-shrink:0;">
                            {initials}
                        </div>
                        <div>
                            <div style="font-size:20px;font-weight:700;letter-spacing:-0.3px;display:flex;align-items:center;gap:8px;flex-wrap:wrap;margin-bottom:4px;">
                                {name.clone()}
                                <span class={type_class} style="font-size:9px;">{type_label}</span>
                                <span style=format!("font-size:9px;font-weight:700;padding:2px 7px;border-radius:3px;border:1px solid {0};background:transparent;color:{0};", status_color)>
                                    {status_label}
                                </span>
                            </div>
                            // Sub-identity row
                            <div style="display:flex;gap:12px;flex-wrap:wrap;margin-bottom:10px;font-size:12px;color:var(--text-secondary);">
                                {(!domain_val.is_empty() && domain_val != "—").then(|| {
                                    let d = domain_val.clone();
                                    view! { <span>"🌐 " {d}</span> }
                                })}
                                {(!industry_val.is_empty() && industry_val != "—").then(|| {
                                    let i = industry_val.clone();
                                    view! { <span>{"🏭 "} {i}</span> }
                                })}
                                {(location_str != "—").then(|| {
                                    let l = location_str.clone();
                                    view! { <span>"📍 " {l}</span> }
                                })}
                            </div>
                            // Channel row
                            <div style="display:flex;gap:10px;flex-wrap:wrap;font-size:12px;">
                                {(!phone_val.is_empty() && phone_val != "—").then(|| {
                                    let p = phone_val.clone();
                                    view! {
                                        <span style="display:flex;align-items:center;gap:5px;color:var(--text-secondary);">
                                            <span style="width:18px;height:18px;border-radius:3px;background:var(--green-dim);color:var(--green);display:flex;align-items:center;justify-content:center;font-size:9px;">"☎"</span>
                                            {p}
                                        </span>
                                    }
                                })}
                                {(!email_val.is_empty() && email_val != "—").then(|| {
                                    let e = email_val.clone();
                                    view! {
                                        <span style="display:flex;align-items:center;gap:5px;color:var(--text-link);">
                                            <span style="width:18px;height:18px;border-radius:3px;background:var(--cobalt-dim);color:var(--cobalt);display:flex;align-items:center;justify-content:center;font-size:9px;">"✉"</span>
                                            {e}
                                        </span>
                                    }
                                })}
                                {(!website_val.is_empty() && website_val != "—").then(|| {
                                    let w = website_val.clone();
                                    let w2 = w.clone();
                                    view! {
                                        <a href={w} target="_blank" style="display:flex;align-items:center;gap:5px;color:var(--text-link);text-decoration:none;">
                                            <span style="width:18px;height:18px;border-radius:3px;background:var(--violet-dim);color:var(--violet);display:flex;align-items:center;justify-content:center;font-size:9px;">"↗"</span>
                                            {w2}
                                        </a>
                                    }
                                })}
                            </div>
                        </div>
                    </div>
                    // Top-right: ID
                    <div style="text-align:right;font-size:10px;color:var(--text-muted);font-family:monospace;flex-shrink:0;">
                        {account_id.get_value()}
                    </div>
                </div>
            </div>

            // ── KPI Strip ─────────────────────────────────────────────────────
            <div style="display:flex;border-bottom:1px solid var(--border-default);flex-shrink:0;">
                <div class="kpi">
                    <div class="kpi-label">"Contacts"</div>
                    <div class="kpi-value mono">{move || contacts_res.get().map(|c| c.len().to_string()).unwrap_or_else(|| "—".into())}</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Open Opps"</div>
                    <div class="kpi-value" style="color:var(--text-muted);">"—"</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Employees"</div>
                    <div class="kpi-value mono">{employees_val.clone()}</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Annual Revenue"</div>
                    <div class="kpi-value mono">{revenue_val.clone()}</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Status"</div>
                    <div class="kpi-value" style=format!("color:{};", status_color)>{status_label}</div>
                </div>
            </div>

            // ── Tab Bar ───────────────────────────────────────────────────────
            <div class="tab-bar" style="display:flex;padding:0 24px;border-bottom:1px solid var(--border-default);flex-shrink:0;">
                <button class=move || format!("tab {}", if active_tab.get() == "overview"  { "active" } else { "" }) on:click=move |_| active_tab.set("overview") >"Overview"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "contacts"  { "active" } else { "" }) on:click=move |_| active_tab.set("contacts") >"Contacts"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "details"   { "active" } else { "" }) on:click=move |_| active_tab.set("details")  >"Details · atlas_accounts"</button>
            </div>

            // ── Tab Content ───────────────────────────────────────────────────
            <div class="content-body" style="flex:1;overflow-y:auto;padding:20px 24px;">
                {move || match active_tab.get() {

                    // ── Overview ──────────────────────────────────────────────
                    "overview" => view! {
                        <div class="col-7-5">
                            // Left — primary contacts list
                            <div>
                                <div class="card">
                                    <div class="card-hdr" style="display:flex;align-items:center;justify-content:space-between;padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                        <span class="card-title" style="font-size:11.5px;font-weight:600;">"Contacts"</span>
                                        <a href={format!("/contacts?account_id={}", account_id.get_value())} style="font-size:11px;color:var(--text-link);text-decoration:none;">"View all →"</a>
                                    </div>
                                    <Suspense fallback=move || view! { <div style="padding:16px;color:var(--text-muted);font-size:12px;">"Loading contacts…"</div> }>
                                        {move || {
                                            let contacts = contacts_res.get().unwrap_or_default();
                                            if contacts.is_empty() {
                                                return view! {
                                                    <div style="padding:24px;text-align:center;color:var(--text-muted);font-size:12px;">"No contacts linked to this account."</div>
                                                }.into_any();
                                            }
                                            view! {
                                                <div>
                                                    {contacts.into_iter().map(|c| {
                                                        let c_id = c.id.clone();
                                                        let name = c.display_name().to_string();
                                                        let name2 = name.clone();
                                                        let email = c.email.clone().unwrap_or_default();
                                                        let title = c.title.clone().unwrap_or_default();
                                                        let is_primary = c.is_primary;
                                                        view! {
                                                            <a href={format!("/contacts/{}", c_id)}
                                                                style="display:flex;align-items:center;gap:10px;padding:10px 14px;border-bottom:1px solid var(--border-subtle);text-decoration:none;cursor:pointer;">
                                                                <div style="width:30px;height:30px;border-radius:50%;background:var(--violet-dim);border:1px solid var(--violet);display:flex;align-items:center;justify-content:center;font-size:11px;font-weight:700;color:var(--violet);flex-shrink:0;">
                                                                    {name2.chars().next().unwrap_or('?').to_string()}
                                                                </div>
                                                                <div style="flex:1;">
                                                                    <div style="display:flex;align-items:center;gap:6px;">
                                                                        <span style="font-size:12.5px;font-weight:500;color:var(--text-primary);">{name}</span>
                                                                        {is_primary.then(|| view! {
                                                                            <span class="tag" style="font-size:9px;color:var(--amber);border-color:var(--amber);">"Primary"</span>
                                                                        })}
                                                                    </div>
                                                                    <div style="font-size:11px;color:var(--text-muted);margin-top:2px;">{if !title.is_empty() { title } else { email }}</div>
                                                                </div>
                                                                <span style="font-size:11px;color:var(--text-link);">"→"</span>
                                                            </a>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            }.into_any()
                                        }}
                                    </Suspense>
                                </div>
                            </div>

                            // Right rail — Firmographics + Address
                            <div>
                                <crate::pages::billing::scorecard_panel::ScorecardPanel
                                    entity_type="atlas_account".to_string()
                                    entity_id=account_id.get_value()
                                    subject_label=name.clone()
                                />
                                // Firmographics
                                <div class="card" style="margin-bottom:14px;">
                                    <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                        <span class="card-title" style="font-size:11.5px;font-weight:600;">"Firmographics"</span>
                                    </div>
                                    {[
                                        ("Industry",    industry_val.clone()),
                                        ("Employees",   employees_val.clone()),
                                        ("Revenue",     revenue_val.clone()),
                                        ("Domain",      domain_val.clone()),
                                        ("Website",     website_val.clone()),
                                    ].into_iter().map(|(label, value)| {
                                        let is_set = value != "—";
                                        let color = if is_set { "var(--text-primary)" } else { "var(--text-muted)" };
                                        view! {
                                            <div style="display:flex;justify-content:space-between;align-items:center;padding:7px 14px;border-bottom:1px solid var(--border-subtle);">
                                                <span style="font-size:12px;color:var(--text-secondary);">{label}</span>
                                                <span style=format!("font-size:12px;font-weight:500;color:{};", color)>{value}</span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>

                                // Address
                                <div class="card">
                                    <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                        <span class="card-title" style="font-size:11.5px;font-weight:600;">"Address"</span>
                                    </div>
                                    {[
                                        ("Street",  fmt_opt(&a.street_address)),
                                        ("City",    fmt_opt(&a.city)),
                                        ("State",   fmt_opt(&a.state)),
                                        ("ZIP",     fmt_opt(&a.postal_code)),
                                        ("Country", fmt_opt(&a.country)),
                                    ].into_iter().map(|(label, value)| {
                                        let is_set = value != "—";
                                        let color = if is_set { "var(--text-primary)" } else { "var(--text-muted)" };
                                        view! {
                                            <div style="display:flex;justify-content:space-between;align-items:center;padding:7px 14px;border-bottom:1px solid var(--border-subtle);">
                                                <span style="font-size:12px;color:var(--text-secondary);">{label}</span>
                                                <span style=format!("font-size:12px;font-weight:500;color:{};", color)>{value}</span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        </div>
                    }.into_any(),

                    // ── Contacts tab ──────────────────────────────────────────
                    "contacts" => view! {
                        <div class="card">
                            <div class="card-hdr" style="display:flex;align-items:center;justify-content:space-between;padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                <span class="card-title" style="font-size:11.5px;font-weight:600;">"All Contacts · atlas_contacts"</span>
                            </div>
                            <table style="width:100%;border-collapse:collapse;">
                                <thead>
                                    <tr>
                                        <th style="font-size:9.5px;font-weight:600;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);padding:6px 12px;text-align:left;border-bottom:1px solid var(--border-default);">"Name"</th>
                                        <th style="font-size:9.5px;font-weight:600;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);padding:6px 12px;text-align:left;border-bottom:1px solid var(--border-default);">"Title"</th>
                                        <th style="font-size:9.5px;font-weight:600;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);padding:6px 12px;text-align:left;border-bottom:1px solid var(--border-default);">"Email"</th>
                                        <th style="font-size:9.5px;font-weight:600;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);padding:6px 12px;text-align:left;border-bottom:1px solid var(--border-default);">"Phone"</th>
                                        <th style="font-size:9.5px;font-weight:600;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);padding:6px 12px;text-align:left;border-bottom:1px solid var(--border-default);">"Role"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <Suspense>
                                        {move || contacts_res.get().unwrap_or_default().into_iter().map(|c| {
                                            let c_id = c.id.clone();
                                            let name = c.display_name().to_string();
                                            let email = c.email.clone().unwrap_or_else(|| "—".into());
                                            let phone = c.phone.clone().unwrap_or_else(|| "—".into());
                                            let title = c.title.clone().unwrap_or_else(|| "—".into());
                                            let is_primary = c.is_primary;
                                            view! {
                                                <tr style="border-bottom:1px solid var(--border-subtle);cursor:pointer;"
                                                    on:click=move |_| { let _ = web_sys::window().and_then(|w| w.location().set_href(&format!("/contacts/{}", c_id)).ok()); }>
                                                    <td style="padding:8px 12px;">
                                                        <div style="font-size:12px;font-weight:500;color:var(--text-primary);">{name}</div>
                                                    </td>
                                                    <td style="padding:8px 12px;font-size:12px;color:var(--text-secondary);">{title}</td>
                                                    <td style="padding:8px 12px;font-size:12px;color:var(--text-secondary);">{email}</td>
                                                    <td style="padding:8px 12px;font-size:12px;color:var(--text-secondary);">{phone}</td>
                                                    <td style="padding:8px 12px;">
                                                        {is_primary.then(|| view! {
                                                            <span class="tag" style="font-size:9px;color:var(--amber);border-color:var(--amber);">"Primary"</span>
                                                        })}
                                                    </td>
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </Suspense>
                                </tbody>
                            </table>
                        </div>
                    }.into_any(),

                    // ── Details: full atlas_accounts field grid ───────────────
                    "details" => view! {
                        <div class="card">
                            <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                <span class="card-title" style="font-size:11.5px;font-weight:600;">"All Fields · atlas_accounts"</span>
                            </div>
                            <div style="display:grid;grid-template-columns:1fr 1fr;">
                                {detail_rows.get_value().into_iter().enumerate().map(|(i, (label, value, mono))| {
                                    let border_right = if i % 2 == 0 { "1px solid var(--border-subtle)" } else { "none" };
                                    let is_set = value != "—";
                                    let color = if is_set { "var(--text-primary)" } else { "var(--text-muted)" };
                                    let font_family = if mono { "font-family:monospace;" } else { "" };
                                    view! {
                                        <div style=format!("display:flex;flex-direction:column;padding:8px 14px;border-bottom:1px solid var(--border-subtle);border-right:{};", border_right)>
                                            <span style="font-size:9.5px;font-weight:600;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);margin-bottom:2px;">{label}</span>
                                            <span style=format!("font-size:12px;color:{};{}", color, font_family)>{value}</span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any(),

                    _ => view! { <div></div> }.into_any(),
                }}
            </div>
        </div>
    }
}
