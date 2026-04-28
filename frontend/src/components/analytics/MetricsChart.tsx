"use client";

import { useMemo, useState } from "react";
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Legend,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { PortfolioMetric } from "@/hooks/useAnalytics";

type Range = "7d" | "30d" | "90d";
type ChartType = "tvl" | "volume" | "activity";

const RANGE_DAYS: Record<Range, number> = { "7d": 7, "30d": 30, "90d": 90 };

interface MetricsChartProps {
  data: PortfolioMetric[];
  loading?: boolean;
}

export function AnalyticsMetricsChart({ data, loading = false }: MetricsChartProps) {
  const [range, setRange] = useState<Range>("30d");
  const [chartType, setChartType] = useState<ChartType>("tvl");

  const filtered = useMemo(() => {
    if (!data.length) return [];
    const days = RANGE_DAYS[range];
    return data.slice(-days).map((p) => ({
      ...p,
      label: new Date(p.date).toLocaleDateString("en-US", { month: "short", day: "numeric" }),
    }));
  }, [data, range]);

  return (
    <div className="rounded-xl border border-gray-200 bg-white p-5">
      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <div className="flex gap-1 rounded-lg bg-gray-100 p-1 text-xs">
          {(["tvl", "volume", "activity"] as ChartType[]).map((t) => (
            <button
              key={t}
              type="button"
              onClick={() => setChartType(t)}
              className={`rounded-md px-3 py-1 font-medium capitalize transition ${chartType === t ? "bg-white text-blue-700 shadow-sm" : "text-gray-500 hover:text-gray-800"}`}
            >
              {t === "tvl" ? "TVL" : t}
            </button>
          ))}
        </div>
        <div className="flex gap-1 rounded-lg bg-gray-100 p-1 text-xs">
          {(["7d", "30d", "90d"] as Range[]).map((r) => (
            <button
              key={r}
              type="button"
              onClick={() => setRange(r)}
              className={`rounded-md px-3 py-1 font-medium transition ${range === r ? "bg-white text-blue-700 shadow-sm" : "text-gray-500 hover:text-gray-800"}`}
            >
              {r.toUpperCase()}
            </button>
          ))}
        </div>
      </div>

      <div className="relative h-64">
        {loading && (
          <div className="absolute inset-0 z-10 flex items-center justify-center bg-white/60">
            <div className="h-5 w-5 animate-spin rounded-full border-2 border-blue-600 border-t-transparent" />
          </div>
        )}
        {!loading && filtered.length === 0 ? (
          <div className="flex h-full items-center justify-center text-sm text-gray-400">No data available.</div>
        ) : chartType === "activity" ? (
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={filtered} margin={{ top: 4, right: 8, left: -16, bottom: 0 }}>
              <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#f0f0f0" />
              <XAxis dataKey="label" tick={{ fontSize: 11 }} axisLine={false} tickLine={false} />
              <YAxis tick={{ fontSize: 11 }} axisLine={false} tickLine={false} />
              <Tooltip contentStyle={{ borderRadius: 8, fontSize: 12 }} />
              <Legend wrapperStyle={{ fontSize: 11 }} />
              <Bar dataKey="loans" fill="#2563eb" name="Loans" radius={[3, 3, 0, 0]} />
              <Bar dataKey="escrows" fill="#10b981" name="Escrows" radius={[3, 3, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={filtered} margin={{ top: 4, right: 8, left: -16, bottom: 0 }}>
              <defs>
                <linearGradient id="metricFill" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#2563eb" stopOpacity={0.3} />
                  <stop offset="100%" stopColor="#2563eb" stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#f0f0f0" />
              <XAxis dataKey="label" tick={{ fontSize: 11 }} axisLine={false} tickLine={false} />
              <YAxis
                tick={{ fontSize: 11 }}
                axisLine={false}
                tickLine={false}
                tickFormatter={(v: number) => `$${v >= 1000000 ? `${(v / 1000000).toFixed(1)}M` : v >= 1000 ? `${Math.round(v / 1000)}k` : v}`}
              />
              <Tooltip
                formatter={(v) => [`$${Number(Array.isArray(v) ? v[0] : (v ?? 0)).toLocaleString()}`, chartType === "tvl" ? "TVL" : "Volume"]}
                contentStyle={{ borderRadius: 8, fontSize: 12 }}
              />
              <Area
                type="monotone"
                dataKey={chartType}
                stroke="#2563eb"
                strokeWidth={2}
                fill="url(#metricFill)"
              />
            </AreaChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
}
