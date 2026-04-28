"use client";

import { useEffect } from "react";
import Link from "next/link";
import { BarChart2, FileText, TrendingUp } from "lucide-react";
import { AnalyticsMetricsChart } from "@/components/analytics/MetricsChart";
import { RiskAnalysis } from "@/components/analytics/RiskAnalysis";
import { PerformanceTable } from "@/components/analytics/PerformanceTable";
import { useAnalytics } from "@/hooks/useAnalytics";

export default function AnalyticsPage() {
  const { stats, protocol, portfolio, loading, fetchStats, fetchProtocol, fetchPortfolio } = useAnalytics();

  useEffect(() => {
    fetchStats();
    fetchProtocol();
    fetchPortfolio();
  }, [fetchStats, fetchProtocol, fetchPortfolio]);

  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-24 pt-6 text-gray-950 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-7xl space-y-6">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="text-sm font-semibold text-blue-800">Platform intelligence</p>
            <h1 className="mt-2 text-3xl font-bold tracking-tight sm:text-4xl">Analytics</h1>
            <p className="mt-2 max-w-2xl text-gray-600">
              Real-time platform metrics, risk analysis, and performance reporting.
            </p>
          </div>
          <div className="flex gap-3">
            <Link href="/analytics/portfolio" className="inline-flex min-h-10 items-center gap-2 rounded-lg border border-gray-300 bg-white px-4 text-sm font-semibold text-gray-700">
              <TrendingUp className="h-4 w-4" /> Portfolio
            </Link>
            <Link href="/analytics/reports" className="inline-flex min-h-10 items-center gap-2 rounded-lg bg-blue-900 px-4 text-sm font-semibold text-white">
              <FileText className="h-4 w-4" /> Reports
            </Link>
          </div>
        </div>

        {/* KPI cards */}
        {stats && (
          <section className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            {[
              { label: "Total Volume", value: `$${Number(stats.totalVolumeUSDC).toLocaleString()}`, sub: "USDC" },
              { label: "Active Loans", value: String(stats.activeLoans), sub: `of ${stats.totalLoans} total` },
              { label: "Total Users", value: String(stats.totalUsers), sub: `${stats.activeWallets} active wallets` },
              { label: "Governance", value: String(stats.governanceProposals), sub: `${(stats.participationRate * 100).toFixed(0)}% participation` },
            ].map(({ label, value, sub }) => (
              <div key={label} className="rounded-lg border border-gray-200 bg-white p-4">
                <p className="text-sm text-gray-500">{label}</p>
                <p className="mt-1 text-2xl font-bold">{value}</p>
                <p className="text-xs text-gray-400">{sub}</p>
              </div>
            ))}
          </section>
        )}

        {/* Charts */}
        <AnalyticsMetricsChart data={portfolio} loading={loading} />

        {/* Risk + Performance */}
        {stats && protocol && (
          <div className="grid gap-6 lg:grid-cols-2">
            <RiskAnalysis stats={stats} protocol={protocol} />
            <div className="rounded-xl border border-gray-200 bg-white p-5">
              <h3 className="mb-3 text-sm font-semibold text-gray-700">Protocol Summary</h3>
              <div className="space-y-3 text-sm">
                {[
                  { label: "TVL", value: `$${Number(protocol.tvl).toLocaleString()}` },
                  { label: "Avg Interest Rate", value: `${(Number(protocol.avgInterestRate) * 100).toFixed(2)}%` },
                  { label: "Default Rate", value: `${(protocol.defaultRate * 100).toFixed(1)}%` },
                  { label: "Funded Escrows", value: String(stats.fundedEscrows) },
                  { label: "Disputed Escrows", value: String(stats.disputedEscrows) },
                ].map(({ label, value }) => (
                  <div key={label} className="flex justify-between border-b border-gray-50 pb-2">
                    <span className="text-gray-500">{label}</span>
                    <span className="font-semibold">{value}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {stats && protocol && <PerformanceTable stats={stats} protocol={protocol} />}

        {!stats && !loading && (
          <div className="flex items-center justify-center gap-2 rounded-lg border border-dashed border-gray-300 bg-white p-12 text-gray-400">
            <BarChart2 className="h-6 w-6" />
            <span>No analytics data available.</span>
          </div>
        )}
      </div>
    </main>
  );
}
