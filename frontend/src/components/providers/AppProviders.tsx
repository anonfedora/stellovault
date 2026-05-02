"use client";

import dynamic from 'next/dynamic';

const OnboardingProvider = dynamic(() => import('@/components/onboarding/OnboardingProvider').then(mod => mod.OnboardingProvider), {
  ssr: false
});

export function AppProviders({ children }: { children: React.ReactNode }) {
  return <OnboardingProvider>{children}</OnboardingProvider>;
}

