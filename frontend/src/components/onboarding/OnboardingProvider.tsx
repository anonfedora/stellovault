"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ACTIONS,
  EVENTS,
  Joyride,
  STATUS,
  type EventData,
  type Step,
} from "react-joyride";
import { usePathname, useRouter } from "next/navigation";
import Image from "next/image";
import {
  getTransactionCount,
  isOnboardingDismissed,
  resetOnboarding,
  setOnboardingCompleted,
  setOnboardingDismissed,
} from "@/utils/onboarding";
import { OnboardingContext, type OnboardingContextValue } from "./useOnboarding";

type OnboardingStep = Step & { route: string };

const ROUTES = {
  dashboard: "/dashboard",
  collateral: "/dashboard/collateral",
  escrows: "/dashboard/escrows",
  loans: "/loans",
} as const;

export function OnboardingProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const router = useRouter();
  const pathname = usePathname();
  const autoStartedRef = useRef(false);

  const steps = useMemo<OnboardingStep[]>(
    () => [
      {
        route: ROUTES.dashboard,
        target: "#sv-onboarding-connect-wallet",
        placement: "bottom",
        content: (
          <div className="space-y-3">
            <div>
              <p className="font-semibold text-gray-900">Connect your wallet</p>
              <p className="text-sm text-gray-600 mt-1">
                StelloVault uses your Stellar wallet to sign actions and interact
                with Soroban contracts.
              </p>
            </div>
            <Image
              src="/tutorials/connect-wallet.svg"
              alt="Connect wallet tutorial"
              width={640}
              height={360}
              className="w-full h-auto rounded-lg border border-gray-200"
              priority
            />
          </div>
        ),
      },
      {
        route: ROUTES.collateral,
        target: "#sv-onboarding-upload-collateral",
        placement: "bottom",
        content: (
          <div className="space-y-3">
            <div>
              <p className="font-semibold text-gray-900">Upload collateral</p>
              <p className="text-sm text-gray-600 mt-1">
                Tokenize invoices, receivables, or inventory to unlock trade
                financing.
              </p>
            </div>
            <Image
              src="/tutorials/upload-collateral.svg"
              alt="Upload collateral tutorial"
              width={640}
              height={360}
              className="w-full h-auto rounded-lg border border-gray-200"
            />
          </div>
        ),
      },
      {
        route: ROUTES.escrows,
        target: "#sv-onboarding-create-escrow",
        placement: "bottom",
        content: (
          <div className="space-y-3">
            <div>
              <p className="font-semibold text-gray-900">Create an escrow</p>
              <p className="text-sm text-gray-600 mt-1">
                Escrows protect both buyer and seller by releasing funds only
                when conditions are met.
              </p>
            </div>
            <Image
              src="/tutorials/create-escrow.svg"
              alt="Create escrow tutorial"
              width={640}
              height={360}
              className="w-full h-auto rounded-lg border border-gray-200"
            />
          </div>
        ),
      },
      {
        route: ROUTES.loans,
        target: "#sv-onboarding-monitor-loan",
        placement: "bottom",
        content: (
          <div className="space-y-3">
            <div>
              <p className="font-semibold text-gray-900">Monitor your loan</p>
              <p className="text-sm text-gray-600 mt-1">
                Track status, repayments, and maturity in real time.
              </p>
            </div>
            <Image
              src="/tutorials/monitor-loan.svg"
              alt="Monitor loan tutorial"
              width={640}
              height={360}
              className="w-full h-auto rounded-lg border border-gray-200"
            />
          </div>
        ),
      },
    ],
    [],
  );

  const [run, setRun] = useState(false);
  const [stepIndex, setStepIndex] = useState(0);
  const [pendingStepIndex, setPendingStepIndex] = useState<number | null>(null);

  const goToStep = useCallback(
    (nextIndex: number) => {
      if (nextIndex < 0 || nextIndex >= steps.length) return false;
      const nextRoute = steps[nextIndex]?.route;
      if (nextRoute && nextRoute !== pathname) {
        setRun(false);
        setPendingStepIndex(nextIndex);
        router.push(nextRoute);
        return false;
      }
      setStepIndex(nextIndex);
      return true;
    },
    [pathname, router, steps],
  );

  const stopTour = useCallback(() => {
    setRun(false);
    setPendingStepIndex(null);
    setStepIndex(0);
  }, []);

  const startTour = useCallback(() => {
    const isReady = goToStep(0);
    if (isReady) setRun(true);
  }, [goToStep]);

  const restartTour = useCallback(() => {
    resetOnboarding();
    const isReady = goToStep(0);
    if (isReady) setRun(true);
  }, [goToStep]);

  const handleJoyrideEvent = useCallback(
    (data: EventData) => {
      const { action, index, status, type } = data;

      if (status === STATUS.FINISHED) {
        setOnboardingDismissed(true);
        setOnboardingCompleted(true);
        stopTour();
        return;
      }

      if (status === STATUS.SKIPPED) {
        setOnboardingDismissed(true);
        stopTour();
        return;
      }

      if (type === EVENTS.TARGET_NOT_FOUND) {
        goToStep(index + 1);
        return;
      }

      if (type !== EVENTS.STEP_AFTER) return;

      if (action === ACTIONS.NEXT) {
        goToStep(index + 1);
      } else if (action === ACTIONS.PREV) {
        goToStep(index - 1);
      }
    },
    [goToStep, stopTour],
  );

  useEffect(() => {
    if (pendingStepIndex == null) return;
    const pendingRoute = steps[pendingStepIndex]?.route;
    if (pendingRoute !== pathname) return;

    const id = window.setTimeout(() => {
      setStepIndex(pendingStepIndex);
      setPendingStepIndex(null);
      setRun(true);
    }, 60);

    return () => window.clearTimeout(id);
  }, [pathname, pendingStepIndex, steps]);

  useEffect(() => {
    if (autoStartedRef.current) return;
    if (pathname !== ROUTES.dashboard) return;

    const txCount = getTransactionCount();
    if (txCount !== 0) return;
    if (isOnboardingDismissed()) return;

    autoStartedRef.current = true;
    const id = window.setTimeout(() => {
      startTour();
    }, 0);
    return () => window.clearTimeout(id);
  }, [pathname, startTour]);

  const value: OnboardingContextValue = useMemo(
    () => ({
      isRunning: run,
      startTour,
      restartTour,
      stopTour,
    }),
    [restartTour, run, startTour, stopTour],
  );

  const showLauncher =
    pathname.startsWith("/dashboard") || pathname.startsWith("/loans");

  return (
    <OnboardingContext.Provider value={value}>
      {children}

      {showLauncher && (
        <button
          type="button"
          onClick={restartTour}
          className="fixed bottom-6 right-6 z-[9998] rounded-full bg-blue-900 text-white px-4 py-2 text-sm font-medium shadow-lg hover:bg-blue-800 transition-colors"
        >
          Restart Tour
        </button>
      )}

      <Joyride
        steps={steps}
        run={run}
        stepIndex={stepIndex}
        continuous
        onEvent={handleJoyrideEvent}
        options={{
          zIndex: 9999,
          primaryColor: "#1E3A8A",
          textColor: "#111827",
          arrowColor: "#ffffff",
          backgroundColor: "#ffffff",
          overlayColor: "rgba(17, 24, 39, 0.55)",
          overlayClickAction: false,
          showProgress: true,
          closeButtonAction: "skip",
          skipBeacon: true,
          buttons: ["back", "close", "skip", "primary"],
        }}
      />
    </OnboardingContext.Provider>
  );
}
