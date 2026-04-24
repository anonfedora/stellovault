import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import { verifyToken, refreshAccessToken } from './lib/auth';

export async function proxy(request: NextRequest) {
    const { pathname } = request.nextUrl;

    // Check for token
    const accessToken = request.cookies.get('accessToken')?.value;
    const refreshToken = request.cookies.get('refreshToken')?.value;

    let isValid = false;
    const response = NextResponse.next();

    if (accessToken) {
        const payload = await verifyToken(accessToken);
        if (payload) {
            isValid = true;
        }
    }

    // Silent refresh if accessToken is invalid but refreshToken exists
    if (!isValid && refreshToken) {
        const refreshResult = await refreshAccessToken(refreshToken);
        if (refreshResult) {
            isValid = true;
            // Set the new access token in the response cookies
            response.cookies.set('accessToken', refreshResult.accessToken, {
                httpOnly: true,
                secure: process.env.NODE_ENV === 'production',
                sameSite: 'strict',
                maxAge: 3600,
                path: '/',
            });
        }
    }

    // If trying to access protected route (dashboard) and not valid
    if (pathname.startsWith('/dashboard') && !isValid) {
        const loginUrl = new URL('/login', request.url);
        loginUrl.searchParams.set('from', pathname);
        return NextResponse.redirect(loginUrl);
    }

    // If accessing login page while already valid
    if (pathname === '/login' && isValid) {
        return NextResponse.redirect(new URL('/dashboard', request.url));
    }

    return response;
}

export const config = {
    matcher: [
        '/dashboard/:path*',
        '/login',
    ],
};
