"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useLoans } from "@/hooks/useLoans";
import { LoanStatus } from "@/types";
import { LoanCard } from "@/components/loans/LoanCard";
import { Plus, ChevronLeft, ChevronRight, Loader2, Search, X, BarChart3 } from "lucide-react";
import { markQuickStartDone } from "@/utils/onboarding";

const STATUSES: Array<LoanStatus | "ALL"> = [
  "ALL",
  LoanStatus.PENDING,
  LoanStatus.ACTIVE,
  LoanStatus.REPAID,
  LoanStatus.DEFAULTED,
];

const STATUS_TAB_STYLES: Record<string, string> = {
  ALL: "hover:bg-gray-100 dark:hover:bg-gray-800",
  [LoanStatus.PENDING]: "hover:bg-yellow-50 dark:hover:bg-yellow-900/20",
  [LoanStatus.ACTIVE]: "hover:bg-green-50 dark:hover:bg-green-900/20",
  [LoanStatus.REPAID]: "hover:bg-blue-50 dark:hover:bg-blue-900/20",
  [LoanStatus.DEFAULTED]: "hover:bg-red-50 dark:hover:bg-red-900/20",
};

export default function LoansPage() {
  const { loans, loading, error, totalPages, fetchLoans } = useLoans();
  const [activeStatus, setActiveStatus] = useState<LoanStatus | "ALL">("ALL");
  const [page, setPage] = useState(1);
  const [searchQuery, setSearchQuery] = useState("");
  const [comparisonMode, setComparisonMode] = useState(false);
  const [selectedLoans, setSelectedLoans] = useState<Set<string>>(new Set());

  useEffect(() => {
    fetchLoans(activeStatus, page);
  }, [activeStatus, page, fetchLoans]);

  useEffect(() => {
    markQuickStartDone("monitorLoan");
  }, []);

  const handleStatusChange = (status: LoanStatus | "ALL") => {
    setActiveStatus(status);
    setPage(1);
  };

  const filteredLoans = loans.filter((loan) => {
    if (!searchQuery) return true;
    const query = searchQuery.toLowerCase();
    return (
      loan.id.toLowerCase().includes(query) ||
      loan.borrower.toLowerCase().includes(query) ||
      loan.collateralAssetType.toLowerCase().includes(query) ||
      loan.status.toLowerCase().includes(query)
    );
  });

  const toggleLoanSelection = (loanId: string) => {
    const newSelection = new Set(selectedLoans);
    if (newSelection.has(loanId)) {
      newSelection.delete(loanId);
    } else {
      if (newSelection.size >= 3) {
        return; // Max 3 loans for comparison
      }
      newSelection.add(loanId);
    }
    setSelectedLoans(newSelection);
  };

  const clearSelection = () => {
    setSelectedLoans(new Set());
    setComparisonMode(false);
  };

  const selectedLoanData = loans.filter((loan) => selectedLoans.has(loan.id));

  return (
    <div className="container mx-auto px-4 py-8">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
            Loans
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            Browse, filter, and manage your trade finance loans.
          </p>
        </div>
        <div className="flex items-center gap-3 mt-4 sm:mt-0">
          <button
            onClick={() => setComparisonMode(!comparisonMode)}
            className={`inline-flex items-center gap-2 px-4 py-3 rounded-lg font-medium transition-all ${
              comparisonMode
                ? "bg-blue-900 text-white"
                : "bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700"
            }`}
          >
            <BarChart3 className="w-5 h-5" />
            Compare
          </button>
          <Link
            href="/loans/new"
            className="inline-flex items-center gap-2 bg-blue-900 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-800 hover:shadow-lg transition-all group"
          >
            <Plus className="w-5 h-5 group-hover:rotate-90 transition-transform" />
            New Loan
          </Link>
        </div>
      </div>

      {/* Search bar */}
      <div className="mb-6">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
          <input
            type="text"
            placeholder="Search by loan ID, borrower, collateral type, or status..."
            value={searchQuery}
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition"
          />
        </div>
      </div>

      {/* Status filter tabs */}
      <div
        id="sv-onboarding-monitor-loan"
        className="flex gap-1 mb-6 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-1.5 overflow-x-auto"
      >
        {STATUSES.map((status) => (
          <button
            key={status}
            onClick={() => handleStatusChange(status)}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-all whitespace-nowrap ${
              activeStatus === status
                ? "bg-blue-900 text-white shadow-sm"
                : `text-gray-600 dark:text-gray-400 ${STATUS_TAB_STYLES[status]}`
            }`}
          >
            {status === "ALL"
              ? "All Loans"
              : status.charAt(0) + status.slice(1).toLowerCase()}
          </button>
        ))}
      </div>

      {/* Content */}
      {loading ? (
        <div className="flex items-center justify-center py-20">
          <Loader2 className="w-8 h-8 text-blue-600 animate-spin" />
        </div>
      ) : error ? (
        <div className="text-center py-20">
          <p className="text-red-500">Error: {error}</p>
          <button
            onClick={() => fetchLoans(activeStatus, page)}
            className="mt-4 text-blue-600 hover:underline text-sm"
          >
            Try again
          </button>
        </div>
      ) : filteredLoans.length === 0 ? (
        <div className="text-center py-20 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700">
          <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-100 dark:bg-gray-700 flex items-center justify-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth={1.5}
              stroke="currentColor"
              className="w-8 h-8 text-gray-400"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M2.25 18.75a60.07 60.07 0 0115.797 2.101c.727.198 1.453-.342 1.453-1.096V18.75M3.75 4.5v.75A.75.75 0 013 4.5zM3 12a9 9 0 1018 0 9 9 0 00-18 0z"
              />
            </svg>
          </div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            No loans found
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1 mb-4">
            {searchQuery
              ? "No loans match your search criteria."
              : activeStatus === "ALL"
              ? "You don't have any loans yet."
              : `No ${activeStatus.toLowerCase()} loans.`}
          </p>
          {!searchQuery && (
            <Link
              href="/loans/new"
              className="inline-flex items-center gap-2 text-blue-600 hover:text-blue-700 font-medium text-sm"
            >
              <Plus className="w-4 h-4" />
              Create your first loan
            </Link>
          )}
        </div>
      ) : (
        <>
          {/* Loan cards grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6 mb-8">
            {filteredLoans.map((loan) => (
              <LoanCard
                key={loan.id}
                loan={loan}
                selectable={comparisonMode}
                selected={selectedLoans.has(loan.id)}
                onSelect={toggleLoanSelection}
              />
            ))}
          </div>

          {/* Comparison bar */}
          {comparisonMode && selectedLoans.size > 0 && (
            <div className="fixed bottom-0 left-0 right-0 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 p-4 shadow-lg z-40">
              <div className="container mx-auto flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    {selectedLoans.size} loan{selectedLoans.size !== 1 ? 's' : ''} selected
                  </span>
                  {selectedLoans.size >= 2 && (
                    <button
                      onClick={() => setComparisonMode(false)}
                      className="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                    >
                      View comparison
                    </button>
                  )}
                </div>
                <button
                  onClick={clearSelection}
                  className="inline-flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100"
                >
                  <X className="w-4 h-4" />
                  Clear selection
                </button>
              </div>
            </div>
          )}

          {/* Comparison table */}
          {!comparisonMode && selectedLoanData.length >= 2 && (
            <div className="mt-8 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
              <div className="flex items-center justify-between mb-6">
                <h3 className="text-xl font-bold text-gray-900 dark:text-gray-100">
                  Loan Comparison
                </h3>
                <button
                  onClick={clearSelection}
                  className="inline-flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100"
                >
                  <X className="w-4 h-4" />
                  Close
                </button>
              </div>
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-gray-200 dark:border-gray-700">
                      <th className="text-left py-3 px-4 text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                        Metric
                      </th>
                      {selectedLoanData.map((loan) => (
                        <th key={loan.id} className="text-center py-3 px-4 text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                          #{loan.id}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    <tr className="border-b border-gray-100 dark:border-gray-700/50">
                      <td className="py-3 px-4 text-gray-700 dark:text-gray-300 font-medium">Status</td>
                      {selectedLoanData.map((loan) => (
                        <td key={loan.id} className="py-3 px-4 text-center">
                          <span className={`text-xs font-semibold px-2 py-1 rounded-full ${STATUS_TAB_STYLES[loan.status]}`}>
                            {loan.status}
                          </span>
                        </td>
                      ))}
                    </tr>
                    <tr className="border-b border-gray-100 dark:border-gray-700/50">
                      <td className="py-3 px-4 text-gray-700 dark:text-gray-300 font-medium">Principal</td>
                      {selectedLoanData.map((loan) => (
                        <td key={loan.id} className="py-3 px-4 text-center font-bold text-gray-900 dark:text-gray-100">
                          ${loan.principal.toLocaleString()}
                        </td>
                      ))}
                    </tr>
                    <tr className="border-b border-gray-100 dark:border-gray-700/50">
                      <td className="py-3 px-4 text-gray-700 dark:text-gray-300 font-medium">Interest Rate</td>
                      {selectedLoanData.map((loan) => (
                        <td key={loan.id} className="py-3 px-4 text-center text-gray-900 dark:text-gray-100">
                          {loan.interestRate}%
                        </td>
                      ))}
                    </tr>
                    <tr className="border-b border-gray-100 dark:border-gray-700/50">
                      <td className="py-3 px-4 text-gray-700 dark:text-gray-300 font-medium">Term</td>
                      {selectedLoanData.map((loan) => (
                        <td key={loan.id} className="py-3 px-4 text-center text-gray-900 dark:text-gray-100">
                          {loan.termMonths} months
                        </td>
                      ))}
                    </tr>
                    <tr className="border-b border-gray-100 dark:border-gray-700/50">
                      <td className="py-3 px-4 text-gray-700 dark:text-gray-300 font-medium">Collateral Value</td>
                      {selectedLoanData.map((loan) => (
                        <td key={loan.id} className="py-3 px-4 text-center font-bold text-gray-900 dark:text-gray-100">
                          ${loan.collateralValue.toLocaleString()}
                        </td>
                      ))}
                    </tr>
                    <tr className="border-b border-gray-100 dark:border-gray-700/50">
                      <td className="py-3 px-4 text-gray-700 dark:text-gray-300 font-medium">LTV Ratio</td>
                      {selectedLoanData.map((loan) => (
                        <td key={loan.id} className="py-3 px-4 text-center text-gray-900 dark:text-gray-100">
                          {((loan.principal / loan.collateralValue) * 100).toFixed(1)}%
                        </td>
                      ))}
                    </tr>
                    <tr>
                      <td className="py-3 px-4 text-gray-700 dark:text-gray-300 font-medium">Total Repayment</td>
                      {selectedLoanData.map((loan) => (
                        <td key={loan.id} className="py-3 px-4 text-center font-bold text-blue-700 dark:text-blue-300">
                          ${(loan.principal * (1 + loan.interestRate / 100)).toLocaleString(undefined, { maximumFractionDigits: 2 })}
                        </td>
                      ))}
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex items-center justify-center gap-3">
              <button
                onClick={() => setPage(Math.max(1, page - 1))}
                disabled={page === 1}
                className="p-2 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800 disabled:opacity-30 disabled:cursor-not-allowed transition"
              >
                <ChevronLeft className="w-5 h-5" />
              </button>
              <span className="text-sm text-gray-600 dark:text-gray-400">
                Page{" "}
                <span className="font-bold text-gray-900 dark:text-gray-100">
                  {page}
                </span>{" "}
                of{" "}
                <span className="font-bold text-gray-900 dark:text-gray-100">
                  {totalPages}
                </span>
              </span>
              <button
                onClick={() => setPage(Math.min(totalPages, page + 1))}
                disabled={page === totalPages}
                className="p-2 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800 disabled:opacity-30 disabled:cursor-not-allowed transition"
              >
                <ChevronRight className="w-5 h-5" />
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
