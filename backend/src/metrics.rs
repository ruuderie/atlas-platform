use lazy_static::lazy_static;
use prometheus::{CounterVec, HistogramVec, Opts, Registry};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    pub static ref MAGIC_LINK_REQUESTS: CounterVec = CounterVec::new(
        Opts::new("magic_link_requests_total", "Total number of magic link requests"),
        &["tenant_id", "status"]
    ).unwrap();

    pub static ref MAGIC_LINK_DUPLICATES_PREVENTED: CounterVec = CounterVec::new(
        Opts::new("magic_link_duplicates_prevented_total", "Number of duplicate magic link requests prevented by the partial unique index"),
        &["tenant_id"]
    ).unwrap();

    pub static ref AUTH_REQUESTS: CounterVec = CounterVec::new(
        Opts::new("auth_requests_total", "Total auth-related requests (magic link, verify, session)"),
        &["action", "status"]
    ).unwrap();

    pub static ref AUTH_LATENCY: HistogramVec = HistogramVec::new(
        Opts::new("auth_request_duration_seconds", "Latency of auth operations"),
        &["action"]
    ).unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(MAGIC_LINK_REQUESTS.clone())).unwrap();
    REGISTRY.register(Box::new(MAGIC_LINK_DUPLICATES_PREVENTED.clone())).unwrap();
    REGISTRY.register(Box::new(AUTH_REQUESTS.clone())).unwrap();
    REGISTRY.register(Box::new(AUTH_LATENCY.clone())).unwrap();
}

pub fn metrics_handler() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
