# Specification: Billing & Monetization (Payment Abstraction Layer)

## 1. Overview
The **Billing & Monetization** module is the revenue engine of the Atlas Platform. Instead of building tight couplings to a single vendor (like Stripe), we will implement a **Payment Gateway Abstraction Layer**. This ensures sovereign control, allows for cryptocurrency-native flows (Bitcoin, USDT), and supports hot-swapping fiat gateways (Stripe, Adyen, Paddle) natively.

## 2. Core Objectives
- Unify multi-tenant subscription tracking.
- Meter tenant usage (active listings, API volume).
- Process Fiat (Credit Cards) and Crypto (BTC/USDT) agnostically.
- Enable Stripe Connect style split-payments for marketplace tenants independent of the processor.

## 3. Architecture & Data Model

### `PaymentProvider` Interface (Rust Trait)
```rust
pub trait PaymentProvider {
    fn create_subscription(&self, tenant_id: Uuid, plan_id: String) -> Result<Subscription, Error>;
    fn capture_payment(&self, invoice_id: Uuid, amount: u64, currency: Currency) -> Result<Transaction, Error>;
    fn setup_tenant_payout_route(&self, tenant_id: Uuid) -> Result<PayoutRoute, Error>;
}
```

### Supported Implementations
1. **`StripeProvider`**: Uses Stripe REST APIs for CC, SEPA, Apple Pay.
2. **`BTCPayProvider` (or custom lightning node)**: Generates invoices and listens for on-chain / lightning confirmations via Webhook.
3. **`USDTProvider`**: Smart contract listener for ERC20/TRC20 payments.

### New Database Entities
- `BillingPlans`: Platform-wide plans (e.g., "Network Starter", "Enterprise Anchor").
- `TenantSubscriptions`: Links `tenant_id` to a `BillingPlan`, tracks `status` (Active, Past_Due, Canceled).
- `Transactions`: A unified ledger of all payments. Columns: `id`, `tenant_id`, `provider` (Enum), `amount`, `currency`, `provider_tx_id`, `status`.

## 4. Platform Admin UI UX
1. **Financial Dashboard**: Global MRR, Crypto vs Fiat visualization, Churn metrics.
2. **Tenant Ledger**: Viewing a specific network/anchor's billing history.
3. **Gateway Toggles**: A UI to Enable/Disable Stripe or BTC payments system-wide.

## 5. Security & Compliance
- Ensure no PCI data touches Atlas servers for fiat.
- Require multi-sig wallets for USDT/Crypto treasury.
- Enforce strict state-machines on Webhook ingestion to prevent replay attacks or false-positive activations.
