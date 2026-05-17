use webauthn_rs::prelude::*;
use std::sync::Arc;
use moka::future::Cache;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use tracing::{info, warn};

use crate::entities::app_domain;

/// Thread-safe registry that creates and caches `Webauthn` instances per tenant origin.
///
/// # DoS protection
/// The cache is capped at `max_capacity` entries. Requests for origins that do not
/// correspond to a verified tenant in the `app_domains` table are rejected before any
/// Webauthn instance is constructed, preventing RAM exhaustion from spoofed origins.
///
/// # Concurrency
/// `get_or_create` uses `moka::future::Cache::try_get_with`, which guarantees that
/// concurrent requests for the same uncached origin execute the initialiser exactly
/// once. Subsequent callers block on the in-flight future rather than racing to the
/// database, preventing thundering-herd and duplicate-insert issues.
pub struct WebauthnRegistry {
    /// Caches the Webauthn instance keyed by normalised origin string.
    /// Max capacity prevents memory exhaustion from spoofed origins.
    cache: Cache<String, Arc<Webauthn>>,
    db: DatabaseConnection,
}

impl WebauthnRegistry {
    /// Create a new registry with a capped cache capacity (e.g. 10,000 tenants).
    pub fn new(db: DatabaseConnection, max_capacity: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .build(),
            db,
        }
    }

    /// Pre-warm the cache with a known-good origin at server startup.
    ///
    /// Call this for every first-party origin (platform admin, UAT, production) so
    /// that those origins never hit the database lookup path. This also eliminates
    /// the need for any hard-coded bypass inside `verify_origin_in_db`.
    pub async fn seed(&self, rp_id: &str, origin_url: &Url) -> Result<(), String> {
        let webauthn = Arc::new(
            WebauthnBuilder::new(rp_id, origin_url)
                .map_err(|e| format!("Invalid WebauthnBuilder arguments during seed: {e}"))?
                .rp_name("Atlas Platform")
                .build()
                .map_err(|e| format!("Failed to build Webauthn on seed: {e}"))?,
        );

        let origin_str = origin_url.as_str().trim_end_matches('/').to_string();
        self.cache.insert(origin_str.clone(), webauthn).await;

        info!("Seeded WebauthnRegistry with platform origin: {}", origin_str);
        Ok(())
    }

    /// Retrieve a cached `Webauthn` instance for `origin`, or build and cache one if
    /// the origin belongs to a verified tenant.
    ///
    /// Uses `try_get_with` so that concurrent requests for the same new origin share a
    /// single in-flight initialiser — no duplicate DB queries or cache inserts.
    ///
    /// # rp_id derivation
    /// `rp_id` is always derived as eTLD+1 (registrable domain), never the full
    /// subdomain host. The WebAuthn spec requires that `rpId` be a suffix of the
    /// origin host, but passkeys are bound to the `rpId` at registration time. Using
    /// the full subdomain (e.g. "dev.buildwithruud.com") as `rpId` would cause browsers
    /// to reject challenges for credentials registered under "buildwithruud.com".
    pub async fn get_or_create(&self, origin: &str) -> Result<Arc<Webauthn>, String> {
        // Normalise: strip any trailing slash for consistent cache keys.
        let origin = origin.trim_end_matches('/').to_string();

        self.cache
            .try_get_with(origin.clone(), async {
                // Only reached on a cache miss — verify the origin belongs to a real tenant.
                if !self.verify_origin_in_db(&origin).await {
                    warn!("Rejected unauthorized WebAuthn origin request: {}", origin);
                    return Err("Unauthorized origin".to_string());
                }

                let origin_url = Url::parse(&origin)
                    .map_err(|_| "Invalid origin URL".to_string())?;

                // FIX: use eTLD+1 as rpId, not the full subdomain host.
                // "dev.buildwithruud.com" → "buildwithruud.com"
                // "buildwithruud.com"     → "buildwithruud.com"
                // This ensures passkeys registered from any subdomain share the same
                // rpId and can authenticate from any sibling subdomain of that tenant.
                let host = origin_url
                    .host_str()
                    .ok_or_else(|| "No host in origin".to_string())?;
                let rp_id = effective_tld_plus_one(host);

                let webauthn = Arc::new(
                    WebauthnBuilder::new(&rp_id, &origin_url)
                        .map_err(|e| format!("Invalid WebauthnBuilder arguments: {e}"))?
                        .rp_name("Atlas Platform")
                        .build()
                        .map_err(|e| format!("Failed to build Webauthn instance: {e}"))?,
                );

                info!(
                    "Cached WebAuthn instance for origin: {} (rpId: {})",
                    origin, rp_id
                );
                Ok(webauthn)
            })
            .await
            // try_get_with wraps the error in Arc; unwrap to the inner String.
            .map_err(|e| e.to_string())
    }

    /// Query the `app_domains` table to confirm the origin host is tied to an active tenant.
    ///
    /// First-party origins (platform admin, UAT, production) must be pre-seeded via
    /// `seed()` at startup and will hit the cache before this function is ever called.
    /// There is intentionally no hard-coded bypass here — seeding is the right mechanism.
    ///
    /// The `app_domains.domain_name` column stores the bare host (e.g. "anchor.atlas.oply.co"
    /// or "mycompany.com") — no scheme or port.
    async fn verify_origin_in_db(&self, origin: &str) -> bool {
        let host = match Url::parse(origin) {
            Ok(url) => match url.host_str() {
                Some(h) => h.to_string(),
                None => return false,
            },
            Err(_) => return false,
        };

        // Generate all possible base domains to support subdomain matching.
        // e.g. "uat.buildwithruud.com" -> ["uat.buildwithruud.com", "buildwithruud.com"]
        let mut possible_domains = vec![host.clone()];
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() > 2 {
            for i in 1..parts.len() - 1 {
                possible_domains.push(parts[i..].join("."));
            }
        }

        match app_domain::Entity::find()
            .filter(app_domain::Column::DomainName.is_in(possible_domains))
            .one(&self.db)
            .await
        {
            Ok(Some(_)) => true,
            _ => false,
        }
    }
}

/// Returns the registrable domain (eTLD+1) for a given hostname.
///
/// This is used to derive the WebAuthn `rpId`. The WebAuthn spec requires `rpId` to be
/// the eTLD+1 so that passkeys registered from any subdomain (e.g. `dev.example.com`)
/// can be used from the apex domain (`example.com`) and vice versa.
///
/// # Examples
/// - `"dev.buildwithruud.com"` → `"buildwithruud.com"`
/// - `"buildwithruud.com"`     → `"buildwithruud.com"`
/// - `"localhost"`             → `"localhost"`
///
/// # Limitations
/// This naive implementation splits on `.` and takes the last two labels. It is correct
/// for single-label TLDs (`.com`, `.co`, `.io`). For multi-label TLDs (`.co.uk`) the
/// result would be incorrect (e.g. `"co.uk"` instead of `"example.co.uk"`).
/// If multi-label TLD support is required, replace with the `publicsuffix` crate.
pub fn effective_tld_plus_one(host: &str) -> String {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() > 2 {
        parts[parts.len() - 2..].join(".")
    } else {
        host.to_string()
    }
}

