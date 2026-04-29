"use client";

import { useState } from "react";
import { Download } from "lucide-react";
import type { PlatformStats, ProtocolAnalytics } from "@/hooks/useAnalytics";

type ExportFormat = "CSV" | "JSON";

const AVAILABLE_METRICS = [
  { key: "totalEscrows", label: "Total Escrows" },
  { key: "fundedEscrows", label: "Funded Escrows" },
  { key: "releasedEscrows", label: "Released Escrows" },
  { key: "totalLoans", label: "Total Loans" },
  { key: "activeLoans", label: "Active Loans" },
  { key: "totalVolumeUSDC", label: "Total Volume (USDC)" },
  { key: "totalUsers", label: "Total Users" },
  { key: "tvl", label: "TVL" },
  { key: "avgInterestRate", label: "Avg Interest Rate" },
  { key: "defaultRate", label: "Default Rate" },
];

interface ReportBuilderProps {
  stats: PlatformStats | null;
  protocol: ProtocolAnalytics | null;
}

export function ReportBuilder({ stats, protocol }: ReportBuilderProps) {
  const [selected, setSelected] = useState<Set<string>>(new Set(["totalEscrows", "totalLoans", "totalVolumeUSDC", "tvl"]));
  const [format, setFormat] = useState<ExportFormat>("CSV");

  function toggle(key: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      next.has(key) ? next.delete(key) : next.add(key);
      return next;
    });
  }

  function buildData(): Record<string, unknown> {
    const combined = { ...stats, ...protocol };
    const result: Record<string, unknown> = {};
    for (const key of selected) {
      if (key in (combined as object)) result[key] = (combined as Record<string, unknown>)[key];
    }
    return result;
  }

  function exportReport() {
    const data = buildData();
    let content: string;
    let mime: string;
    let ext: string;

    if (format === "JSON") {
      content = JSON.stringify(data, null, 2);
      mime = "application/json";
      ext = "json";
    } else {
      const headers = Object.keys(data).join(",");
      const values = Object.values(data).join(",");
      content = `${headers}\n${values}`;
      mime = "text/csv";
      ext = "csv";
    }

    const blob = new Blob([content], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `stellovault-report-${new Date().toISOString().split("T")[0]}.${ext}`;
    a.click();
    URL.revokeObjectURL(url);
  }

  return (
    <div className="rounded-xl border border-gray-200 bg-white p-5 space-y-4">
      <h3 className="text-sm font-semibold text-gray-700">Report Builder</h3>
      <p className="text-xs text-gray-500">Select metrics to include in your export.</p>

      <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
        {AVAILABLE_METRICS.map(({ key, label }) => (
          <label key={key} className="flex cursor-pointer items-center gap-2 rounded-lg border border-gray-200 p-2 text-xs hover:border-blue-300">
            <input
              type="checkbox"
              checked={selected.has(key)}
              onChange={() => toggle(key)}
              className="accent-blue-700"
            />
            {label}
          </label>
        ))}
      </div>

      <div className="flex items-center gap-3">
        <select
          value={format}
          onChange={(e) => setFormat(e.target.value as ExportFormat)}
          className="rounded-lg border border-gray-300 px-3 py-2 text-sm"
        >
          <option>CSV</option>
          <option>JSON</option>
        </select>
        <button
          type="button"
          onClick={exportReport}
          disabled={selected.size === 0 || (!stats && !protocol)}
          className="flex items-center gap-2 rounded-lg bg-blue-900 px-4 py-2 text-sm font-semibold text-white disabled:opacity-50"
        >
          <Download className="h-4 w-4" />
          Export {format}
        </button>
      </div>
    </div>
  );
}
