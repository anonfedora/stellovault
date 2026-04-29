'use client';

import { useLanguage } from './LanguageProvider';
import { useTransition } from 'react';
import { Globe, ChevronDown } from 'lucide-react';
import { clsx } from 'clsx';

export const localeNames: Record<string, string> = {
  en: 'English',
  es: 'Español',
  fr: 'Français',
  ar: 'العربية',
  he: 'עברית',
  zh: '中文',
  ja: '日本語',
};

export const localeFlags: Record<string, string> = {
  en: '🇺🇸',
  es: '🇪🇸',
  fr: '🇫🇷',
  ar: '🇸🇦',
  he: '🇮🇱',
  zh: '🇨🇳',
  ja: '🇯🇵',
};

interface LanguageSwitcherProps {
  className?: string;
}

export function LanguageSwitcher({ className }: LanguageSwitcherProps) {
  const { locale, setLocale, isLoading } = useLanguage();
  const [, startTransition] = useTransition();

  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const newLocale = e.target.value;
    startTransition(() => {
      setLocale(newLocale);
    });
  };

  if (isLoading) {
    return (
      <div className={clsx("flex items-center gap-2", className)}>
        <Globe className="w-4 h-4" />
        <div className="w-20 h-8 bg-muted rounded animate-pulse" />
      </div>
    );
  }

  return (
    <div className={clsx("relative flex items-center", className)}>
      <Globe className="w-4 h-4 text-muted-foreground absolute left-3 pointer-events-none" />
      <select
        value={locale}
        onChange={handleChange}
        className="appearance-none bg-background border border-border rounded-md pl-9 pr-8 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-primary cursor-pointer"
        aria-label="Select language"
      >
        {Object.keys(localeNames).map((loc) => (
          <option key={loc} value={loc}>
            {localeFlags[loc]} {localeNames[loc]}
          </option>
        ))}
      </select>
      <ChevronDown className="absolute right-2 w-4 h-4 text-muted-foreground pointer-events-none" />
    </div>
  );
}

export function LanguageButton({ 
  locale: targetLocale, 
  className 
}: { 
  locale: string; 
  className?: string;
}) {
  const { locale, setLocale } = useLanguage();
  const [, startTransition] = useTransition();

  if (locale === targetLocale) {
    return null;
  }

  const handleClick = () => {
    startTransition(() => {
      setLocale(targetLocale);
    });
  };

  return (
    <button
      onClick={handleClick}
      className={clsx(
        "flex items-center gap-2 px-3 py-2 rounded-md text-sm hover:bg-accent transition-colors",
        className
      )}
      aria-label={`Switch to ${localeNames[targetLocale]}`}
    >
      <span>{localeFlags[targetLocale]}</span>
      <span>{localeNames[targetLocale]}</span>
    </button>
  );
}