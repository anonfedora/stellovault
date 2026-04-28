"use client";

import Link from "next/link";
import { Plus, RefreshCcw } from "lucide-react";
import { useEffect, useState } from "react";
import { OracleCard } from "@/components/oracles/OracleCard";
import { useOracles } from "@/hooks/useOracles";

export default function OraclesPage() {
  const { oracles, networkStatus, loading, fetchOracles, fetchNetworkStatus } = useOracles();
  const [activeFilter, setActiveFilter] = useState<boolean | undefined>(undefined);

  useEffect(() => {
    fetchOracles(activeFilter);
    fetchNetworkStatus();
  }, [fetchOracles, fetchNetworkStatus, activeFilter]);

  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-24 pt-6 text-gray-950 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="text-sm font-semibold text-blue-800">Oracle network</p>
            <h1 className="mt-2 text-3xl font-bold tracking-tight sm:text-4xl">Oracle Nodes</h1>
            <p className="mt-2 max-w-2xl text-gray-600">
              Monitor oracle operators, reputation scores, and network health.
            </p>
          </div>
          <Link
            href="/oracles/register"
            className="inline-flex min-h-12 items-center justify-center gap-2 rounded-lg bg-blue-900 px-4 font-semibold text-white"
          >
            <Plus className="h-4 w-4" />
            Register Oracle
          </Link>
        </div>

        {/* Network status */}
        {networkStatus && (
          <section className="mt-6 grid gap-3 sm:grid-cols-4">
            <div className="rounded-lg border border-gray-200 bg-white p-4">
              <p className="text-sm text-gray-500">Active oracles</p>
              <p className="mt-1 text-2xl font-bold">{networkStatus.activeOracles}</p>
            </div>
            <div className="rounded-lg border border-gray-200 bg-white p-4">
              <p className="text-sm text-gray-500">Network health</p>
              <p className="mt-1 text-2xl font-bold">{networkStatus.networkHealth.toFixed(0)}%</p>
            </div>
            <div className="rounded-lg border border-gray-200 bg-white p-4">
              <p className="text-sm text-gray-500">Avg reputation</p>
              <p className="mt-1 text-2xl font-bold">{networkStatus.averageReputation.score.toFixed(0)}</p>
            </div>
            <div className="rounded-lg border border-gray-200 bg-white p-4">
              <p className="flex items-center gap-1 text-sm text-gray-500">
                <RefreshCcw className="h-3 w-3" /> Recent activity
              </p>
              <p className="mt-1 text-2xl font-bold">{networkStatus.recentActivity}</p>
            </div>
          </section>
        )}

        {/* Filter */}
        <div className="mt-6 flex gap-1 rounded-lg border border-gray-200 bg-white p-3 w-fit">
          {([undefined, true, false] as Array<boolean | undefined>).map((v) => (
            <button
              key={String(v)}
              type="button"
              onClick={() => setActiveFilter(v)}
              className={`rounded-md px-3 py-1.5 text-xs font-medium transition ${activeFilter === v ? "bg-blue-900 text-white" : "text-gray-500 hover:text-gray-800"}`}
            >
              {v === undefined ? "All" : v ? "Active" : "Inactive"}
            </button>
          ))}
        </div>

        <section className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {loading ? (
            Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="h-44 animate-pulse rounded-lg bg-gray-200" />
            ))
          ) : oracles.length > 0 ? (
            oracles.map((o) => <OracleCard key={o.id} oracle={o} />)
          ) : (
            <div className="col-span-full rounded-lg border border-dashed border-gray-300 bg-white p-8 text-center text-gray-500">
              No oracles found.
            </div>
          )}
        </section>
      </div>
    </main>
  );
}
