import { NextRequest, NextResponse } from 'next/server';
import { clearAuthCookies } from '@/lib/auth';

const NO_STORE = { 'Cache-Control': 'no-store' };

export async function POST(request: NextRequest) {
    // CSRF guard: verify the Origin header matches the expected host.
    // Cookies are also SameSite=Strict which blocks cross-site requests natively.
    const origin = request.headers.get('origin');
    const host = request.headers.get('host');
    if (origin && host) {
        try {
            const originHost = new URL(origin).host;
            if (originHost !== host) {
                return NextResponse.json(
                    { error: 'Forbidden' },
                    { status: 403, headers: NO_STORE }
                );
            }
        } catch {
            return NextResponse.json(
                { error: 'Forbidden' },
                { status: 403, headers: NO_STORE }
            );
        }
    }

    try {
        await clearAuthCookies();
        return NextResponse.json({ success: true }, { headers: NO_STORE });
    } catch (error) {
        // Structured log — swap for your project logger when available
        console.error('[auth/logout] Failed to clear auth cookies', { error });
        return NextResponse.json(
            { error: 'Internal Server Error' },
            { status: 500, headers: NO_STORE }
        );
    }
}
