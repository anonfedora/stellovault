import { getLocaleConfig } from './config';

export function formatNumber(
  value: number,
  locale: string,
  options?: Intl.NumberFormatOptions
): string {
  const config = getLocaleConfig(locale);
  return value.toLocaleString(config.locale, options);
}

export function formatDecimal(
  value: number,
  locale: string,
  decimals: number = 2
): string {
  const config = getLocaleConfig(locale);
  return value.toLocaleString(config.locale, {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}

export function formatPercentage(
  value: number,
  locale: string,
  decimals: number = 2
): string {
  const config = getLocaleConfig(locale);
  return (value / 100).toLocaleString(config.locale, {
    style: 'percent',
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}

export function formatScientific(
  value: number,
  locale: string
): string {
  const config = getLocaleConfig(locale);
  return value.toLocaleString(config.locale, {
    notation: 'scientific',
    maximumFractionDigits: 3,
  });
}

export function formatCompact(
  value: number,
  locale: string
): string {
  const config = getLocaleConfig(locale);
  return value.toLocaleString(config.locale, {
    notation: 'compact',
    compactDisplay: 'short',
  });
}