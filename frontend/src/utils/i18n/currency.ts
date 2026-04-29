import { getLocaleConfig } from './config';

export interface CurrencyConfig {
  code: string;
  symbol: string;
  name: string;
  decimals: number;
}

export const currencyConfigs: Record<string, CurrencyConfig> = {
  USD: { code: 'USD', symbol: '$', name: 'US Dollar', decimals: 2 },
  EUR: { code: 'EUR', symbol: '€', name: 'Euro', decimals: 2 },
  GBP: { code: 'GBP', symbol: '£', name: 'British Pound', decimals: 2 },
  JPY: { code: 'JPY', symbol: '¥', name: 'Japanese Yen', decimals: 0 },
  CNY: { code: 'CNY', symbol: '¥', name: 'Chinese Yuan', decimals: 2 },
  SAR: { code: 'SAR', symbol: 'ر.س', name: 'Saudi Riyal', decimals: 2 },
  ILS: { code: 'ILS', symbol: '₪', name: 'Israeli Shekel', decimals: 2 },
  XLM: { code: 'XLM', symbol: 'XLM', name: 'Stellar Lumens', decimals: 7 },
};

const exchangeRates: Record<string, number> = {
  USD: 1,
  EUR: 0.92,
  GBP: 0.79,
  JPY: 149.5,
  CNY: 7.24,
  SAR: 3.75,
  ILS: 3.68,
};

export function formatCurrency(
  value: number,
  currency: string = 'USD',
  locale: string,
  options?: Intl.NumberFormatOptions
): string {
  const config = getLocaleConfig(locale);
  const currencyConfig = currencyConfigs[currency] || currencyConfigs.USD;
  
  return value.toLocaleString(config.locale, {
    style: 'currency',
    currency: currency,
    minimumFractionDigits: currencyConfig.decimals,
    maximumFractionDigits: currencyConfig.decimals,
    ...options,
  });
}

export function formatCurrencySymbol(
  value: number,
  currency: string = 'USD'
): string {
  const currencyConfig = currencyConfigs[currency] || currencyConfigs.USD;
  return `${currencyConfig.symbol}${value.toLocaleString(undefined, {
    minimumFractionDigits: currencyConfig.decimals,
    maximumFractionDigits: currencyConfig.decimals,
  })}`;
}

export function convertCurrency(
  value: number,
  fromCurrency: string,
  toCurrency: string
): number {
  const fromRate = exchangeRates[fromCurrency] || 1;
  const toRate = exchangeRates[toCurrency] || 1;
  const usdValue = value / fromRate;
  return usdValue * toRate;
}

export function getExchangeRate(fromCurrency: string, toCurrency: string): number {
  const fromRate = exchangeRates[fromCurrency] || 1;
  const toRate = exchangeRates[toCurrency] || 1;
  return toRate / fromRate;
}

export function formatCrypto(
  value: number,
  symbol: string = 'XLM',
  decimals: number = 7
): string {
  return `${value.toFixed(decimals)} ${symbol}`;
}