import {clsx} from "clsx";
import {create} from "zustand";
import {persist} from "zustand/middleware";

interface LocaleStore {
  locale: string;
  direction: 'ltr' | 'rtl';
  translations: Record<string, any>;
  isLoading: boolean;
  setLocale: (locale: string) => void;
  setTranslations: (translations: Record<string, any>) => void;
  setLoading: (loading: boolean) => void;
}

const rtlLocales = ['ar', 'he'];

export const useLocaleStore = create<LocaleStore>()(
  persist(
    (set) => ({
      locale: 'en',
      direction: 'ltr',
      translations: {},
      isLoading: true,
      setLocale: (locale) => set({ 
        locale, 
        direction: rtlLocales.includes(locale) ? 'rtl' : 'ltr'
      }),
      setTranslations: (translations) => set({ translations }),
      setLoading: (isLoading) => set({ isLoading }),
    }),
    {
      name: 'locale-storage',
      partialize: (state) => ({ locale: state.locale }),
    }
  )
);

export function useTranslation() {
  const { locale, translations, isLoading } = useLocaleStore();

  const t = (key: string): string => {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  };

  return { t, locale, isLoading };
}