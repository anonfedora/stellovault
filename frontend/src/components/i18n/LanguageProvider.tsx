'use client';

import { createContext, useContext, useEffect, ReactNode } from 'react';
import { useLocaleStore } from '@/utils/i18n/store';
import { routing } from '@/i18n/routing';

interface LanguageContextType {
  locale: string;
  direction: 'ltr' | 'rtl';
  setLocale: (locale: string) => void;
  t: (key: string) => string;
  isLoading: boolean;
}

const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

const rtlLocales = ['ar', 'he'];

export function LanguageProvider({ 
  children, 
  initialLocale 
}: { 
  children: ReactNode; 
  initialLocale?: string;
}) {
  const { locale, direction, translations, isLoading, setLocale: setStoreLocale, setTranslations, setLoading } = useLocaleStore();

  useEffect(() => {
    const initLocale = async () => {
      let savedLocale = initialLocale || 'en';
      
      if (typeof window !== 'undefined') {
        savedLocale = localStorage.getItem('locale') || '';
        if (!savedLocale || !routing.locales.includes(savedLocale as typeof routing.locales[number])) {
          savedLocale = 'en';
        }
      }
      
      if (!routing.locales.includes(savedLocale as typeof routing.locales[number])) {
        savedLocale = routing.defaultLocale;
      }

      setStoreLocale(savedLocale);
      document.documentElement.lang = savedLocale;
      document.documentElement.dir = rtlLocales.includes(savedLocale) ? 'rtl' : 'ltr';

      try {
        const msgs = await import(`@/i18n/locales/${savedLocale}.json`);
        setTranslations(msgs.default);
      } catch {
        const msgs = await import('@/i18n/locales/en.json');
        setTranslations(msgs.default);
      }
      setLoading(false);
    };

    if (!locale || locale === 'en') {
      initLocale();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const setLocale = (newLocale: string) => {
    if (!routing.locales.includes(newLocale as typeof routing.locales[number])) {
      return;
    }
    
    setStoreLocale(newLocale);
    document.documentElement.lang = newLocale;
    document.documentElement.dir = rtlLocales.includes(newLocale) ? 'rtl' : 'ltr';
    
    if (typeof window !== 'undefined') {
      localStorage.setItem('locale', newLocale);
      document.cookie = `locale=${newLocale};path=/;max-age=31536000`;
    }
    
    import(`@/i18n/locales/${newLocale}.json`).then((msgs) => {
      setTranslations(msgs.default);
    }).catch(() => {});
  };

  const t = (key: string): string => {
    const keys = key.split('.');
    let value: unknown = translations;
    for (const k of keys) {
      if (value && typeof value === 'object') {
        value = (value as Record<string, unknown>)[k];
      } else {
        value = undefined;
      }
    }
    return (value as string) || key;
  };

  return (
    <LanguageContext.Provider value={{ 
      locale: locale || initialLocale || 'en', 
      direction: direction || (rtlLocales.includes(locale || initialLocale || 'en') ? 'rtl' : 'ltr'),
      setLocale,
      t,
      isLoading
    }}>
      {children}
    </LanguageContext.Provider>
  );
}

export function useLanguage() {
  const context = useContext(LanguageContext);
  if (!context) {
    throw new Error('useLanguage must be used within LanguageProvider');
  }
  return context;
}

export function useTranslation() {
  const context = useContext(LanguageContext);
  if (!context) {
    throw new Error('useTranslation must be used within LanguageProvider');
  }
  return { t: context.t, locale: context.locale };
}