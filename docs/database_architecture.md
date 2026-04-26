# Atlas Platform Database Architecture

The Atlas Platform operates on a **Single-Database, Multi-Tenant** architecture using PostgreSQL. All applications (`backend`, `anchor-app`, `network-app`) communicate with this strict, unified schema. Data segmentation is primarily enforced at the application layer through `tenant_id` foreign keys, preventing data leakage across organizational boundaries while keeping infrastructure costs minimal.

This diagram demonstrates how tables are grouped logically into distinct domains:

```mermaid
erDiagram
    %% Core Multi-Tenant Architecture
    TENANT ||--o{ USER_ACCOUNT : "manages"
    TENANT ||--o{ TENANT_SETTING : "configures"
    TENANT ||--o{ TENANT_SUBSCRIPTION : "billed via"
    
    USER ||--o{ USER_ACCOUNT : "belongs to"
    USER ||--o{ SESSION : "authenticates"
    USER ||--o{ PASSKEY : "secures"
    
    %% Application Distribution & Routing
    TENANT ||--o{ APP_INSTANCE : "owns"
    APP_INSTANCE ||--o{ APP_DOMAIN : "runs on"
    APP_INSTANCE ||--o{ APP_PAGE : "renders"
    APP_INSTANCE ||--o{ APP_MENU : "navigates"
    
    %% CRM & Sales Operations (Network App)
    TENANT ||--o{ CUSTOMER : "tracks"
    TENANT ||--o{ LEAD : "captures"
    CUSTOMER ||--o{ DEAL : "negotiates"
    LEAD ||--o{ DEAL : "converts to"
    DEAL ||--o{ ACTIVITY : "tracks touches"
    DEAL ||--o{ DEAL_CONTACT : "involves"
    CUSTOMER ||--o{ CASE : "supports"
    
    %% Content Management (Anchor App Base)
    TENANT ||--o{ RESUME_PROFILE : "publishes"
    TENANT ||--o{ SERVICES : "offers"
    RESUME_PROFILE ||--o{ RESUME_ENTRY : "lists"
    TENANT ||--o{ LISTING : "showcases"
    
    %% Telemetry, Logs & Webhooks
    TENANT ||--o{ AUDIT_LOG : "audits"
    TENANT ||--o{ WEBHOOK_ENDPOINT : "integrates via"
    WEBHOOK_ENDPOINT ||--o{ WEBHOOK_DELIVERY : "receives"
    
    %% Global Tables
    REQUEST_LOG {
        uuid id
        string uri
        string method
        int status_code
    }
    
    TELEMETRY_EVENTS {
        uuid id
        string event_type
        jsonb properties
    }
```

### Key Architectural Concepts

**1. Tenant Context Engine (Multi-Tenancy)**
Every table specific to user data holds a `tenant_id` UUID physically mapping back to the `tenant` table. The backend server functions (using `Axum`) inject the correct `tenant_id` into the request context dynamically based on the requesting URL's `domain_name` (via `app_domains`). Queries utilizing SeaORM are actively filtered by this globally injected context.

**2. Dynamic "App Instance" Resolution**
Instead of spinning up physically separate codebases or databases for each micro-SaaS you deploy, Atlas routes HTTP hostnames to an `app_instance` record. The `app_type` flag (e.g. `Network`, `anchor`) tells Leptos which SSR rendering engine to dispatch, and pulls localized `settings` JSON to customize colors, copy, and layout on exactly the same infrastructure.

**3. Headless CMS / Anchor Data Models**
Tables like `resume_entries`, `services`, and `app_pages` serve as a headless CMS layer. When a user requests `buildwithruud.com`, the API looks up the assigned `tenant_id` from the `app_domains` list, then grabs the associated `resume_profiles` strictly associated with that ID.

**4. Fully Synchronized Billing & Webhooks**
By placing `tenant_subscription` and `webhook_endpoint` entirely inside the unified schema, you eliminate split-brain architectural issues. When a webhook hits `/api/webhooks/paddle/`, it can directly map the Stripe/Paddle reference token onto the exact `tenant_id`, instantly updating user entitlements across all connected `app_instances`.
