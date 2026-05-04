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

                // For subdomains the RP ID is the effective registrable domain.
                // For custom tenant domains it is the custom domain itself.
                let rp_id = origin_url
                    .host_str()
                    .ok_or_else(|| "No host in origin".to_string())?;

                let webauthn = Arc::new(
                    WebauthnBuilder::new(rp_id, &origin_url)
                        .map_err(|e| format!("Invalid WebauthnBuilder arguments: {e}"))?
                        .rp_name("Atlas Platform")
                        .build()
                        .map_err(|e| format!("Failed to build Webauthn instance: {e}"))?,
                );

                info!(
                    "Dynamically allocated and cached WebAuthn instance for origin: {}",
                    origin
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

        match app_domain::Entity::find()
            .filter(app_domain::Column::DomainName.eq(host))
            .one(&self.db)
            .await
        {
            Ok(Some(_)) => true,
            _ => false,
        }
    }
}
