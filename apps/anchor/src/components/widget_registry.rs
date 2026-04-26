/// Widget Registry — Data-driven, tenant-configurable widget system
///
/// Architecture: The platform provides renderer shells (WidgetRenderer); tenants
/// configure their own widget instances with any DataSource via JSONB in
/// app_instances.settings.widgets. No platform code change is needed to add
/// a new widget instance for a tenant.
///
/// Security: RestEndpoint URLs are validated against an SSRF allowlist on save.
///           PlatformTable queries are always scoped to tenant_id automatically.
///           WidgetInstance is fully typed via serde — no raw SQL interpolation.
///
/// Scalability: Nav widgets are fetched in one server round-trip via get_site_settings().
///              External API results are cached server-side with per-widget TTL.
use leptos::*;
use serde::{Deserialize, Serialize};

// ─── Renderer Types ───────────────────────────────────────────────────────────

/// Platform-provided renderer shells. Tenants wire their own DataSource into these.
/// Adding a new variant here enables a new renderer type for ALL tenants.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "renderer", rename_all = "snake_case")]
pub enum WidgetRenderer {
    /// Bitcoin block height clock — specialized renderer for bitcoin_blocks data
    BlockClock,
    /// Single key-value stat card (label + value + optional unit)
    StatCard {
        label: String,
        #[serde(default)]
        unit: Option<String>,
    },
    /// Horizontally scrolling ticker tape of values
    TickerTape,
    /// Line/bar/area chart
    LiveChart {
        #[serde(default = "default_chart_type")]
        chart_type: String,
    },
    /// Arbitrary HTML for advanced tenants (sanitized server-side)
    CustomHtml,
}

fn default_chart_type() -> String {
    "line".to_string()
}

// ─── Data Source Types ────────────────────────────────────────────────────────

/// Tenant-configurable data source. The source_type tag determines which variant
/// is parsed from the JSONB config in app_instances.settings.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "source_type", rename_all = "snake_case")]
pub enum DataSource {
    /// Internal platform table — always scoped to tenant_id automatically.
    /// `table` must be in the platform allowlist (validated on save).
    PlatformTable {
        table: String,
        column: String,
        #[serde(default)]
        filter: Option<String>,
    },
    /// External REST endpoint — tenant provides their own API key via headers.
    /// URL is validated against SSRF allowlist on save (no RFC-1918 or loopback).
    RestEndpoint {
        url: String,
        #[serde(default)]
        json_path: Option<String>,
        #[serde(default)]
        headers: Option<serde_json::Value>,
    },
    /// Static value — no data fetching, value is embedded in config.
    Static {
        value: serde_json::Value,
    },
    /// WebSocket stream — tenant provides the URL.
    WebSocket {
        url: String,
    },
}

// ─── Placement ────────────────────────────────────────────────────────────────

/// Where the widget appears in the UI.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WidgetPlacement {
    Nav,
    Landing,
    Dashboard,
}

// ─── Widget Instance ──────────────────────────────────────────────────────────

/// A complete widget configuration — 100% serializable to JSONB.
/// Owned by the tenant in app_instances.settings.widgets[].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WidgetInstance {
    /// Unique slug for this widget (used as reactive key)
    pub id: String,
    /// Human-readable name shown in the admin widget builder
    pub name: String,
    pub renderer: WidgetRenderer,
    pub data_source: DataSource,
    pub placement: Vec<WidgetPlacement>,
    /// Poll interval in seconds. None = render once on SSR only.
    #[serde(default)]
    pub refresh_seconds: Option<u32>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl WidgetInstance {
    pub fn is_nav_widget(&self) -> bool {
        self.enabled && self.placement.contains(&WidgetPlacement::Nav)
    }

    pub fn is_landing_widget(&self) -> bool {
        self.enabled && self.placement.contains(&WidgetPlacement::Landing)
    }
}

// ─── Validation (server-side only) ───────────────────────────────────────────

/// Validate a WidgetInstance before saving to the database.
/// Called from the admin API handler — NOT during rendering.
#[cfg(feature = "ssr")]
pub fn validate_widget_instance(widget: &WidgetInstance) -> Result<(), String> {
    if widget.id.is_empty() || widget.name.is_empty() {
        return Err("Widget id and name are required".into());
    }
    match &widget.data_source {
        DataSource::RestEndpoint { url, .. } => validate_widget_url(url)?,
        DataSource::PlatformTable { table, .. } => validate_platform_table(table)?,
        _ => {}
    }
    Ok(())
}

/// Validate a list of widget instances for a tenant.
/// Enforces the per-tenant widget cap.
#[cfg(feature = "ssr")]
pub fn validate_widget_list(widgets: &[WidgetInstance]) -> Result<(), String> {
    if widgets.len() > MAX_WIDGETS_PER_TENANT {
        return Err(format!(
            "Exceeded maximum of {} widgets per tenant",
            MAX_WIDGETS_PER_TENANT
        ));
    }
    for widget in widgets {
        validate_widget_instance(widget)?;
    }
    Ok(())
}

pub const MAX_WIDGETS_PER_TENANT: usize = 10;

/// SSRF protection: reject RFC-1918, loopback, and cloud metadata addresses.
/// Also enforces HTTPS in production.
#[cfg(feature = "ssr")]
pub fn validate_widget_url(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;

    // Enforce HTTPS (allow http only in dev/test via LEPTOS_ENV)
    let env = std::env::var("LEPTOS_ENV").unwrap_or_default();
    if parsed.scheme() != "https" && env != "development" && env != "DEV" {
        return Err("RestEndpoint URL must use HTTPS in non-development environments".into());
    }

    let host = parsed.host_str().unwrap_or("").to_lowercase();

    // Block loopback
    if host == "localhost" || host == "127.0.0.1" || host == "::1" {
        return Err(format!("RestEndpoint URL host '{}' is not allowed (loopback)", host));
    }

    // Block AWS/GCP/cloud metadata endpoints
    if host == "169.254.169.254" || host == "metadata.google.internal" {
        return Err(format!("RestEndpoint URL host '{}' is not allowed (cloud metadata)", host));
    }

    // Block RFC-1918 private ranges (basic string prefix check)
    if host.starts_with("10.")
        || host.starts_with("192.168.")
        || (host.starts_with("172.") && {
            // Check 172.16.0.0/12 range
            host.split('.')
                .nth(1)
                .and_then(|s| s.parse::<u8>().ok())
                .map(|n| (16..=31).contains(&n))
                .unwrap_or(false)
        })
    {
        return Err(format!("RestEndpoint URL host '{}' is not allowed (private IP range)", host));
    }

    Ok(())
}

/// Platform table allowlist — only these tables can be queried by PlatformTable widgets.
/// Sensitive tables (user, session, api_tokens, audit_logs) are explicitly excluded.
#[cfg(feature = "ssr")]
pub fn validate_platform_table(table: &str) -> Result<(), String> {
    const ALLOWED_TABLES: &[&str] = &[
        "bitcoin_blocks",
        "tenant_entries",
        "app_pages",
        "app_menus",
        "footer_items",
        "blog_posts",
    ];
    if ALLOWED_TABLES.contains(&table) {
        Ok(())
    } else {
        Err(format!(
            "PlatformTable '{}' is not in the allowlist. Allowed tables: {:?}",
            table, ALLOWED_TABLES
        ))
    }
}

// ─── Leptos Components ────────────────────────────────────────────────────────

/// Dispatch component — renders the correct shell based on WidgetRenderer variant.
/// Called once per enabled nav/landing widget from Nav and Landing respectively.
#[component]
pub fn WidgetShell(widget: WidgetInstance) -> impl IntoView {
    match widget.renderer {
        WidgetRenderer::BlockClock => {
            view! { <BitcoinNavWidget /> }.into_view()
        }
        WidgetRenderer::StatCard { label, unit } => {
            view! { <StatCardWidget label=label unit=unit source=widget.data_source /> }.into_view()
        }
        // Future renderers: TickerTape, LiveChart, CustomHtml
        _ => view! {}.into_view(),
    }
}

/// Bitcoin Block Clock — renders the current block height linked to mempool.space.
/// Data is fetched via the existing get_block_height() server function, which
/// already scopes its query to the active tenant_id.
#[component]
pub fn BitcoinNavWidget() -> impl IntoView {
    use std::time::Duration;

    let (tick, set_tick) = create_signal(0u32);

    create_effect(move |_| {
        let handle = set_interval_with_handle(
            move || set_tick.update(|t| *t += 1),
            Duration::from_secs(60),
        )
        .ok();
        on_cleanup(move || {
            if let Some(h) = handle { h.clear(); }
        });
    });

    let height_resource = create_resource(
        move || tick.get(),
        |_| crate::components::nav::get_block_height(),
    );

    view! {
        <Suspense fallback=move || view! {
            <a href="#" class="bg-surface border border-outline-variant/30 px-6 py-2 jetbrains text-[0.65rem] font-bold tracking-wider opacity-50 block whitespace-nowrap">
                <div class="flex flex-col items-center leading-none justify-center">
                    <span class="text-[0.55rem] text-on-surface-variant uppercase font-medium">"CURRENT BLOCK"</span>
                    <div class="mt-1 flex items-center text-on-surface">
                        <span class="material-symbols-outlined text-[0.8rem] inline mr-1 text-[#f7931a] align-text-bottom">"currency_bitcoin"</span>
                        <span>"..."</span>
                    </div>
                </div>
            </a>
        }>
            {move || {
                let h = height_resource.get().unwrap_or(Ok(None)).unwrap_or(None);
                if let Some(height) = h {
                    view! {
                        <a href=format!("https://mempool.space/block/{}", height)
                           target="_blank" rel="noopener noreferrer"
                           class="bg-surface border border-outline-variant/50 hover:border-[#f7931a]/50 shadow-sm px-6 py-2 jetbrains text-[0.65rem] font-bold tracking-wider hover:bg-surface-container-low transition-all block whitespace-nowrap">
                            <div class="flex flex-col items-center leading-none justify-center">
                                <span class="text-[0.55rem] text-on-surface-variant uppercase font-medium tracking-[0.1em]">"CURRENT BLOCK"</span>
                                <div class="mt-1 flex items-center text-on-surface">
                                    <span class="material-symbols-outlined text-[0.8rem] inline mr-1 text-[#f7931a] align-text-bottom">"currency_bitcoin"</span>
                                    <span>"#" {height}</span>
                                </div>
                            </div>
                        </a>
                    }.into_view()
                } else {
                    view! {
                        <a href="#" class="bg-surface border border-[#f7931a]/30 shadow-sm px-6 py-2 jetbrains text-[0.65rem] font-bold tracking-wider animate-pulse block whitespace-nowrap">
                            <div class="flex flex-col items-center leading-none justify-center">
                                <span class="text-[0.55rem] text-on-surface-variant uppercase font-medium tracking-[0.1em]">"STATE"</span>
                                <div class="mt-1 flex items-center text-on-surface">
                                    <span class="material-symbols-outlined text-[0.8rem] inline mr-1 text-[#f7931a] align-text-bottom animate-spin">"sync"</span>
                                    <span class="text-[#f7931a]">"SYNCING..."</span>
                                </div>
                            </div>
                        </a>
                    }.into_view()
                }
            }}
        </Suspense>
    }
}

/// Generic StatCard widget renderer.
/// Displays a label + value from a configured DataSource.
/// Currently supports Static data source; RestEndpoint/PlatformTable fetching
/// is implemented server-side in the background job / cache layer (Phase 2).
#[component]
pub fn StatCardWidget(
    label: String,
    #[prop(default = None)] unit: Option<String>,
    source: DataSource,
) -> impl IntoView {
    // For static sources, render immediately
    let value = match &source {
        DataSource::Static { value } => value.to_string().trim_matches('"').to_string(),
        _ => "—".to_string(), // RestEndpoint/PlatformTable: Phase 2 client polling
    };
    view! {
        <div class="flex flex-col items-center px-4 py-2 bg-surface border border-outline-variant/30 jetbrains text-[0.65rem]">
            <span class="text-[0.55rem] text-on-surface-variant uppercase tracking-[0.1em]">{label}</span>
            <span class="text-on-surface font-bold mt-0.5">
                {value}
                {unit.map(|u| view! { <span class="text-outline ml-0.5">{u}</span> })}
            </span>
        </div>
    }
}

// ─── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_widget(id: &str, enabled: bool, placement: Vec<WidgetPlacement>) -> WidgetInstance {
        WidgetInstance {
            id: id.to_string(),
            name: format!("Widget {id}"),
            renderer: WidgetRenderer::BlockClock,
            data_source: DataSource::Static { value: json!(0) },
            placement,
            refresh_seconds: None,
            enabled,
        }
    }

    // ── Deserialization ────────────────────────────────────────────────────────

    #[test]
    fn test_widget_instance_deserializes_block_clock() {
        let raw = json!({
            "id": "bitcoin_block_clock", "name": "BTC", "enabled": true,
            "renderer": {"renderer": "block_clock"},
            "data_source": {"source_type": "platform_table", "table": "bitcoin_blocks", "column": "height"},
            "placement": ["nav"], "refresh_seconds": 600
        });
        let w: WidgetInstance = serde_json::from_value(raw).unwrap();
        assert_eq!(w.renderer, WidgetRenderer::BlockClock);
        assert!(w.placement.contains(&WidgetPlacement::Nav));
        assert_eq!(w.refresh_seconds, Some(600));
    }

    #[test]
    fn test_widget_instance_deserializes_stat_card() {
        let raw = json!({
            "id": "my_stat", "name": "My Stat", "enabled": true,
            "renderer": {"renderer": "stat_card", "label": "AAPL", "unit": "USD"},
            "data_source": {"source_type": "static", "value": 189.43},
            "placement": ["landing"]
        });
        let w: WidgetInstance = serde_json::from_value(raw).unwrap();
        assert!(matches!(w.renderer, WidgetRenderer::StatCard { .. }));
        assert!(w.is_landing_widget());
        assert!(!w.is_nav_widget());
    }

    #[test]
    fn test_disabled_widget_is_filtered() {
        let widgets = vec![
            make_widget("w1", true, vec![WidgetPlacement::Nav]),
            make_widget("w2", false, vec![WidgetPlacement::Nav]),
        ];
        let nav: Vec<_> = widgets.into_iter().filter(|w| w.is_nav_widget()).collect();
        assert_eq!(nav.len(), 1);
        assert_eq!(nav[0].id, "w1");
    }

    // ── Security: RestEndpoint URL validation ──────────────────────────────────

    #[cfg(feature = "ssr")]
    #[test]
    fn test_rest_endpoint_rejects_private_ip_10x() {
        assert!(validate_widget_url("http://10.0.0.1/internal").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_rest_endpoint_rejects_private_ip_192168() {
        assert!(validate_widget_url("http://192.168.1.1/secret").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_rest_endpoint_rejects_localhost() {
        assert!(validate_widget_url("http://localhost/admin").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_rest_endpoint_rejects_metadata_endpoint() {
        assert!(validate_widget_url("http://169.254.169.254/latest/meta-data").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_rest_endpoint_rejects_http_in_prod() {
        // LEPTOS_ENV is unset in test — treated as production
        // http:// should fail
        assert!(validate_widget_url("http://api.example.com/data").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_rest_endpoint_accepts_valid_https() {
        assert!(validate_widget_url("https://api.coinbase.com/v2/prices").is_ok());
    }

    // ── Security: PlatformTable allowlist ──────────────────────────────────────

    #[cfg(feature = "ssr")]
    #[test]
    fn test_platform_table_allowlist_bitcoin_blocks() {
        assert!(validate_platform_table("bitcoin_blocks").is_ok());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_platform_table_allowlist_tenant_entries() {
        assert!(validate_platform_table("tenant_entries").is_ok());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_platform_table_rejects_user_table() {
        assert!(validate_platform_table("user").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_platform_table_rejects_session_table() {
        assert!(validate_platform_table("session").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_platform_table_rejects_api_tokens() {
        assert!(validate_platform_table("api_tokens").is_err());
    }

    // ── Placement filtering ────────────────────────────────────────────────────

    #[test]
    fn test_nav_placement_filter() {
        let widgets = vec![
            make_widget("w1", true, vec![WidgetPlacement::Landing]),
            make_widget("w2", true, vec![WidgetPlacement::Nav, WidgetPlacement::Landing]),
        ];
        let nav: Vec<_> = widgets.iter().filter(|w| w.is_nav_widget()).collect();
        assert_eq!(nav.len(), 1);
        assert_eq!(nav[0].id, "w2");
    }

    // ── Scalability: widget cap ────────────────────────────────────────────────

    #[cfg(feature = "ssr")]
    #[test]
    fn test_widget_count_cap_enforced() {
        let too_many: Vec<WidgetInstance> = (0..=MAX_WIDGETS_PER_TENANT)
            .map(|i| make_widget(&format!("w{i}"), true, vec![]))
            .collect();
        assert!(validate_widget_list(&too_many).is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_widget_count_at_cap_is_ok() {
        let at_cap: Vec<WidgetInstance> = (0..MAX_WIDGETS_PER_TENANT)
            .map(|i| make_widget(&format!("w{i}"), true, vec![]))
            .collect();
        assert!(validate_widget_list(&at_cap).is_ok());
    }
}
