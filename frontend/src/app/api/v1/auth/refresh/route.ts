import { NextRequest, NextResponse } from 'next/server';
import { refreshAccessToken, setAuthCookies } from '@/lib/auth';

const NO_STORE = { 'Cache-Control': 'no-store' };

export async function POST(request: NextRequest) {
  const refreshToken = request.cookies.get('refreshToken')?.value;

  if (!refreshToken) {
    return NextResponse.json({ error: 'No refresh token' }, { status: 401, headers: NO_STORE });
  }

  const result = await refreshAccessToken(refreshToken);
  if (!result) {
    return NextResponse.json({ error: 'Invalid or expired refresh token' }, { status: 401, headers: NO_STORE });
  }

  // Re-issue access token cookie
  await setAuthCookies(result.accessToken, refreshToken);
  return NextResponse.json({ success: true }, { headers: NO_STORE });
}
