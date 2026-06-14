use leptos::prelude::*;

#[component]
pub fn IntelSidebar() -> impl IntoView {
    view! {
        <aside class="intel-sidebar">
            // Anomaly Alerts
            <div class="intel-panel">
                <div class="intel-panel-header">
                    <span class="intel-panel-title">
                        <span class="live-dot"></span>
                        "Anomaly Alerts"
                    </span>
                    <span style="font-size:10px;color:var(--text-muted)">"4 active"</span>
                </div>
                <div class="alert-card red">
                    <span class="alert-type">"Score Critical"</span>
                    <span class="alert-title">"Blue Ridge Holdings — 4.1"</span>
                    <span class="alert-desc">"Tenant score fell below 5.0 threshold. 3 open maintenance cases unresolved >7 days."</span>
                    <span class="alert-time">"18 min ago"</span>
                </div>
                <div class="alert-card amber">
                    <span class="alert-type">"Outbox Lag"</span>
                    <span class="alert-title">"OutboxWorker — 12s behind"</span>
                    <span class="alert-desc">"Scorecard aggregate jobs queued. recompute_scorecard_aggregates queue depth: 47."</span>
                    <span class="alert-time">"4 min ago"</span>
                </div>
                <div class="alert-card red">
                    <span class="alert-type">"Verification Backlog"</span>
                    <span class="alert-title">"G-06 Queue — 3 pending"</span>
                    <span class="alert-desc">"3 verification requests older than 24h without review. Tenant: Meridian Brokerage."</span>
                    <span class="alert-time">"1h 12m ago"</span>
                </div>
                <div class="alert-card cobalt">
                    <span class="alert-type">"New Lead Burst"</span>
                    <span class="alert-title">"Atlas STR Miami — 88 waitlist"</span>
                    <span class="alert-desc">"Pre-launch product received 88 leads in 6h via campaign. Conversion rate: 3.2%."</span>
                    <span class="alert-time">"2h ago"</span>
                </div>
            </div>

            // Top Scoring Leads (G-27)
            <div class="intel-panel">
                <div class="intel-panel-header">
                    <span class="intel-panel-title">"Top Leads · G-27 Score"</span>
                    <a href="/crm" class="section-action" style="font-size:10px">"All →"</a>
                </div>
                <div class="lead-score-row">
                    <div class="lead-initials" style="background:var(--cobalt-dim);color:var(--cobalt)">"JS"</div>
                    <div class="lead-meta">
                        <div class="lead-name">"João Silva"</div>
                        <div class="lead-co">"Ruud Commercial · fmcsa import"</div>
                    </div>
                    <div class="score-badge">
                        <span class="score-dot" style="background:var(--tier-outstanding)"></span>
                        <span class="mono">"9.3"</span>
                    </div>
                </div>
                <div class="lead-score-row">
                    <div class="lead-initials" style="background:var(--green-dim);color:var(--green)">"AC"</div>
                    <div class="lead-meta">
                        <div class="lead-name">"Ana Carvalho"</div>
                        <div class="lead-co">"Nexus Property · business_leads"</div>
                    </div>
                    <div class="score-badge">
                        <span class="score-dot" style="background:var(--tier-above)"></span>
                        <span class="mono">"8.1"</span>
                    </div>
                </div>
                <div class="lead-score-row">
                    <div class="lead-initials" style="background:var(--amber-dim);color:var(--amber)">"MT"</div>
                    <div class="lead-meta">
                        <div class="lead-name">"Marcus Thompson"</div>
                        <div class="lead-co">"Vizcaya STR · direct form"</div>
                    </div>
                    <div class="score-badge">
                        <span class="score-dot" style="background:var(--tier-above)"></span>
                        <span class="mono">"7.6"</span>
                    </div>
                </div>
                <div class="lead-score-row">
                    <div class="lead-initials" style="background:rgba(124,58,237,0.12);color:var(--violet)">"RL"</div>
                    <div class="lead-meta">
                        <div class="lead-name">"Rita Lacerda"</div>
                        <div class="lead-co">"Nexus Property · dot_registry"</div>
                    </div>
                    <div class="score-badge">
                        <span class="score-dot" style="background:var(--tier-at)"></span>
                        <span class="mono">"6.8"</span>
                    </div>
                </div>
            </div>

            // AI Task Queue (G-08)
            <div class="intel-panel">
                <div class="intel-panel-header">
                    <span class="intel-panel-title">
                        <span class="live-dot"></span>
                        "AI Tasks · G-08"
                    </span>
                    <a href="/analytics" class="section-action" style="font-size:10px">"Monitor →"</a>
                </div>
                <div class="job-row">
                    <div class="job-status running">"↻"</div>
                    <div class="flex-col" style="flex:1;gap:1px">
                        <span class="job-name">"localize_product_page"</span>
                        <span class="job-tenant">"Atlas STR · Miami · es-419"</span>
                    </div>
                    <span class="job-duration">"44s"</span>
                </div>
                <div class="job-row">
                    <div class="job-status running">"↻"</div>
                    <div class="flex-col" style="flex:1;gap:1px">
                        <span class="job-name">"localize_product_page"</span>
                        <span class="job-tenant">"Atlas STR · Brazil · pt-BR"</span>
                    </div>
                    <span class="job-duration">"1m 2s"</span>
                </div>
                <div class="job-row">
                    <div class="job-status done">"✓"</div>
                    <div class="flex-col" style="flex:1;gap:1px">
                        <span class="job-name">"recompute_scorecard_aggregates"</span>
                        <span class="job-tenant">"Nexus Property Group"</span>
                    </div>
                    <span class="job-duration">"0.8s"</span>
                </div>
                <div class="job-row">
                    <div class="job-status done">"✓"</div>
                    <div class="flex-col" style="flex:1;gap:1px">
                        <span class="job-name">"calibrate_scorecard_contributors"</span>
                        <span class="job-tenant">"Ruud Commercial"</span>
                    </div>
                    <span class="job-duration">"2.1s"</span>
                </div>
                <div class="job-row">
                    <div class="job-status failed">"✗"</div>
                    <div class="flex-col" style="flex:1;gap:1px">
                        <span class="job-name">"ota_sync · Airbnb Connect"</span>
                        <span class="job-tenant">"Vizcaya STR Partners"</span>
                    </div>
                    <span class="job-duration" style="color:var(--red)">"Err"</span>
                </div>
                <div class="job-row">
                    <div class="job-status queued">"·"</div>
                    <div class="flex-col" style="flex:1;gap:1px">
                        <span class="job-name">"rebuild_scorecard_portfolio"</span>
                        <span class="job-tenant">"Blue Ridge Holdings"</span>
                    </div>
                    <span class="job-duration">"Queued"</span>
                </div>
            </div>

            // Background Worker Health
            <div class="intel-panel" style="border-bottom:none">
                <div class="intel-panel-header">
                    <span class="intel-panel-title">"Worker Health"</span>
                </div>
                <div style="padding:10px 14px;display:flex;flex-direction:column;gap:8px;">
                    <div style="display:flex;align-items:center;justify-content:space-between;font-size:11px;">
                        <span style="color:var(--text-secondary)">"OutboxWorker"</span>
                        <span style="color:var(--amber);font-weight:600">"Lagging 12s"</span>
                    </div>
                    <div style="display:flex;align-items:center;justify-content:space-between;font-size:11px;">
                        <span style="color:var(--text-secondary)">"TelemetryService"</span>
                        <span style="color:var(--green);font-weight:600">"● OK"</span>
                    </div>
                    <div style="display:flex;align-items:center;justify-content:space-between;font-size:11px;">
                        <span style="color:var(--text-secondary)">"WebhookSweeper"</span>
                        <span style="color:var(--green);font-weight:600">"● OK"</span>
                    </div>
                    <div style="display:flex;align-items:center;justify-content:space-between;font-size:11px;">
                        <span style="color:var(--text-secondary)">"DataSyncService"</span>
                        <span style="color:var(--green);font-weight:600">"● OK"</span>
                    </div>
                    <div style="display:flex;align-items:center;justify-content:space-between;font-size:11px;">
                        <span style="color:var(--text-secondary)">"Scorecard Aggregates"</span>
                        <span style="color:var(--text-muted)">"Next: 4m 33s"</span>
                    </div>
                </div>
            </div>
        </aside>
    }
}
