"use client";

import { useState } from "react";
import { CheckCircle2, Upload } from "lucide-react";
import { incrementTransactionCount, markQuickStartDone } from "@/utils/onboarding";

export default function DashboardCollateralPage() {
  const [uploaded, setUploaded] = useState(false);

  const handleUpload = () => {
    markQuickStartDone("uploadCollateral");
    incrementTransactionCount(1);
    setUploaded(true);
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4 mb-8">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
            Collateral Portfolio
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">
            Upload and tokenize assets you can use to secure trade financing.
          </p>
        </div>
      </div>

      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              Upload collateral
            </h2>
            <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
              Add an invoice, receivable, or inventory proof to mint a collateral
              token.
            </p>
          </div>

          <button
            id="sv-onboarding-upload-collateral"
            type="button"
            onClick={handleUpload}
            className="inline-flex items-center justify-center gap-2 px-5 py-2.5 rounded-lg bg-blue-900 text-white text-sm font-medium hover:bg-blue-800 transition-colors"
          >
            <Upload className="w-4 h-4" />
            Upload Collateral
          </button>
        </div>

        {uploaded && (
          <div className="mt-5 flex items-start gap-2 rounded-lg border border-green-200 bg-green-50 px-4 py-3 text-sm text-green-800 dark:border-green-900/30 dark:bg-green-900/10 dark:text-green-200">
            <CheckCircle2 className="w-5 h-5 mt-0.5 shrink-0" />
            <div>
              <p className="font-semibold">Collateral uploaded</p>
              <p className="mt-0.5">
                This is a demo flow — swap in the real tokenization transaction
                when the Soroban contract call is ready.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

