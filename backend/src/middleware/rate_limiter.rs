use std::sync::Arc;
use std::time::{Duration, Instant};
use axum::http::StatusCode;
use dashmap::DashMap;
use uuid::Uuid;

const MAX_REQUESTS: u32 = 100;
const WINDOW_SIZE: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct RateLimiter {
    store: Arc<DashMap<String, (Instant, u32)>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        tracing::info!("Initializing rate limiter with max {} requests per {} seconds", 
            MAX_REQUESTS, WINDOW_SIZE.as_secs());
        RateLimiter {
            store: Arc::new(DashMap::new()),
        }
    }

    pub async fn check_rate_limit(&self, ip: &str) -> Result<(), StatusCode> {
        let request_id = Uuid::new_v4();
        tracing::info!("[{}] Rate limit check started for IP: {}", request_id, ip);
        
        let now = Instant::now();
        let mut should_allow = true;
        let mut current_count = 0;

        // Check if this IP is already in our store
        if let Some(mut entry) = self.store.get_mut(ip) {
            let (window_start, count) = &mut *entry;
            let elapsed = now.duration_since(*window_start);
            
            tracing::debug!("[{}] Existing rate limit entry found - Count: {}, Window age: {:.2}s", 
                request_id, *count, elapsed.as_secs_f32());
                
            // Reset counter if window has expired
            if elapsed > WINDOW_SIZE {
                tracing::debug!("[{}] Rate limit window expired, resetting counter", request_id);
                *window_start = now;
                *count = 1;
                current_count = 1;
            } else {
                // Increment counter within current window
                *count += 1;
                current_count = *count;
                
                tracing::debug!("[{}] Incremented request count to {} (max: {})", 
                    request_id, current_count, MAX_REQUESTS);
                    
                // Check if rate limit exceeded
                if current_count > MAX_REQUESTS {
                    should_allow = false;
                    tracing::warn!("[{}] Rate limit exceeded for IP: {} - {} requests in {:.2}s", 
                        request_id, ip, current_count, elapsed.as_secs_f32());
                }
            }
        } else {
            // First request from this IP
            tracing::debug!("[{}] First request from IP: {}, initializing rate limit counter", request_id, ip);
            self.store.insert(ip.to_string(), (now, 1));
            current_count = 1;
        }

        // Log current rate limit status
        if should_allow {
            tracing::debug!("[{}] Request allowed: {} of {} in current window for IP: {}", 
                request_id, current_count, MAX_REQUESTS, ip);
                
            // Log when approaching limit
            if current_count > MAX_REQUESTS * 3 / 4 {
                tracing::info!("[{}] IP approaching rate limit: {} ({:.0}% of max {})", 
                    request_id, ip, (current_count as f32 / MAX_REQUESTS as f32) * 100.0, MAX_REQUESTS);
            }
            
            Ok(())
        } else {
            // Rate limit exceeded
            tracing::warn!("[{}] Rate limit exceeded - returning 429 Too Many Requests for IP: {}", 
                request_id, ip);
                
            // Periodically clean up the store to prevent memory leaks
            if current_count % 1000 == 0 {
                tracing::info!("Rate limiter store size: {} entries", self.store.len());
                self.cleanup_expired_entries();
            }
            
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
    }
    
    // Clean up expired entries to prevent memory leaks
    fn cleanup_expired_entries(&self) {
        let now = Instant::now();
        let mut removed_count = 0;
        
        tracing::debug!("Starting cleanup of expired rate limit entries");
        
        self.store.retain(|_, (window_start, _)| {
            let retain = now.duration_since(*window_start) <= WINDOW_SIZE * 2;
            if !retain {
                removed_count += 1;
            }
            retain
        });
        
        tracing::info!("Rate limiter cleanup complete - removed {} expired entries", removed_count);
    }
    
    // Get current statistics about rate limiting
    pub fn get_stats(&self) -> RateLimiterStats {
        let total_entries = self.store.len();
        let mut blocked_ips = 0;
        let mut high_usage_ips = 0;
        
        for entry in self.store.iter() {
            let (_, count) = *entry.value();
            if count > MAX_REQUESTS {
                blocked_ips += 1;
            } else if count > MAX_REQUESTS * 3 / 4 {
                high_usage_ips += 1;
            }
        }
        
        RateLimiterStats {
            total_tracked_ips: total_entries,
            blocked_ips,
            high_usage_ips,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiterStats {
    pub total_tracked_ips: usize,
    pub blocked_ips: usize,
    pub high_usage_ips: usize,
}
