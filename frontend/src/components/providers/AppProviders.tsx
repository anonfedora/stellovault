"use client";

import { OnboardingProvider } from "@/components/onboarding/OnboardingProvider";

export function AppProviders({ children }: { children: React.ReactNode }) {
  return <OnboardingProvider>{children}</OnboardingProvider>;
}

