"use client";

import { useMemo, useState } from "react";
import {
  Area,
  AreaChart,
  CartesianGrid,
  Legend,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { MetricsPoint } from "@/hooks/useDashboard";

interface MetricsChartProps {
  data: MetricsPoint[];
  loading?: boolean;
}

type Range = "7d" | "30d" | "90d" | "all";

const RANGE_LABEL: Record<Range, string> = {
  "7d": "7D",
  "30d": "30D",
  "90d": "90D",
  all: "All",
};

const RANGE_DAYS: Record<Range, number | null> = {
  "7d": 7,
  "30d": 30,
  "90d": 90,
  all: null,
};

const tickFormatter = (value: string) => {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
};

/**
 * Filter metrics points so only those within `days` of the latest data point
 * are returned. Using the most recent point as the anchor keeps this pure and
 * stable across renders (no Date.now() needed in render bodies).
 */
export const filterMetricsByRange = (
  data: MetricsPoint[],
  range: Range,
): MetricsPoint[] => {
  const days = RANGE_DAYS[range];
  if (days === null || data.length === 0) return data;
  const anchor = new Date(data[data.length - 1].date).getTime();
  const cutoff = anchor - days * 24 * 60 * 60 * 1000;
  const slice = data.filter((p) => new Date(p.date).getTime() >= cutoff);
  return slice.length === 0 ? data.slice(-1) : slice;
};

export const MetricsChart = ({ data, loading = false }: MetricsChartProps) => {
  const [range, setRange] = useState<Range>("30d");
  const filtered = useMemo(
    () => filterMetricsByRange(data, range),
    [data, range],
  );

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
      <div className="flex items-start justify-between mb-4">
        <div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Performance
          </h2>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            Outstanding balance and cumulative repayments over time.
          </p>
        </div>
        <div className="inline-flex bg-gray-100 dark:bg-gray-700 rounded-lg p-1 text-xs">
          {(Object.keys(RANGE_LABEL) as Range[]).map((key) => (
            <button
              key={key}
              type="button"
              onClick={() => setRange(key)}
              className={`px-2.5 py-1 rounded-md font-medium transition-colors ${
                range === key
                  ? "bg-white dark:bg-gray-800 text-blue-600 dark:text-blue-300 shadow-sm"
                  : "text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100"
              }`}
            >
              {RANGE_LABEL[key]}
            </button>
          ))}
        </div>
      </div>

      <div className="relative h-[280px]">
        {loading && (
          <div className="absolute inset-0 z-10 flex items-center justify-center bg-white/60 dark:bg-gray-800/60 rounded-lg">
            <div className="w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin" />
          </div>
        )}

        {!loading && filtered.length === 0 ? (
          <div className="h-full flex items-center justify-center text-sm text-gray-500 dark:text-gray-400">
            No metrics available yet.
          </div>
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart
              data={filtered}
              margin={{ top: 10, right: 12, left: -10, bottom: 0 }}
            >
              <defs>
                <linearGradient id="outstandingFill" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#2563eb" stopOpacity={0.35} />
                  <stop offset="100%" stopColor="#2563eb" stopOpacity={0} />
                </linearGradient>
                <linearGradient id="repaidFill" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#10b981" stopOpacity={0.35} />
                  <stop offset="100%" stopColor="#10b981" stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#e5e7eb" />
              <XAxis
                dataKey="date"
                tickFormatter={tickFormatter}
                tick={{ fontSize: 12, fill: "#6b7280" }}
                axisLine={false}
                tickLine={false}
              />
              <YAxis
                tick={{ fontSize: 12, fill: "#6b7280" }}
                axisLine={false}
                tickLine={false}
                tickFormatter={(v: number) =>
                  v >= 1000 ? `$${Math.round(v / 1000)}k` : `$${v}`
                }
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: "#ffffff",
                  border: "1px solid #e5e7eb",
                  borderRadius: 8,
                }}
                labelFormatter={(label) => tickFormatter(String(label))}
                formatter={(value, name) => [
                  typeof value === "number"
                    ? `$${Math.round(value).toLocaleString()}`
                    : String(value ?? ""),
                  name,
                ]}
              />
              <Legend wrapperStyle={{ fontSize: 12 }} />
              <Area
                type="monotone"
                dataKey="outstanding"
                stroke="#2563eb"
                strokeWidth={2}
                fill="url(#outstandingFill)"
                name="Outstanding"
              />
              <Area
                type="monotone"
                dataKey="repaid"
                stroke="#10b981"
                strokeWidth={2}
                fill="url(#repaidFill)"
                name="Repaid"
              />
            </AreaChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
};
