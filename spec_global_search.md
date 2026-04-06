# Specification: Global Command Search (Omnibar)

## 1. Overview
The **Global Command Search Bar** replaces the redundant Top Nav links in the UI, serving as an omnibar (accessible via `Cmd+K` or `/`). It provides a lightning-fast, Elasticsearch-style lookup across structured relational data, ignoring tenant boundaries when accessed by Platform Admins.

## 2. Core Objectives
- Allow instantaneous lookups of User Emails, Network IDs, Category Tags, and CRM Deals without navigating through sidebars.
- Act as a command palette to quickly execute application-wide administrative actions (e.g., "> Impersonate User", "> Clear Platform Cache").
- Respect RBAC (Role-Based Access Control) to filter search results based on the searcher's permission scope.

## 3. Architecture & Data Model

### The Search Index
PostgreSQL's built-in Full Text Search (FTS) feature (using `tsvector` and `tsquery`) handles multi-table searching.

### `global_search_index` Materialized View (or separate table)
A single indexed table aggregating search targets:
- `id`: Uuid (Primary Key)
- `entity_type`: Enum (`User`, `Network`, `Listing`, `Deal`)
- `entity_id`: Uuid
- `tenant_id`: Uuid
- `searchable_text`: `tsvector`
- `metadata`: JSONB (For UI rendering hints, like Avatar URLs or Status chips)

### Background Synchronization
Since computing `tsvector` across millions of rows live is slow, the index must be kept fresh using Database Triggers or asynchronous Rust handlers applying mutations to the `global_search_index` whenever an entity changes.

## 4. Platform Admin UI UX
1. **The Modal UI**: Pressing `Cmd+K` triggers a fixed, centered glassmorphism modal with a blurred background.
2. **Keyboard Navigation**: Up and Down arrows iterate through search results; `Enter` navigates to the detailed view.
3. **Segmented Results**: Searching "John" should yield groups: 
   - *Users* (John Doe admin@john.com) 
   - *Listings* (John's Bakery) 
   - *Networks* (JohnCo Anchor)

## 5. Security & Isolation
- The `tenant_id` column in the search index is critical. Platform Admins run queries without `tenant_id` constraints. Tenant operators will implicitly have `WHERE tenant_id = $1` injected into their search execution plans.
