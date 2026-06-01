use std::sync::Arc;
use std::time::{Duration, Instant};
use axum::http::StatusCode;
use dashmap::DashMap;
use uuid::Uuid;

const MAX_REQUESTS: u32 = 100;
const WINDOW_SIZE: Duration = Duration::from_secs(60);

const AUTH_MAX_IP_REQUESTS: u32 = 5;
const AUTH_MAX_EMAIL_REQUESTS: u32 = 3;
const AUTH_WINDOW_SIZE: Duration = Duration::from_secs(600); // 10 minutes

// ⚠️  REPLICA BYPASS LIMITATION
// This rate limiter uses in-process DashMap state. With replicas > 1, each pod
// has independent counters \u2014 an attacker can send (replicas × limit) requests
// before either pod triggers a block.
//
// Short-term: keep `replicas: 1` in the backend Deployment manifest.
// Long-term: replace DashMap stores with a Redis INCR+EXPIRE pattern, or
// enforce primary rate limiting at the Cloudflare WAF layer (rate limit rule
// on /magic-links/request by IP \u2014 Cloudflare sees the true IP pre-pod).

#[derive(Clone)]
pub struct RateLimiter {
    store: Arc<DashMap<String, (Instant, u32)>>,
    auth_ip_store: Arc<DashMap<String, (Instant, u32)>>,
    auth_email_store: Arc<DashMap<String, (Instant, u32)>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        tracing::info!("Initializing rate limiter with max {} requests per {} seconds", 
            MAX_REQUESTS, WINDOW_SIZE.as_secs());
        RateLimiter {
            store: Arc::new(DashMap::new()),
            auth_ip_store: Arc::new(DashMap::new()),
            auth_email_store: Arc::new(DashMap::new()),
        }
    }

    pub async fn check_rate_limit(&self, ip: &str) -> Result<(), StatusCode> {
        let request_id = Uuid::new_v4();
        tracing::info!("[{}] Rate limit check started for IP: {}", request_id, ip);
        
        let now = Instant::now();
        let mut should_allow = true;
        #[allow(unused_assignments)]
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
    
    pub async fn check_auth_rate_limit(&self, ip: &str, email: &str) -> Result<(), StatusCode> {
        let now = Instant::now();
        let mut allow = true;

        // Check IP limit
        if let Some(mut entry) = self.auth_ip_store.get_mut(ip) {
            let (window_start, count) = &mut *entry;
            let elapsed = now.duration_since(*window_start);
            if elapsed > AUTH_WINDOW_SIZE {
                *window_start = now;
                *count = 1;
            } else {
                *count += 1;
                if *count > AUTH_MAX_IP_REQUESTS { allow = false; }
            }
        } else {
            self.auth_ip_store.insert(ip.to_string(), (now, 1));
        }

        // Check Email limit
        if let Some(mut entry) = self.auth_email_store.get_mut(email) {
            let (window_start, count) = &mut *entry;
            let elapsed = now.duration_since(*window_start);
            if elapsed > AUTH_WINDOW_SIZE {
                *window_start = now;
                *count = 1;
            } else {
                *count += 1;
                if *count > AUTH_MAX_EMAIL_REQUESTS { allow = false; }
            }
        } else {
            self.auth_email_store.insert(email.to_string(), (now, 1));
        }

        if allow {
            Ok(())
        } else {
            tracing::warn!("Auth rate limit exceeded for IP: {} or Email: {}", ip, email);
            // Clean up occasionally
            if rand::random::<u16>() % 100 == 0 {
                self.auth_ip_store.retain(|_, (ws, _)| now.duration_since(*ws) <= AUTH_WINDOW_SIZE * 2);
                self.auth_email_store.retain(|_, (ws, _)| now.duration_since(*ws) <= AUTH_WINDOW_SIZE * 2);
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
