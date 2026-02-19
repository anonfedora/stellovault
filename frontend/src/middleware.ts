import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import { verifyToken } from './lib/auth';

export async function middleware(request: NextRequest) {
    const { pathname } = request.nextUrl;

    // Public paths that don't need auth (besides login, which has special handling)
    const isPublicPath = pathname === '/login' || pathname === '/' || pathname.startsWith('/api/');

    // Check for token
    const token = request.cookies.get('accessToken')?.value;
    let isValid = false;

    if (token) {
        const payload = await verifyToken(token);
        if (payload) {
            isValid = true;
        }
    }

    // If trying to access protected route (dashboard) and not valid
    if (pathname.startsWith('/dashboard') && !isValid) {
        return NextResponse.redirect(new URL('/login', request.url));
    }

    // If accessing login page while already valid
    if (pathname === '/login' && isValid) {
        return NextResponse.redirect(new URL('/dashboard', request.url));
    }

    return NextResponse.next();
}

export const config = {
    matcher: [
        '/dashboard/:path*',
        '/login',
    ],
};
