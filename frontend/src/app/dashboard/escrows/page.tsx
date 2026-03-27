"use client";

import { useState } from "react";
import { CheckCircle2, ShieldCheck } from "lucide-react";
import { incrementTransactionCount, markQuickStartDone } from "@/utils/onboarding";

export default function DashboardEscrowsPage() {
  const [created, setCreated] = useState(false);

  const handleCreate = () => {
    markQuickStartDone("createEscrow");
    incrementTransactionCount(1);
    setCreated(true);
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4 mb-8">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
            Escrow Monitoring
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">
            Create and track escrows that protect both sides of a trade.
          </p>
        </div>
      </div>

      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              Create escrow
            </h2>
            <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
              Lock funds on-chain and release them automatically when conditions
              are satisfied.
            </p>
          </div>

          <button
            id="sv-onboarding-create-escrow"
            type="button"
            onClick={handleCreate}
            className="inline-flex items-center justify-center gap-2 px-5 py-2.5 rounded-lg bg-blue-900 text-white text-sm font-medium hover:bg-blue-800 transition-colors"
          >
            <ShieldCheck className="w-4 h-4" />
            Create Escrow
          </button>
        </div>

        {created && (
          <div className="mt-5 flex items-start gap-2 rounded-lg border border-green-200 bg-green-50 px-4 py-3 text-sm text-green-800 dark:border-green-900/30 dark:bg-green-900/10 dark:text-green-200">
            <CheckCircle2 className="w-5 h-5 mt-0.5 shrink-0" />
            <div>
              <p className="font-semibold">Escrow created</p>
              <p className="mt-0.5">
                This is a demo placeholder — wire it to the escrow Soroban
                contract call when ready.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

