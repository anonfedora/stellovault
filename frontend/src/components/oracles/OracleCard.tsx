import Link from "next/link";
import { Activity, CheckCircle2, XCircle } from "lucide-react";
import type { Oracle } from "@/hooks/useOracles";

const TYPE_COLORS: Record<string, string> = {
  SHIPPING: "bg-blue-100 text-blue-800",
  QUALITY: "bg-purple-100 text-purple-800",
  PRICE: "bg-amber-100 text-amber-800",
  WEATHER: "bg-cyan-100 text-cyan-800",
  GENERAL: "bg-gray-100 text-gray-700",
};

export function OracleCard({ oracle }: { oracle: Oracle }) {
  const successRate =
    oracle.totalConfirmations > 0
      ? Math.round((oracle.successfulConfirmations / oracle.totalConfirmations) * 100)
      : 0;

  return (
    <Link
      href={`/oracles/${oracle.id}`}
      className="block rounded-lg border border-gray-200 bg-white p-4 shadow-sm transition hover:border-blue-300 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 sm:p-5"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 space-y-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className={`rounded-full px-2 py-0.5 text-xs font-semibold ${TYPE_COLORS[oracle.oracleType] ?? TYPE_COLORS.GENERAL}`}>
              {oracle.oracleType}
            </span>
            {oracle.isActive ? (
              <span className="flex items-center gap-1 text-xs font-medium text-green-700">
                <CheckCircle2 className="h-3 w-3" /> Active
              </span>
            ) : (
              <span className="flex items-center gap-1 text-xs font-medium text-gray-400">
                <XCircle className="h-3 w-3" /> Inactive
              </span>
            )}
          </div>
          <p className="truncate font-mono text-xs text-gray-500">{oracle.address}</p>
        </div>
        {oracle.reputation && (
          <div className="shrink-0 text-right">
            <p className="text-2xl font-bold text-blue-900">{oracle.reputation.score}</p>
            <p className="text-xs text-gray-400">score</p>
          </div>
        )}
      </div>

      <div className="mt-4 grid grid-cols-3 gap-3 text-center text-xs">
        <div>
          <p className="font-bold text-gray-900">{oracle.totalConfirmations}</p>
          <p className="text-gray-400">Confirmations</p>
        </div>
        <div>
          <p className="font-bold text-gray-900">{successRate}%</p>
          <p className="text-gray-400">Success rate</p>
        </div>
        <div>
          <p className="font-bold text-gray-900">
            {oracle.stake ? `$${Number(oracle.stake.amount).toLocaleString()}` : "—"}
          </p>
          <p className="text-gray-400">Staked</p>
        </div>
      </div>

      {oracle.lastActiveAt && (
        <p className="mt-3 flex items-center gap-1 text-xs text-gray-400">
          <Activity className="h-3 w-3" />
          Last active {new Date(oracle.lastActiveAt).toLocaleDateString()}
        </p>
      )}
    </Link>
  );
}
