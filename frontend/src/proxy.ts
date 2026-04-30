import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'
import { verifyToken, refreshAccessToken } from './lib/auth'

const locales = ['en', 'es', 'fr', 'ar', 'he', 'zh', 'ja']
const defaultLocale = 'en'

const rtlLocales = ['ar', 'he']

function getLocaleFromHeader(request: NextRequest): string {
  const acceptLanguage = request.headers.get('Accept-Language')
  if (!acceptLanguage) return defaultLocale

  const preferredLocales = acceptLanguage
    .split(',')
    .map(lang => lang.split(';')[0].trim().substring(0, 2))

  for (const pref of preferredLocales) {
    const matched = locales.find(l => l.startsWith(pref))
    if (matched) return matched
  }

  return defaultLocale
}

function getLocaleFromCookie(request: NextRequest): string | null {
  const cookie = request.cookies.get('locale')
  return cookie?.value || null
}

export default async function proxy(request: NextRequest) {
  const { pathname } = request.nextUrl

  // --- 1. Authentication Logic ---
  const accessToken = request.cookies.get('accessToken')?.value;
  const refreshToken = request.cookies.get('refreshToken')?.value;

  let isValid = false;
  let newAccessToken: string | null = null;

  if (accessToken) {
      const payload = await verifyToken(accessToken);
      if (payload) {
          isValid = true;
      }
  }

  if (!isValid && refreshToken) {
      const refreshResult = await refreshAccessToken(refreshToken);
      if (refreshResult) {
          isValid = true;
          newAccessToken = refreshResult.accessToken;
      }
  }

  // Determine if it's a dashboard or login route (ignoring locale prefix if present)
  // e.g., /en/dashboard, /dashboard, /es/login, /login
  const isDashboardRoute = /^\/([^\/]+\/)?dashboard(\/.*)?$/.test(pathname);
  const isLoginRoute = /^\/([^\/]+\/)?login(\/.*)?$/.test(pathname);

  // If redirecting, preserve locale if possible
  const currentLocale = locales.find(locale => pathname.startsWith(`/${locale}/`) || pathname === `/${locale}`) || defaultLocale;

  if (isDashboardRoute && !isValid) {
      const loginUrl = new URL(`/${currentLocale}/login`, request.url);
      loginUrl.searchParams.set('from', pathname);
      return NextResponse.redirect(loginUrl);
  }

  if (isLoginRoute && isValid) {
      return NextResponse.redirect(new URL(`/${currentLocale}/dashboard`, request.url));
  }

  // --- 2. Internationalization Logic ---
  const pathnameHasLocale = locales.some(
    locale => pathname.startsWith(`/${locale}/`) || pathname === `/${locale}`
  )

  let response: NextResponse;

  if (pathnameHasLocale) {
    const locale = pathname.split('/')[1] || defaultLocale
    const newPathname = pathname.replace(`/${locale}`, '') || '/'
    
    // Create a new URL for the rewrite
    const rewriteUrl = new URL(newPathname, request.url)
    request.nextUrl.searchParams.forEach((value, key) => {
        rewriteUrl.searchParams.append(key, value)
    })
    
    response = NextResponse.rewrite(rewriteUrl)
    response.headers.set('x-locale', locale)
    response.headers.set('x-dir', rtlLocales.includes(locale) ? 'rtl' : 'ltr')
  } else if (
    pathname.startsWith('/_next') ||
    pathname.startsWith('/api') ||
    pathname.includes('.') ||
    pathname === '/favicon.ico'
  ) {
    response = NextResponse.next()
  } else {
    let locale = getLocaleFromCookie(request) || getLocaleFromHeader(request)

    if (!locales.includes(locale)) {
      locale = defaultLocale
    }

    const redirectUrl = new URL(`/${locale}${pathname}`, request.url)
    request.nextUrl.searchParams.forEach((value, key) => {
        redirectUrl.searchParams.append(key, value);
    });

    response = NextResponse.redirect(redirectUrl)
    response.headers.set('x-locale', locale)
    response.headers.set('x-dir', rtlLocales.includes(locale) ? 'rtl' : 'ltr')
  }

  // --- 3. Set refreshed token cookie if necessary ---
  if (newAccessToken) {
    response.cookies.set('accessToken', newAccessToken, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'strict',
        maxAge: 3600,
        path: '/',
    });
  }

  return response
}

export const config = {
  matcher: [
    '/((?!api|_next/static|_next/image|favicon.ico|.*\\..*).*)',
  ],
}