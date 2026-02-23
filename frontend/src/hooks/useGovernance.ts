"use client";

import { useState, useCallback } from "react";

export type ProposalStatus = "OPEN" | "PASSED" | "REJECTED" | "EXECUTED";
export type VoteType = "For" | "Against" | "Abstain";

export interface Proposal {
  id: string;
  title: string;
  description: string;
  status: ProposalStatus;
  creator: string;
  createdAt: number;
  expiresAt: number;
  votes: {
    for: number;
    against: number;
    abstain: number;
  };
  type: string;
}

// Mock initial data
const INITIAL_PROPOSALS: Proposal[] = [
  {
    id: "1",
    title: "Increase Liquidity Reward for USDC/XLM Pool",
    description:
      "Proposal to increase the daily reward distribution for the USDC/XLM pool from 5,000 to 7,500 XLM to attract more liquidity providers.",
    status: "OPEN",
    creator: "GE...A3BS",
    createdAt: Date.now() - 86400000 * 2,
    expiresAt: Date.now() + 86400000 * 3,
    votes: { for: 1250000, against: 450000, abstain: 120000 },
    type: "Reward Parameters",
  },
  {
    id: "2",
    title: "Update Governance Veto Threshold",
    description:
      "Set the veto threshold for critical protocol changes to 33.3% of total staked tokens to ensure decentralization.",
    status: "PASSED",
    creator: "GC...P8Q1",
    createdAt: Date.now() - 86400000 * 10,
    expiresAt: Date.now() - 86400000 * 5,
    votes: { for: 5000000, against: 1000000, abstain: 500000 },
    type: "Protocol Settings",
  },
  {
    id: "3",
    title: "Allocate 100k XLM for Community Marketing",
    description:
      "Grant to fund community-led marketing initiatives for the next quarter, including local meetups and educational content.",
    status: "REJECTED",
    creator: "GA...L2X9",
    createdAt: Date.now() - 86400000 * 15,
    expiresAt: Date.now() - 86400000 * 10,
    votes: { for: 800000, against: 2500000, abstain: 300000 },
    type: "Treasury Grant",
  },
];

export function useGovernance() {
  const [proposals, setProposals] = useState<Proposal[]>(INITIAL_PROPOSALS);
  const [votingState, setVotingState] = useState<{
    id: string | null;
    loading: boolean;
    error: string | null;
  }>({
    id: null,
    loading: false,
    error: null,
  });
  const [userVoted, setUserVoted] = useState<Record<string, boolean>>({});
  const [walletConnected] = useState(true); // Mocking connected state

  const fetchProposals = useCallback(async () => {
    // In a real app, this would be an API call
    return proposals;
  }, [proposals]);

  const fetchProposalById = useCallback(
    async (id: string) => {
      return proposals.find((p) => p.id === id) || null;
    },
    [proposals],
  );

  const vote = useCallback(
    async (proposalId: string, type: VoteType) => {
      if (!walletConnected) {
        alert("Please connect your wallet first.");
        return;
      }

      if (userVoted[proposalId]) {
        setVotingState({
          id: proposalId,
          loading: false,
          error: "409: Already voted on this proposal",
        });
        return;
      }

      setVotingState({ id: proposalId, loading: true, error: null });

      try {
        // Simulate XDR signing flow
        console.log(
          `Signing XDR for proposal ${proposalId} with vote ${type}...`,
        );
        await new Promise((resolve) => setTimeout(resolve, 1500));

        setProposals((prev) =>
          prev.map((p) => {
            if (p.id === proposalId) {
              const newVotes = { ...p.votes };
              if (type === "For") newVotes.for += 100000; // Simplified weight
              if (type === "Against") newVotes.against += 100000;
              if (type === "Abstain") newVotes.abstain += 100000;
              return { ...p, votes: newVotes };
            }
            return p;
          }),
        );

        setUserVoted((prev) => ({ ...prev, [proposalId]: true }));
        setVotingState({ id: null, loading: false, error: null });
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : "Failed to vote";
        setVotingState({ id: proposalId, loading: false, error: message });
      }
    },
    [walletConnected, userVoted],
  );

  const createProposal = useCallback(
    async (data: {
      title: string;
      description: string;
      type: string;
      duration: number;
    }) => {
      const newProposal: Proposal = {
        id: Math.random().toString(36).substr(2, 9),
        title: data.title,
        description: data.description,
        status: "OPEN",
        creator: "ME...USER",
        createdAt: Date.now(),
        expiresAt: Date.now() + 86400000 * data.duration,
        votes: { for: 0, against: 0, abstain: 0 },
        type: data.type,
      };

      setProposals((prev) => [newProposal, ...prev]);
      return newProposal;
    },
    [],
  );

  return {
    proposals,
    votingState,
    walletConnected,
    fetchProposals,
    fetchProposalById,
    vote,
    createProposal,
    userVoted,
  };
}
