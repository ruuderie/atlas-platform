# G-19 Direct Mail Provider Contract

Plug-ready adapters for landlord acquisition via physical mail. **v1 ships `dm_manual` only.** Lob and PropertyRadar register as stubs (`501` / `NotImplemented`) until wired.

## Provider IDs

| ID | Status | Role |
|----|--------|------|
| `dm_manual` | Implemented | CSV export + manual spend entry |
| `dm_lob` | Stub | Print/mail API + webhooks (future) |
| `dm_property_radar` | Stub | List + mail (future) |

Trait: `crate::services::pm::direct_mail::DirectMailProvider`  
Resolve: `resolve_direct_mail_provider(id)`  
Webhook: `POST /api/admin/integrations/dm/{provider}/webhook`

## UTM conventions

```text
utm_source   = manual | lob | property_radar
utm_medium   = direct_mail
utm_campaign = <atlas_campaigns.global_name or utm_campaign>
utm_content  = <mail_drop utm_content / creative slug>
```

Example:

```text
https://folio1.atlas.oply.co/?utm_source=manual&utm_medium=direct_mail&utm_campaign=miami_q3_dm&utm_content=drop_postcard_a&offer_code=MIAMI-A
```

## Offer codes

Table `atlas_campaign_offer_codes` — unique code → campaign (+ optional mail_drop).  
Waitlist body field `offer_code` resolves campaign and increments `redemption_count`.

## Spend

`POST /api/admin/campaigns/{id}/spend` `{ "cents": 120000, "source": "mail_house_invoice", "external_ref": "INV-1" }`  
Increments `atlas_campaigns.spent_cents`. CAC = spent / conversions in admin UI.

## Attribution

Waitlist + LP events write G-20 `atlas_attribution_touchpoints`.  
Channel `direct_mail` when `utm_medium` ∈ {direct_mail, postcard, mail} or offer_code present.  
OTP verify accepts optional `anonymous_id` for identity resolve (sentinel tenant).

## Stripe / paid conversion

Helper: `attribution_hooks::record_paid_conversion`.  
`POST /api/pub/products/{slug}/pre-order` accepts optional `offer_code`, `campaign_id`, `anonymous_id` and stamps Stripe Checkout `client_reference_id` + metadata (`conversion_id`, `attribution_tenant_id`, …).  
`checkout.session.completed` (and `payment_intent.succeeded` without ledger) records the paid conversion.

## Pixel ops checklist (before first drop)

1. Platform-admin → Products → Folio → **Pixels** tab  
2. Add GA4 + Meta (minimum); confirm `inject_at: head`  
3. Curl homepage product payload — `pixels` must be non-empty  
4. Verify tags fire on Folio marketing SSR  

Admin API: `GET/POST /api/admin/platform/products/{id}/pixels`  
Public product JSON includes `pixels: [{ pixel_type, snippet, inject_at }]`.

## Feature flags

Seeded keys (FlagService catalog):

- `acquisition.dm_tracking` — enable G-20 capture paths (default **on**)  
- `acquisition.open_signup` — when **false**, organic stays waitlist-only  

Toggle via platform-admin `/flags`.

## Admin APIs (DM)

- `GET/POST …/campaigns/{id}/mail-drops`  
- `GET/POST …/campaigns/{id}/offer-codes`  
- `POST …/campaigns/{id}/spend`  
- `GET …/campaigns/{id}/attribution`  
- `GET …/campaigns/{id}/qr`  

## Stitch

`designs/stitch/project_pm/platform_admin/marketing/_direct_mail/`  
`designs/stitch/project_pm/folio/marketing/lp_direct_mail/code.html`
