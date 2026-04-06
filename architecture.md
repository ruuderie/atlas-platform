```mermaid
erDiagram
    %% Original Network System Relationships
    User ||--o{ UserAccount : has
    User ||--o{ Session : has
    User ||--o{ RequestLog : has
    Account ||--o{ UserAccount : has
    Account ||--o{ Profile : has
    Account }|--|| Tenant : belongs_to
    Tenant ||--o{ AppInstance : installs
    Tenant ||--|{ Profile : has
    Tenant ||--|{ Listing : has
    Tenant ||--|{ Template : has
    AppInstance ||--|{ AppDomain : routes
    Profile ||--o{ Listing : creates
    Profile ||--o{ AdPurchase : makes
    Listing }|--|| Category : belongs_to
    Listing ||--o{ ListingAttribute : has
    Listing ||--o{ AdPurchase : has
    Template }|--|| Category : belongs_to
    Template }|--|| Category : belongs_to
    Template }|--|| Tenant : belongs_to
    Template ||--o{ ListingAttribute : has
    Category ||--o{ Category : has_subcategories

    %% New CRM Relationships
    Customer ||--o{ Deal : has
    Customer ||--o{ Contact : has
    Customer ||--o{ Case : has
    Customer ||--o{ Activity : has
    Customer ||--o{ Note : has
    Deal ||--o{ DealContact : has
    Deal ||--o{ Activity : has
    Deal ||--o{ Note : has
    Contact ||--o{ DealContact : has
    Contact ||--o{ Activity : has
    Contact ||--o{ Note : has
    Lead ||--o{ Activity : has
    Lead ||--o{ Note : has
    Case ||--o{ Activity : has
    Case ||--o{ Note : has
    Activity ||--o{ Note : has
    FileAssociation }|--|| File : belongs_to

    %% File Associations
    Customer ||--o{ FileAssociation : has
    Deal ||--o{ FileAssociation : has
    Contact ||--o{ FileAssociation : has
    Lead ||--o{ FileAssociation : has
    Case ||--o{ FileAssociation : has
    Activity ||--o{ FileAssociation : has
    Note ||--o{ FileAssociation : has

    User {
        UUID id PK
        string username
        string email
        string password_hash
        boolean is_admin
        boolean is_active
        datetime created_at
        datetime updated_at
    }

    Account {
        UUID id PK
        UUID tenant_id FK
        string name
        boolean is_active
        datetime created_at
        datetime updated_at
    }

    UserAccount {
        UUID id PK
        UUID user_id FK
        UUID account_id FK
        string role
        boolean is_active
        datetime created_at
        datetime updated_at
    }

    Profile {
        UUID id PK
        UUID account_id FK
        UUID tenant_id FK
        string profile_type
        string display_name
        string contact_info
        string business_name
        string business_address
        string business_phone
        string business_website
        json additional_info
        boolean is_active
        datetime created_at
        datetime updated_at
    }

    Tenant {
        UUID id PK
        string name
        string description
        string site_status
        json custom_settings
        datetime created_at
        datetime updated_at
    }

    AppInstance {
        UUID id PK
        UUID tenant_id FK
        string app_type
        string database_url
        json settings
        datetime created_at
        datetime updated_at
    }

    AppDomain {
        UUID id PK
        UUID app_instance_id FK
        string domain
        boolean is_primary
    }

    Listing {
        UUID id PK
        UUID profile_id FK
        UUID tenant_id FK
        UUID category_id FK
        UUID based_on_template_id FK
        string title
        string description
        string listing_type
        int64 price
        string price_type
        string country
        string state
        string city
        string neighborhood
        float latitude
        float longitude
        json additional_info
        string status
        boolean is_featured
        boolean is_based_on_template
        boolean is_ad_placement
        boolean is_active
        datetime created_at
        datetime updated_at
    }

    ListingAttribute {
        UUID id PK
        UUID listing_id FK
        UUID template_id FK
        string attribute_type
        string attribute_key
        json value
    }

    Category {
        UUID id PK
        UUID parent_category_id FK
        string name
        string description
        boolean is_custom
        boolean is_active
        datetime created_at
        datetime updated_at
    }

    Template {
        UUID id PK
        UUID tenant_id FK
        UUID category_id FK
        string name
        string description
        string template_type
        boolean is_active
        datetime created_at
        datetime updated_at
    }

    AdPurchase {
        UUID id PK
        UUID listing_id FK
        UUID profile_id FK
        datetime start_date
        datetime end_date
        float price
        string status
    }

    Session {
        UUID id PK
        UUID user_id FK
        string bearer_token
        string refresh_token
        datetime token_expiration
        boolean is_active
    }

    RequestLog {
        UUID id PK
        UUID user_id FK
        string ip_address
        string path
        string method
        int status_code
        string request_type
    }

    Customer {
        UUID id PK
        string name
        UUID primary_contact_id FK
        string customer_type
        json attributes
        string cpf
        string cnpj
        string tin
        string email
        string phone
        string whatsapp
        string telegram
        string twitter
        string instagram
        string facebook
        string website
        float annual_revenue
        int employee_count
        boolean is_active
        json billing_address
        json shipping_address
        datetime created_at
        datetime updated_at
    }

    Deal {
        UUID id PK
        UUID customer_id FK
        string name
        float amount
        string status
        string stage
        datetime close_date
        boolean is_active
        datetime created_at
        datetime updated_at
    }

    Contact {
        UUID id PK
        UUID customer_id FK
        string name
        string email
        string phone
        string role
        datetime created_at
        datetime updated_at
    }

    Lead {
        UUID id PK
        UUID associated_deal_id FK
        string name
        string email
        string phone
        string status
        boolean is_converted
        datetime created_at
        datetime updated_at
    }

    Case {
        UUID id PK
        UUID customer_id FK
        string title
        string description
        string status
        string priority
        UUID assigned_to FK
        datetime closed_at
        datetime created_at
        datetime updated_at
    }

    Activity {
        UUID id PK
        UUID account_id FK
        UUID deal_id FK
        UUID customer_id FK
        UUID lead_id FK
        UUID contact_id FK
        UUID case_id FK
        string activity_type
        string title
        string description
        string status
        datetime due_date
        datetime completed_at
        json associated_entities
        UUID created_by FK
        UUID assigned_to FK
        datetime created_at
        datetime updated_at
    }

    Note {
        UUID id PK
        string content
        UUID created_by FK
        string entity_type
        UUID entity_id
        datetime created_at
        datetime updated_at
    }

    File {
        UUID id PK
        string name
        string file_type
        string storage_path
        int64 size
        datetime created_at
        datetime updated_at
    }

    FileAssociation {
        UUID id PK
        UUID file_id FK
        string associated_entity_type
        UUID associated_entity_id
    }

    DealContact {
        UUID deal_id FK
        UUID contact_id FK
    }
```

## Deployment & Infrastructure Architecture

```mermaid
graph TD
    User[End User] -->|Visits app.domain.com| Proxy[Caddy Reverse Proxy / K8s Ingress]
    Proxy -->|Preserves Host Header| AppInst[App Server SSR/CSR e.g. Network, Anchor]
    AppInst -->|API Lookup with Host| Backend[Rust Axum Backend API]
    Backend -->|Resolves Tenant via AppDomain| DB[(PostgreSQL Database)]
    Admin[Platform Admin] -->|Visits admin.domain.com| Proxy
    Proxy --> AdminApp[Platform Admin CSR App]
    AdminApp --> Backend
```

The system uses a highly scalable Docker multi-stage environment natively supporting dynamic multi-tenancy. A single `Tenant` (Organization) can run multiple `AppInstances` (e.g. Network, Anchor) which share the underlying CRM and Data APIs, isolated securely via `tenant_id`. GitHub Actions automates CI/CD, pushing `ghcr.io` containers into Kubernetes using Kustomize overlays.

---

&copy; Copyright Ruud Salym Erie & Oplyst International, LLC. All Rights Reserved.