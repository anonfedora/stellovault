"use client";

import Link from "next/link";
import { Plus, Search } from "lucide-react";
import { useEffect, useState } from "react";
import { CollateralCard } from "@/components/collateral/CollateralCard";
import { useCollateral } from "@/hooks/useCollateral";
import type { CollateralStatus } from "@/hooks/useCollateral";

const STATUSES: Array<CollateralStatus | "ALL"> = ["ALL", "LOCKED", "RELEASED", "LIQUIDATED"];

export default function CollateralPage() {
  const { collaterals, loading, fetchCollaterals } = useCollateral();
  const [statusFilter, setStatusFilter] = useState<CollateralStatus | "ALL">("ALL");
  const [search, setSearch] = useState("");

  useEffect(() => {
    fetchCollaterals(undefined, statusFilter === "ALL" ? undefined : statusFilter);
  }, [fetchCollaterals, statusFilter]);

  const filtered = collaterals.filter((c) => {
    if (!search) return true;
    const q = search.toLowerCase();
    return (
      c.description?.toLowerCase().includes(q) ||
      c.issuer?.toLowerCase().includes(q) ||
      c.assetType?.toLowerCase().includes(q) ||
      c.id.toLowerCase().includes(q)
    );
  });

  const totalValue = collaterals.reduce((sum, c) => sum + Number(c.amount), 0);
  const lockedCount = collaterals.filter((c) => c.status === "LOCKED").length;

  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-24 pt-6 text-gray-950 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="text-sm font-semibold text-blue-800">Collateral management</p>
            <h1 className="mt-2 text-3xl font-bold tracking-tight sm:text-4xl">Collateral Tokens</h1>
            <p className="mt-2 max-w-2xl text-gray-600">
              Tokenize real-world assets as Stellar collateral and manage their lifecycle.
            </p>
          </div>
          <Link
            href="/collateral/new"
            className="inline-flex min-h-12 items-center justify-center gap-2 rounded-lg bg-blue-900 px-4 font-semibold text-white"
          >
            <Plus className="h-4 w-4" />
            Tokenize Asset
          </Link>
        </div>

        <section className="mt-6 grid gap-3 sm:grid-cols-3">
          <div className="rounded-lg border border-gray-200 bg-white p-4">
            <p className="text-sm text-gray-500">Total collateral value</p>
            <p className="mt-1 text-2xl font-bold">${totalValue.toLocaleString()}</p>
          </div>
          <div className="rounded-lg border border-gray-200 bg-white p-4">
            <p className="text-sm text-gray-500">Locked assets</p>
            <p className="mt-1 text-2xl font-bold">{lockedCount}</p>
          </div>
          <div className="rounded-lg border border-gray-200 bg-white p-4">
            <p className="text-sm text-gray-500">Total assets</p>
            <p className="mt-1 text-2xl font-bold">{collaterals.length}</p>
          </div>
        </section>

        <div className="mt-6 flex flex-col gap-3 rounded-lg border border-gray-200 bg-white p-4 sm:flex-row">
          <div className="relative flex-1">
            <Search className="pointer-events-none absolute left-3 top-3 h-4 w-4 text-gray-400" />
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search by description, issuer, or ID"
              className="min-h-10 w-full rounded-lg border border-gray-300 pl-9 pr-3 text-sm"
            />
          </div>
          <div className="flex gap-1 rounded-lg bg-gray-100 p-1 text-xs">
            {STATUSES.map((s) => (
              <button
                key={s}
                type="button"
                onClick={() => setStatusFilter(s)}
                className={`rounded-md px-3 py-1.5 font-medium transition ${statusFilter === s ? "bg-white text-blue-700 shadow-sm" : "text-gray-500 hover:text-gray-800"}`}
              >
                {s}
              </button>
            ))}
          </div>
        </div>

        <section className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {loading ? (
            Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="h-40 animate-pulse rounded-lg bg-gray-200" />
            ))
          ) : filtered.length > 0 ? (
            filtered.map((c) => <CollateralCard key={c.id} collateral={c} />)
          ) : (
            <div className="col-span-full rounded-lg border border-dashed border-gray-300 bg-white p-8 text-center text-gray-500">
              No collateral assets found.
            </div>
          )}
        </section>
      </div>
    </main>
  );
}
