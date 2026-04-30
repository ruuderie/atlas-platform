# Seed Script Archive

This directory contains the **original raw SQL scripts** that define the demo and
test data for platform apps. These files are **reference-only**.

## ⚠️ Do Not Execute Directly

The canonical, executable seed packs are implemented in Rust at:

```
backend/src/atlas_apps/seeds/
```

Running these SQL files directly against the database will bypass tenant scoping,
UUID generation, and the idempotency guards enforced by the Rust seed executors.

## Contents

| File | Description |
|---|---|
| `add_categories.sql` | Category trees for T&L, Automotive, Construction, Beauty, Financial, Healthcare |
| `seed_50_mock_listings.sql` | 55 construction listings — CT Build Pros style |
| `construction_listings.sql` | Additional construction/service listings |
| `more_listings.sql` | Cross-industry sample listings |
| `listsings_for_profiles.sql` | Large listing set scoped to profiles |
| `random_directory.sql` | 5 network types + sample networks per industry |
| `random_profiles.sql` | 100 random profile rows |
| `random_businesses.sql` | 100 random business rows |
| `random_listings.sql` | 500 random listing rows |
| `random_listing_attributes.sql` | Listing attribute rows |
| `random_ad_purchases.sql` | 200 random ad purchase rows |
| `create_directory_type.sql` | Network type definitions |
| `patch.sql` | Misc one-off data patches |
| `create_users.sql` | Test user data |
| `listing_attributes.sql` | Listing attribute definitions |
