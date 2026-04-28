"use client";

import { useEffect } from "react";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { AnalyticsMetricsChart } from "@/components/analytics/MetricsChart";
import { PerformanceTable } from "@/components/analytics/PerformanceTable";
import { useAnalytics } from "@/hooks/useAnalytics";

export default function PortfolioPage() {
  const { stats, protocol, portfolio, loading, fetchStats, fetchProtocol, fetchPortfolio } = useAnalytics();

  useEffect(() => {
    fetchStats();
    fetchProtocol();
    fetchPortfolio();
  }, [fetchStats, fetchProtocol, fetchPortfolio]);

  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-24 pt-6 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-5xl space-y-6">
        <Link href="/analytics" className="inline-flex items-center gap-1 text-sm text-gray-500 hover:text-gray-800">
          <ArrowLeft className="h-4 w-4" /> Back to analytics
        </Link>

        <div>
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">Portfolio Analysis</h1>
          <p className="mt-2 text-gray-600">
            Historical TVL, volume trends, and detailed performance attribution.
          </p>
        </div>

        {stats && protocol && (
          <div className="grid gap-3 sm:grid-cols-3">
            <div className="rounded-lg border border-gray-200 bg-white p-4">
              <p className="text-sm text-gray-500">TVL</p>
              <p className="mt-1 text-2xl font-bold">${Number(protocol.tvl).toLocaleString()}</p>
            </div>
            <div className="rounded-lg border border-gray-200 bg-white p-4">
              <p className="text-sm text-gray-500">Total Volume</p>
              <p className="mt-1 text-2xl font-bold">${Number(stats.totalVolumeUSDC).toLocaleString()}</p>
            </div>
            <div className="rounded-lg border border-gray-200 bg-white p-4">
              <p className="text-sm text-gray-500">Active Loans</p>
              <p className="mt-1 text-2xl font-bold">{stats.activeLoans}</p>
            </div>
          </div>
        )}

        <AnalyticsMetricsChart data={portfolio} loading={loading} />

        {stats && protocol && <PerformanceTable stats={stats} protocol={protocol} />}
      </div>
    </main>
  );
}
