import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'

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

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl

  const pathnameHasLocale = locales.some(
    locale => pathname.startsWith(`/${locale}/`) || pathname === `/${locale}`
  )

  if (pathnameHasLocale) {
    const locale = pathname.split('/')[1] || defaultLocale
    const response = NextResponse.next()
    response.headers.set('x-locale', locale)
    response.headers.set('x-dir', rtlLocales.includes(locale) ? 'rtl' : 'ltr')
    return response
  }

  if (
    pathname.startsWith('/_next') ||
    pathname.startsWith('/api') ||
    pathname.includes('.') ||
    pathname === '/favicon.ico'
  ) {
    return NextResponse.next()
  }

  let locale = getLocaleFromCookie(request) || getLocaleFromHeader(request)

  if (!locales.includes(locale)) {
    locale = defaultLocale
  }

  const response = NextResponse.redirect(
    new URL(`/${locale}${pathname}`, request.url)
  )

  response.headers.set('x-locale', locale)
  response.headers.set('x-dir', rtlLocales.includes(locale) ? 'rtl' : 'ltr')

  return response
}

export const config = {
  matcher: [
    '/((?!api|_next/static|_next/image|favicon.ico|.*\\..*).*)',
  ],
}