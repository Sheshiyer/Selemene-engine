// Note: portal session creation lives on the Rust API at
// `POST /api/v1/billing/portal`. The biofield-web client calls it directly
// via `createPortalSession(token)` from src/lib/api.ts.

export async function POST(): Promise<Response> {
  return Response.json(
    {
      error: "use Rust API",
      detail:
        "portal session creation is at POST /api/v1/billing/portal on noesis-api",
    },
    { status: 410 },
  );
}
