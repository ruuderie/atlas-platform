# Observability Setup: OpenTelemetry Abstraction

This document outlines the observability pipeline for the Anchor application, specifically focusing on how we capture metrics in a vendor-agnostic way and export them for reporting.

## Architecture Overview

To ensure the Anchor application is never tightly coupled to a specific monitoring visualization tool (like Grafana, Datadog, or Honeycomb), we employ an architecture using the **OpenTelemetry (OTel) Collector**.

The flow is configured as follows:
```text
[Anchor App] -> (Prometheus format exposed) -> [OTel Collector Service] -> (OTLP Internal) -> [Grafana via Prometheus Exporter]
```

1. **The Application Layer (`anchor-app`)**
   The Rust backend is instrumented using `axum-prometheus`. It automatically collects RED metrics (Rate, Errors, Duration) and exposes them on a local `/metrics` endpoint. The app has no knowledge of the external environment or where these metrics go.
   
2. **The Abstraction Layer (`otel-collector`)**
   A centralized `otel-collector` deployment runs in the same Kubernetes namespace as the application. This collector is explicitly configured to scrape the `anchor-app` pod. It ingests the raw metrics, converts them into standard OpenTelemetry Protocol (OTLP) data in memory, and then exports them out.

3. **The Visualization Layer (Grafana via Prometheus)**
   For your immediate reporting needs, the `otel-collector` is configured with a Prometheus exporter. We attached `prometheus.io/scrape` annotations directly to the `otel-collector` Pod, allowing your cluster's Prometheus to pull the metrics.

---

## Grafana Dashboard Configuration

Because the metrics arrive through the OTel Collector in a standardized format, you can immediately begin building reports in Grafana.

### Key PromQL Metrics

When building your Grafana panels, use the following core metrics exported by the Axum middleware:

*   **Request Rate (Throughput):**
    `rate(axum_http_requests_total[5m])`
*   **Request Latency (Duration):**
    `histogram_quantile(0.95, sum(rate(axum_http_requests_duration_seconds_bucket[5m])) by (le))`
*   **Active Connections/Requests:**
    `axum_http_requests_pending`

You can use the `path` and `method` labels in Grafana to filter traffic to specific routes (e.g., separating API traffic from static page leads).

### Plugging In Alternative Providers

The primary benefit of this abstracted setup is future flexibility. Should you wish to switch from Grafana to an enterprise APM provider (e.g., Datadog, Splunk, New Relic) in the future:

1. **Do not modify the Rust source code.**
2. Open `k8s/base/otel-collector.yaml`.
3. Under `exporters:`, add the new provider's OTLP endpoint and authentication headers.
4. Modify the `metrics` pipeline to include the new exporter:
   ```yaml
   service:
     pipelines:
       metrics:
         receivers: [prometheus]
         exporters: [datadog] # Replaced prometheus exporter
   ```
5. Apply the Kubernetes manifest. Your application's telemetry will instantly start flowing to the new vendor without any downtime or code changes.
