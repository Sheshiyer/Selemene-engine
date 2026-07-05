// Inbound Dodo Payments webhook ingress.
//
// Pipeline:
//   1. Standard Webhooks signature verification via @dodopayments/core
//   2. Forward the verified raw body + canonical webhook-id to the Rust
//      noesis-api endpoint POST /internal/billing/events
//   3. Rust handles idempotency, persistence, and tier mirror updates
//
// Contract: .context/billing/contracts.md § API.
// Runtime: Node (Standard Webhooks library uses Node crypto).

import { verifyWebhookPayload } from "@dodopayments/core/webhook";
import type { DodoInboundEventType } from "@noesis/sdk";

export const runtime = "nodejs";
export const dynamic = "force-dynamic"; // never cache webhook handlers

const NOESIS_API_URL = process.env.NOESIS_API_URL ?? "http://localhost:8080";

export async function POST(request: Request): Promise<Response> {
  const webhookKey = process.env.DODO_PAYMENTS_WEBHOOK_KEY;
  const forwardSecret = process.env.DODO_INTERNAL_FORWARD_SECRET;

  if (!webhookKey || !forwardSecret) {
    console.error(
      "[dodo-webhook] missing DODO_PAYMENTS_WEBHOOK_KEY or DODO_INTERNAL_FORWARD_SECRET",
    );
    return new Response("misconfigured", { status: 500 });
  }

  const webhookId = request.headers.get("webhook-id") ?? "";
  const webhookSignature = request.headers.get("webhook-signature") ?? "";
  const webhookTimestamp = request.headers.get("webhook-timestamp") ?? "";

  if (!webhookId || !webhookSignature || !webhookTimestamp) {
    return new Response("missing standard-webhooks headers", { status: 400 });
  }

  const rawBody = await request.text();

  try {
    await verifyWebhookPayload({
      webhookKey,
      headers: {
        "webhook-id": webhookId,
        "webhook-signature": webhookSignature,
        "webhook-timestamp": webhookTimestamp,
      },
      body: rawBody,
    });
  } catch (err) {
    console.warn("[dodo-webhook] signature verification failed", err);
    return new Response("invalid signature", { status: 401 });
  }

  let dodoEvent: { type?: string };
  try {
    dodoEvent = JSON.parse(rawBody) as { type?: string };
  } catch {
    return new Response("invalid JSON body", { status: 400 });
  }

  const eventType = dodoEvent.type;
  if (!eventType) {
    return new Response("missing event type", { status: 400 });
  }

  const forwardResp = await fetch(`${NOESIS_API_URL}/internal/billing/events`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Forward-Secret": forwardSecret,
    },
    body: JSON.stringify({
      webhook_id: webhookId,
      webhook_timestamp: webhookTimestamp,
      event_type: eventType as DodoInboundEventType,
      payload: dodoEvent,
    }),
  });

  if (!forwardResp.ok) {
    const detail = await forwardResp.text().catch(() => "");
    console.error(
      `[dodo-webhook] Rust forward ${forwardResp.status}: ${detail.slice(0, 200)}`,
    );
    // 502 tells Dodo to retry; 4xx from Rust (bad payload shape) shouldn't
    // be retried, so collapse those to 200 with a warning to avoid retry storms.
    if (forwardResp.status >= 400 && forwardResp.status < 500) {
      return Response.json(
        { status: "forwarded_4xx", upstream: forwardResp.status },
        { status: 200 },
      );
    }
    return new Response("forward failed", { status: 502 });
  }

  return Response.json({ status: "ok" }, { status: 200 });
}
