"use client";

import {
  PolarAngleAxis,
  PolarGrid,
  Radar,
  RadarChart,
  ResponsiveContainer,
  Tooltip,
} from "recharts";
import type { OracleReputation } from "@/hooks/useOracles";

interface ReputationChartProps {
  reputation: OracleReputation;
}

export function ReputationChart({ reputation }: ReputationChartProps) {
  const data = [
    { metric: "Score", value: reputation.score },
    { metric: "Accuracy", value: Number(reputation.accuracy) },
    { metric: "Reliability", value: Number(reputation.reliability) },
    { metric: "Responsiveness", value: Number(reputation.responsiveness) },
  ];

  return (
    <div className="rounded-xl border border-gray-200 bg-white p-5">
      <h3 className="mb-1 text-sm font-semibold text-gray-700">Reputation Breakdown</h3>
      <div className="grid grid-cols-2 gap-2 mb-4 text-xs">
        <div className="rounded-lg bg-gray-50 p-2">
          <p className="text-gray-400">Total Votes</p>
          <p className="font-bold text-gray-900">{reputation.totalVotes}</p>
        </div>
        <div className="rounded-lg bg-gray-50 p-2">
          <p className="text-gray-400">Positive / Negative</p>
          <p className="font-bold text-gray-900">
            {reputation.positiveVotes} / {reputation.negativeVotes}
          </p>
        </div>
      </div>
      <div className="h-52">
        <ResponsiveContainer width="100%" height="100%">
          <RadarChart data={data}>
            <PolarGrid />
            <PolarAngleAxis dataKey="metric" tick={{ fontSize: 11 }} />
            <Radar
              dataKey="value"
              stroke="#2563eb"
              fill="#2563eb"
              fillOpacity={0.25}
              strokeWidth={2}
            />
            <Tooltip formatter={(v) => [`${Number(Array.isArray(v) ? v[0] : (v ?? 0)).toFixed(1)}`, ""]} contentStyle={{ fontSize: 12, borderRadius: 8 }} />
          </RadarChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
