export const rtlLocales = ['ar', 'he'];

export const localeConfig: Record<string, { 
  dir: 'ltr' | 'rtl';
  locale: string;
  currencyLocale?: string;
  dateLocale?: string;
}> = {
  en: { dir: 'ltr', locale: 'en-US', currencyLocale: 'en-US', dateLocale: 'en-US' },
  es: { dir: 'ltr', locale: 'es-ES', currencyLocale: 'es-ES', dateLocale: 'es-ES' },
  fr: { dir: 'ltr', locale: 'fr-FR', currencyLocale: 'fr-FR', dateLocale: 'fr-FR' },
  ar: { dir: 'rtl', locale: 'ar-SA', currencyLocale: 'ar-SA', dateLocale: 'ar-SA' },
  he: { dir: 'rtl', locale: 'he-IL', currencyLocale: 'he-IL', dateLocale: 'he-IL' },
  zh: { dir: 'ltr', locale: 'zh-CN', currencyLocale: 'zh-CN', dateLocale: 'zh-CN' },
  ja: { dir: 'ltr', locale: 'ja-JP', currencyLocale: 'ja-JP', dateLocale: 'ja-JP' },
};

export function getDirection(locale: string): 'ltr' | 'rtl' {
  return rtlLocales.includes(locale) ? 'rtl' : 'ltr';
}

export function getLocaleConfig(locale: string) {
  return localeConfig[locale] || localeConfig.en;
}