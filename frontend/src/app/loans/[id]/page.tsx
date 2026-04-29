"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { useLoans } from "@/hooks/useLoans";
import { RepaymentSchedule } from "@/components/loans/RepaymentSchedule";
import { Button } from "@/components/ui/Button";
import { shortenAddress } from "@/utils/stellar";
import { signTransaction } from "@stellar/freighter-api";
import { Networks } from "@stellar/stellar-sdk";
import { ArrowLeft, Loader2, X } from "lucide-react";

const STATUS_STYLES: Record<string, string> = {
  PENDING:
    "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300",
  ACTIVE:
    "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300",
  REPAID: "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300",
  DEFAULTED: "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300",
};

export default function LoanDetailPage() {
  const params = useParams();
  const id = params.id as string;
  const { loan, loading, error, fetchLoanById } = useLoans();
  const [showRepaymentModal, setShowRepaymentModal] = useState(false);
  const [repaymentAmount, setRepaymentAmount] = useState("");
  const [repaymentLoading, setRepaymentLoading] = useState(false);
  const [repaymentError, setRepaymentError] = useState<string | null>(null);
  const [repaymentXdr, setRepaymentXdr] = useState<string | null>(null);
  const [repaymentSuccess, setRepaymentSuccess] = useState(false);

  useEffect(() => {
    if (id) fetchLoanById(id);
  }, [id, fetchLoanById]);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <Loader2 className="w-8 h-8 text-blue-600 animate-spin" />
      </div>
    );
  }

  if (error || !loan) {
    return (
      <div className="container mx-auto px-4 py-8">
        <Link
          href="/loans"
          className="inline-flex items-center gap-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 mb-6 transition"
        >
          <ArrowLeft className="w-4 h-4" /> Back to Loans
        </Link>
        <div className="text-center py-20">
          <p className="text-red-500 text-lg">{error || "Loan not found"}</p>
        </div>
      </div>
    );
  }

  const totalOwed = loan.principal * (1 + loan.interestRate / 100);
  const totalRepaid = loan.repayments.reduce((s, r) => s + r.amount, 0);
  const remainingBalance = Math.max(0, totalOwed - totalRepaid);

  const handleRepayment = async () => {
    setRepaymentLoading(true);
    setRepaymentError(null);
    try {
      const amount = parseFloat(repaymentAmount);
      if (isNaN(amount) || amount <= 0) {
        throw new Error("Please enter a valid repayment amount");
      }
      if (amount > remainingBalance) {
        throw new Error(`Amount exceeds remaining balance of $${remainingBalance.toLocaleString()}`);
      }

      // Generate XDR from backend
      const res = await fetch('/api/v1/loans/repayment', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ loanId: loan.id, amount }),
      });

      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body.error || 'Failed to generate repayment transaction');
      }

      const data = await res.json();
      setRepaymentXdr(data.xdr);
    } catch (err) {
      setRepaymentError(err instanceof Error ? err.message : 'Failed to initiate repayment');
    } finally {
      setRepaymentLoading(false);
    }
  };

  const handleSignRepayment = async () => {
    setRepaymentLoading(true);
    setRepaymentError(null);
    try {
      if (!repaymentXdr) {
        throw new Error('No transaction XDR available');
      }

      // Sign with Freighter
      const { signedTxXdr, error: signError } = await signTransaction(repaymentXdr, {
        networkPassphrase: Networks.TESTNET,
      });

      if (signError || !signedTxXdr) {
        throw new Error(signError || 'Signing failed. Please try again.');
      }

      // Submit signed XDR
      const res = await fetch('/api/v1/loans/repayment/submit', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ signedXdr: signedTxXdr, loanId: id, amount: repaymentAmount }),
      });

      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body.error || 'Failed to submit repayment');
      }

      setRepaymentSuccess(true);
      // Refresh loan data
      await fetchLoanById(id);
      // Close modal after delay
      setTimeout(() => {
        setShowRepaymentModal(false);
        setRepaymentSuccess(false);
        setRepaymentXdr(null);
        setRepaymentAmount("");
      }, 2000);
    } catch (err) {
      setRepaymentError(err instanceof Error ? err.message : 'Failed to sign repayment');
    } finally {
      setRepaymentLoading(false);
    }
  };

  return (
    <div className="container mx-auto px-4 py-8">
      {/* Back button */}
      <Link
        href="/loans"
        className="inline-flex items-center gap-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 mb-6 transition text-sm"
      >
        <ArrowLeft className="w-4 h-4" /> Back to Loans
      </Link>

      {/* Loan header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between mb-8">
        <div>
          <div className="flex items-center gap-3 mb-1">
            <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
              Loan #{loan.id}
            </h1>
            <span
              className={`text-xs font-semibold px-3 py-1 rounded-full ${STATUS_STYLES[loan.status]}`}
            >
              {loan.status}
            </span>
          </div>
          <p className="text-sm text-gray-500 dark:text-gray-400">
            Created{" "}
            {new Date(loan.createdAt).toLocaleDateString("en-US", {
              year: "numeric",
              month: "long",
              day: "numeric",
            })}
          </p>
        </div>
        {loan.status === "ACTIVE" && (
          <Button
            size="lg"
            className="mt-4 sm:mt-0 bg-blue-900 hover:bg-blue-800"
            onClick={() => setShowRepaymentModal(true)}
          >
            Make Repayment
          </Button>
        )}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left: Loan info + Collateral */}
        <div className="lg:col-span-1 space-y-6">
          {/* Loan details card */}
          <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
              Loan Details
            </h3>
            <dl className="space-y-4">
              <div className="flex justify-between">
                <dt className="text-sm text-gray-500 dark:text-gray-400">
                  Borrower
                </dt>
                <dd className="text-sm font-mono text-gray-900 dark:text-gray-100">
                  {shortenAddress(loan.borrower, 6)}
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-sm text-gray-500 dark:text-gray-400">
                  Principal
                </dt>
                <dd className="text-sm font-bold text-gray-900 dark:text-gray-100">
                  ${loan.principal.toLocaleString()}
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-sm text-gray-500 dark:text-gray-400">
                  Interest Rate
                </dt>
                <dd className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  {loan.interestRate}%
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-sm text-gray-500 dark:text-gray-400">
                  Term
                </dt>
                <dd className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  {loan.termMonths} months
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-sm text-gray-500 dark:text-gray-400">
                  Total Owed
                </dt>
                <dd className="text-sm font-bold text-blue-700 dark:text-blue-300">
                  $
                  {totalOwed.toLocaleString(undefined, {
                    maximumFractionDigits: 2,
                  })}
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-sm text-gray-500 dark:text-gray-400">
                  Total Repaid
                </dt>
                <dd className="text-sm font-bold text-green-600 dark:text-green-400">
                  ${totalRepaid.toLocaleString()}
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-sm text-gray-500 dark:text-gray-400">
                  Maturity Date
                </dt>
                <dd className="text-sm text-gray-900 dark:text-gray-100">
                  {new Date(loan.maturityDate).toLocaleDateString("en-US", {
                    year: "numeric",
                    month: "short",
                    day: "numeric",
                  })}
                </dd>
              </div>
            </dl>
          </div>

          {/* Collateral info card */}
          <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
              Collateral Info
            </h3>
            <dl className="space-y-4">
              <div className="flex justify-between">
                <dt className="text-sm text-gray-500 dark:text-gray-400">
                  Token ID
                </dt>
                <dd className="text-sm font-mono text-gray-900 dark:text-gray-100">
                  {loan.collateralTokenId}
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-sm text-gray-500 dark:text-gray-400">
                  Asset Type
                </dt>
                <dd>
                  <span className="text-xs font-medium uppercase tracking-wider text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 px-2 py-0.5 rounded">
                    {loan.collateralAssetType}
                  </span>
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-sm text-gray-500 dark:text-gray-400">
                  Value
                </dt>
                <dd className="text-sm font-bold text-gray-900 dark:text-gray-100">
                  ${loan.collateralValue.toLocaleString()}
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-sm text-gray-500 dark:text-gray-400">
                  LTV Ratio
                </dt>
                <dd className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  {loan.collateralValue > 0
                    ? ((loan.principal / loan.collateralValue) * 100).toFixed(1)
                    : 0}
                  %
                </dd>
              </div>
            </dl>
          </div>
        </div>

        {/* Right: Repayment schedule */}
        <div className="lg:col-span-2">
          <RepaymentSchedule
            repayments={loan.repayments}
            totalOwed={totalOwed}
          />
        </div>
      </div>

      {/* Repayment Modal */}
      {showRepaymentModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 max-w-md w-full">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-xl font-bold text-gray-900 dark:text-gray-100">
                Make Repayment
              </h3>
              <button
                onClick={() => {
                  setShowRepaymentModal(false);
                  setRepaymentError(null);
                  setRepaymentXdr(null);
                  setRepaymentAmount("");
                  setRepaymentSuccess(false);
                }}
                className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            {/* Error display */}
            {repaymentError && (
              <div className="mb-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-700 rounded-lg p-3">
                <p className="text-sm text-red-700 dark:text-red-300">{repaymentError}</p>
              </div>
            )}

            {/* Success display */}
            {repaymentSuccess ? (
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
                  Repayment Successful!
                </h3>
                <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                  Your payment has been submitted to the network.
                </p>
              </div>
            ) : repaymentXdr ? (
              /* Sign step */
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
                  Review the transaction XDR and sign with your Freighter wallet.
                </p>
                <div className="bg-gray-900 dark:bg-black rounded-lg p-3 overflow-x-auto mb-4">
                  <code className="text-xs text-green-400 font-mono break-all whitespace-pre-wrap">
                    {repaymentXdr}
                  </code>
                </div>
                <Button
                  onClick={handleSignRepayment}
                  loading={repaymentLoading}
                  fullWidth
                  className="bg-blue-900 hover:bg-blue-800"
                >
                  Sign with Freighter
                </Button>
              </div>
            ) : (
              /* Amount input step */
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
                  Enter the amount you want to repay. Remaining balance:{" "}
                  <span className="font-semibold text-gray-900 dark:text-gray-100">
                    ${remainingBalance.toLocaleString()}
                  </span>
                </p>
                <div className="mb-4">
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Repayment Amount ($)
                  </label>
                  <input
                    type="number"
                    min={0.01}
                    max={remainingBalance}
                    step={0.01}
                    value={repaymentAmount}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setRepaymentAmount(e.target.value)}
                    className="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition"
                    placeholder="e.g. 1000"
                  />
                </div>
                <Button
                  onClick={handleRepayment}
                  loading={repaymentLoading}
                  fullWidth
                  className="bg-blue-900 hover:bg-blue-800"
                >
                  Generate Transaction
                </Button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
