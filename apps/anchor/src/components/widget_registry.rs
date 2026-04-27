/// Widget Registry — Data-driven, tenant-configurable widget system
///
/// Architecture: The platform provides renderer shells (WidgetRenderer); tenants
/// configure their own widget instances with any DataSource via JSONB in
/// app_instances.settings.widgets. No platform code change is needed to add
/// a new widget instance for a tenant.
///
/// Security — Two-layer SSRF defense for RestEndpoint data sources:
///
///   Layer 1 (static, sync): validate_widget_url()
///     Called on widget save. Uses url::Host enum dispatch + IpAddr stdlib
///     methods (is_loopback, is_private, is_link_local, to_ipv4_mapped) to
///     block direct IP address attacks. Domain names intentionally pass Layer 1
///     because DNS cannot be resolved at parse time.
///
///   Layer 2 (async, post-DNS): enforce_ssrf_safe_fetch()
///     Called before every outbound HTTP request to a tenant URL. Resolves all
///     DNS A/AAAA records via tokio::net::lookup_host and validates every
///     returned IpAddr. This is the only effective defense against DNS rebinding
///     and custom domain redirects to private IPs. MUST be called before any
///     reqwest/hyper call to a tenant-supplied URL.
///
///   PlatformTable queries are always scoped to tenant_id automatically.
///   WidgetInstance is fully typed via serde — no raw SQL interpolation.
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

// ─── IP Classification ────────────────────────────────────────────────────────
// These functions operate on parsed IpAddr values — never on strings.
// This eliminates entire classes of bypass (bracket formatting, prefix tricks,
// IPv4-mapped IPv6, alternate loopback IPs in the 127.0.0.0/8 range, etc.)

#[cfg(feature = "ssr")]
fn is_dangerous_ipv4(addr: std::net::Ipv4Addr) -> bool {
    addr.is_loopback()      // Full 127.0.0.0/8 — not just 127.0.0.1
    || addr.is_private()    // 10.x, 172.16-31.x, 192.168.x
    || addr.is_link_local() // 169.254.0.0/16 — AWS/GCP cloud metadata
    || addr.is_broadcast()  // 255.255.255.255
    || addr.is_unspecified()// 0.0.0.0
    // Carrier-grade NAT (100.64.0.0/10) — often used in cloud environments
    || (addr.octets()[0] == 100 && (addr.octets()[1] & 0xC0) == 64)
}

#[cfg(feature = "ssr")]
fn is_dangerous_ipv6(addr: std::net::Ipv6Addr) -> bool {
    // Loopback (::1) and unspecified (::)
    if addr.is_loopback() || addr.is_unspecified() {
        return true;
    }
    // IPv4-mapped IPv6 (::ffff:x.x.x.x) and IPv4-compatible (::x.x.x.x)
    // Both to_ipv4_mapped() and to_ipv4() return Some for these forms.
    // This is the critical check for ::ffff:169.254.169.254 bypass.
    if let Some(v4) = addr.to_ipv4_mapped().or_else(|| addr.to_ipv4()) {
        return is_dangerous_ipv4(v4);
    }
    // IPv6 link-local (fe80::/10) — analogous to 169.254.x.x
    if (addr.segments()[0] & 0xffc0) == 0xfe80 {
        return true;
    }
    // IPv6 unique-local (fc00::/7) — analogous to RFC-1918
    if (addr.segments()[0] & 0xfe00) == 0xfc00 {
        return true;
    }
    false
}

#[cfg(feature = "ssr")]
fn is_dangerous_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => is_dangerous_ipv4(v4),
        std::net::IpAddr::V6(v6) => is_dangerous_ipv6(v6),
    }
}

// ─── Layer 1: Static pre-flight URL validation ────────────────────────────────
//
// Operates on the parsed URL *before* any network activity.
// Guards against direct IP address attacks but CANNOT guard against
// DNS rebinding (where a domain resolves to a private IP at request time).
// Domain names are explicitly allowed here — DNS validation happens in Layer 2.
//
// Called synchronously on widget save from validate_widget_instance().

#[cfg(feature = "ssr")]
pub fn validate_widget_url(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;

    // Require a host
    let host = parsed.host().ok_or_else(|| "URL must have a host".to_string())?;

    // Enforce HTTPS (allow http only in dev/test via LEPTOS_ENV)
    let env = std::env::var("LEPTOS_ENV").unwrap_or_default();
    if parsed.scheme() != "https" && env != "development" && env != "DEV" {
        return Err("RestEndpoint URL must use HTTPS in non-development environments".into());
    }

    // Static IP validation — only reachable for literal IP addresses in the URL.
    // Bypasses involving DNS (e.g. attacker.com → 10.0.0.1) are caught in Layer 2.
    match host {
        url::Host::Ipv4(addr) => {
            if is_dangerous_ipv4(addr) {
                return Err(format!("IP address {addr} is not allowed (loopback/private/link-local)"));
            }
        }
        url::Host::Ipv6(addr) => {
            // url::Host::Ipv6 gives us the Ipv6Addr directly — no bracket/formatting ambiguity.
            if is_dangerous_ipv6(addr) {
                return Err(format!("IPv6 address {addr} is not allowed (loopback/private/link-local/IPv4-mapped)"));
            }
        }
        url::Host::Domain(_domain) => {
            // Domain names are allowed at static validation time.
            // The actual SSRF defense for domain names is enforce_ssrf_safe_fetch()
            // which resolves DNS and validates each returned IpAddr before connecting.
            // DO NOT attempt string matching on domain names here — it cannot be made safe.
        }
    }

    Ok(())
}

// ─── Layer 2: Post-DNS resolution validation ─────────────────────────────────
//
// This is the MANDATORY second layer. Static URL validation alone is insufficient
// because a domain name can resolve to a private IP (DNS rebinding attack).
//
// MUST be called server-side before any outbound HTTP request to a tenant-
// configured RestEndpoint URL. Never call reqwest/hyper directly on a tenant URL
// without first passing through this function.
//
// Usage (in the background fetch job / Phase 2 polling):
//   enforce_ssrf_safe_fetch("https://api.example.com/price").await?;
//   // Only reaches here if all resolved IPs are safe
//   let resp = reqwest_client.get(url).send().await?;

#[cfg(feature = "ssr")]
pub async fn enforce_ssrf_safe_fetch(url: &str) -> Result<(), String> {
    use tokio::net::lookup_host;

    let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;

    // Re-run static validation as a sanity guard
    validate_widget_url(url)?;

    let host_str = parsed.host_str().ok_or("URL has no host")?;
    // Default to port 443 for HTTPS, 80 for HTTP
    let port = parsed.port_or_known_default().unwrap_or(443);

    // Resolve all DNS records for this host+port combination
    let addrs: Vec<_> = lookup_host(format!("{host_str}:{port}"))
        .await
        .map_err(|e| format!("DNS resolution failed for '{host_str}': {e}"))?
        .collect();

    if addrs.is_empty() {
        return Err(format!("DNS resolution returned no addresses for '{host_str}'"));
    }

    // Every single resolved IP must be safe.
    // Accepting any unsafe IP in a multi-address response is a bypass.
    for addr in &addrs {
        let ip = addr.ip();
        if is_dangerous_ip(ip) {
            return Err(format!(
                "Host '{host_str}' resolved to blocked IP {ip} (loopback/private/link-local). \
                 Request denied."
            ));
        }
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
            "PlatformTable '{table}' is not in the allowlist. Allowed tables: {ALLOWED_TABLES:?}"
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

    // ── Security: IP classification (direct IpAddr tests) ─────────────────────
    // These test the core classification logic independently of URL parsing.

    #[cfg(feature = "ssr")]
    #[test]
    fn test_ipv4_loopback_full_range() {
        use std::net::Ipv4Addr;
        // The full 127.0.0.0/8 range must be blocked, not just 127.0.0.1
        // Bypass vector: http://127.0.0.2 or http://127.123.0.1
        assert!(is_dangerous_ipv4(Ipv4Addr::new(127, 0, 0, 1)));
        assert!(is_dangerous_ipv4(Ipv4Addr::new(127, 0, 0, 2)));   // bypass attempt
        assert!(is_dangerous_ipv4(Ipv4Addr::new(127, 123, 45, 67))); // bypass attempt
        assert!(is_dangerous_ipv4(Ipv4Addr::new(127, 255, 255, 255)));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_ipv4_private_ranges() {
        use std::net::Ipv4Addr;
        assert!(is_dangerous_ipv4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(is_dangerous_ipv4(Ipv4Addr::new(10, 255, 255, 255)));
        assert!(is_dangerous_ipv4(Ipv4Addr::new(172, 16, 0, 1)));
        assert!(is_dangerous_ipv4(Ipv4Addr::new(172, 31, 255, 255)));
        assert!(is_dangerous_ipv4(Ipv4Addr::new(192, 168, 0, 1)));
        assert!(is_dangerous_ipv4(Ipv4Addr::new(192, 168, 255, 255)));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_ipv4_link_local_metadata() {
        use std::net::Ipv4Addr;
        // Cloud metadata endpoint (AWS IMDSv1/v2, GCP)
        assert!(is_dangerous_ipv4(Ipv4Addr::new(169, 254, 169, 254)));
        assert!(is_dangerous_ipv4(Ipv4Addr::new(169, 254, 0, 1)));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_ipv4_carrier_grade_nat() {
        use std::net::Ipv4Addr;
        // 100.64.0.0/10 — carrier-grade NAT, sometimes used in cloud
        assert!(is_dangerous_ipv4(Ipv4Addr::new(100, 64, 0, 1)));
        assert!(is_dangerous_ipv4(Ipv4Addr::new(100, 127, 255, 255)));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_ipv4_public_addresses_are_safe() {
        use std::net::Ipv4Addr;
        assert!(!is_dangerous_ipv4(Ipv4Addr::new(1, 1, 1, 1)));       // Cloudflare DNS
        assert!(!is_dangerous_ipv4(Ipv4Addr::new(8, 8, 8, 8)));       // Google DNS
        assert!(!is_dangerous_ipv4(Ipv4Addr::new(52, 12, 0, 1)));     // Example AWS public
    }

    // ── Bypass vector 1: Alternate loopback IPs (127.x.x.x range) ────────────
    // Old code checked host == "127.0.0.1" only. 127.0.0.2, 127.1.0.1, etc. bypassed it.

    #[cfg(feature = "ssr")]
    #[test]
    fn test_bypass_alternate_loopback_127_0_0_2() {
        // http://127.0.0.2 was NOT caught by the old host == "127.0.0.1" check
        assert!(validate_widget_url("https://127.0.0.2/admin").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_bypass_alternate_loopback_127_1_0_1() {
        assert!(validate_widget_url("https://127.1.0.1/internal").is_err());
    }

    // ── Bypass vector 2: IPv6 bracket formatting ──────────────────────────────
    // Old code checked host == "::1" but url::Host::Ipv6 wraps in brackets in
    // some representations. Using url::Host enum gives us Ipv6Addr directly.

    #[cfg(feature = "ssr")]
    #[test]
    fn test_bypass_ipv6_loopback_bracket_format() {
        // The url crate parses http://[::1] into Host::Ipv6(::1) — we get the
        // Ipv6Addr and call .is_loopback() on it. No string comparison needed.
        assert!(validate_widget_url("https://[::1]/secret").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_bypass_ipv6_link_local() {
        // fe80::/10 — IPv6 link-local, analogous to 169.254.x.x
        assert!(validate_widget_url("https://[fe80::1]/metadata").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_bypass_ipv6_unique_local() {
        // fc00::/7 — IPv6 unique-local, analogous to RFC-1918
        assert!(validate_widget_url("https://[fc00::1]/internal").is_err());
    }

    // ── Bypass vector 3: IPv4-mapped IPv6 ────────────────────────────────────
    // Old string check started_with("10.") missed ::ffff:10.0.0.1 entirely.
    // to_ipv4_mapped() catches these.

    #[cfg(feature = "ssr")]
    #[test]
    fn test_bypass_ipv4_mapped_ipv6_metadata() {
        // ::ffff:169.254.169.254 — IPv4-mapped IPv6 form of the cloud metadata endpoint
        assert!(validate_widget_url("https://[::ffff:169.254.169.254]/latest/meta-data").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_bypass_ipv4_mapped_ipv6_private() {
        // ::ffff:10.0.0.1 — IPv4-mapped IPv6 form of a private IP
        assert!(validate_widget_url("https://[::ffff:10.0.0.1]/internal").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_bypass_ipv4_mapped_ipv6_loopback() {
        // ::ffff:127.0.0.1 — IPv4-mapped IPv6 loopback
        assert!(validate_widget_url("https://[::ffff:127.0.0.1]/admin").is_err());
    }

    // ── Bypass vector 4: DNS rebinding ────────────────────────────────────────
    // Static validation cannot detect this. Layer 2 (enforce_ssrf_safe_fetch)
    // is required. We verify the function exists and is async (compile-time check).
    // Integration tests against a real DNS resolver are in /tests/widget_ssrf.rs.

    #[cfg(feature = "ssr")]
    #[test]
    fn test_dns_rebinding_defense_function_exists() {
        // enforce_ssrf_safe_fetch is the mandatory Layer 2 defense.
        // Any domain that passes Layer 1 MUST go through this before fetching.
        // This test asserts the function is callable and returns the right type.
        // The actual rebinding defense requires a live DNS resolver — tested in integration tests.
        let _: fn(&str) -> _ = enforce_ssrf_safe_fetch;
    }

    // ── Static validation: domains still pass Layer 1 (by design) ────────────

    #[cfg(feature = "ssr")]
    #[test]
    fn test_valid_public_domain_passes_layer1() {
        // Domain names pass Layer 1 — DNS resolution happens in Layer 2.
        // This is correct: we cannot know at parse time what IP a domain resolves to.
        assert!(validate_widget_url("https://api.coinbase.com/v2/prices").is_ok());
        assert!(validate_widget_url("https://api.example.com/data").is_ok());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_http_rejected_in_prod() {
        // LEPTOS_ENV is unset in test context — treated as production
        assert!(validate_widget_url("http://api.example.com/data").is_err());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_url_without_host_rejected() {
        assert!(validate_widget_url("https:///no-host").is_err());
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
