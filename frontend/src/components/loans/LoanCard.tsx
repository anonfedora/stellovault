"use client";

import Link from "next/link";
import type { Loan } from "@/types";
import { shortenAddress } from "@/utils/stellar";
import { formatAmount } from "@/utils/stellar";

// Status badge colour map
const STATUS_STYLES: Record<string, string> = {
  PENDING:
    "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300",
  ACTIVE:
    "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300",
  REPAID: "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300",
  DEFAULTED: "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300",
};

interface LoanCardProps {
  loan: Loan;
}

export const LoanCard = ({ loan }: LoanCardProps) => {
  const totalOwed = loan.principal * (1 + loan.interestRate / 100);
  const totalRepaid = loan.repayments.reduce((s, r) => s + r.amount, 0);
  const progress = Math.min(100, (totalRepaid / totalOwed) * 100);

  return (
    <Link
      href={`/loans/${loan.id}`}
      className="block bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 hover:shadow-lg hover:border-blue-300 dark:hover:border-blue-600 transition-all duration-200 group"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <span className="text-sm font-mono text-gray-500 dark:text-gray-400">
          #{loan.id}
        </span>
        <span
          className={`text-xs font-semibold px-2.5 py-1 rounded-full ${STATUS_STYLES[loan.status]}`}
        >
          {loan.status}
        </span>
      </div>

      {/* Borrower */}
      <div className="mb-4">
        <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">
          Borrower
        </p>
        <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
          {shortenAddress(loan.borrower, 6)}
        </p>
      </div>

      {/* Financial details grid */}
      <div className="grid grid-cols-3 gap-3 mb-4">
        <div>
          <p className="text-xs text-gray-500 dark:text-gray-400">Principal</p>
          <p className="text-sm font-bold text-gray-900 dark:text-gray-100">
            ${loan.principal.toLocaleString()}
          </p>
        </div>
        <div>
          <p className="text-xs text-gray-500 dark:text-gray-400">Interest</p>
          <p className="text-sm font-bold text-gray-900 dark:text-gray-100">
            {loan.interestRate}%
          </p>
        </div>
        <div>
          <p className="text-xs text-gray-500 dark:text-gray-400">Term</p>
          <p className="text-sm font-bold text-gray-900 dark:text-gray-100">
            {loan.termMonths}mo
          </p>
        </div>
      </div>

      {/* Mini progress bar */}
      <div>
        <div className="flex justify-between text-xs text-gray-500 dark:text-gray-400 mb-1">
          <span>Repaid</span>
          <span>{progress.toFixed(0)}%</span>
        </div>
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 overflow-hidden">
          <div
            className="h-full rounded-full transition-all duration-500 bg-gradient-to-r from-blue-500 to-blue-700"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>

      {/* Collateral type tag */}
      <div className="mt-4 flex items-center gap-2">
        <span className="text-[10px] font-medium uppercase tracking-wider text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-700 px-2 py-0.5 rounded">
          {loan.collateralAssetType}
        </span>
        <span className="text-xs text-gray-400 dark:text-gray-500">
          Collateral: ${loan.collateralValue.toLocaleString()}
        </span>
      </div>

      {/* Hover hint */}
      <p className="mt-3 text-xs text-blue-600 dark:text-blue-400 opacity-0 group-hover:opacity-100 transition-opacity">
        View details →
      </p>
    </Link>
  );
};
