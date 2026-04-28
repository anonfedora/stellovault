"use client";

import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";
import type { PlatformStats, ProtocolAnalytics } from "@/hooks/useAnalytics";

const COLORS = ["#2563eb", "#10b981", "#f59e0b", "#ef4444"];

interface RiskAnalysisProps {
  stats: PlatformStats;
  protocol: ProtocolAnalytics;
}

export function RiskAnalysis({ stats, protocol }: RiskAnalysisProps) {
  const escrowData = [
    { name: "Funded", value: stats.fundedEscrows },
    { name: "Released", value: stats.releasedEscrows },
    { name: "Disputed", value: stats.disputedEscrows },
    { name: "Other", value: Math.max(0, stats.totalEscrows - stats.fundedEscrows - stats.releasedEscrows - stats.disputedEscrows) },
  ].filter((d) => d.value > 0);

  const defaultRate = (protocol.defaultRate * 100).toFixed(1);
  const avgRate = (Number(protocol.avgInterestRate) * 100).toFixed(2);

  return (
    <div className="rounded-xl border border-gray-200 bg-white p-5 space-y-4">
      <h3 className="text-sm font-semibold text-gray-700">Risk Analysis</h3>
      <div className="grid gap-4 sm:grid-cols-2">
        <div>
          <p className="mb-2 text-xs text-gray-500">Escrow Distribution</p>
          <div className="h-40">
            <ResponsiveContainer width="100%" height="100%">
              <PieChart>
                <Pie data={escrowData} dataKey="value" nameKey="name" cx="50%" cy="50%" outerRadius={60} label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`} labelLine={false}>
                  {escrowData.map((_, i) => (
                    <Cell key={i} fill={COLORS[i % COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip contentStyle={{ fontSize: 12, borderRadius: 8 }} />
              </PieChart>
            </ResponsiveContainer>
          </div>
        </div>
        <div className="space-y-3">
          <div className="rounded-lg bg-amber-50 p-3">
            <p className="text-xs text-amber-600">Default Rate</p>
            <p className="text-2xl font-bold text-amber-900">{defaultRate}%</p>
            <p className="text-xs text-amber-600">{stats.defaultedLoans} of {stats.totalLoans} loans</p>
          </div>
          <div className="rounded-lg bg-blue-50 p-3">
            <p className="text-xs text-blue-600">Avg Interest Rate</p>
            <p className="text-2xl font-bold text-blue-900">{avgRate}%</p>
          </div>
          <div className="rounded-lg bg-red-50 p-3">
            <p className="text-xs text-red-600">Disputed Escrows</p>
            <p className="text-2xl font-bold text-red-900">{stats.disputedEscrows}</p>
          </div>
        </div>
      </div>
    </div>
  );
}
