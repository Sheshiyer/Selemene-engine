# Dodo Payments Foundation Contract

Date: 2026-03-31
Status: frozen for foundation implementation slice

## Scope

This contract covers only the first Dodo Payments foundation wave:

- environment/config loading
- provider-facing billing emitter scaffolding
- database persistence surfaces for customers and webhook events
- subscription schema expansion for Dodo-ready states and metadata

It does not yet implement:

- checkout session routes
- portal session routes
- webhook ingestion handlers
- entitlement cutover from legacy tier reads
- user-facing billing UI

## Environment Contract

Supported environment variables:

- `DODO_PAYMENTS_API_KEY`
- `DODO_PAYMENTS_WEBHOOK_KEY`
- `DODO_PAYMENTS_ENV`

Backward-compatible aliases currently accepted:

- `DODO_API_KEY`
- `DODO_WEBHOOK_KEY`

Allowed `DODO_PAYMENTS_ENV` values:

- `test`
- `live`

## Data Contract

### `billing_customers`

Purpose:
- maps one internal user to one provider customer record per provider

Key fields:
- `user_id`
- `provider`
- `provider_customer_id`
- `email`
- `mode`
- `metadata`

Uniqueness:
- unique on `(provider, provider_customer_id)`
- unique on `(user_id, provider)`

### `billing_webhook_events`

Purpose:
- stores raw webhook deliveries and their processing state

Key fields:
- `provider`
- `provider_event_id`
- `event_type`
- `payload`
- `signature`
- `processing_state`
- `received_at`
- `processed_at`
- `error_message`

Allowed processing states:
- `pending`
- `processed`
- `ignored`
- `failed`

Uniqueness:
- unique on `(provider, provider_event_id)`

### `billing_subscriptions`

Foundation additions:
- `provider_product_id`
- `provider_price_id`
- `metadata`

Expanded status vocabulary:
- `trialing`
- `active`
- `past_due`
- `on_hold`
- `canceled`
- `expired`
- `incomplete`
- `failed`

## Runtime Contract

The runtime now recognizes Dodo as a first-class billing provider scaffold through:

- config loading in `ApiConfig`
- `DodoWebhookEmitter` for structured billing event formatting

This does not mean Dodo is fully wired into checkout or webhook routes yet.

## Next Lock Zones

The next implementation wave will touch:

- `crates/noesis-api/src/lib.rs`
- `crates/noesis-api/src/handlers/*`
- `crates/noesis-data/src/repositories/*`
- route registration and request/response contracts

Those changes should be treated as a separate wave from this foundation slice.
