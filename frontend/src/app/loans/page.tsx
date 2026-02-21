"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useLoans } from "@/hooks/useLoans";
import { LoanCard } from "@/components/loans/LoanCard";
import type { LoanStatus } from "@/types";
import { Plus, ChevronLeft, ChevronRight, Loader2 } from "lucide-react";

const STATUSES: Array<LoanStatus | "ALL"> = [
  "ALL",
  "PENDING",
  "ACTIVE",
  "REPAID",
  "DEFAULTED",
];

const STATUS_TAB_STYLES: Record<string, string> = {
  ALL: "hover:bg-gray-100 dark:hover:bg-gray-800",
  PENDING: "hover:bg-yellow-50 dark:hover:bg-yellow-900/20",
  ACTIVE: "hover:bg-green-50 dark:hover:bg-green-900/20",
  REPAID: "hover:bg-blue-50 dark:hover:bg-blue-900/20",
  DEFAULTED: "hover:bg-red-50 dark:hover:bg-red-900/20",
};

export default function LoansPage() {
  const { loans, loading, error, totalPages, fetchLoans } = useLoans();
  const [activeStatus, setActiveStatus] = useState<LoanStatus | "ALL">("ALL");
  const [page, setPage] = useState(1);

  useEffect(() => {
    fetchLoans(activeStatus, page);
  }, [activeStatus, page, fetchLoans]);

  const handleStatusChange = (status: LoanStatus | "ALL") => {
    setActiveStatus(status);
    setPage(1);
  };

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
        <Link
          href="/loans/new"
          className="mt-4 sm:mt-0 inline-flex items-center gap-2 bg-blue-900 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-800 hover:shadow-lg transition-all group"
        >
          <Plus className="w-5 h-5 group-hover:rotate-90 transition-transform" />
          New Loan
        </Link>
      </div>

      {/* Status filter tabs */}
      <div className="flex gap-1 mb-6 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-1.5 overflow-x-auto">
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
      ) : loans.length === 0 ? (
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
            {activeStatus === "ALL"
              ? "You don't have any loans yet."
              : `No ${activeStatus.toLowerCase()} loans.`}
          </p>
          <Link
            href="/loans/new"
            className="inline-flex items-center gap-2 text-blue-600 hover:text-blue-700 font-medium text-sm"
          >
            <Plus className="w-4 h-4" />
            Create your first loan
          </Link>
        </div>
      ) : (
        <>
          {/* Loan cards grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6 mb-8">
            {loans.map((loan) => (
              <LoanCard key={loan.id} loan={loan} />
            ))}
          </div>

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
