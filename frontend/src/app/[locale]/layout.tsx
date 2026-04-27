import { NextIntlClientProvider } from 'next-intl';
import { LanguageProvider } from '@/components/i18n/LanguageProvider';
import { rtlLocales } from '@/utils/i18n/config';
import type { ReactNode } from 'react';

export function generateStaticParams() {
  return [
    { locale: 'en' },
    { locale: 'es' },
    { locale: 'fr' },
    { locale: 'ar' },
    { locale: 'he' },
    { locale: 'zh' },
    { locale: 'ja' },
  ];
}

export default async function LocaleLayout({
  children,
  params,
}: {
  children: ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const messages = await import(`@/i18n/locales/${locale}.json`);
  const direction = rtlLocales.includes(locale) ? 'rtl' : 'ltr';

  return (
    <html lang={locale} dir={direction} suppressHydrationWarning>
      <head>
        <script
          dangerouslySetInnerHTML={{
            __html: `
              try {
                var locale = '${locale}';
                var rtlLocales = ['ar', 'he'];
                document.documentElement.lang = locale;
                document.documentElement.dir = rtlLocales.indexOf(locale) >= 0 ? 'rtl' : 'ltr';
              } catch (e) {}
            `,
          }}
        />
      </head>
      <body>
        <NextIntlClientProvider locale={locale} messages={messages.default}>
          <LanguageProvider initialLocale={locale}>
            {children}
          </LanguageProvider>
        </NextIntlClientProvider>
      </body>
    </html>
  );
}