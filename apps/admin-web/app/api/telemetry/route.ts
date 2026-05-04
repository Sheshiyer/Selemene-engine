import type { NextRequest } from "next/server";
import { NextResponse } from "next/server";

export const runtime = "edge";

interface TelemetryEvent {
  name: string;
  props?: Record<string, unknown>;
  ts?: number;
}

export async function POST(request: NextRequest): Promise<NextResponse> {
  try {
    const event = (await request.json()) as TelemetryEvent;
    // Structured log to stdout — Railway captures these as structured events.
    console.log(
      JSON.stringify({
        source: "admin-telemetry",
        name: event.name,
        props: event.props ?? {},
        ts: event.ts ?? Date.now(),
        ip: request.headers.get("x-forwarded-for") ?? "unknown"
      })
    );
    return NextResponse.json({ ok: true });
  } catch {
    return NextResponse.json({ ok: false }, { status: 400 });
  }
}
