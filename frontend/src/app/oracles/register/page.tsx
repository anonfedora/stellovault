"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { ArrowLeft, CheckCircle2, ChevronRight } from "lucide-react";
import { useOracles } from "@/hooks/useOracles";
import type { OracleType } from "@/hooks/useOracles";

const ORACLE_TYPES: OracleType[] = ["GENERAL", "SHIPPING", "QUALITY", "PRICE", "WEATHER"];
const STEPS = ["Oracle Details", "Staking", "Review"];

export default function OracleRegisterPage() {
  const router = useRouter();
  const { registerOracle, loading } = useOracles();
  const [step, setStep] = useState(0);
  const [address, setAddress] = useState("");
  const [oracleType, setOracleType] = useState<OracleType>("GENERAL");
  const [stakeAmount, setStakeAmount] = useState("");
  const [assetCode, setAssetCode] = useState("USDC");
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);

  async function handleSubmit() {
    setError(null);
    try {
      await registerOracle({
        address,
        oracleType,
        stakeAmount: stakeAmount ? Number(stakeAmount) : undefined,
        assetCode,
      });
      setDone(true);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Registration failed");
    }
  }

  if (done) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-gray-50 px-4">
        <div className="max-w-md text-center space-y-4">
          <CheckCircle2 className="mx-auto h-14 w-14 text-green-500" />
          <h2 className="text-2xl font-bold text-gray-900">Oracle Registered</h2>
          <p className="text-gray-600">Your oracle node is now active on the network.</p>
          <button
            type="button"
            onClick={() => router.push("/oracles")}
            className="rounded-lg bg-blue-900 px-6 py-2.5 text-sm font-semibold text-white"
          >
            View Oracle Network
          </button>
        </div>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-24 pt-6 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-xl">
        <Link href="/oracles" className="inline-flex items-center gap-1 text-sm text-gray-500 hover:text-gray-800">
          <ArrowLeft className="h-4 w-4" /> Back to oracles
        </Link>

        <div className="mt-6">
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">Register Oracle</h1>
          <p className="mt-2 text-gray-600">Join the oracle network to earn rewards for data confirmations.</p>
        </div>

        <div className="mt-8 rounded-xl border border-gray-200 bg-white p-6 space-y-6">
          {/* Step indicator */}
          <div className="flex items-center gap-2">
            {STEPS.map((label, i) => (
              <div key={label} className="flex items-center gap-2">
                <div className={`flex h-7 w-7 items-center justify-center rounded-full text-xs font-bold ${i <= step ? "bg-blue-900 text-white" : "bg-gray-200 text-gray-500"}`}>
                  {i + 1}
                </div>
                <span className={`hidden text-xs sm:block ${i === step ? "font-semibold text-gray-900" : "text-gray-400"}`}>{label}</span>
                {i < STEPS.length - 1 && <ChevronRight className="h-4 w-4 text-gray-300" />}
              </div>
            ))}
          </div>

          {step === 0 && (
            <div className="space-y-4">
              <label className="block space-y-1 text-sm font-medium text-gray-700">
                Stellar Address
                <input
                  required
                  value={address}
                  onChange={(e) => setAddress(e.target.value)}
                  placeholder="G..."
                  className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 font-mono text-sm"
                />
              </label>
              <label className="block space-y-1 text-sm font-medium text-gray-700">
                Oracle Type
                <select
                  value={oracleType}
                  onChange={(e) => setOracleType(e.target.value as OracleType)}
                  className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
                >
                  {ORACLE_TYPES.map((t) => <option key={t}>{t}</option>)}
                </select>
              </label>
              <button
                type="button"
                disabled={!address.trim()}
                onClick={() => setStep(1)}
                className="w-full rounded-lg bg-blue-900 py-2.5 text-sm font-semibold text-white disabled:opacity-50"
              >
                Next: Staking
              </button>
            </div>
          )}

          {step === 1 && (
            <div className="space-y-4">
              <p className="text-sm text-gray-600">
                Stake tokens to participate in the oracle network. Higher stakes increase your reputation weight.
              </p>
              <label className="block space-y-1 text-sm font-medium text-gray-700">
                Stake Amount (optional)
                <input
                  type="number"
                  min="0"
                  value={stakeAmount}
                  onChange={(e) => setStakeAmount(e.target.value)}
                  placeholder="e.g. 1000"
                  className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
                />
              </label>
              <label className="block space-y-1 text-sm font-medium text-gray-700">
                Asset Code
                <input
                  value={assetCode}
                  onChange={(e) => setAssetCode(e.target.value)}
                  className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
                />
              </label>
              <div className="flex gap-3">
                <button type="button" onClick={() => setStep(0)} className="flex-1 rounded-lg border border-gray-300 py-2.5 text-sm font-semibold text-gray-700">Back</button>
                <button type="button" onClick={() => setStep(2)} className="flex-1 rounded-lg bg-blue-900 py-2.5 text-sm font-semibold text-white">Next: Review</button>
              </div>
            </div>
          )}

          {step === 2 && (
            <div className="space-y-4">
              <div className="rounded-lg border border-gray-200 bg-gray-50 p-4 text-sm space-y-2">
                <div className="flex justify-between"><span className="text-gray-500">Address</span><span className="font-mono text-xs truncate max-w-[200px]">{address}</span></div>
                <div className="flex justify-between"><span className="text-gray-500">Type</span><span className="font-medium">{oracleType}</span></div>
                <div className="flex justify-between"><span className="text-gray-500">Stake</span><span className="font-medium">{stakeAmount ? `${stakeAmount} ${assetCode}` : "None"}</span></div>
              </div>
              {error && <p className="text-xs text-red-600">{error}</p>}
              <div className="flex gap-3">
                <button type="button" onClick={() => setStep(1)} className="flex-1 rounded-lg border border-gray-300 py-2.5 text-sm font-semibold text-gray-700">Back</button>
                <button type="button" onClick={handleSubmit} disabled={loading} className="flex-1 rounded-lg bg-blue-900 py-2.5 text-sm font-semibold text-white disabled:opacity-50">
                  {loading ? "Registering…" : "Register Oracle"}
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </main>
  );
}
