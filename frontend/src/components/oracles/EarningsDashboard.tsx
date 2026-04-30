import type { OracleStake } from "@/hooks/useOracles";
import { TrendingUp, Award, AlertTriangle } from "lucide-react";

interface EarningsDashboardProps {
  stake: OracleStake;
  totalConfirmations: number;
}

export function EarningsDashboard({ stake, totalConfirmations }: EarningsDashboardProps) {
  const rewardAmount = Number(stake.rewardAmount);
  const slashedAmount = Number(stake.slashedAmount);
  const stakedAmount = Number(stake.amount);
  const netEarnings = rewardAmount - slashedAmount;
  const avgPerConfirmation = totalConfirmations > 0 ? rewardAmount / totalConfirmations : 0;

  return (
    <div className="rounded-xl border border-gray-200 bg-white p-5 space-y-4">
      <h3 className="text-sm font-semibold text-gray-700">Earnings Dashboard</h3>
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        <div className="rounded-lg bg-blue-50 p-3">
          <p className="text-xs text-blue-600">Staked</p>
          <p className="mt-1 text-lg font-bold text-blue-900">
            ${stakedAmount.toLocaleString()} <span className="text-xs font-normal">{stake.assetCode}</span>
          </p>
        </div>
        <div className="rounded-lg bg-green-50 p-3">
          <div className="flex items-center gap-1 text-xs text-green-600">
            <TrendingUp className="h-3 w-3" /> Total Rewards
          </div>
          <p className="mt-1 text-lg font-bold text-green-900">${rewardAmount.toLocaleString()}</p>
        </div>
        <div className="rounded-lg bg-red-50 p-3">
          <div className="flex items-center gap-1 text-xs text-red-600">
            <AlertTriangle className="h-3 w-3" /> Slashed
          </div>
          <p className="mt-1 text-lg font-bold text-red-900">${slashedAmount.toLocaleString()}</p>
        </div>
        <div className="rounded-lg bg-gray-50 p-3">
          <div className="flex items-center gap-1 text-xs text-gray-500">
            <Award className="h-3 w-3" /> Net Earnings
          </div>
          <p className={`mt-1 text-lg font-bold ${netEarnings >= 0 ? "text-gray-900" : "text-red-700"}`}>
            ${netEarnings.toLocaleString()}
          </p>
        </div>
      </div>
      <div className="rounded-lg bg-gray-50 p-3 text-xs text-gray-600">
        Avg reward per confirmation:{" "}
        <span className="font-semibold">${avgPerConfirmation.toFixed(2)}</span>
        {" · "}Staked since {new Date(stake.stakedAt).toLocaleDateString()}
      </div>
    </div>
  );
}
