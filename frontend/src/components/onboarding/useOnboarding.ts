"use client";

import { createContext, useContext } from "react";

export interface OnboardingContextValue {
  isRunning: boolean;
  startTour: () => void;
  restartTour: () => void;
  stopTour: () => void;
}

export const OnboardingContext = createContext<OnboardingContextValue | null>(
  null,
);

export function useOnboarding(): OnboardingContextValue {
  const ctx = useContext(OnboardingContext);
  if (!ctx) {
    throw new Error("useOnboarding must be used within <OnboardingProvider />");
  }
  return ctx;
}

