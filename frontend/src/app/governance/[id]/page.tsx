"use client";

import { useParams } from "next/navigation";
import { useGovernance } from "@/hooks/useGovernance";
import { VoteTallyBar } from "@/components/governance/VoteTallyBar";
import { VoteButton } from "@/components/governance/VoteButton";
import Link from "next/link";
import { useMemo } from "react";

export default function ProposalDetailPage() {
  const params = useParams();
  const id = params?.id as string;
  const { proposals, vote, votingState, userVoted } = useGovernance();

  // Derived state for the current proposal
  const proposal = useMemo(() => {
    return proposals.find((p) => p.id === id) || null;
  }, [proposals, id]);

  const error = !proposal && id ? "Proposal not found" : null;

  const handleVote = async (type: "For" | "Against" | "Abstain") => {
    await vote(id, type);
  };

  if (error && !proposal) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-zinc-50 dark:bg-black">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-4">{error}</h2>
          <Link href="/governance" className="text-blue-600 hover:underline">
            Back to Governance
          </Link>
        </div>
      </div>
    );
  }

  if (!proposal) return null;

  const showSuccessToast =
    userVoted[id] && !votingState.loading && !votingState.error;
  const showErrorToast = votingState.id === id && votingState.error;

  return (
    <div className="min-h-screen bg-zinc-50 dark:bg-black py-12 px-4 sm:px-6">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <Link
            href="/governance"
            className="inline-flex items-center gap-2 text-sm font-semibold text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-50 transition-colors"
          >
            ← Back to Proposals
          </Link>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          <div className="lg:col-span-2 space-y-6">
            <div className="bg-white dark:bg-zinc-900 p-8 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm">
              <div className="flex items-center gap-3 mb-6">
                <span
                  className={`px-3 py-1 rounded-full text-xs font-bold tracking-tight bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-400`}
                >
                  {proposal.status}
                </span>
                <span className="text-xs font-medium text-zinc-400">
                  ID: {proposal.id}
                </span>
              </div>

              <h1 className="text-3xl font-extrabold mb-4 text-zinc-900 dark:text-zinc-50 leading-tight">
                {proposal.title}
              </h1>

              <div className="flex items-center gap-4 mb-8 pb-8 border-b border-zinc-100 dark:border-zinc-800">
                <div className="p-2 bg-zinc-100 dark:bg-zinc-800 rounded-lg">
                  <div className="h-10 w-10 rounded-full bg-gradient-to-tr from-blue-500 to-emerald-500" />
                </div>
                <div>
                  <p className="text-xs font-bold text-zinc-400 uppercase tracking-widest">
                    Creator
                  </p>
                  <p className="font-mono text-sm font-semibold text-zinc-700 dark:text-zinc-300">
                    {proposal.creator}
                  </p>
                </div>
              </div>

              <div className="prose dark:prose-invert max-w-none">
                <p className="text-zinc-700 dark:text-zinc-300 whitespace-pre-wrap leading-relaxed">
                  {proposal.description}
                </p>
              </div>
            </div>
          </div>

          <aside className="space-y-6">
            <div className="bg-white dark:bg-zinc-900 p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm">
              <h2 className="text-lg font-bold mb-6 text-zinc-900 dark:text-zinc-50">
                Vote Results
              </h2>
              <VoteTallyBar votes={proposal.votes} />
              <div className="mt-8 space-y-3">
                <div className="flex justify-between text-sm">
                  <span className="text-zinc-500">Threshold</span>
                  <span className="font-semibold">66.7%</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-zinc-500">Quorum</span>
                  <span className="font-semibold text-emerald-500">
                    Reached
                  </span>
                </div>
              </div>
            </div>

            {proposal.status === "OPEN" && (
              <div className="bg-white dark:bg-zinc-900 p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm relative overflow-hidden">
                <h2 className="text-lg font-bold mb-6 text-zinc-900 dark:text-zinc-50">
                  Cast Your Vote
                </h2>

                {showErrorToast && (
                  <div className="mb-4 p-4 rounded-xl bg-rose-50 border border-rose-100 dark:bg-rose-950/20 dark:border-rose-900/30 text-rose-600 dark:text-rose-400 text-sm font-medium animate-in slide-in-from-top duration-300">
                    {votingState.error}
                  </div>
                )}

                {showSuccessToast && (
                  <div className="mb-4 p-4 rounded-xl bg-emerald-50 border border-emerald-100 dark:bg-emerald-950/20 dark:border-emerald-900/30 text-emerald-600 dark:text-emerald-400 text-sm font-medium animate-in slide-in-from-top duration-300">
                    Vote cast successfully!
                  </div>
                )}

                <div className="space-y-3">
                  <VoteButton
                    type="For"
                    onClick={() => handleVote("For")}
                    isLoading={votingState.id === id && votingState.loading}
                    disabled={userVoted[id] || false}
                  />
                  <VoteButton
                    type="Against"
                    onClick={() => handleVote("Against")}
                    isLoading={votingState.id === id && votingState.loading}
                    disabled={userVoted[id] || false}
                  />
                  <VoteButton
                    type="Abstain"
                    onClick={() => handleVote("Abstain")}
                    isLoading={votingState.id === id && votingState.loading}
                    disabled={userVoted[id] || false}
                  />
                </div>

                {userVoted[id] && (
                  <p className="mt-4 text-center text-xs font-semibold text-emerald-500 uppercase tracking-widest">
                    You have voted on this proposal
                  </p>
                )}
              </div>
            )}
          </aside>
        </div>
      </div>
    </div>
  );
}
