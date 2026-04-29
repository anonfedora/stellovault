"use client";

import { useState } from "react";
import { ChevronDown, ChevronUp } from "lucide-react";
import type { PlatformStats, ProtocolAnalytics } from "@/hooks/useAnalytics";

interface Row {
  metric: string;
  value: string;
  category: string;
  trend?: "up" | "down" | "neutral";
}

function buildRows(stats: PlatformStats, protocol: ProtocolAnalytics): Row[] {
  return [
    { metric: "Total Volume (USDC)", value: `$${Number(stats.totalVolumeUSDC).toLocaleString()}`, category: "Volume", trend: "up" },
    { metric: "TVL", value: `$${Number(protocol.tvl).toLocaleString()}`, category: "Volume", trend: "up" },
    { metric: "Total Escrows", value: String(stats.totalEscrows), category: "Escrows", trend: "neutral" },
    { metric: "Funded Escrows", value: String(stats.fundedEscrows), category: "Escrows", trend: "up" },
    { metric: "Released Escrows", value: String(stats.releasedEscrows), category: "Escrows", trend: "up" },
    { metric: "Disputed Escrows", value: String(stats.disputedEscrows), category: "Escrows", trend: "down" },
    { metric: "Total Loans", value: String(stats.totalLoans), category: "Loans", trend: "neutral" },
    { metric: "Active Loans", value: String(stats.activeLoans), category: "Loans", trend: "up" },
    { metric: "Defaulted Loans", value: String(stats.defaultedLoans), category: "Loans", trend: "down" },
    { metric: "Default Rate", value: `${(protocol.defaultRate * 100).toFixed(1)}%`, category: "Risk", trend: protocol.defaultRate > 0.05 ? "down" : "neutral" },
    { metric: "Avg Interest Rate", value: `${(Number(protocol.avgInterestRate) * 100).toFixed(2)}%`, category: "Risk", trend: "neutral" },
    { metric: "Total Users", value: String(stats.totalUsers), category: "Users", trend: "up" },
    { metric: "Active Wallets", value: String(stats.activeWallets), category: "Users", trend: "up" },
    { metric: "Governance Proposals", value: String(stats.governanceProposals), category: "Governance", trend: "neutral" },
    { metric: "Participation Rate", value: `${(stats.participationRate * 100).toFixed(0)}%`, category: "Governance", trend: "neutral" },
  ];
}

const TREND_ICONS = {
  up: <ChevronUp className="h-3 w-3 text-green-600" />,
  down: <ChevronDown className="h-3 w-3 text-red-500" />,
  neutral: <span className="h-3 w-3 inline-block" />,
};

interface PerformanceTableProps {
  stats: PlatformStats;
  protocol: ProtocolAnalytics;
}

export function PerformanceTable({ stats, protocol }: PerformanceTableProps) {
  const [categoryFilter, setCategoryFilter] = useState("All");
  const rows = buildRows(stats, protocol);
  const categories = ["All", ...Array.from(new Set(rows.map((r) => r.category)))];
  const filtered = categoryFilter === "All" ? rows : rows.filter((r) => r.category === categoryFilter);

  return (
    <div className="rounded-xl border border-gray-200 bg-white p-5 space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h3 className="text-sm font-semibold text-gray-700">Performance Metrics</h3>
        <div className="flex gap-1 rounded-lg bg-gray-100 p-1 text-xs">
          {categories.map((c) => (
            <button
              key={c}
              type="button"
              onClick={() => setCategoryFilter(c)}
              className={`rounded-md px-2.5 py-1 font-medium transition ${categoryFilter === c ? "bg-white text-blue-700 shadow-sm" : "text-gray-500 hover:text-gray-800"}`}
            >
              {c}
            </button>
          ))}
        </div>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-100 text-left text-xs text-gray-400">
              <th className="pb-2 font-medium">Metric</th>
              <th className="pb-2 font-medium">Category</th>
              <th className="pb-2 text-right font-medium">Value</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((row) => (
              <tr key={row.metric} className="border-b border-gray-50 hover:bg-gray-50">
                <td className="py-2 font-medium text-gray-800">{row.metric}</td>
                <td className="py-2 text-gray-400">{row.category}</td>
                <td className="py-2 text-right">
                  <span className="inline-flex items-center gap-1 font-semibold text-gray-900">
                    {TREND_ICONS[row.trend ?? "neutral"]}
                    {row.value}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
