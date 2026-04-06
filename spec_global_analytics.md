# Specification: Global Analytics & Telemetry

## 1. Overview
The **Global Analytics** module is a macro reporting tool for Platform Administrators. It aggregates data from strictly partitioned tenant databases (Networks and Anchors) to provide a unified perspective on platform growth, engagement, and data volume without breaching cross-tenant leakages.

## 2. Core Objectives
- Monitor platform health metrics (Total Tenants, Active Users).
- Measure network liquidity (Listings created vs Listings approved).
- Visualize traffic (Pageviews, API usage spikes) to aid in infrastructure scaling.

## 3. Architecture & Data Model

### The Telemetry Sink Pattern
Directly querying `COUNT(*)` across thousands of tenant tables is computationally expensive. 
Instead, we will implement an event-driven sink architecture.

1. **`TelemetryEvent` Struct**:
   - `tenant_id: Uuid`
   - `event_type: String` (e.g., `user_signed_up`, `listing_published`)
   - `timestamp: DateTime`

### Batch Processing
A background cron (or Rust task) aggregates these logs nightly or hourly into a time-series database (or a partitioned PostgreSQL `platform_metrics` hypertable if using TimescaleDB).

### Database Entities
- `PlatformMetricsDaily`: Aggregated rows for fast UI queries.
  - Columns: `date`, `tenant_id`, `metric_key` (String), `metric_value` (Integer).

## 4. Platform Admin UI UX
1. **Macro Dashboard**: Big KPI numbers (Total ARR, Total Users, Active Listings).
2. **Growth Graphs**: Leptos-rendered D3/Chart.js graphs for historical trends.
3. **Tenant Leaderboard**: "Top 10 Fastest Growing Networks" based on MAU or Listing submissions.

## 5. Performance Considerations
- All telemetry writes must be non-blocking. Use an message queue (RabbitMQ/Redis Streams) or an asynchronous direct `INSERT` pool.
- Caching: The Platform Admin dashboard should retrieve KPIs from a Redis cache, refreshed every 15 minutes, rather than executing fresh SQL aggregations.
