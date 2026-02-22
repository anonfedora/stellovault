"use client";

import { useState } from "react";
import type { LoanFormData } from "@/types";
import { Button } from "@/components/ui/Button";

// Mock collateral tokens the user owns
const MOCK_COLLATERALS = [
  {
    id: "ct-101",
    assetType: "INVOICE" as const,
    assetValue: 50000,
    metadata: "Invoice #INV-2026-001 – Export goods to Kenya",
  },
  {
    id: "ct-102",
    assetType: "COMMODITY" as const,
    assetValue: 75000,
    metadata: "Gold reserves warehouse – Accra lot #42",
  },
  {
    id: "ct-103",
    assetType: "RECEIVABLE" as const,
    assetValue: 30000,
    metadata: "Receivable – Acme Corp payment due Mar 2026",
  },
];

const STEPS = [
  "Select Collateral",
  "Loan Terms",
  "Review",
  "Sign & Submit",
] as const;

interface LoanFormProps {
  onSubmit?: (data: LoanFormData) => Promise<void>;
}

export const LoanForm = ({ onSubmit }: LoanFormProps) => {
  const [step, setStep] = useState(0);
  const [submitting, setSubmitting] = useState(false);
  const [xdr, setXdr] = useState<string | null>(null);
  const [signed, setSigned] = useState(false);

  const [form, setForm] = useState<LoanFormData>({
    selectedCollateralId: "",
    principal: 0,
    termMonths: 6,
    interestRate: 7.5,
  });

  const selectedCollateral = MOCK_COLLATERALS.find(
    (c) => c.id === form.selectedCollateralId,
  );

  const canProceed = (): boolean => {
    switch (step) {
      case 0:
        return !!form.selectedCollateralId;
      case 1:
        return (
          form.principal > 0 && form.termMonths > 0 && form.interestRate > 0
        );
      case 2:
        return true;
      case 3:
        return signed;
      default:
        return false;
    }
  };

  const handleNext = async () => {
    if (step < 3) {
      setStep(step + 1);
      // When reaching the sign step, generate mock XDR
      if (step === 2) {
        setSubmitting(true);
        // Simulating POST /api/v1/loans → returns XDR
        await new Promise((r) => setTimeout(r, 800));
        const mockXdr =
          "AAAAAgAAAAD" + btoa(JSON.stringify(form)).slice(0, 60) + "...AAAA";
        setXdr(mockXdr);
        setSubmitting(false);
      }
    }
  };

  const handleSign = async () => {
    setSubmitting(true);
    try {
      // TODO: Replace with real Freighter signTransaction()
      // const signedXdr = await window.freighterApi.signTransaction(xdr, { networkPassphrase: Networks.TESTNET });
      await new Promise((r) => setTimeout(r, 1200));
      setSigned(true);
      if (onSubmit) await onSubmit(form);
    } catch {
      // handle error
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
      {/* Step indicator */}
      <div className="flex items-center mb-10">
        {STEPS.map((label, i) => (
          <div key={label} className="flex items-center flex-1">
            <div className="flex flex-col items-center flex-1">
              <div
                className={`w-10 h-10 rounded-full flex items-center justify-center text-sm font-bold transition-all duration-300 ${
                  i < step
                    ? "bg-blue-600 text-white"
                    : i === step
                      ? "bg-blue-900 text-white ring-4 ring-blue-200 dark:ring-blue-900/40"
                      : "bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400"
                }`}
              >
                {i < step ? "✓" : i + 1}
              </div>
              <span
                className={`text-xs mt-2 text-center ${i <= step ? "text-blue-700 dark:text-blue-300 font-medium" : "text-gray-400 dark:text-gray-500"}`}
              >
                {label}
              </span>
            </div>
            {i < STEPS.length - 1 && (
              <div
                className={`h-0.5 flex-1 -mt-5 mx-1 transition-colors ${i < step ? "bg-blue-500" : "bg-gray-200 dark:bg-gray-700"}`}
              />
            )}
          </div>
        ))}
      </div>

      {/* Step content */}
      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-8">
        {/* Step 0: Select Collateral */}
        {step === 0 && (
          <div>
            <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">
              Select Collateral
            </h2>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
              Choose the collateral token to back your loan.
            </p>
            <div className="space-y-3">
              {MOCK_COLLATERALS.map((c) => (
                <button
                  key={c.id}
                  type="button"
                  onClick={() =>
                    setForm({ ...form, selectedCollateralId: c.id })
                  }
                  className={`w-full text-left p-4 rounded-lg border-2 transition-all ${
                    form.selectedCollateralId === c.id
                      ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20 dark:border-blue-400"
                      : "border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
                  }`}
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-700 px-2 py-0.5 rounded">
                      {c.assetType}
                    </span>
                    <span className="text-sm font-bold text-gray-900 dark:text-gray-100">
                      ${c.assetValue.toLocaleString()}
                    </span>
                  </div>
                  <p className="text-sm text-gray-700 dark:text-gray-300 mt-1">
                    {c.metadata}
                  </p>
                  <p className="text-xs font-mono text-gray-400 mt-1">{c.id}</p>
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Step 1: Loan Terms */}
        {step === 1 && (
          <div>
            <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">
              Loan Terms
            </h2>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
              Define the amount, duration, and interest rate.
            </p>
            <div className="space-y-5">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Principal Amount ($)
                </label>
                <input
                  type="number"
                  min={1}
                  max={
                    selectedCollateral ? selectedCollateral.assetValue : 999999
                  }
                  value={form.principal || ""}
                  onChange={(e) =>
                    setForm({ ...form, principal: Number(e.target.value) })
                  }
                  className="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition"
                  placeholder="e.g. 25000"
                />
                {selectedCollateral && (
                  <p className="text-xs text-gray-400 mt-1">
                    Max: ${selectedCollateral.assetValue.toLocaleString()}{" "}
                    (collateral value)
                  </p>
                )}
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Term (months)
                  </label>
                  <select
                    value={form.termMonths}
                    onChange={(e) =>
                      setForm({ ...form, termMonths: Number(e.target.value) })
                    }
                    className="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition"
                  >
                    {[3, 6, 9, 12, 18, 24].map((m) => (
                      <option key={m} value={m}>
                        {m} months
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Interest Rate (%)
                  </label>
                  <input
                    type="number"
                    min={0.1}
                    max={30}
                    step={0.1}
                    value={form.interestRate}
                    onChange={(e) =>
                      setForm({ ...form, interestRate: Number(e.target.value) })
                    }
                    className="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition"
                  />
                </div>
              </div>

              {form.principal > 0 && (
                <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4 border border-blue-200 dark:border-blue-700">
                  <p className="text-sm text-gray-700 dark:text-gray-300">
                    Total repayment:{" "}
                    <span className="font-bold text-blue-900 dark:text-blue-300">
                      $
                      {(
                        form.principal *
                        (1 + form.interestRate / 100)
                      ).toLocaleString(undefined, { maximumFractionDigits: 2 })}
                    </span>
                  </p>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Step 2: Review */}
        {step === 2 && (
          <div>
            <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">
              Review Your Loan
            </h2>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
              Confirm all details before signing.
            </p>
            <div className="space-y-4">
              <div className="flex justify-between py-3 border-b border-gray-100 dark:border-gray-700">
                <span className="text-sm text-gray-500 dark:text-gray-400">
                  Collateral
                </span>
                <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  {selectedCollateral?.assetType} — $
                  {selectedCollateral?.assetValue.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between py-3 border-b border-gray-100 dark:border-gray-700">
                <span className="text-sm text-gray-500 dark:text-gray-400">
                  Principal
                </span>
                <span className="text-sm font-bold text-gray-900 dark:text-gray-100">
                  ${form.principal.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between py-3 border-b border-gray-100 dark:border-gray-700">
                <span className="text-sm text-gray-500 dark:text-gray-400">
                  Term
                </span>
                <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  {form.termMonths} months
                </span>
              </div>
              <div className="flex justify-between py-3 border-b border-gray-100 dark:border-gray-700">
                <span className="text-sm text-gray-500 dark:text-gray-400">
                  Interest Rate
                </span>
                <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  {form.interestRate}%
                </span>
              </div>
              <div className="flex justify-between py-3">
                <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                  Total Repayment
                </span>
                <span className="text-sm font-bold text-blue-900 dark:text-blue-300">
                  $
                  {(
                    form.principal *
                    (1 + form.interestRate / 100)
                  ).toLocaleString(undefined, { maximumFractionDigits: 2 })}
                </span>
              </div>
            </div>
          </div>
        )}

        {/* Step 3: Sign XDR */}
        {step === 3 && (
          <div>
            <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">
              Sign Transaction
            </h2>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
              Review the transaction XDR and sign with your Freighter wallet.
            </p>

            {xdr && (
              <div className="mb-6">
                <label className="block text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-2">
                  Transaction XDR
                </label>
                <div className="bg-gray-900 dark:bg-black rounded-lg p-4 overflow-x-auto">
                  <code className="text-xs text-green-400 font-mono break-all whitespace-pre-wrap">
                    {xdr}
                  </code>
                </div>
              </div>
            )}

            {signed ? (
              <div className="text-center py-6">
                <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-green-100 dark:bg-green-900/30 flex items-center justify-center">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    strokeWidth={2}
                    stroke="currentColor"
                    className="w-8 h-8 text-green-600 dark:text-green-400"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      d="M4.5 12.75l6 6 9-13.5"
                    />
                  </svg>
                </div>
                <h3 className="text-lg font-bold text-green-700 dark:text-green-300">
                  Transaction Signed & Submitted!
                </h3>
                <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                  Your loan request has been submitted to the network.
                </p>
              </div>
            ) : (
              <Button
                onClick={handleSign}
                loading={submitting}
                fullWidth
                size="lg"
                className="bg-blue-900 hover:bg-blue-800"
              >
                Sign with Freighter
              </Button>
            )}
          </div>
        )}
      </div>

      {/* Navigation buttons */}
      {!signed && (
        <div className="flex justify-between mt-6">
          <Button
            variant="outline"
            onClick={() => setStep(Math.max(0, step - 1))}
            disabled={step === 0}
          >
            Back
          </Button>
          {step < 3 && (
            <Button
              onClick={handleNext}
              disabled={!canProceed()}
              loading={submitting}
              className="bg-blue-900 hover:bg-blue-800"
            >
              {step === 2 ? "Generate XDR" : "Continue"}
            </Button>
          )}
        </div>
      )}
    </div>
  );
};
