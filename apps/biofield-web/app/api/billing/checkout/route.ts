// Note: checkout creation lives on the Rust API at
// `POST /api/v1/billing/checkout`. The biofield-web client calls it directly
// with `Authorization: Bearer <token>` (matching the existing api.ts pattern).
// This Next.js route exists only to redirect anyone who hits it directly
// during development — there is no Next.js-side logic to add.

export async function POST(): Promise<Response> {
  return Response.json(
    {
      error: "use Rust API",
      detail:
        "checkout creation is at POST /api/v1/billing/checkout on noesis-api; this Next.js route is intentionally empty",
    },
    { status: 410 },
  );
}
