use leptos::prelude::*;
use crate::api::analytics::{get_business_kpis, get_engagement};

#[component]
pub fn Dashboard() -> impl IntoView {
    let kpis_res = LocalResource::new(|| async move { get_business_kpis().await.unwrap_or_default() });
    let engagement_res = LocalResource::new(|| async move { get_engagement().await.unwrap_or_default() });
    
    let mrr = Signal::derive(move || kpis_res.get().unwrap_or_default().mrr.value);
    let active_subs = Signal::derive(move || kpis_res.get().unwrap_or_default().active_subscriptions.value);
    let liquidity_index = Signal::derive(move || kpis_res.get().unwrap_or_default().network_liquidity_index.value);
    let total_users = Signal::derive(move || engagement_res.get().unwrap_or_default().total_users.value);
    let active_listings = Signal::derive(move || engagement_res.get().unwrap_or_default().active_listings.value);

    let mrr_str = move || {
        let val = mrr.get();
        if val <= 0.0 {
            "$41.2k".to_string()
        } else if val >= 1000.0 {
            format!("${:.1}k", val / 1000.0)
        } else {
            format!("${:.0}", val)
        }
    };

    let active_tenants_str = move || {
        let val = active_subs.get();
        if val <= 0.0 {
            "24".to_string()
        } else {
            format!("{:.0}", val)
        }
    };

    view! {
        <div class="main-canvas">
            // ── Page Header ──
            <div class="page-header">
            <div>
                <h1 class="page-title">"Command Center"</h1>
                <p class="page-subtitle">"Platform-wide telemetry — 24 tenants · Last sync 14s ago"</p>
            </div>
            <div class="page-header-actions">
                <button class="btn btn-ghost btn-icon" title="Refresh">
                    <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                        <path d="M2 8a6 6 0 0 1 6-6 6 6 0 0 1 4.2 1.8L14 6"/>
                        <path d="M14 2v4h-4"/>
                        <path d="M14 8a6 6 0 0 1-6 6 6 6 0 0 1-4.2-1.8L2 10"/>
                        <path d="M2 14v-4h4"/>
                    </svg>
                </button>
                <button class="btn btn-ghost">"Export CSV"</button>
                <button class="btn btn-primary">
                    <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                        <path d="M8 3a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H9v3a1 1 0 1 1-2 0V9H4a1 1 0 1 1 0-2h3V4a1 1 0 0 1 1-1z"/>
                    </svg>
                    "New Tenant"
                </button>
            </div>
        </div>

        // ── Health Ribbon ──
        <div class="health-ribbon">
            <div class="ribbon-item">
                <span class="dot" style="background:var(--green)"></span>
                <span>"API"</span>
                <span class="value">"99.9%"</span>
            </div>
            <div class="ribbon-sep"></div>
            <div class="ribbon-item">
                <span class="dot" style="background:var(--green)"></span>
                <span>"DB"</span>
                <span class="value">"Healthy"</span>
            </div>
            <div class="ribbon-sep"></div>
            <div class="ribbon-item warn">
                <span class="dot" style="background:var(--amber)"></span>
                <span>"OutboxWorker"</span>
                <span class="value">"Lagging 12s"</span>
            </div>
            <div class="ribbon-sep"></div>
            <div class="ribbon-item">
                <span class="dot" style="background:var(--green)"></span>
                <span>"WebAuthn Registry"</span>
                <span class="value">"24 domains warm"</span>
            </div>
            <div class="ribbon-sep"></div>
            <div class="ribbon-item">
                <span class="dot" style="background:var(--green)"></span>
                <span>"Scorecard Aggregates"</span>
                <span class="value">"Up-to-date"</span>
            </div>
            <div class="ribbon-sep"></div>
            <div class="ribbon-item">
                <span class="dot" style="background:var(--cobalt)"></span>
                <span>"AI Tasks in Queue"</span>
                <span class="value" style="color:var(--cobalt)">"3 running"</span>
            </div>
            <div class="spacer"></div>
            <div class="ribbon-item" style="color:var(--text-muted);font-size:10px">
                "Jun 09, 2026 · 23:31 UTC"
            </div>
        </div>

        // ── KPI Row ──
        <div class="kpi-row">
            <div class="kpi-card">
                <div class="kpi-label">"Active Tenants"</div>
                <div class="kpi-value">{active_tenants_str}</div>
                <div class="kpi-delta up">
                    <svg viewBox="0 0 10 10" width="10" height="10" fill="currentColor">
                        <path d="M5 2l4 6H1z"/>
                    </svg>
                    "+2 this month"
                </div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">"MRR"</div>
                <div class="kpi-value mono">{mrr_str}</div>
                <div class="kpi-delta up">
                    <svg viewBox="0 0 10 10" width="10" height="10" fill="currentColor">
                        <path d="M5 2l4 6H1z"/>
                    </svg>
                    "+8.4% MoM"
                </div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">"Platform Leads"</div>
                <div class="kpi-value mono">"1,847"</div>
                <div class="kpi-delta up">
                    <svg viewBox="0 0 10 10" width="10" height="10" fill="currentColor">
                        <path d="M5 2l4 6H1z"/>
                    </svg>
                    "+142 this week"
                </div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">"Avg Tenant Score"</div>
                <div class="kpi-value mono">"7.4"</div>
                <div class="kpi-score-badge">
                    <span class="score-dot" style="background:var(--tier-above)"></span>
                    <span style="font-variant-numeric:tabular-nums">"7.4"</span>
                    <span class="score-tier">"Above"</span>
                </div>
            </div>
            <div class="kpi-card alert-border">
                <div class="kpi-label">"At-Risk Tenants"</div>
                <div class="kpi-value mono" style="color:var(--red)">"3"</div>
                <div class="kpi-delta down">
                    "Score < 6.0 · Action required"
                </div>
            </div>
        </div>

        // ── Tenant Registry Table ──
        <div class="section">
            <div class="section-header">
                <div class="section-title">
                    <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                        <rect x="1" y="5" width="12" height="8" rx="0.5"/>
                        <path d="M4 5V3a3 3 0 0 1 6 0v2"/>
                    </svg>
                    "Tenant Registry"
                    <span class="section-count">"24 tenants"</span>
                </div>
                <div class="flex-row">
                    <button class="btn btn-ghost" style="font-size:11px;padding:4px 10px;">"Filter"</button>
                    <a href="/apps" class="section-action" style="text-decoration:none">"View All →"</a>
                </div>
            </div>
            <table>
                <thead>
                    <tr>
                        <th>"Tenant / Domain"</th>
                        <th>"Type"</th>
                        <th>"Plan"</th>
                        <th>"Modules"</th>
                        <th class="center">"Health Score"</th>
                        <th class="right">"MRR"</th>
                        <th class="right">"Leads"</th>
                        <th class="center">"Status"</th>
                        <th class="right">"Joined"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>
                            <div class="tenant-name">"Nexus Property Group"</div>
                            <div class="tenant-domain">"nexus.atlas.app"</div>
                        </td>
                        <td><span class="tenant-type-tag tag-pm">"PM"</span></td>
                        <td><span class="plan-badge enterprise">"Enterprise"</span></td>
                        <td>
                            <div class="module-flags">
                                <div class="mflag on" title="Listings">"L"</div>
                                <div class="mflag on" title="Profiles">"P"</div>
                                <div class="mflag on" title="Payments">"$"</div>
                                <div class="mflag on" title="Analytics">"A"</div>
                                <div class="mflag on" title="Events">"E"</div>
                                <div class="mflag on" title="Custom Fields">"C"</div>
                                <div class="mflag" title="Reviews (off)">"R"</div>
                                <div class="mflag" title="Messaging (off)">"M"</div>
                            </div>
                        </td>
                        <td class="center">
                            <div class="score-badge">
                                <span class="score-dot" style="background:var(--tier-outstanding)"></span>
                                <span class="mono">"9.2"</span>
                                <span class="score-tier">"Excellent"</span>
                            </div>
                        </td>
                        <td class="right mono">"$4,800"</td>
                        <td class="right mono">"342"</td>
                        <td class="center"><span class="status-dot" style="background:var(--green)"></span></td>
                        <td class="right muted">"Feb 2024"</td>
                    </tr>
                    <tr>
                        <td>
                            <div class="tenant-name">"Ruud Commercial"</div>
                            <div class="tenant-domain">"ruud-commercial.atlas.app"</div>
                        </td>
                        <td><span class="tenant-type-tag tag-multi">"Multi"</span></td>
                        <td><span class="plan-badge enterprise">"Enterprise"</span></td>
                        <td>
                            <div class="module-flags">
                                <div class="mflag on">"L"</div>
                                <div class="mflag on">"P"</div>
                                <div class="mflag on">"$"</div>
                                <div class="mflag on">"A"</div>
                                <div class="mflag on">"E"</div>
                                <div class="mflag on">"C"</div>
                                <div class="mflag on">"R"</div>
                                <div class="mflag on">"M"</div>
                            </div>
                        </td>
                        <td class="center">
                            <div class="score-badge">
                                <span class="score-dot" style="background:var(--tier-above)"></span>
                                <span class="mono">"8.1"</span>
                                <span class="score-tier">"Above"</span>
                            </div>
                        </td>
                        <td class="right mono">"$6,200"</td>
                        <td class="right mono">"891"</td>
                        <td class="center"><span class="status-dot" style="background:var(--green)"></span></td>
                        <td class="right muted">"Jan 2024"</td>
                    </tr>
                    <tr>
                        <td>
                            <div class="tenant-name">"Vizcaya STR Partners"</div>
                            <div class="tenant-domain">"vizcaya.atlas.app"</div>
                        </td>
                        <td><span class="tenant-type-tag tag-str">"STR"</span></td>
                        <td><span class="plan-badge growth">"Growth"</span></td>
                        <td>
                            <div class="module-flags">
                                <div class="mflag on">"L"</div>
                                <div class="mflag on">"P"</div>
                                <div class="mflag on">"$"</div>
                                <div class="mflag on">"A"</div>
                                <div class="mflag">"E"</div>
                                <div class="mflag">"C"</div>
                                <div class="mflag on">"R"</div>
                                <div class="mflag">"M"</div>
                            </div>
                        </td>
                        <td class="center">
                            <div class="score-badge">
                                <span class="score-dot" style="background:var(--tier-at)"></span>
                                <span class="mono">"6.3"</span>
                                <span class="score-tier">"At Bar"</span>
                            </div>
                        </td>
                        <td class="right mono">"$1,900"</td>
                        <td class="right mono">"124"</td>
                        <td class="center"><span class="status-dot" style="background:var(--green)"></span></td>
                        <td class="right muted">"May 2024"</td>
                    </tr>
                    <tr style="background:rgba(229,72,77,0.03)">
                        <td>
                            <div class="tenant-name">"Blue Ridge Holdings"</div>
                            <div class="tenant-domain">"blueridge.atlas.app"</div>
                        </td>
                        <td><span class="tenant-type-tag tag-pm">"PM"</span></td>
                        <td><span class="plan-badge growth">"Growth"</span></td>
                        <td>
                            <div class="module-flags">
                                <div class="mflag on">"L"</div>
                                <div class="mflag on">"P"</div>
                                <div class="mflag on">"$"</div>
                                <div class="mflag">"A"</div>
                                <div class="mflag">"E"</div>
                                <div class="mflag">"C"</div>
                                <div class="mflag">"R"</div>
                                <div class="mflag">"M"</div>
                            </div>
                        </td>
                        <td class="center">
                            <div class="score-badge" style="border-color:var(--red);background:var(--red-dim)">
                                <span class="score-dot" style="background:var(--tier-avoid)"></span>
                                <span class="mono" style="color:var(--red)">"4.1"</span>
                                <span class="score-tier" style="color:var(--red)">"Critical"</span>
                            </div>
                        </td>
                        <td class="right mono">"$1,200"</td>
                        <td class="right mono">"56"</td>
                        <td class="center"><span class="status-dot" style="background:var(--red)"></span></td>
                        <td class="right muted">"Aug 2024"</td>
                    </tr>
                    <tr>
                        <td>
                            <div class="tenant-name">"Meridian Brokerage"</div>
                            <div class="tenant-domain">"meridian.atlas.app"</div>
                        </td>
                        <td><span class="tenant-type-tag tag-biz">"Biz"</span></td>
                        <td><span class="plan-badge starter">"Starter"</span></td>
                        <td>
                            <div class="module-flags">
                                <div class="mflag on">"L"</div>
                                <div class="mflag on">"P"</div>
                                <div class="mflag">"$"</div>
                                <div class="mflag on">"A"</div>
                                <div class="mflag">"E"</div>
                                <div class="mflag">"C"</div>
                                <div class="mflag">"R"</div>
                                <div class="mflag">"M"</div>
                            </div>
                        </td>
                        <td class="center">
                            <div class="score-badge">
                                <span class="score-dot" style="background:var(--tier-above)"></span>
                                <span class="mono">"7.8"</span>
                                <span class="score-tier">"Above"</span>
                            </div>
                        </td>
                        <td class="right mono">"$480"</td>
                        <td class="right mono">"89"</td>
                        <td class="center"><span class="status-dot" style="background:var(--amber)"></span></td>
                        <td class="right muted">"Mar 2025"</td>
                    </tr>
                    <tr style="opacity:0.6">
                        <td>
                            <div class="tenant-name">"Solaris Logistics"</div>
                            <div class="tenant-domain">"solaris.atlas.app"</div>
                        </td>
                        <td><span class="tenant-type-tag tag-biz">"Biz"</span></td>
                        <td><span class="plan-badge starter">"Starter"</span></td>
                        <td>
                            <div class="module-flags">
                                <div class="mflag on">"L"</div>
                                <div class="mflag">"P"</div>
                                <div class="mflag">"$"</div>
                                <div class="mflag">"A"</div>
                                <div class="mflag">"E"</div>
                                <div class="mflag">"C"</div>
                                <div class="mflag">"R"</div>
                                <div class="mflag">"M"</div>
                            </div>
                        </td>
                        <td class="center">
                            <div class="score-badge">
                                <span class="score-dot" style="background:var(--text-muted)"></span>
                                <span class="mono" style="color:var(--text-muted)">"—"</span>
                                <span class="score-tier">"No data"</span>
                            </div>
                        </td>
                        <td class="right mono secondary">"$0"</td>
                        <td class="right mono secondary">"0"</td>
                        <td class="center"><span class="status-dot" style="background:var(--text-muted)"></span></td>
                        <td class="right muted">"Jun 2025"</td>
                    </tr>
                </tbody>
            </table>
        </div>

        // ── Bottom Split: Products + Generics Coverage ──
        <div class="two-col">
            // Platform Products
            <div class="section">
                <div class="section-header">
                    <div class="section-title">
                        <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                            <circle cx="7" cy="7" r="5"/>
                            <path d="M7 4v3l2 2"/>
                        </svg>
                        "Platform Products"
                        <span class="section-count">"7 active"</span>
                    </div>
                    <a href="/apps" class="section-action" style="text-decoration:none">"Manage →"</a>
                </div>

                <div class="product-row">
                    <span class="product-mode-dot" style="background:var(--green)"></span>
                    <div class="product-info">
                        <div class="product-name">"Atlas PM — Residential"</div>
                        <div class="product-domain">"pm.buildwithruud.com"</div>
                    </div>
                    <div class="product-meta">
                        <div class="score-badge">
                            <span class="score-dot" style="background:var(--tier-outstanding)"></span>
                            <span class="mono">"9.1"</span>
                        </div>
                        <span style="font-size:10px;color:var(--text-muted)">"342 leads"</span>
                    </div>
                </div>
                <div class="product-row">
                    <span class="product-mode-dot" style="background:var(--green)"></span>
                    <div class="product-info">
                        <div class="product-name">"Atlas Commercial"</div>
                        <div class="product-domain">"commercial.buildwithruud.com"</div>
                    </div>
                    <div class="product-meta">
                        <div class="score-badge">
                            <span class="score-dot" style="background:var(--tier-above)"></span>
                            <span class="mono">"7.8"</span>
                        </div>
                        <span style="font-size:10px;color:var(--text-muted)">"124 leads"</span>
                    </div>
                </div>
                <div class="product-row">
                    <span class="product-mode-dot" style="background:var(--cobalt)"></span>
                    <div class="product-info">
                        <div class="product-name">"Atlas STR — Miami"</div>
                        <div class="product-domain">"str-miami.buildwithruud.com"</div>
                    </div>
                    <div class="product-meta">
                        <span style="font-size:10px;color:var(--cobalt);border:1px solid var(--cobalt);border-radius:3px;padding:1px 5px;font-weight:600;">"PRE-LAUNCH"</span>
                        <span style="font-size:10px;color:var(--text-muted)">"88 waitlist"</span>
                    </div>
                </div>
                <div class="product-row">
                    <span class="product-mode-dot" style="background:var(--amber)"></span>
                    <div class="product-info">
                        <div class="product-name">"Atlas Wholesale — USVI"</div>
                        <div class="product-domain">"wholesale-usvi.buildwithruud.com"</div>
                    </div>
                    <div class="product-meta">
                        <span style="font-size:10px;color:var(--amber);border:1px solid var(--amber);border-radius:3px;padding:1px 5px;font-weight:600;">"BETA"</span>
                        <span style="font-size:10px;color:var(--text-muted)">"41 leads"</span>
                    </div>
                </div>
                <div class="product-row">
                    <span class="product-mode-dot" style="background:var(--text-muted)"></span>
                    <div class="product-info">
                        <div class="product-name">"Atlas STR — Brazil"</div>
                        <div class="product-domain">"str-brasil.buildwithruud.com"</div>
                    </div>
                    <div class="product-meta">
                        <span style="font-size:10px;color:var(--violet);border:1px solid var(--violet);border-radius:3px;padding:1px 5px;font-weight:600;">"AI LOCALIZE"</span>
                        <span style="font-size:10px;color:var(--text-muted)">"—"</span>
                    </div>
                </div>
            </div>

            // Generics Coverage Panel
            <div class="section">
                <div class="section-header">
                    <div class="section-title">
                        <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                            <rect x="1" y="1" width="12" height="12" rx="1"/>
                            <path d="M1 5h12M5 5v8"/>
                        </svg>
                        "Platform Generics — G01–G31"
                        <span class="section-count">"Coverage"</span>
                    </div>
                    <a href="/billing/scorecards" class="section-action" style="text-decoration:none">"View Spec →"</a>
                </div>
                <div style="padding:12px 16px;display:flex;flex-direction:column;gap:8px;">
                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--cobalt);border:1px solid var(--cobalt);border-radius:3px;padding:1px 5px;background:var(--cobalt-dim)">"G27"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Scorecards"</span>
                        <span style="font-size:10px;color:var(--green);font-weight:600">"● Deployed + UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--cobalt);border:1px solid var(--cobalt);border-radius:3px;padding:1px 5px;background:var(--cobalt-dim)">"G31"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Leads (atlas_lead)"</span>
                        <span style="font-size:10px;color:var(--green);font-weight:600">"● Full UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G08"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"AI Tasks"</span>
                        <span style="font-size:10px;color:var(--amber);font-weight:600">"⚠ No UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G19"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Campaigns"</span>
                        <span style="font-size:10px;color:var(--amber);font-weight:600">"⚠ No UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G21"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Events"</span>
                        <span style="font-size:10px;color:var(--amber);font-weight:600">"⚠ No UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G23"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Reservations"</span>
                        <span style="font-size:10px;color:var(--amber);font-weight:600">"⚠ No UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G24"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Quotes"</span>
                        <span style="font-size:10px;color:var(--amber);font-weight:600">"⚠ No UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G06"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Verification Queue"</span>
                        <span style="font-size:10px;color:var(--red);font-weight:600">"✗ 3 pending"</span>
                    </div>
                </div>
            </div>
        </div>
        </div>
    }
}
