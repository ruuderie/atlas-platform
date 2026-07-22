# Atlas Platform — Backlog

Known gaps and deferred epics that are **not** yet in the schema or shipped product surface.
Read this before planning net-new work — the item may already be scoped here.

---

## External PMC property/book handoff (from landlord hub)

**Status:** Deferred (after same-tenant Delegate to PM V1)

**Job:** Landlord hands a property or book to an **external PMC tenant**; becomes Owner-lite (`/o`); day-to-day ops run in the PMC client book via `managed_account_id`.

**Depends on:**

- Same-tenant Delegate to PM UX patterns (Property Hub Management sheet) — shipped Rev 14
- PMC client APIs already at `/api/folio/pm/clients*`

**Open questions for that epic:**

- Invite-into-PMC vs landlord-initiated join code
- Partial portfolio vs full book transfer
- Billing / owner-statement continuity across tenants

**Non-goals of V1 (already shipped):** Same-tenant hire of a PM onto the landlord’s own Folio book (invite + `management_agreement` + asset grants).
