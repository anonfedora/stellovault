"use client";

import type { Repayment } from "@/types";
import { getExplorerUrl } from "@/utils/stellar";

interface RepaymentScheduleProps {
  repayments: Repayment[];
  totalOwed: number;
}

export const RepaymentSchedule = ({
  repayments,
  totalOwed,
}: RepaymentScheduleProps) => {
  const totalRepaid = repayments.reduce((s, r) => s + r.amount, 0);
  const progress =
    totalOwed > 0 ? Math.min(100, (totalRepaid / totalOwed) * 100) : 0;
  const remaining = Math.max(0, totalOwed - totalRepaid);

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
        Repayment Progress
      </h3>

      {/* Progress bar */}
      <div className="mb-6">
        <div className="flex justify-between text-sm mb-2">
          <span className="text-gray-600 dark:text-gray-400">
            ${totalRepaid.toLocaleString()} repaid
          </span>
          <span className="font-semibold text-gray-900 dark:text-gray-100">
            ${totalOwed.toLocaleString()} total
          </span>
        </div>
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-4 overflow-hidden relative">
          <div
            className="h-full rounded-full transition-all duration-700 ease-out bg-gradient-to-r from-blue-500 via-blue-600 to-indigo-600"
            style={{ width: `${progress}%` }}
          />
          <span className="absolute inset-0 flex items-center justify-center text-[10px] font-bold text-white drop-shadow">
            {progress.toFixed(1)}%
          </span>
        </div>
        <div className="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
          <span>
            {repayments.length} payment{repayments.length !== 1 ? "s" : ""} made
          </span>
          <span>${remaining.toLocaleString()} remaining</span>
        </div>
      </div>

      {/* Repayment table */}
      {repayments.length > 0 ? (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-200 dark:border-gray-700">
                <th className="text-left py-3 px-2 text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                  Date
                </th>
                <th className="text-right py-3 px-2 text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                  Amount
                </th>
                <th className="text-right py-3 px-2 text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                  Tx Hash
                </th>
              </tr>
            </thead>
            <tbody>
              {repayments.map((r) => (
                <tr
                  key={r.id}
                  className="border-b border-gray-100 dark:border-gray-700/50 hover:bg-gray-50 dark:hover:bg-gray-700/30 transition-colors"
                >
                  <td className="py-3 px-2 text-gray-700 dark:text-gray-300">
                    {new Date(r.paidAt).toLocaleDateString("en-US", {
                      year: "numeric",
                      month: "short",
                      day: "numeric",
                    })}
                  </td>
                  <td className="py-3 px-2 text-right font-medium text-gray-900 dark:text-gray-100">
                    ${r.amount.toLocaleString()}
                  </td>
                  <td className="py-3 px-2 text-right">
                    <a
                      href={getExplorerUrl(r.txHash)}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-blue-600 dark:text-blue-400 hover:underline font-mono text-xs"
                    >
                      {r.txHash.slice(0, 8)}…
                    </a>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div className="text-center py-8 text-gray-500 dark:text-gray-400">
          <p className="text-sm">No repayments recorded yet.</p>
        </div>
      )}
    </div>
  );
};
