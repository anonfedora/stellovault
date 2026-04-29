"use client";

import { useEffect } from "react";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { ReportBuilder } from "@/components/analytics/ReportBuilder";
import { useAnalytics } from "@/hooks/useAnalytics";

export default function ReportsPage() {
  const { stats, protocol, loading, fetchStats, fetchProtocol } = useAnalytics();

  useEffect(() => {
    fetchStats();
    fetchProtocol();
  }, [fetchStats, fetchProtocol]);

  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-24 pt-6 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-3xl space-y-6">
        <Link href="/analytics" className="inline-flex items-center gap-1 text-sm text-gray-500 hover:text-gray-800">
          <ArrowLeft className="h-4 w-4" /> Back to analytics
        </Link>

        <div>
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">Report Builder</h1>
          <p className="mt-2 text-gray-600">
            Select metrics and export custom reports in CSV or JSON format.
          </p>
        </div>

        {loading ? (
          <div className="h-64 animate-pulse rounded-xl bg-gray-200" />
        ) : (
          <ReportBuilder stats={stats} protocol={protocol} />
        )}
      </div>
    </main>
  );
}
