'use client';

import { useReportWebVitals } from 'next/web-vitals';

export function PerformanceMonitor() {
  useReportWebVitals((metric) => {
    // In a real app, you'd send this to an analytics endpoint
    console.log(`[Performance Metric] ${metric.name}:`, metric.value);
    
    // Performance Budget Check
    const budgets: Record<string, number> = {
      FCP: 1500, // First Contentful Paint < 1.5s
      LCP: 2500, // Largest Contentful Paint < 2.5s
      CLS: 0.1,  // Cumulative Layout Shift < 0.1
      FID: 100,  // First Input Delay < 100ms
      TTFB: 600, // Time to First Byte < 600ms
    };

    if (budgets[metric.name] && metric.value > budgets[metric.name]) {
      console.warn(`[Performance Warning] ${metric.name} exceeded budget: ${metric.value.toFixed(2)}ms (Budget: ${budgets[metric.name]}ms)`);
    }
  });

  return null;
}
