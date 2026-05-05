// Wave 1.3 stub. T19 calls Rust → client.customers.createCustomerPortalSession()
// and returns the portal URL. Contract: .context/billing/contracts.md § API.

export async function POST(): Promise<Response> {
  return Response.json(
    { status: "stub", note: "portal lands in T19" },
    { status: 200 },
  );
}
