"use client";

import Link from "next/link";
import { useGovernance } from "@/hooks/useGovernance";
import { ProposalCard } from "@/components/governance/ProposalCard";

export default function GovernancePage() {
  const { proposals } = useGovernance();

  return (
    <div className="min-h-screen bg-zinc-50 dark:bg-black py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-4xl mx-auto">
        <header className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-6 mb-12">
          <div>
            <h1 className="text-4xl font-extrabold tracking-tight text-zinc-900 dark:text-zinc-50">
              Governance
            </h1>
            <p className="mt-2 text-lg text-zinc-600 dark:text-zinc-400">
              Shape the future of the DAO by voting on active proposals.
            </p>
          </div>

          <Link
            href="/governance/new"
            className="flex h-12 items-center justify-center gap-2 rounded-full bg-zinc-900 px-6 font-bold text-white transition-all hover:bg-zinc-800 active:scale-95 dark:bg-white dark:text-zinc-950 dark:hover:bg-zinc-100"
          >
            Create Proposal
          </Link>
        </header>

        <div className="grid grid-cols-1 gap-6">
          {proposals.length > 0 ? (
            proposals.map((proposal) => (
              <ProposalCard key={proposal.id} proposal={proposal} />
            ))
          ) : (
            <div className="text-center py-20 bg-white dark:bg-zinc-900 rounded-3xl border border-dashed border-zinc-200 dark:border-zinc-800">
              <p className="text-zinc-500 font-medium">No proposals found.</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
