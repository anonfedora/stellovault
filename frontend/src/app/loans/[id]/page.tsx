"use client";

import { useEffect } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { useLoans } from "@/hooks/useLoans";
import { RepaymentSchedule } from "@/components/loans/RepaymentSchedule";
import { Button } from "@/components/ui/Button";
import { shortenAddress, getExplorerUrl } from "@/utils/stellar";
import { ArrowLeft, Loader2, ExternalLink } from "lucide-react";

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
            onClick={() =>
              alert("Repayment flow would trigger XDR signing here")
            }
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
    </div>
  );
}
