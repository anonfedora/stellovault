import {defineRouting} from 'next-intl/routing';

export const routing = defineRouting({
  locales: ['en', 'es', 'fr', 'ar', 'he', 'zh', 'ja'],
  defaultLocale: 'en'
});