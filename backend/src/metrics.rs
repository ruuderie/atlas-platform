use lazy_static::lazy_static;
use prometheus::{CounterVec, HistogramVec, Opts, Registry};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    pub static ref MAGIC_LINK_REQUESTS: CounterVec = CounterVec::new(
        Opts::new("magic_link_requests_total", "Total number of magic link requests"),
        &["tenant_id", "app_instance_id", "status"]
    ).unwrap();

    pub static ref MAGIC_LINK_DUPLICATES_PREVENTED: CounterVec = CounterVec::new(
        Opts::new("magic_link_duplicates_prevented_total", "Number of duplicate magic link requests prevented"),
        &["tenant_id", "app_instance_id"]
    ).unwrap();

    pub static ref AUTH_REQUESTS: CounterVec = CounterVec::new(
        Opts::new("auth_requests_total", "Total auth-related requests"),
        &["tenant_id", "app_instance_id", "action", "status"]
    ).unwrap();

    pub static ref AUTH_LATENCY: HistogramVec = HistogramVec::new(
        Opts::new("auth_request_duration_seconds", "Latency of auth operations"),
        &["tenant_id", "app_instance_id", "action"]
    ).unwrap();

    // === PASSKEY METRICS ===
    pub static ref PASSKEY_REGISTRATION_STARTED: CounterVec = CounterVec::new(
        Opts::new("passkey_registration_started_total", "Number of passkey registration flows started"),
        &["tenant_id", "app_instance_id"]
    ).unwrap();

    pub static ref PASSKEY_REGISTRATION_SUCCESS: CounterVec = CounterVec::new(
        Opts::new("passkey_registration_success_total", "Number of successful passkey registrations"),
        &["tenant_id", "app_instance_id"]
    ).unwrap();

    pub static ref PASSKEY_AUTH_SUCCESS: CounterVec = CounterVec::new(
        Opts::new("passkey_auth_success_total", "Number of successful passkey authentications"),
        &["tenant_id", "app_instance_id"]
    ).unwrap();

    // === FRONTEND HYDRATION PANICS ===
    pub static ref FRONTEND_HYDRATION_PANICS: CounterVec = CounterVec::new(
        Opts::new("frontend_hydration_panics_total", "Number of detected Leptos hydration panics"),
        &["tenant_id", "app_instance_id", "component"]
    ).unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(MAGIC_LINK_REQUESTS.clone())).unwrap();
    REGISTRY.register(Box::new(MAGIC_LINK_DUPLICATES_PREVENTED.clone())).unwrap();
    REGISTRY.register(Box::new(AUTH_REQUESTS.clone())).unwrap();
    REGISTRY.register(Box::new(AUTH_LATENCY.clone())).unwrap();
    REGISTRY.register(Box::new(PASSKEY_REGISTRATION_STARTED.clone())).unwrap();
    REGISTRY.register(Box::new(PASSKEY_REGISTRATION_SUCCESS.clone())).unwrap();
    REGISTRY.register(Box::new(PASSKEY_AUTH_SUCCESS.clone())).unwrap();
    REGISTRY.register(Box::new(FRONTEND_HYDRATION_PANICS.clone())).unwrap();
}

pub fn metrics_handler() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
