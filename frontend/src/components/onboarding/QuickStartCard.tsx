"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import {
  CheckCircle2,
  Circle,
  ArrowRight,
  RotateCcw,
  Sparkles,
} from "lucide-react";
import { getQuickStartState, getTransactionCount } from "@/utils/onboarding";
import { useOnboarding } from "./useOnboarding";

const QUICK_START_ITEMS = [
  {
    key: "connectWallet" as const,
    title: "Connect Wallet",
    description: "Sign actions securely with your Stellar wallet.",
    href: "/login",
  },
  {
    key: "uploadCollateral" as const,
    title: "Upload Collateral",
    description: "Tokenize invoices or inventory to back a deal.",
    href: "/dashboard/collateral",
  },
  {
    key: "createEscrow" as const,
    title: "Create Escrow",
    description: "Lock funds with clear release conditions.",
    href: "/dashboard/escrows",
  },
  {
    key: "monitorLoan" as const,
    title: "Monitor Loan",
    description: "Track status, repayments, and maturity.",
    href: "/loans",
  },
] as const;

export function QuickStartCard() {
  const { restartTour } = useOnboarding();
  const [txCount, setTxCount] = useState(0);
  const [state, setState] = useState(() => getQuickStartState());

  useEffect(() => {
    const id = window.setTimeout(() => {
      setTxCount(getTransactionCount());
      setState(getQuickStartState());
    }, 0);
    return () => window.clearTimeout(id);
  }, []);

  const completedCount = useMemo(() => {
    return QUICK_START_ITEMS.reduce((sum, item) => sum + (state[item.key] ? 1 : 0), 0);
  }, [state]);

  const isNew = txCount === 0;
  const isComplete = completedCount === QUICK_START_ITEMS.length;

  if (!isNew && isComplete) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-5 mb-8 flex items-center justify-between gap-4">
        <div>
          <p className="text-sm font-semibold text-gray-900 dark:text-gray-100">
            Need a refresher?
          </p>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
            Restart the guided onboarding walkthrough anytime.
          </p>
        </div>
        <button
          type="button"
          onClick={() => {
            restartTour();
            setTxCount(getTransactionCount());
            setState(getQuickStartState());
          }}
          className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-blue-900 text-white text-sm font-medium hover:bg-blue-800 transition-colors"
        >
          <RotateCcw className="w-4 h-4" />
          Restart Tour
        </button>
      </div>
    );
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 mb-8">
      <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4 mb-5">
        <div>
          <div className="flex items-center gap-2">
            <Sparkles className="w-5 h-5 text-blue-700 dark:text-blue-400" />
            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              Quick Start
            </h2>
          </div>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
            {isNew
              ? "Complete these steps to launch your first deal."
              : "Finish any remaining steps or restart the tour."}
          </p>
        </div>

        <button
          type="button"
          onClick={() => {
            restartTour();
            setTxCount(getTransactionCount());
            setState(getQuickStartState());
          }}
          className="inline-flex items-center justify-center gap-2 px-4 py-2 rounded-lg bg-blue-900 text-white text-sm font-medium hover:bg-blue-800 transition-colors"
        >
          <RotateCcw className="w-4 h-4" />
          Restart Tour
        </button>
      </div>

      <div className="flex items-center justify-between text-sm mb-4">
        <p className="text-gray-600 dark:text-gray-400">
          Progress:{" "}
          <span className="font-semibold text-gray-900 dark:text-gray-100">
            {completedCount}/{QUICK_START_ITEMS.length}
          </span>
        </p>
        {isNew && (
          <span className="text-xs font-medium px-2 py-1 rounded-full bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300">
            New account
          </span>
        )}
      </div>

      <div className="space-y-3">
        {QUICK_START_ITEMS.map((item) => {
          const done = state[item.key];
          return (
            <div
              key={item.key}
              className="flex items-start justify-between gap-4 rounded-xl border border-gray-100 dark:border-gray-700 p-4 hover:border-blue-200 dark:hover:border-blue-700 transition-colors"
            >
              <div className="flex items-start gap-3">
                {done ? (
                  <CheckCircle2 className="w-5 h-5 text-green-600 mt-0.5" />
                ) : (
                  <Circle className="w-5 h-5 text-gray-400 mt-0.5" />
                )}
                <div>
                  <p className="text-sm font-semibold text-gray-900 dark:text-gray-100">
                    {item.title}
                  </p>
                  <p className="text-sm text-gray-600 dark:text-gray-400 mt-0.5">
                    {item.description}
                  </p>
                </div>
              </div>

              <Link
                href={item.href}
                className="shrink-0 inline-flex items-center gap-1.5 text-sm font-medium text-blue-700 dark:text-blue-300 hover:underline"
              >
                {done ? "View" : "Start"}
                <ArrowRight className="w-4 h-4" />
              </Link>
            </div>
          );
        })}
      </div>
    </div>
  );
}
