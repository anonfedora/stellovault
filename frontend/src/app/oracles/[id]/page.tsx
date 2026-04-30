"use client";

import { useEffect } from "react";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { ReputationChart } from "@/components/oracles/ReputationChart";
import { EarningsDashboard } from "@/components/oracles/EarningsDashboard";
import { ConfirmationForm } from "@/components/oracles/ConfirmationForm";
import { useOracles } from "@/hooks/useOracles";

export default function OracleDetailPage({ params }: { params: { id: string } }) {
  const { oracle, loading, fetchOracleById, submitConfirmation } = useOracles();

  useEffect(() => {
    fetchOracleById(params.id);
  }, [fetchOracleById, params.id]);

  if (loading) {
    return (
      <main className="min-h-screen bg-gray-50 px-4 pt-6 sm:px-6 lg:px-8">
        <div className="mx-auto max-w-4xl space-y-4">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="h-32 animate-pulse rounded-lg bg-gray-200" />
          ))}
        </div>
      </main>
    );
  }

  if (!oracle) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-gray-50">
        <div className="text-center">
          <p className="text-gray-500">Oracle not found.</p>
          <Link href="/oracles" className="mt-3 text-sm text-blue-700 hover:underline">Back to oracles</Link>
        </div>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-24 pt-6 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-4xl space-y-6">
        <Link href="/oracles" className="inline-flex items-center gap-1 text-sm text-gray-500 hover:text-gray-800">
          <ArrowLeft className="h-4 w-4" /> Back to oracles
        </Link>

        {/* Header */}
        <div className="rounded-xl border border-gray-200 bg-white p-6">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div>
              <div className="flex items-center gap-2">
                <span className={`h-2 w-2 rounded-full ${oracle.isActive ? "bg-green-500" : "bg-gray-400"}`} />
                <span className="text-sm font-semibold text-gray-500">{oracle.oracleType}</span>
              </div>
              <p className="mt-1 break-all font-mono text-sm text-gray-700">{oracle.address}</p>
            </div>
            <span className={`rounded-full px-3 py-1 text-xs font-semibold ${oracle.isActive ? "bg-green-100 text-green-800" : "bg-gray-100 text-gray-600"}`}>
              {oracle.isActive ? "Active" : "Inactive"}
            </span>
          </div>
          <div className="mt-4 grid grid-cols-3 gap-3 text-center text-sm">
            <div className="rounded-lg bg-gray-50 p-3">
              <p className="text-xs text-gray-400">Total Confirmations</p>
              <p className="font-bold text-gray-900">{oracle.totalConfirmations}</p>
            </div>
            <div className="rounded-lg bg-green-50 p-3">
              <p className="text-xs text-green-600">Successful</p>
              <p className="font-bold text-green-900">{oracle.successfulConfirmations}</p>
            </div>
            <div className="rounded-lg bg-red-50 p-3">
              <p className="text-xs text-red-500">Failed</p>
              <p className="font-bold text-red-900">{oracle.failedConfirmations}</p>
            </div>
          </div>
        </div>

        {/* Reputation */}
        {oracle.reputation && <ReputationChart reputation={oracle.reputation} />}

        {/* Earnings */}
        {oracle.stake && (
          <EarningsDashboard stake={oracle.stake} totalConfirmations={oracle.totalConfirmations} />
        )}

        {/* Submit confirmation */}
        {oracle.isActive && (
          <div className="rounded-xl border border-gray-200 bg-white p-5">
            <h2 className="mb-4 text-sm font-semibold text-gray-700">Submit Confirmation</h2>
            <ConfirmationForm
              oracleAddress={oracle.address}
              onSubmit={submitConfirmation}
              loading={loading}
            />
          </div>
        )}
      </div>
    </main>
  );
}
