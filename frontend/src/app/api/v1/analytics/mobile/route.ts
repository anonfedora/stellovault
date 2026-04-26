import { NextRequest, NextResponse } from "next/server";

export async function POST(request: NextRequest) {
  const payload = await request.json().catch(() => ({}));

  console.info("mobile_runtime_signal", {
    ...payload,
    ip: request.headers.get("x-forwarded-for"),
  });

  return NextResponse.json({ ok: true });
}
